use anchor_lang::prelude::*;

#[error_code]
pub enum OracleError {
    #[msg("oracle config is paused")]
    OraclePaused,
    #[msg("caller is not authorized")]
    Unauthorized,
    #[msg("publisher is not allowlisted")]
    UnauthorizedPublisher,
    #[msg("pending admin is missing")]
    PendingAdminMissing,
    #[msg("too many publishers")]
    TooManyPublishers,
    #[msg("soft stale threshold must be non-negative and less than or equal to hard stale threshold")]
    InvalidThresholds,
    #[msg("only price scale 1e8 is supported by this deployment")]
    InvalidPriceScale,
    #[msg("sequence must be strictly monotonic")]
    SequenceNotMonotonic,
    #[msg("publish time cannot move backwards")]
    PublishTimeWentBackwards,
    #[msg("observed time must be greater than or equal to publish time")]
    InvalidTimestamps,
    #[msg("confidence must be non-zero")]
    InvalidConfidence,
    #[msg("source count must be non-zero")]
    InvalidSourceCount,
    #[msg("feed price must be positive")]
    InvalidPrice,
    #[msg("peg deviation requires a TWAP price")]
    InvalidPegDeviation,
    #[msg("feed does not belong to this oracle config")]
    FeedConfigMismatch,
    #[msg("publisher set does not belong to this oracle config")]
    PublisherSetMismatch,
    #[msg("feed symbol is too long")]
    FeedSymbolTooLong,
    #[msg("asset symbol is too long")]
    AssetSymbolTooLong,
    #[msg("deviation exceeds halt threshold and reject mode is enabled")]
    HaltedDeviationRejected,
    #[msg("feed status cannot be forced to an invalid value")]
    InvalidFeedStatus,
    #[msg("publisher set account is already initialized for another config")]
    PublisherSetAlreadyBound,
}
