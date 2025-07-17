pub mod mv;
use clap::Parser;
use clap::Subcommand;
use clap_verbosity_flag::InfoLevel;
use clap_verbosity_flag::Verbosity;

use crate::cli::mv::MVCommand;

/// Tools for TagStudio
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long, hide = true)]
    pub markdown_help: bool,

    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    // If provided, outputs the completion file for given shell
    // #[arg(long = "generate", value_enum)]
    // generator: Option<Shell>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl Cli {
    pub async fn run(&self) {
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

        if let Some(command) = &self.command {
            command.run().await;
        }

        
    }

    // fn print_completions<G: Generator>(gene: G, cmd: &mut Command) {
    //     generate(gene, cmd, cmd.get_name().to_string(), &mut io::stdout());
    // }
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    MV(MVCommand)
}

impl Commands {
    pub async fn run(&self)  {
        match self {
            Self::MV(val) => val.run().await,
        }
    }
}