use anchor_client::{
    solana_sdk::signature::Keypair as AnchorKeypair, Client, Cluster, Program,
};
use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use kzte_common::{load_toml_file, CliConfig};
use kzte_feeder::{load_config as load_feeder_config, run_once};
use kzte_feeder::metrics::FeederMetrics;
use kzte_oracle::state::{FeedAccount, FeedStatus as ChainFeedStatus, HaltBehavior as ChainHaltBehavior, OracleConfig};
use serde_json::json;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{read_keypair_file, Signature, Signer},
    system_program,
};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Parser)]
#[command(name = "kzte-cli", about = "CLI for the KZTE reference oracle")]
struct Args {
    #[arg(long, env = "CLI_CONFIG_PATH", default_value = "config/cli.example.toml")]
    config: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init {
        #[arg(long)]
        config_keypair: String,
        #[arg(long)]
        soft_stale_seconds: i64,
        #[arg(long)]
        hard_stale_seconds: i64,
        #[arg(long)]
        warn_deviation_bps: u32,
        #[arg(long)]
        halt_deviation_bps: u32,
        #[arg(long, value_enum, default_value_t = HaltBehaviorArg::StoreHalted)]
        halt_behavior: HaltBehaviorArg,
        #[arg(long, value_delimiter = ',')]
        publishers: Vec<String>,
    },
    CreateFeed {
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        base_symbol: String,
        #[arg(long)]
        quote_symbol: String,
        #[arg(long)]
        is_reference_feed: bool,
        #[arg(long, default_value_t = 1)]
        metadata_version: u32,
    },
    ReadConfig {
        #[arg(long)]
        address: Option<String>,
    },
    ReadFeed {
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        address: Option<String>,
    },
    SetPublishers {
        #[arg(long, value_delimiter = ',')]
        publishers: Vec<String>,
    },
    SetThresholds {
        #[arg(long)]
        soft_stale_seconds: i64,
        #[arg(long)]
        hard_stale_seconds: i64,
        #[arg(long)]
        warn_deviation_bps: u32,
        #[arg(long)]
        halt_deviation_bps: u32,
        #[arg(long, value_enum, default_value_t = HaltBehaviorArg::StoreHalted)]
        halt_behavior: HaltBehaviorArg,
    },
    Pause,
    Resume,
    TransferAdmin {
        #[arg(long)]
        new_admin: String,
    },
    AcceptAdmin,
    ForceSetStatus {
        #[arg(long)]
        symbol: String,
        #[arg(long, value_enum)]
        status: FeedStatusArg,
        #[arg(long, default_value_t = true)]
        keep_last_good_price: bool,
    },
    Update {
        #[arg(long, env = "FEEDER_CONFIG_PATH", default_value = "config/feeder.example.toml")]
        feeder_config: String,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum HaltBehaviorArg {
    Reject,
    StoreHalted,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FeedStatusArg {
    Active,
    CarryForward,
    Stale,
    Diverged,
    Paused,
    Halted,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config: CliConfig = load_toml_file(&args.config)
        .with_context(|| format!("failed to load CLI config from {}", args.config))?;

    match args.command {
        Command::Init {
            config_keypair,
            soft_stale_seconds,
            hard_stale_seconds,
            warn_deviation_bps,
            halt_deviation_bps,
            halt_behavior,
            publishers,
        } => {
            let payer = load_keypair(&config.rpc.keypair_path)?;
            let admin_pubkey = payer.pubkey();
            let program_id = parse_pubkey(&config.rpc.program_id)?;
            let config_keypair = load_keypair(&config_keypair)?;
            let program = program(&config, payer)?;
            let publisher_set = publisher_set_pda(&config_keypair.pubkey(), &program_id);
            let publisher_keys = parse_pubkeys(&publishers)?;

            let signature = program
                .request()
                .accounts(kzte_oracle::accounts::InitializeOracleConfig {
                    config: config_keypair.pubkey(),
                    publisher_set,
                    admin: admin_pubkey,
                    system_program: system_program::ID,
                })
                .args(kzte_oracle::instruction::InitializeOracleConfig {
                    args: kzte_oracle::InitializeOracleConfigArgs {
                        initial_publishers: publisher_keys,
                        soft_stale_seconds,
                        hard_stale_seconds,
                        warn_deviation_bps,
                        halt_deviation_bps,
                        price_scale: 100_000_000,
                        halt_behavior: halt_behavior.into(),
                    },
                })
                .signer(&config_keypair)
                .send()?;

            print_json(json!({
                "signature": signature.to_string(),
                "config_pubkey": config_keypair.pubkey().to_string(),
                "publisher_set_pubkey": publisher_set.to_string(),
            }));
        }
        Command::CreateFeed {
            symbol,
            base_symbol,
            quote_symbol,
            is_reference_feed,
            metadata_version,
        } => {
            let payer = load_keypair(&config.rpc.keypair_path)?;
            let admin_pubkey = payer.pubkey();
            let program_id = parse_pubkey(&config.rpc.program_id)?;
            let program = program(&config, payer)?;
            let config_pubkey = parse_pubkey(&config.oracle.config_pubkey)?;
            let feed = feed_pda(&config_pubkey, &symbol, &program_id);

            let signature = program
                .request()
                .accounts(kzte_oracle::accounts::CreateFeed {
                    config: config_pubkey,
                    feed,
                    admin: admin_pubkey,
                    system_program: system_program::ID,
                })
                .args(kzte_oracle::instruction::CreateFeed {
                    args: kzte_oracle::CreateFeedArgs {
                        symbol: symbol.clone(),
                        base_symbol,
                        quote_symbol,
                        is_reference_feed,
                        metadata_version,
                    },
                })
                .send()?;

            print_json(json!({
                "signature": signature.to_string(),
                "feed_symbol": symbol,
                "feed_pubkey": feed.to_string(),
            }));
        }
        Command::ReadConfig { address } => {
            let payer = load_keypair(&config.rpc.keypair_path)?;
            let program = program(&config, payer)?;
            let address = address
                .map(|value| parse_pubkey(&value))
                .transpose()?
                .unwrap_or(parse_pubkey(&config.oracle.config_pubkey)?);
            let account: OracleConfig = program.account(address)?;
            print_json(json!({
                "address": address.to_string(),
                "admin": account.admin.to_string(),
                "pending_admin": account.pending_admin.map(|value| value.to_string()),
                "paused": account.paused,
                "publisher_set": account.publisher_set.to_string(),
                "soft_stale_seconds": account.soft_stale_seconds,
                "hard_stale_seconds": account.hard_stale_seconds,
                "warn_deviation_bps": account.warn_deviation_bps,
                "halt_deviation_bps": account.halt_deviation_bps,
                "price_scale": account.price_scale,
                "last_sequence": account.last_sequence,
                "halt_behavior": format!("{:?}", account.halt_behavior),
            }));
        }
        Command::ReadFeed { symbol, address } => {
            let payer = load_keypair(&config.rpc.keypair_path)?;
            let program_id = parse_pubkey(&config.rpc.program_id)?;
            let program = program(&config, payer)?;
            let address = match (symbol, address) {
                (_, Some(address)) => parse_pubkey(&address)?,
                (Some(symbol), None) => feed_address_from_config(&config, &symbol, &program_id)?,
                (None, None) => return Err(anyhow!("either --symbol or --address must be provided")),
            };

            let account: FeedAccount = program.account(address)?;
            print_json(json!({
                "address": address.to_string(),
                "price": account.price,
                "conf": account.conf,
                "expo": account.expo,
                "publish_time": account.publish_time,
                "prev_publish_time": account.prev_publish_time,
                "status": format!("{:?}", account.status),
                "source_count": account.source_count,
                "last_good_price": account.last_good_price,
                "peg_deviation_bps": account.peg_deviation_bps,
                "sequence": account.sequence,
                "is_reference_feed": account.is_reference_feed,
                "twap_price": account.twap_price,
                "metadata_hash_hex": hex::encode(account.metadata_hash),
                "metadata_version": account.metadata_version,
            }));
        }
        Command::SetPublishers { publishers } => {
            let payer = load_keypair(&config.rpc.keypair_path)?;
            let admin_pubkey = payer.pubkey();
            let program = program(&config, payer)?;
            let config_pubkey = parse_pubkey(&config.oracle.config_pubkey)?;
            let publisher_set = parse_pubkey(&config.oracle.publisher_set_pubkey)?;
            let signature = program
                .request()
                .accounts(kzte_oracle::accounts::SetPublishers {
                    config: config_pubkey,
                    publisher_set,
                    admin: admin_pubkey,
                })
                .args(kzte_oracle::instruction::SetPublishers {
                    publishers: parse_pubkeys(&publishers)?,
                })
                .send()?;
            print_signature(signature);
        }
        Command::SetThresholds {
            soft_stale_seconds,
            hard_stale_seconds,
            warn_deviation_bps,
            halt_deviation_bps,
            halt_behavior,
        } => {
            let payer = load_keypair(&config.rpc.keypair_path)?;
            let admin_pubkey = payer.pubkey();
            let program = program(&config, payer)?;
            let config_pubkey = parse_pubkey(&config.oracle.config_pubkey)?;
            let signature = program
                .request()
                .accounts(kzte_oracle::accounts::AdminOnly {
                    config: config_pubkey,
                    admin: admin_pubkey,
                })
                .args(kzte_oracle::instruction::SetThresholds {
                    args: kzte_oracle::SetThresholdsArgs {
                        soft_stale_seconds,
                        hard_stale_seconds,
                        warn_deviation_bps,
                        halt_deviation_bps,
                        price_scale: 100_000_000,
                        halt_behavior: halt_behavior.into(),
                    },
                })
                .send()?;
            print_signature(signature);
        }
        Command::Pause => {
            let signature = admin_only_call(&config, |program, config_pubkey, admin_pubkey| {
                Ok(program
                    .request()
                    .accounts(kzte_oracle::accounts::AdminOnly {
                        config: config_pubkey,
                        admin: admin_pubkey,
                    })
                    .args(kzte_oracle::instruction::PauseOracle {})
                    .send()?)
            })?;
            print_signature(signature);
        }
        Command::Resume => {
            let signature = admin_only_call(&config, |program, config_pubkey, admin_pubkey| {
                Ok(program
                    .request()
                    .accounts(kzte_oracle::accounts::AdminOnly {
                        config: config_pubkey,
                        admin: admin_pubkey,
                    })
                    .args(kzte_oracle::instruction::ResumeOracle {})
                    .send()?)
            })?;
            print_signature(signature);
        }
        Command::TransferAdmin { new_admin } => {
            let signature = admin_only_call(&config, |program, config_pubkey, admin_pubkey| {
                Ok(program
                    .request()
                    .accounts(kzte_oracle::accounts::AdminOnly {
                        config: config_pubkey,
                        admin: admin_pubkey,
                    })
                    .args(kzte_oracle::instruction::TransferAdmin {
                        new_admin: parse_pubkey(&new_admin)?,
                    })
                    .send()?)
            })?;
            print_signature(signature);
        }
        Command::AcceptAdmin => {
            let payer = load_keypair(&config.rpc.keypair_path)?;
            let admin_pubkey = payer.pubkey();
            let program = program(&config, payer)?;
            let config_pubkey = parse_pubkey(&config.oracle.config_pubkey)?;
            let signature = program
                .request()
                .accounts(kzte_oracle::accounts::AcceptAdmin {
                    config: config_pubkey,
                    new_admin: admin_pubkey,
                })
                .args(kzte_oracle::instruction::AcceptAdmin {})
                .send()?;
            print_signature(signature);
        }
        Command::ForceSetStatus {
            symbol,
            status,
            keep_last_good_price,
        } => {
            let payer = load_keypair(&config.rpc.keypair_path)?;
            let admin_pubkey = payer.pubkey();
            let program_id = parse_pubkey(&config.rpc.program_id)?;
            let program = program(&config, payer)?;
            let config_pubkey = parse_pubkey(&config.oracle.config_pubkey)?;
            let feed = feed_address_from_config(&config, &symbol, &program_id)?;
            let signature = program
                .request()
                .accounts(kzte_oracle::accounts::ForceSetStatus {
                    config: config_pubkey,
                    feed,
                    admin: admin_pubkey,
                })
                .args(kzte_oracle::instruction::ForceSetStatus {
                    status: status.into(),
                    keep_last_good_price,
                })
                .send()?;
            print_signature(signature);
        }
        Command::Update { feeder_config } => {
            let feeder_config = load_feeder_config(&feeder_config)?;
            let metrics = Arc::new(FeederMetrics::default());
            let summary = run_once(&feeder_config, &metrics).await?;
            print_json(json!({
                "submitted": summary.submitted,
                "skipped": summary.skipped,
                "signatures": summary.signatures,
                "used_official_sources": summary.used_official_sources,
                "carry_forward": summary.carry_forward,
            }));
        }
    }

    Ok(())
}

fn program(config: &CliConfig, payer: AnchorKeypair) -> Result<Program<Rc<AnchorKeypair>>> {
    let cluster = Cluster::Custom(config.rpc.http_url.clone(), config.rpc.ws_url.clone());
    let client = Client::new_with_options(cluster, Rc::new(payer), CommitmentConfig::confirmed());
    client
        .program(parse_pubkey(&config.rpc.program_id)?)
        .context("failed to create Anchor program client")
}

fn load_keypair(path: impl AsRef<Path>) -> Result<AnchorKeypair> {
    let expanded = expand_tilde(path.as_ref())?;
    read_keypair_file(&expanded).map_err(|error| {
        anyhow!(
            "failed to read keypair from {}: {}",
            expanded.display(),
            error
        )
    })
}

fn expand_tilde(path: &Path) -> Result<PathBuf> {
    let raw = path.to_string_lossy();
    if let Some(stripped) = raw.strip_prefix("~/") {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .context("could not resolve user home directory")?;
        Ok(PathBuf::from(home).join(stripped))
    } else {
        Ok(path.to_path_buf())
    }
}

fn publisher_set_pda(config_pubkey: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[kzte_oracle::state::PublisherSet::SEED_PREFIX, config_pubkey.as_ref()],
        program_id,
    )
    .0
}

fn feed_pda(config_pubkey: &Pubkey, symbol: &str, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[kzte_oracle::state::FeedAccount::SEED_PREFIX, config_pubkey.as_ref(), symbol.as_bytes()],
        program_id,
    )
    .0
}

fn feed_address_from_config(config: &CliConfig, symbol: &str, program_id: &Pubkey) -> Result<Pubkey> {
    match symbol {
        "KZTE/KZT" => parse_pubkey(&config.feeds.kzte_kzt),
        "KZTE/USD" => parse_pubkey(&config.feeds.kzte_usd),
        "KZTE/USDC" if !config.feeds.kzte_usdc.trim().is_empty() => parse_pubkey(&config.feeds.kzte_usdc),
        other => {
            let config_pubkey = parse_pubkey(&config.oracle.config_pubkey)?;
            Ok(feed_pda(&config_pubkey, other, program_id))
        }
    }
}

fn parse_pubkeys(values: &[String]) -> Result<Vec<Pubkey>> {
    values.iter().map(|value| parse_pubkey(value)).collect()
}

fn parse_pubkey(value: &str) -> Result<Pubkey> {
    Pubkey::from_str(value).with_context(|| format!("invalid pubkey {}", value))
}

fn admin_only_call(
    config: &CliConfig,
    call: impl FnOnce(&Program<Rc<AnchorKeypair>>, Pubkey, Pubkey) -> Result<Signature>,
) -> Result<Signature> {
    let payer = load_keypair(&config.rpc.keypair_path)?;
    let admin_pubkey = payer.pubkey();
    let program = program(config, payer)?;
    let config_pubkey = parse_pubkey(&config.oracle.config_pubkey)?;
    call(&program, config_pubkey, admin_pubkey)
}

fn print_signature(signature: Signature) {
    print_json(json!({ "signature": signature.to_string() }));
}

fn print_json(value: serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(&value).unwrap());
}

impl From<HaltBehaviorArg> for ChainHaltBehavior {
    fn from(value: HaltBehaviorArg) -> Self {
        match value {
            HaltBehaviorArg::Reject => ChainHaltBehavior::Reject,
            HaltBehaviorArg::StoreHalted => ChainHaltBehavior::StoreHalted,
        }
    }
}

impl From<FeedStatusArg> for ChainFeedStatus {
    fn from(value: FeedStatusArg) -> Self {
        match value {
            FeedStatusArg::Active => ChainFeedStatus::Active,
            FeedStatusArg::CarryForward => ChainFeedStatus::CarryForward,
            FeedStatusArg::Stale => ChainFeedStatus::Stale,
            FeedStatusArg::Diverged => ChainFeedStatus::Diverged,
            FeedStatusArg::Paused => ChainFeedStatus::Paused,
            FeedStatusArg::Halted => ChainFeedStatus::Halted,
        }
    }
}
