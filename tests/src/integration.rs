use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};

use kzte_oracle::state::{FeedAccount, PublisherSet};

#[tokio::test]
async fn rejects_unauthorized_signer() {
    let mut context = setup().await;
    let config = Keypair::new();
    let authorized_publisher = Keypair::new();
    let unauthorized_publisher = Keypair::new();

    fund(&mut context, &authorized_publisher, 5_000_000_000).await;
    fund(&mut context, &unauthorized_publisher, 5_000_000_000).await;

    let publisher_set = publisher_set_pda(&config.pubkey());
    initialize_config(
        &mut context,
        &config,
        vec![authorized_publisher.pubkey()],
    )
    .await;
    create_feed(&mut context, &config, "KZTE/USD", "KZTE", "USD", true).await;

    let feed = feed_pda(&config.pubkey(), "KZTE/USD");
    let result = submit_update(
        &mut context,
        &config.pubkey(),
        &publisher_set,
        &feed,
        &unauthorized_publisher,
        1,
        212_558,
        101,
        101,
    )
    .await;

    assert_anchor_error(result, 6002);
}

#[tokio::test]
async fn rejects_replay_sequence() {
    let mut context = setup().await;
    let config = Keypair::new();
    let publisher = Keypair::new();

    fund(&mut context, &publisher, 5_000_000_000).await;

    let publisher_set = publisher_set_pda(&config.pubkey());
    initialize_config(&mut context, &config, vec![publisher.pubkey()]).await;
    create_feed(&mut context, &config, "KZTE/USD", "KZTE", "USD", true).await;

    let feed = feed_pda(&config.pubkey(), "KZTE/USD");
    submit_update(
        &mut context,
        &config.pubkey(),
        &publisher_set,
        &feed,
        &publisher,
        1,
        212_558,
        101,
        101,
    )
    .await
    .unwrap();

    let result = submit_update(
        &mut context,
        &config.pubkey(),
        &publisher_set,
        &feed,
        &publisher,
        1,
        212_558,
        102,
        102,
    )
    .await;

    assert_anchor_error(result, 6007);
}

async fn setup() -> ProgramTestContext {
    ProgramTest::new("kzte_oracle", kzte_oracle::id(), processor!(kzte_oracle::entry))
        .start_with_context()
        .await
}

async fn fund(context: &mut ProgramTestContext, recipient: &Keypair, lamports: u64) {
    let transfer = system_instruction::transfer(&context.payer.pubkey(), &recipient.pubkey(), lamports);
    let transaction = Transaction::new_signed_with_payer(
        &[transfer],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(transaction).await.unwrap();
    context.last_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
}

async fn initialize_config(
    context: &mut ProgramTestContext,
    config: &Keypair,
    publishers: Vec<solana_sdk::pubkey::Pubkey>,
) {
    let publisher_set = publisher_set_pda(&config.pubkey());
    let instruction = solana_sdk::instruction::Instruction {
        program_id: kzte_oracle::id(),
        accounts: kzte_oracle::accounts::InitializeOracleConfig {
            config: config.pubkey(),
            publisher_set,
            admin: context.payer.pubkey(),
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: kzte_oracle::instruction::InitializeOracleConfig {
            args: kzte_oracle::InitializeOracleConfigArgs {
                initial_publishers: publishers,
                soft_stale_seconds: 86_400,
                hard_stale_seconds: 3 * 86_400,
                warn_deviation_bps: 100,
                halt_deviation_bps: 500,
                price_scale: 100_000_000,
                halt_behavior: kzte_oracle::state::HaltBehavior::StoreHalted,
            },
        }
        .data(),
    };

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, config],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(transaction).await.unwrap();
    context.last_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();

    let publisher_set_account = context
        .banks_client
        .get_account(publisher_set)
        .await
        .unwrap()
        .unwrap();
    let mut data: &[u8] = publisher_set_account.data.as_ref();
    let state = PublisherSet::try_deserialize(&mut data).unwrap();
    assert_eq!(state.publishers.len(), 1);
}

async fn create_feed(
    context: &mut ProgramTestContext,
    config: &Keypair,
    symbol: &str,
    base_symbol: &str,
    quote_symbol: &str,
    is_reference_feed: bool,
) {
    let feed = feed_pda(&config.pubkey(), symbol);
    let instruction = solana_sdk::instruction::Instruction {
        program_id: kzte_oracle::id(),
        accounts: kzte_oracle::accounts::CreateFeed {
            config: config.pubkey(),
            feed,
            admin: context.payer.pubkey(),
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: kzte_oracle::instruction::CreateFeed {
            args: kzte_oracle::CreateFeedArgs {
                symbol: symbol.to_string(),
                base_symbol: base_symbol.to_string(),
                quote_symbol: quote_symbol.to_string(),
                is_reference_feed,
                metadata_version: 1,
            },
        }
        .data(),
    };

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(transaction).await.unwrap();
    context.last_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();

    let account = context.banks_client.get_account(feed).await.unwrap().unwrap();
    let mut data: &[u8] = account.data.as_ref();
    let state = FeedAccount::try_deserialize(&mut data).unwrap();
    assert_eq!(state.publish_time, 0);
}

async fn submit_update(
    context: &mut ProgramTestContext,
    config: &solana_sdk::pubkey::Pubkey,
    publisher_set: &solana_sdk::pubkey::Pubkey,
    feed: &solana_sdk::pubkey::Pubkey,
    publisher: &Keypair,
    sequence: u64,
    price: i64,
    publish_time: i64,
    observed_at: i64,
) -> Result<(), TransportError> {
    let instruction = solana_sdk::instruction::Instruction {
        program_id: kzte_oracle::id(),
        accounts: kzte_oracle::accounts::SubmitUpdate {
            config: *config,
            publisher_set: *publisher_set,
            feed: *feed,
            publisher: publisher.pubkey(),
        }
        .to_account_metas(None),
        data: kzte_oracle::instruction::SubmitUpdate {
            args: kzte_oracle::SubmitUpdateArgs {
                price,
                conf: 100,
                expo: -8,
                publish_time,
                observed_at,
                source_count: 1,
                peg_deviation_bps: 0,
                sequence,
                twap_price: None,
                raw_payload_hash: [1u8; 32],
                metadata_version: 2,
            },
        }
        .data(),
    };

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, publisher],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(transaction).await;
    if result.is_ok() {
        context.last_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    }
    result
}

fn publisher_set_pda(config_pubkey: &solana_sdk::pubkey::Pubkey) -> solana_sdk::pubkey::Pubkey {
    solana_sdk::pubkey::Pubkey::find_program_address(
        &[PublisherSet::SEED_PREFIX, config_pubkey.as_ref()],
        &kzte_oracle::id(),
    )
    .0
}

fn feed_pda(config_pubkey: &solana_sdk::pubkey::Pubkey, symbol: &str) -> solana_sdk::pubkey::Pubkey {
    solana_sdk::pubkey::Pubkey::find_program_address(
        &[FeedAccount::SEED_PREFIX, config_pubkey.as_ref(), symbol.as_bytes()],
        &kzte_oracle::id(),
    )
    .0
}

fn assert_anchor_error(result: Result<(), TransportError>, expected_code: u32) {
    let error = result.expect_err("transaction was expected to fail");
    let code = match error {
        TransportError::TransactionError(TransactionError::InstructionError(_, InstructionError::Custom(code))) => code,
        other => panic!("unexpected error shape: {other:?}"),
    };
    assert_eq!(code, expected_code);
}
