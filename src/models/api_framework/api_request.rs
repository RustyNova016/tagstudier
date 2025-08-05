use serde::de::DeserializeOwned;

use crate::models::api_framework::http_client::HTTPClient;

/// A raw API request, used to send custom requests to the API
pub struct ApiRequest {
    /// The url to fetch
    pub url: String,

    /// The current number of times the request has been tried
    pub tries: u32,

    /// The maximum number of tries of a request
    pub max_tries: u32,

    /// The priority of the request. 0 is the most important
    pub priority: u32,
}

impl ApiRequest {
    pub fn new(url: String) -> Self {
        Self {
            url,
            tries: 0,
            priority: 100,
            max_tries: 10,
        }
    }

    /// Sends a get request to the api. Return a [serde_json::Value]
    pub async fn get_json(self, client: &HTTPClient) -> Result<serde_json::Value, crate::Error> {
        Ok(client.get(self).await?.json::<serde_json::Value>().await?)
    }

    /// Parse a [serde_json::Value] into Musicbrainz structs
    pub fn parse_json<T>(json: serde_json::Value, _url: &str) -> Result<T, crate::Error>
    where
        T: DeserializeOwned,
    {
        // Try to deserialize as our result
        let err = match serde_json::from_value::<T>(json.clone()) {
            Ok(result) => return Ok(result),
            Err(err) => err,
        };

        // it's a problem with out models. Let's send the serde error
        Err(err.into())
    }

    /// Send the request as a get, deal with ratelimits, and retries
    pub async fn get<T>(self, client: &HTTPClient) -> Result<T, crate::Error>
    where
        T: DeserializeOwned,
    {
        let url = self.url.to_string();
        let json = self.get_json(client).await?;
        Self::parse_json(json, &url)
    }
}
