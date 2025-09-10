use std::fs::File;
use std::io::Read as _;

use color_eyre::eyre::Context as _;
use serde::Deserialize;
use serde::Serialize;
use tagstudio_db::Library;
use tracing::instrument;

use crate::ColEyre;
use crate::ColEyreVal;
use crate::models::config::folders::folder_rule::FolderRule;
use crate::pg_counted;
use crate::pg_inc;

pub mod folder_rule;

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderConfig {
    #[serde(alias = "folder")]
    pub folders: Vec<FolderRule>,
}

impl FolderConfig {
    pub fn load(lib: &Library) -> ColEyreVal<Self> {
        let path = lib.path.join(".TagStudio/TSR_folder_rules.toml");
        let mut config = File::open(path)
            .context("Couldn't open the autosort config file. Make sure it exists")?;
        let mut data = String::new();
        config
            .read_to_string(&mut data)
            .context("Couldn't read the autosort config file")?;
        toml::from_str(&data).context("Couldn't parse the autosort config file")
    }

    #[instrument(skip(lib), fields(indicatif.pb_show = tracing::field::Empty))]
    pub async fn apply_folder_rules(&self, lib: &Library) -> ColEyre {
        pg_counted!(2, "Processing folder rules");
        self.apply_folder_tagging(lib).await?;
        pg_inc!();
        self.apply_folder_sorting(lib).await?;
        pg_inc!();
        Ok(())
    }

    #[instrument(skip(lib), fields(indicatif.pb_show = tracing::field::Empty))]
    async fn apply_folder_sorting(&self, lib: &Library) -> ColEyre {
        pg_counted!(self.folders.len(), "Processing entry sorting rules");

        let mut black_list = Vec::new();
        for rule in &self.folders {
            let processed = rule.sort_entries(lib, black_list.clone()).await?;
            black_list.extend(processed);
            pg_inc!();
        }

        Ok(())
    }

    #[instrument(skip(lib), fields(indicatif.pb_show = tracing::field::Empty))]
    async fn apply_folder_tagging(&self, lib: &Library) -> ColEyre {
        pg_counted!(self.folders.len(), "Processing entry tagging rules");

        for rule in &self.folders {
            rule.tag_entries(lib).await?;
            pg_inc!();
        }

        Ok(())
    }
}
