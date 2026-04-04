use anyhow::{Context, Result};
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use kzte_common::{FeederConfig, HaltBehavior};
use kzte_oracle::state::{
    FeedAccount as ChainFeedAccount, HaltBehavior as ChainHaltBehavior, OracleConfig as ChainOracleConfig,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signature, Signer},
    transaction::Transaction,
};
use std::str::FromStr;
use std::sync::Arc;

use crate::aggregator::{FeedSet, FeedSnapshot, OracleThresholds};

pub struct OracleRpcClient {
    rpc: RpcClient,
    signer: Arc<Keypair>,
    program_id: Pubkey,
}

impl OracleRpcClient {
    pub fn new(config: &FeederConfig) -> Result<Self> {
        let signer = Arc::new(
            read_keypair_file(&config.rpc.keypair_path)
                .with_context(|| format!("failed to read keypair at {}", config.rpc.keypair_path))?,
        );
        let program_id = Pubkey::from_str(&config.rpc.program_id)
            .with_context(|| format!("invalid program id {}", config.rpc.program_id))?;

        Ok(Self {
            rpc: RpcClient::new_with_commitment(
                config.rpc.http_url.clone(),
                CommitmentConfig::confirmed(),
            ),
            signer,
            program_id,
        })
    }

    pub fn signer_pubkey(&self) -> Pubkey {
        self.signer.pubkey()
    }

    pub async fn load_state(&self, config: &FeederConfig) -> Result<(OracleThresholds, FeedSet)> {
        let oracle_key = parse_pubkey(&config.oracle.config_pubkey, "oracle config pubkey")?;
        let kzte_kzt_key = parse_pubkey(&config.oracle.feed_kzte_kzt, "KZTE/KZT feed pubkey")?;
        let kzte_usd_key = parse_pubkey(&config.oracle.feed_kzte_usd, "KZTE/USD feed pubkey")?;

        let oracle: ChainOracleConfig = self.fetch_account(&oracle_key).await?;
        let kzte_kzt: ChainFeedAccount = self.fetch_account(&kzte_kzt_key).await?;
        let kzte_usd: ChainFeedAccount = self.fetch_account(&kzte_usd_key).await?;

        let kzte_usdc = if config.oracle.feed_kzte_usdc.trim().is_empty() {
            None
        } else {
            let feed_key = parse_pubkey(&config.oracle.feed_kzte_usdc, "KZTE/USDC feed pubkey")?;
            let feed: ChainFeedAccount = self.fetch_account(&feed_key).await?;
            Some(feed_into_snapshot(feed))
        };

        Ok((
            OracleThresholds {
                soft_stale_seconds: oracle.soft_stale_seconds,
                hard_stale_seconds: oracle.hard_stale_seconds,
                warn_deviation_bps: oracle.warn_deviation_bps,
                halt_deviation_bps: oracle.halt_deviation_bps,
                halt_behavior: map_halt_behavior(oracle.halt_behavior),
                last_sequence: oracle.last_sequence,
            },
            FeedSet {
                kzte_kzt: feed_into_snapshot(kzte_kzt),
                kzte_usd: feed_into_snapshot(kzte_usd),
                kzte_usdc,
            },
        ))
    }

    pub async fn submit_update(
        &self,
        config_pubkey: &Pubkey,
        publisher_set_pubkey: &Pubkey,
        feed_pubkey: &Pubkey,
        submission: &kzte_common::ReferenceSubmission,
    ) -> Result<Signature> {
        let accounts = kzte_oracle::accounts::SubmitUpdate {
            config: *config_pubkey,
            publisher_set: *publisher_set_pubkey,
            feed: *feed_pubkey,
            publisher: self.signer.pubkey(),
        };
        let args = kzte_oracle::SubmitUpdateArgs {
            price: submission.price,
            conf: submission.conf,
            expo: submission.expo,
            publish_time: submission.publish_time,
            observed_at: submission.observed_at,
            source_count: submission.source_count,
            peg_deviation_bps: submission.peg_deviation_bps,
            sequence: submission.sequence,
            twap_price: submission.twap_price,
            raw_payload_hash: submission.raw_payload_hash,
            metadata_version: submission.metadata_version,
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: accounts.to_account_metas(None),
            data: kzte_oracle::instruction::SubmitUpdate { args }.data(),
        };
        let blockhash = self.rpc.get_latest_blockhash().await.context("failed to fetch recent blockhash")?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.signer.pubkey()),
            &[self.signer.as_ref()],
            blockhash,
        );

        self.rpc
            .send_and_confirm_transaction(&transaction)
            .await
            .with_context(|| format!("failed to submit oracle update for {}", submission.feed_symbol))
    }

    pub fn rpc(&self) -> &RpcClient {
        &self.rpc
    }

    async fn fetch_account<T>(&self, pubkey: &Pubkey) -> Result<T>
    where
        T: AccountDeserialize,
    {
        let data = self
            .rpc
            .get_account_data(pubkey)
            .await
            .with_context(|| format!("failed to fetch account {}", pubkey))?;
        let mut slice: &[u8] = data.as_ref();
        T::try_deserialize(&mut slice).with_context(|| format!("failed to deserialize account {}", pubkey))
    }
}

pub fn parse_pubkey(raw: &str, label: &str) -> Result<Pubkey> {
    Pubkey::from_str(raw).with_context(|| format!("invalid {}: {}", label, raw))
}

fn map_halt_behavior(value: ChainHaltBehavior) -> HaltBehavior {
    match value {
        ChainHaltBehavior::Reject => HaltBehavior::Reject,
        ChainHaltBehavior::StoreHalted => HaltBehavior::StoreHalted,
    }
}

fn feed_into_snapshot(feed: ChainFeedAccount) -> FeedSnapshot {
    FeedSnapshot {
        price: feed.price,
        last_good_price: feed.last_good_price,
        publish_time: feed.publish_time,
        source_count: feed.source_count,
        metadata_hash: feed.metadata_hash,
        metadata_version: feed.metadata_version,
    }
}
