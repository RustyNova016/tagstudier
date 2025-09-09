pub mod folders;
#[cfg(feature = "unstable")]
use serde::Deserialize;
#[cfg(feature = "unstable")]
use serde::Serialize;

#[cfg(feature = "unstable")]
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub pixiv_cookies: String,
}
