use std::fs::File;
use std::io::Read;

use color_eyre::eyre::Context;
use serde::Deserialize;
use serde::Serialize;
use spire_enum::prelude::delegated_enum;
use tagstudio_db::Library;

use crate::ColEyre;

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct LintsConfig {
    pub lint: Vec<EntryLintConfig>,
}

impl LintsConfig {
    pub fn load(lib: &Library) -> ColEyre<Self> {
        let path = lib.path.join(".TagStudio/tsr_lints.toml");
        let mut config =
            File::open(path).context("Couldn't open the lint config file. Make sure it exists")?;
        let mut data = String::new();
        config
            .read_to_string(&mut data)
            .context("Couldn't read the lint config file")?;
        toml::from_str(&data).context("Couldn't parse the lint config file")
    }
}

#[delegated_enum(extract_variants(derive(Debug, Serialize, Deserialize)))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "lint_type")]
pub(super) enum EntryLintConfig {
    YesNoLint {
        question: String,

        #[serde(default)]
        tag_blacklist_any: Vec<String>,
        #[serde(default)]
        tag_whitelist_any: Vec<String>,

        #[serde(default)]
        add_yes_tags: Vec<String>,
        #[serde(default)]
        add_no_tags: Vec<String>,
    },
}
