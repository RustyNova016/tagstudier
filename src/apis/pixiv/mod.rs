use reqwest::Client;
use reqwest::header;
use tracing::debug;

use crate::apis::pixiv::models::IllustResponse;

pub mod models;
pub mod tag_on_pixiv;

pub async fn fetch_illust_data(illust_id: &str) -> IllustResponse {
    // TODO: Ratelimit
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
    let text = client.get(url).send().await.unwrap().text().await.unwrap();

    serde_json::from_str(&text).unwrap()
}
