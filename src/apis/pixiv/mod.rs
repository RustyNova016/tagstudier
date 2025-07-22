use std::sync::LazyLock;
use std::num::NonZeroU32;
use governor::clock;
use governor::middleware::NoOpMiddleware;
use governor::state::InMemoryState;
use governor::state::NotKeyed;
use governor::Quota;

use governor::RateLimiter;
use reqwest::Client;
use reqwest::header;
use tracing::debug;

use crate::apis::pixiv::models::IllustResponse;

pub mod models;
pub mod tag_on_pixiv;

pub static RATE_LIMIT: LazyLock<RateLimiter<NotKeyed, InMemoryState, clock::DefaultClock, NoOpMiddleware>> = LazyLock::new(|| {
            let quota =Quota::per_minute(NonZeroU32::new(12).unwrap()).allow_burst(NonZeroU32::new(1).unwrap());
            RateLimiter::direct(quota)
});

pub async fn fetch_illust_data(illust_id: &str) -> IllustResponse {
    // TODO: Error checking

    let mut headers = header::HeaderMap::new();

    headers.insert(
        header::ACCEPT_LANGUAGE,
        header::HeaderValue::from_str("en-US,en;q=0.5").unwrap(),
    );

    let client = Client::builder()      // see : https://github.com/hyperium/hyper/issues/2136
            .pool_max_idle_per_host(0)
            .default_headers(headers)
            .build().expect("Unable to set default user agent, the following values must be set in Cargo.toml : 'name', 'version', 'authors'");

    let url = format!("https://www.pixiv.net/ajax/illust/{illust_id}");
    debug!("Requesting post: {url}");
    RATE_LIMIT.until_ready().await;
    let text = client.get(url).send().await.unwrap().text().await.unwrap();

    serde_json::from_str(&text).unwrap()
}
