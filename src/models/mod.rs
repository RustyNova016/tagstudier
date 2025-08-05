use governor::RateLimiter;
use governor::clock;
use governor::middleware::NoOpMiddleware;
use governor::state::InMemoryState;
use governor::state::NotKeyed;

pub mod api_framework;
pub mod config;
pub mod pixiv;
pub mod tag_providers;

pub type DefaultRateLimiter =
    RateLimiter<NotKeyed, InMemoryState, clock::DefaultClock, NoOpMiddleware>;
