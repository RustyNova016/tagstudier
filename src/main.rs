pub mod apis;
pub(crate) mod cli;
pub(crate) mod error;

use clap::Parser as _;

use crate::cli::Cli;
pub(crate) use crate::error::Error;
use crate::interface::tracing::init_tracer;

pub mod interface;
pub mod utils;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let cli = Cli::parse();
    let _worker_guard = init_tracer(&cli);

    cli.run().await;
}
