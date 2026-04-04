use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod state;

use constants::{EXPO, PRICE_SCALE};
use error::OracleError;
use state::{FeedAccount, FeedStatus, HaltBehavior, OracleConfig, PublisherSet};

declare_id!("9FZHqpq5ffv8HLKtWMjVU8WuNTUH44WDYeKr6MEtx7ex");

#[program]
pub mod kzte_oracle {
    use super::*;

    pub fn initialize_oracle_config(
        ctx: Context<InitializeOracleConfig>,
        args: InitializeOracleConfigArgs,
    ) -> Result<()> {
        validate_thresholds(
            args.soft_stale_seconds,
            args.hard_stale_seconds,
            args.warn_deviation_bps,
            args.halt_deviation_bps,
            args.price_scale,
        )?;
        require!(
            args.initial_publishers.len() <= constants::MAX_PUBLISHERS,
            OracleError::TooManyPublishers
        );

        let config = &mut ctx.accounts.config;
        let publisher_set = &mut ctx.accounts.publisher_set;

        config.admin = ctx.accounts.admin.key();
        config.pending_admin = None;
        config.paused = false;
        config.publisher_set = publisher_set.key();
        config.soft_stale_seconds = args.soft_stale_seconds;
        config.hard_stale_seconds = args.hard_stale_seconds;
        config.warn_deviation_bps = args.warn_deviation_bps;
        config.halt_deviation_bps = args.halt_deviation_bps;
        config.price_scale = args.price_scale;
        config.last_sequence = 0;
        config.halt_behavior = args.halt_behavior;

        publisher_set.replace_publishers(config.key(), args.initial_publishers)?;
        Ok(())
    }

    pub fn transfer_admin(ctx: Context<AdminOnly>, new_admin: Pubkey) -> Result<()> {
        ctx.accounts.config.pending_admin = Some(new_admin);
        Ok(())
    }

    pub fn accept_admin(ctx: Context<AcceptAdmin>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        let pending = config
            .pending_admin
            .ok_or_else(|| error!(OracleError::PendingAdminMissing))?;

        require_keys_eq!(pending, ctx.accounts.new_admin.key(), OracleError::Unauthorized);
        config.admin = pending;
        config.pending_admin = None;
        Ok(())
    }

    pub fn set_publishers(ctx: Context<SetPublishers>, publishers: Vec<Pubkey>) -> Result<()> {
        ctx.accounts
            .publisher_set
            .replace_publishers(ctx.accounts.config.key(), publishers)?;
        Ok(())
    }

    pub fn set_thresholds(ctx: Context<AdminOnly>, args: SetThresholdsArgs) -> Result<()> {
        validate_thresholds(
            args.soft_stale_seconds,
            args.hard_stale_seconds,
            args.warn_deviation_bps,
            args.halt_deviation_bps,
            args.price_scale,
        )?;

        let config = &mut ctx.accounts.config;
        config.soft_stale_seconds = args.soft_stale_seconds;
        config.hard_stale_seconds = args.hard_stale_seconds;
        config.warn_deviation_bps = args.warn_deviation_bps;
        config.halt_deviation_bps = args.halt_deviation_bps;
        config.price_scale = args.price_scale;
        config.halt_behavior = args.halt_behavior;
        Ok(())
    }

    pub fn pause_oracle(ctx: Context<AdminOnly>) -> Result<()> {
        ctx.accounts.config.paused = true;
        Ok(())
    }

    pub fn resume_oracle(ctx: Context<AdminOnly>) -> Result<()> {
        ctx.accounts.config.paused = false;
        Ok(())
    }

    pub fn create_feed(ctx: Context<CreateFeed>, args: CreateFeedArgs) -> Result<()> {
        let feed = &mut ctx.accounts.feed;
        feed.config = ctx.accounts.config.key();
        feed.set_symbols(&args.symbol, &args.base_symbol, &args.quote_symbol)?;
        feed.price = 0;
        feed.conf = 0;
        feed.expo = EXPO;
        feed.publish_time = 0;
        feed.prev_publish_time = 0;
        feed.status = if ctx.accounts.config.paused {
            FeedStatus::Paused
        } else {
            FeedStatus::Stale
        };
        feed.source_count = 0;
        feed.last_good_price = 0;
        feed.peg_deviation_bps = 0;
        feed.sequence = 0;
        feed.is_reference_feed = args.is_reference_feed;
        feed.twap_price = None;
        feed.metadata_hash = [0u8; 32];
        feed.metadata_version = args.metadata_version;
        Ok(())
    }

