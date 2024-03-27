use futures_util::future::join_all;
use leaky_bucket::RateLimiter;
use std::sync::Arc;

use crate::{tiers::Tier, Consumer, State};

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

pub async fn limiter(state: Arc<State>, consumer: &Consumer) {
    let tiers = state.tiers.read().await.clone();
    let tier = tiers.get(&consumer.tier).unwrap();

    if !has_limiter(&state, consumer).await {
        add_limiter(&state, consumer, tier).await;
    }

    let rate_limiter_map = state.limiter.read().await.clone();
    let rates = rate_limiter_map.get(&consumer.key).unwrap();

    join_all(rates.iter().map(|r| async { r.acquire_one().await })).await;
}
