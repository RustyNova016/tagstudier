pub mod autosort;
pub mod download;
pub mod download_bookmarks;
pub mod link_urls;
pub mod merge_tags;
pub mod mv;
pub mod rename_tag;
pub mod tag_import;
use clap::Parser;
use clap::Subcommand;
use clap_verbosity_flag::InfoLevel;
use clap_verbosity_flag::Verbosity;

use crate::cli::autosort::AutosortCommand;
use crate::cli::download::DownloadCommand;
use crate::cli::download_bookmarks::DownloadBookmarksCommand;
use crate::cli::link_urls::LinkUrlsCommand;
use crate::cli::merge_tags::MergeTagCommand;
use crate::cli::mv::MVCommand;
use crate::cli::rename_tag::RenameTagCommand;
use crate::cli::tag_import::TagImportCommand;
use crate::models::cli_utils::cli_data::CLI_DATA;

/// Tools for TagStudio
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long, hide = true)]
    pub markdown_help: bool,

    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    /// The path to the TagStudio library. If left blank, it will try to find a library folder in the parent folder recursively
    #[clap(short, long)]
    pub library: Option<String>,

    // If provided, outputs the completion file for given shell
    // #[arg(long = "generate", value_enum)]
    // generator: Option<Shell>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl Cli {
    pub async fn run(&self) -> crate::ColEyre {
        // Invoked as: `$ my-app --markdown-help`
        // if self.markdown_help {
        //     clap_markdown::print_help_markdown::<Self>();
        //     return Ok(false);
        // }

        // if let Some(generator) = self.generator {
        //     let mut cmd = Self::command();
        //     Self::print_completions(generator, &mut cmd);
        //     return Ok(false);
        // }

        self.load_cli_data();

        if let Some(command) = &self.command {
            command.run().await?;
        }

        Ok(())
    }

    fn load_cli_data(&self) {
        let mut data = CLI_DATA.write().unwrap();

        if let Some(lib) = &self.library {
            data.set_lib_path(lib.to_owned());
        }
    }

    // fn print_completions<G: Generator>(gene: G, cmd: &mut Command) {
    //     generate(gene, cmd, cmd.get_name().to_string(), &mut io::stdout());
    // }
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Autosort(AutosortCommand),
    Download(DownloadCommand),
    DownloadBookmarks(DownloadBookmarksCommand),
    LinkUrls(LinkUrlsCommand),
    MergeTags(MergeTagCommand),
    MV(MVCommand),
    RenameTag(RenameTagCommand),
    TagImport(TagImportCommand),
}

impl Commands {
    pub async fn run(&self) -> crate::ColEyre {
        match self {
            Self::Autosort(val) => val.run().await?,
            Self::Download(val) => val.run().await?,
            Self::DownloadBookmarks(val) => val.run().await?,
            Self::LinkUrls(val) => val.run().await,
            Self::MergeTags(val) => val.run().await,
            Self::MV(val) => val.run().await?,
            Self::RenameTag(val) => val.run().await,
            Self::TagImport(val) => val.run().await?,
        }

        Ok(())
    }
}