    pub fn submit_update(ctx: Context<SubmitUpdate>, args: SubmitUpdateArgs) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(!config.paused, OracleError::OraclePaused);
        require!(
            ctx.accounts.publisher_set.contains(&ctx.accounts.publisher.key()),
            OracleError::UnauthorizedPublisher
        );
        require_keys_eq!(
            ctx.accounts.publisher_set.key(),
            config.publisher_set,
            OracleError::PublisherSetMismatch
        );
        require_keys_eq!(
            ctx.accounts.feed.config,
            config.key(),
            OracleError::FeedConfigMismatch
        );

        validate_submit_timestamps(&args)?;
        require!(args.price > 0, OracleError::InvalidPrice);
        require!(args.conf > 0, OracleError::InvalidConfidence);
        require!(args.source_count > 0, OracleError::InvalidSourceCount);
        require!(args.expo == EXPO, OracleError::InvalidPriceScale);
        require!(
            args.sequence > config.last_sequence && args.sequence > ctx.accounts.feed.sequence,
            OracleError::SequenceNotMonotonic
        );
        require!(
            args.publish_time >= ctx.accounts.feed.publish_time,
            OracleError::PublishTimeWentBackwards
        );
        if args.twap_price.is_none() {
            require!(args.peg_deviation_bps == 0, OracleError::InvalidPegDeviation);
        }

        let next_status = resolve_status(config, &ctx.accounts.feed, &args)?;
        let previous_publish_time = ctx.accounts.feed.publish_time;
        let next_last_good_price = match next_status {
            FeedStatus::Active | FeedStatus::CarryForward | FeedStatus::Diverged => args.price,
            FeedStatus::Stale | FeedStatus::Halted | FeedStatus::Paused => {
                if ctx.accounts.feed.last_good_price > 0 {
                    ctx.accounts.feed.last_good_price
                } else {
                    args.price
                }
            }
        };

        let feed = &mut ctx.accounts.feed;
        feed.price = args.price;
        feed.conf = args.conf;
        feed.expo = args.expo;
        feed.prev_publish_time = previous_publish_time;
        feed.publish_time = args.publish_time;
        feed.status = next_status;
        feed.source_count = args.source_count;
        feed.last_good_price = next_last_good_price;
        feed.peg_deviation_bps = args.peg_deviation_bps;
        feed.sequence = args.sequence;
        feed.twap_price = args.twap_price;
        feed.metadata_hash = args.raw_payload_hash;
        feed.metadata_version = args.metadata_version;

        config.last_sequence = args.sequence;
        Ok(())
    }

    pub fn force_set_status(
        ctx: Context<ForceSetStatus>,
        status: FeedStatus,
        keep_last_good_price: bool,
    ) -> Result<()> {
        let feed = &mut ctx.accounts.feed;
        feed.status = status;
        if !keep_last_good_price {
            feed.last_good_price = feed.price;
        }
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(args: InitializeOracleConfigArgs)]
pub struct InitializeOracleConfig<'info> {
    #[account(init, payer = admin, space = OracleConfig::LEN)]
    pub config: Account<'info, OracleConfig>,
    #[account(
        init,
        payer = admin,
        space = PublisherSet::LEN,
        seeds = [PublisherSet::SEED_PREFIX, config.key().as_ref()],
        bump
    )]
    pub publisher_set: Account<'info, PublisherSet>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    #[account(mut, has_one = admin)]
    pub config: Account<'info, OracleConfig>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct AcceptAdmin<'info> {
    #[account(mut)]
    pub config: Account<'info, OracleConfig>,
    pub new_admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetPublishers<'info> {
    #[account(mut, has_one = admin)]
    pub config: Account<'info, OracleConfig>,
    #[account(
        mut,
        seeds = [PublisherSet::SEED_PREFIX, config.key().as_ref()],
        bump,
        constraint = publisher_set.config == config.key() @ OracleError::PublisherSetMismatch
    )]
    pub publisher_set: Account<'info, PublisherSet>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(args: CreateFeedArgs)]
