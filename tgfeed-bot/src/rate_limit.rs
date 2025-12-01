use std::num::NonZeroU32;
use std::time::Duration;

use governor::clock::DefaultClock;
use governor::state::keyed::DashMapStateStore;
use governor::{Quota, RateLimiter};

pub type KeyedRateLimiter = RateLimiter<i64, DashMapStateStore<i64>, DefaultClock>;

pub struct RateLimiters {
    pub commands: KeyedRateLimiter,
    pub summarize: KeyedRateLimiter,
}

impl RateLimiters {
    pub fn new() -> Self {
        Self {
            commands: RateLimiter::keyed(
                Quota::with_period(Duration::from_secs(1))
                    .unwrap()
                    .allow_burst(NonZeroU32::new(1).unwrap()),
            ),
            summarize: RateLimiter::keyed(
                Quota::with_period(Duration::from_secs(3600))
                    .unwrap()
                    .allow_burst(NonZeroU32::new(1).unwrap()),
            ),
        }
    }
}
