use std::backtrace::Backtrace;

use regex::Regex;
use snafu::OptionExt;
use snafu::Snafu;
use tagstudio_db::Entry;

/// An entry from the database for a pixiv illust page
pub struct PixivEntry {
    entry: Entry,

    pixiv_illust_id: i64,
    illust_page: i64,
}

impl PixivEntry {
    pub fn try_from_entry(entry: Entry) -> Result<Self, TryFromEntryError> {
        let regex = Regex::new(r"(?m)illust_([0-9]+)_p([0-9]+)").unwrap();

        let filename = entry.get_filename().unwrap();
        let filename = filename.to_string_lossy();
        let result = regex.captures(&filename).context(FilenameParseSnafu {
            filename: filename.to_string(),
        })?;

        let id: i64 = result.get(1).unwrap().as_str().parse().unwrap();
        let page: i64 = result.get(2).unwrap().as_str().parse().unwrap();

        Ok(Self {
            entry: entry,
            illust_page: page,
            pixiv_illust_id: id,
        })
    }

    pub fn entry(&self) -> &Entry {
        &self.entry
    }
}

#[derive(Debug, Snafu)]
pub enum TryFromEntryError {
    #[snafu(display("Couldn't parse filename `{filename}`"))]
    FilenameParse {
        filename: String,
        backtrace: Backtrace,
    },
}
