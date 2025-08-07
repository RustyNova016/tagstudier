use core::num::NonZeroU32;
use core::ops::Deref;
use std::env::current_dir;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::LazyLock;

use color_eyre::eyre::Context as _;
use governor::Quota;
use governor::RateLimiter;
use governor::clock;
use governor::middleware::NoOpMiddleware;
use governor::state::InMemoryState;
use governor::state::NotKeyed;
use reqwest::Client;
use reqwest::Url;
use reqwest::cookie::Jar;
use reqwest::header;
use tagstudio_db::Library;

use crate::models::api_framework::http_client::HTTPClient;
use crate::models::config::Config;

pub static PIXIV_CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let current_dir = current_dir()
        .context("Couldn't get current working directory")
        .unwrap();
    let lib = Library::try_new(current_dir.clone())
        .context("Couldn't get the root library")
        .unwrap();
    let file_path = lib.path.join(".TagStudio/tagerine_config.json");
    let file = File::open(file_path).expect("Couldn't read the configuration file");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).expect("Couldn't parse the configuration file. Make sure it is correct")
});

pub static PIXIV_REQWEST_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let mut headers = header::HeaderMap::new();

    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_str("application/json").unwrap(),
    );
    headers.insert(
        header::ACCEPT_LANGUAGE,
        header::HeaderValue::from_str("en-US,en;q=0.5").unwrap(),
    );
    headers.insert(
        header::HOST,
        header::HeaderValue::from_str("www.pixiv.net").unwrap(),
    );
    headers.insert(
        header::REFERER,
        header::HeaderValue::from_str("https://www.pixiv.net/").unwrap(),
    );
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_str(
            "Mozilla/5.0 (X11; Linux x86_64; rv:141.0) Gecko/20100101 Firefox/141.0",
        )
        .unwrap(),
    );

    Client::builder()      // see : https://github.com/hyperium/hyper/issues/2136
            .pool_max_idle_per_host(0)
            .default_headers(headers)
            .cookie_provider(Arc::new(get_cookies()))
            .build().expect("Unable to set default user agent, the following values must be set in Cargo.toml : 'name', 'version', 'authors'")
});

pub static PIXIV_HTTP_CLIENT: LazyLock<Arc<HTTPClient>> = LazyLock::new(|| {
    HTTPClient::new(
        PIXIV_REQWEST_CLIENT.deref().clone(),
        PIXIV_RATE_LIMIT.clone(),
    )
});

pub static IPXIM_HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let mut headers = header::HeaderMap::new();

    headers.insert(
        header::REFERER,
        header::HeaderValue::from_str("https://www.pixiv.net/").unwrap(),
    );

    Client::builder()      // see : https://github.com/hyperium/hyper/issues/2136
            .pool_max_idle_per_host(0)
            .default_headers(headers)
            .cookie_provider(Arc::new(get_cookies()))
            .build().expect("Unable to set default user agent, the following values must be set in Cargo.toml : 'name', 'version', 'authors'")
});

fn get_cookies() -> Jar {
    let jar = Jar::default();

    let full_cookie = &PIXIV_CONFIG.pixiv_cookies;

    for cookie in full_cookie.split("; ") {
        jar.add_cookie_str(cookie, &"https://www.pixiv.net/".parse::<Url>().unwrap());
    }

    jar
}

pub static PIXIV_RATE_LIMIT: LazyLock<
    Arc<RateLimiter<NotKeyed, InMemoryState, clock::DefaultClock, NoOpMiddleware>>,
> = LazyLock::new(|| {
    let quota =
        Quota::per_minute(NonZeroU32::new(12).unwrap()).allow_burst(NonZeroU32::new(1).unwrap());
    Arc::new(RateLimiter::direct(quota))
});
