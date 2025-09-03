use clap::Parser;

use crate::models::cli_utils::cli_data::CLI_DATA;
use crate::models::config::auto_sort::AutosortRules;

/// Add links to images based on their filename
#[derive(Parser, Debug, Clone)]
pub struct AutosortCommand {}

impl AutosortCommand {
    pub async fn run(&self) -> crate::ColEyre {
        let lib = CLI_DATA.read().await.get_library().await?;

        let auto = AutosortRules::load(&lib)?;
        auto.apply(&lib).await?;

        Ok(())
    }
}
