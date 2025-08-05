use std::env::current_dir;

use clap::Parser;
use regex::Regex;
use streamies::TryStreamies as _;
use tagstudio_db::models::entry::Entry;
use tagstudio_db::models::library::Library;
use tagstudio_db::models::text_field::TextField;
use tracing::info;

/// Add links to images based on their filename
#[derive(Parser, Debug, Clone)]
pub struct LinkUrlsCommand {
    /// Do not make actual changes, and just print them out
    #[clap(short, long)]
    pub dry: bool,
}

impl LinkUrlsCommand {
    pub async fn run(&self) {
        let current_dir = current_dir().expect("Couldn't get current working directory");
        let lib = Library::try_new(current_dir.clone()).expect("Couldn't get the root library");
        let conn = &mut *lib
            .db
            .get()
            .await
            .expect("Couldn't open a new connection to the library database");

        let entries = Entry::stream_entries(conn)
            .try_collect_vec()
            .await
            .expect("Couldn't get the entries");
        for entry in entries {
            // Check if there's already an url
            let fields = entry
                .get_text_fields(conn)
                .await
                .expect("Couldn't get entry feilds");
            if fields.iter().any(|field| field.type_key == "URL") {
                continue;
            }

            let name = &entry.filename;

            let url = None.or_else(|| parse_pixiv_mobile(name));

            if let Some(url) = url {
                info!("Adding url `{url}` for `{}`", entry.filename);
                if !self.dry {
                    TextField::insert_text_field(conn, entry.id, "URL", &url)
                        .await
                        .expect("Couldn't save url field");
                }
            }
        }
    }
}

/// Parse the filename of a pixiv pic, downloaded on the mobile app, and return the url if found
///
/// For reference, the filename is constructed like so: illust_{id}_{Date Saved}_{Hour Saved}
fn parse_pixiv_mobile(name: &str) -> Option<String> {
    let regex = Regex::new(r"(?m)illust_([0-9]+)_[0-9]{8}_[0-9]{6}").unwrap();

    let result = regex.captures(name)?;

    let id = result.get(1)?;

    Some(format!("https://www.pixiv.net/en/artworks/{}", id.as_str()))
}

#[cfg(test)]
mod tests {
    use crate::cli::link_urls::parse_pixiv_mobile;

    #[test]
    fn test_parse_pixiv_mobile_valid() {
        assert_eq!(
            parse_pixiv_mobile("illust_12345678_20240601_123456.jpg"),
            Some("https://www.pixiv.net/en/artwork/12345678".to_string())
        );
    }

    #[test]
    fn test_parse_pixiv_mobile_invalid() {
        assert_eq!(
            parse_pixiv_mobile("image_12345678_20240601_123456.jpg"),
            None
        );
    }
}
