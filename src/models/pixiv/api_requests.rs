use std::fmt::Write;

use async_fn_stream::try_fn_stream;
use color_eyre::eyre::Context;
use futures::Stream;

use crate::ColEyreVal;
use crate::models::api_framework::api_request::ApiRequest;
use crate::models::pixiv::PixivProvider;
use crate::models::pixiv::api::bookmark_item::BookmarkItem;
use crate::models::pixiv::api::bookmark_response::BookmarkResponse;
use crate::models::pixiv::api::illust_body::IllustBody;
use crate::models::pixiv::api::illust_pages::IllustPage;
use crate::models::pixiv::api::pixiv_response::PixivResponse;
use crate::models::pixiv::http_clients::PIXIV_HTTP_CLIENT;

impl PixivProvider {
    pub fn bookmarks_api_request(
        user_id: u64,
        offset: u64,
        limit: u64,
        hidden: bool,
    ) -> ApiRequest {
        let mut url = format!(
            "https://www.pixiv.net/ajax/user/{user_id}/illusts/bookmarks?tag=&offset={offset}&limit={limit}&lang=en"
        );

        if hidden {
            write!(url, "&rest=hide").unwrap();
        } else {
            write!(url, "&rest=show").unwrap();
        }

        ApiRequest::new(url)
    }
    pub async fn fetch_bookmarks(
        user_id: u64,
        offset: u64,
        limit: u64,
        hidden: bool,
    ) -> Result<PixivResponse<BookmarkResponse>, crate::Error> {
        Self::bookmarks_api_request(user_id, offset, limit, hidden)
            .get(&PIXIV_HTTP_CLIENT)
            .await
    }

    pub fn stream_bookmarks(
        user_id: u64,
        hidden: bool,
    ) -> impl Stream<Item = ColEyreVal<BookmarkItem>> {
        try_fn_stream(async move |emitter| {
            let mut offset = 0;
            let mut total = 1;

            while offset < total {
                let res = Self::fetch_bookmarks(user_id, offset, 48, hidden).await?;
                let body = res.body().context("Reading response")?.as_object().unwrap();

                total = body.total;
                offset += 48;

                for item in &body.works {
                    emitter.emit(item.to_owned()).await;
                }
            }

            Ok(())
        })
    }

    pub async fn fetch_illust_pages(illust_id: u64) -> ColEyreVal<Vec<IllustPage>> {
        let url = format!("https://www.pixiv.net/ajax/illust/{illust_id}/pages?lang=en");

        ApiRequest::new(url)
            .get::<PixivResponse<IllustPage>>(&PIXIV_HTTP_CLIENT)
            .await
            .context("Fetching illust pages")
            .and_then(|res| res.body().cloned().context("Reading illust pages result"))
            .map(|res| res.as_array().unwrap().to_owned())
    }

    pub async fn fetch_illust(illust_id: u64) -> ColEyreVal<IllustBody> {
        let url = format!("https://www.pixiv.net/ajax/illust/{illust_id}");

        ApiRequest::new(url)
            .get::<PixivResponse<IllustBody>>(&PIXIV_HTTP_CLIENT)
            .await
            .context("Fetching illust")
            .and_then(|res| res.body().cloned().context("Reading illust result"))
            .map(|res| res.as_object().unwrap().to_owned())
    }
}
