use clap::Parser;

use crate::models::cli_utils::cli_data::CLI_DATA;
use crate::models::config::folders::FolderConfig;

/// Manage folders based on simple rules
#[derive(Parser, Debug, Clone)]
pub struct ManageFoldersCommand {}

impl ManageFoldersCommand {
    pub async fn run(&self) -> crate::ColEyre {
        let lib = CLI_DATA.read().await.get_library().await?;

        let auto = FolderConfig::load(&lib)?;
        auto.apply_folder_rules(&lib).await?;

        Ok(())
    }
}
