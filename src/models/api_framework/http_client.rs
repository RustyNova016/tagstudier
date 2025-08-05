use std::sync::Arc;

use reqwest::Response;
use tracing::debug;

use crate::models::DefaultRateLimiter;
use crate::models::api_framework::api_request::ApiRequest;

pub struct HTTPClient {
    client: reqwest::Client,
    rate_limit: Arc<DefaultRateLimiter>,
}

impl HTTPClient {
    pub fn new(client: reqwest::Client, rate_limit: Arc<DefaultRateLimiter>) -> Arc<Self> {
        Arc::new(Self { client, rate_limit })
    }

    pub async fn get(&self, request: ApiRequest) -> Result<Response, crate::Error> {
        while request.tries < request.max_tries {
            self.rate_limit.until_ready().await;

            // Send the query
            let http_request = self.client.get(&request.url);

            debug!(
                "Sending api request `{}` (attempt: {})",
                request.url, request.tries
            );

            //TODO: Retries

            return Ok(http_request.send().await?);
        }

        todo!("Implement retries")
    }
}
