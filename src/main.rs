mod auth;
mod cli;
mod commands;
mod config;
mod output;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        output::error(&e.to_string());
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { action } => commands::auth::run(action).await?,
    }

    Ok(())
}
