#[cfg(feature = "unstable")]
pub mod download;
#[cfg(feature = "unstable")]
pub mod download_bookmarks;
#[cfg(feature = "unstable")]
pub mod link_urls;
pub mod manage_folders;
pub mod merge_tags;
pub mod mv;
#[cfg(feature = "unstable")]
pub mod rename_tag;
#[cfg(feature = "unstable")]
pub mod tag_import;
use clap::Parser;
use clap::Subcommand;
use clap_verbosity_flag::InfoLevel;
use clap_verbosity_flag::Verbosity;

#[cfg(feature = "unstable")]
use crate::cli::autosort::AutosortCommand;
#[cfg(feature = "unstable")]
use crate::cli::download::DownloadCommand;
#[cfg(feature = "unstable")]
use crate::cli::download_bookmarks::DownloadBookmarksCommand;
#[cfg(feature = "unstable")]
use crate::cli::link_urls::LinkUrlsCommand;
use crate::cli::manage_folders::ManageFoldersCommand;
use crate::cli::merge_tags::MergeTagCommand;
use crate::cli::mv::MVCommand;
#[cfg(feature = "unstable")]
use crate::cli::rename_tag::RenameTagCommand;
#[cfg(feature = "unstable")]
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
        if self.markdown_help {
            clap_markdown::print_help_markdown::<Self>();
            return Ok(());
        }

        // if let Some(generator) = self.generator {
        //     let mut cmd = Self::command();
        //     Self::print_completions(generator, &mut cmd);
        //     return Ok(false);
        // }

        self.load_cli_data().await;

        if let Some(command) = &self.command {
            command.run().await?;
        }

        Ok(())
    }

    async fn load_cli_data(&self) {
        let mut data = CLI_DATA.write().await;

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
    #[cfg(feature = "unstable")]
    Download(DownloadCommand),
    #[cfg(feature = "unstable")]
    DownloadBookmarks(DownloadBookmarksCommand),
    #[cfg(feature = "unstable")]
    LinkUrls(LinkUrlsCommand),
    ManageFolders(ManageFoldersCommand),
    MergeTags(MergeTagCommand),
    MV(MVCommand),
    #[cfg(feature = "unstable")]
    RenameTag(RenameTagCommand),
    #[cfg(feature = "unstable")]
    TagImport(TagImportCommand),
}

impl Commands {
    pub async fn run(&self) -> crate::ColEyre {
        match self {
            #[cfg(feature = "unstable")]
            Self::Download(val) => val.run().await?,
            #[cfg(feature = "unstable")]
            Self::DownloadBookmarks(val) => val.run().await?,
            #[cfg(feature = "unstable")]
            Self::LinkUrls(val) => val.run().await,
            Self::ManageFolders(val) => val.run().await?,
            Self::MergeTags(val) => val.run().await?,
            Self::MV(val) => val.run().await?,
            #[cfg(feature = "unstable")]
            Self::RenameTag(val) => val.run().await,
            #[cfg(feature = "unstable")]
            Self::TagImport(val) => val.run().await?,
        }

        Ok(())
    }
}
