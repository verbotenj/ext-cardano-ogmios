use futures_util::future::join_all;
use leaky_bucket::RateLimiter;
use std::sync::Arc;
use std::{error::Error, fmt::Display};

use crate::{tiers::Tier, Consumer, State};

#[derive(Debug)]
pub enum LimiterError {
    PortDeleted,
    InvalidTier,
}
impl Display for LimiterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LimiterError::PortDeleted => f.write_str("Port was deleted"),
            LimiterError::InvalidTier => f.write_str("Tier is invalid"),
        }
    }
}
impl Error for LimiterError {}

async fn has_limiter(state: &State, consumer: &Consumer) -> bool {
    let rate_limiter_map = state.limiter.read().await;
    rate_limiter_map.get(&consumer.key).is_some()
}

async fn add_limiter(state: &State, consumer: &Consumer, tier: &Tier) {
    let rates = tier
        .rates
        .iter()
        .map(|r| {
            Arc::new(
                RateLimiter::builder()
                    .initial(r.limit)
                    .interval(r.interval)
                    .refill(r.limit)
                    .build(),
            )
        })
        .collect();

    state
        .limiter
        .write()
        .await
        .insert(consumer.key.clone(), rates);
}

pub async fn limiter(state: Arc<State>, consumer: &Consumer) -> Result<(), LimiterError> {
    if !has_limiter(&state, consumer).await {
        let consumers = state.consumers.read().await.clone();
        let refreshed_consumer = match consumers.get(&consumer.key) {
            Some(consumer) => consumer,
            None => return Err(LimiterError::PortDeleted),
        };
        let tiers = state.tiers.read().await.clone();
        let tier = match tiers.get(&refreshed_consumer.tier) {
            Some(tier) => tier,
            None => return Err(LimiterError::InvalidTier),
        };
        add_limiter(&state, refreshed_consumer, tier).await;
    }

    let rate_limiter_map = state.limiter.read().await.clone();
    let rates = rate_limiter_map.get(&consumer.key).unwrap();

    join_all(rates.iter().map(|r| async { r.acquire_one().await })).await;
    Ok(())
}
