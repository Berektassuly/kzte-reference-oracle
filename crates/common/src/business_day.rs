use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StalenessTier {
    Fresh,
    SoftStale,
    HardStale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarryForwardDecision {
    FreshPublish,
    CarryForward,
    HardStale,
}

#[derive(Debug, Error)]
pub enum StalenessError {
    #[error("observed time cannot be earlier than publish time")]
    NegativeAge,
    #[error("hard stale window must be greater than or equal to soft stale window")]
    InvalidThresholds,
}

pub fn classify_staleness(
    publish_time: i64,
    observed_at: i64,
    soft_stale_seconds: i64,
    hard_stale_seconds: i64,
) -> Result<StalenessTier, StalenessError> {
    if hard_stale_seconds < soft_stale_seconds {
        return Err(StalenessError::InvalidThresholds);
    }
    if observed_at < publish_time {
        return Err(StalenessError::NegativeAge);
    }

    let age = observed_at - publish_time;
    if age > hard_stale_seconds {
        Ok(StalenessTier::HardStale)
    } else if age > soft_stale_seconds {
        Ok(StalenessTier::SoftStale)
    } else {
        Ok(StalenessTier::Fresh)
    }
}

pub fn carry_forward_decision(
    previous_publish_time: i64,
    candidate_publish_time: i64,
    observed_at: i64,
    hard_stale_seconds: i64,
) -> Result<CarryForwardDecision, StalenessError> {
    if candidate_publish_time > observed_at {
        return Err(StalenessError::NegativeAge);
    }

    if candidate_publish_time == previous_publish_time && previous_publish_time != 0 {
        let age = observed_at - candidate_publish_time;
        if age > hard_stale_seconds {
            Ok(CarryForwardDecision::HardStale)
        } else {
            Ok(CarryForwardDecision::CarryForward)
        }
    } else {
        Ok(CarryForwardDecision::FreshPublish)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weekend_carry_forward_is_allowed_inside_hard_window() {
        let friday = 1_712_018_400_i64;
        let saturday = friday + 86_400;

        let decision = carry_forward_decision(friday, friday, saturday, 3 * 86_400).unwrap();
        assert_eq!(decision, CarryForwardDecision::CarryForward);
    }

    #[test]
    fn hard_stale_is_triggered_after_deadline() {
        let publish_time = 1_712_018_400_i64;
        let observed_at = publish_time + (4 * 86_400);

        let tier = classify_staleness(publish_time, observed_at, 86_400, 3 * 86_400).unwrap();
        assert_eq!(tier, StalenessTier::HardStale);
    }
}
