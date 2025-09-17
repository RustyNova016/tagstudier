#[cfg(feature = "unstable")]
pub mod suggest_tag;
#[cfg(feature = "unstable")]
use governor::RateLimiter;
#[cfg(feature = "unstable")]
use governor::clock;
#[cfg(feature = "unstable")]
use governor::middleware::NoOpMiddleware;
#[cfg(feature = "unstable")]
use governor::state::InMemoryState;
#[cfg(feature = "unstable")]
use governor::state::NotKeyed;

#[cfg(feature = "unstable")]
pub mod api_framework;
pub mod cli_utils;
pub mod config;
#[cfg(feature = "unstable")]
pub mod pixiv;
#[cfg(feature = "unstable")]
pub mod tag_providers;

#[cfg(feature = "unstable")]
pub type DefaultRateLimiter =
    RateLimiter<NotKeyed, InMemoryState, clock::DefaultClock, NoOpMiddleware>;