pub struct CreateFeed<'info> {
    #[account(has_one = admin)]
    pub config: Account<'info, OracleConfig>,
    #[account(
        init,
        payer = admin,
        space = FeedAccount::LEN,
        seeds = [FeedAccount::SEED_PREFIX, config.key().as_ref(), args.symbol.as_bytes()],
        bump
    )]
    pub feed: Account<'info, FeedAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitUpdate<'info> {
    #[account(mut)]
    pub config: Account<'info, OracleConfig>,
    #[account(
        seeds = [PublisherSet::SEED_PREFIX, config.key().as_ref()],
        bump,
        constraint = publisher_set.config == config.key() @ OracleError::PublisherSetMismatch
    )]
    pub publisher_set: Account<'info, PublisherSet>,
    #[account(mut, constraint = feed.config == config.key() @ OracleError::FeedConfigMismatch)]
    pub feed: Account<'info, FeedAccount>,
    pub publisher: Signer<'info>,
}

#[derive(Accounts)]
pub struct ForceSetStatus<'info> {
    #[account(has_one = admin)]
    pub config: Account<'info, OracleConfig>,
    #[account(mut, constraint = feed.config == config.key() @ OracleError::FeedConfigMismatch)]
    pub feed: Account<'info, FeedAccount>,
    pub admin: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitializeOracleConfigArgs {
    pub initial_publishers: Vec<Pubkey>,
    pub soft_stale_seconds: i64,
    pub hard_stale_seconds: i64,
    pub warn_deviation_bps: u32,
    pub halt_deviation_bps: u32,
    pub price_scale: u64,
    pub halt_behavior: HaltBehavior,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SetThresholdsArgs {
    pub soft_stale_seconds: i64,
    pub hard_stale_seconds: i64,
    pub warn_deviation_bps: u32,
    pub halt_deviation_bps: u32,
    pub price_scale: u64,
    pub halt_behavior: HaltBehavior,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CreateFeedArgs {
    pub symbol: String,
    pub base_symbol: String,
    pub quote_symbol: String,
    pub is_reference_feed: bool,
    pub metadata_version: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SubmitUpdateArgs {
    pub price: i64,
    pub conf: u64,
    pub expo: i32,
    pub publish_time: i64,
    pub observed_at: i64,
    pub source_count: u8,
    pub peg_deviation_bps: u32,
    pub sequence: u64,
    pub twap_price: Option<i64>,
    pub raw_payload_hash: [u8; 32],
    pub metadata_version: u32,
}

fn validate_thresholds(
    soft_stale_seconds: i64,
    hard_stale_seconds: i64,
    warn_deviation_bps: u32,
    halt_deviation_bps: u32,
    price_scale: u64,
) -> Result<()> {
    require!(soft_stale_seconds >= 0, OracleError::InvalidThresholds);
    require!(hard_stale_seconds >= soft_stale_seconds, OracleError::InvalidThresholds);
    require!(halt_deviation_bps >= warn_deviation_bps, OracleError::InvalidThresholds);
    require!(price_scale == PRICE_SCALE, OracleError::InvalidPriceScale);
    Ok(())
}

fn validate_submit_timestamps(args: &SubmitUpdateArgs) -> Result<()> {
    require!(args.observed_at >= args.publish_time, OracleError::InvalidTimestamps);
    Ok(())
}

fn resolve_status(config: &OracleConfig, feed: &FeedAccount, args: &SubmitUpdateArgs) -> Result<FeedStatus> {
    let age = args
        .observed_at
        .checked_sub(args.publish_time)
        .ok_or_else(|| error!(OracleError::InvalidTimestamps))?;

    if args.twap_price.is_some() && args.peg_deviation_bps > config.halt_deviation_bps {
        return match config.halt_behavior {
            HaltBehavior::Reject => err!(OracleError::HaltedDeviationRejected),
            HaltBehavior::StoreHalted => Ok(FeedStatus::Halted),
        };
    }

    if age > config.hard_stale_seconds {
        return Ok(FeedStatus::Stale);
    }

    if args.twap_price.is_some() && args.peg_deviation_bps > config.warn_deviation_bps {
        return Ok(FeedStatus::Diverged);
    }

    if feed.publish_time != 0 && args.publish_time == feed.publish_time {
        return Ok(FeedStatus::CarryForward);
    }

    Ok(FeedStatus::Active)
}
