use tagstudio_db::sqlx::SqliteConnection;

use crate::apis::pixiv::fetch_illust_data;

pub struct PixivTagProvider {

}

impl PixivTagProvider {
    pub fn parse_pixiv_id(url: &str) ->  &str {
        url.split("/").last().expect("TODO")
    }

    pub fn load_data(conn: &mut SqliteConnection, url: &str) {
        let data = fetch_illust_data(Self::parse_pixiv_id(url)).await;
        
    }
}