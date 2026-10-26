mod build;
mod query;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cookdb", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build the recipe database
    Build(build::BuildArgs),
    /// Run a read-only SQL query against the database
    Query(query::QueryArgs),
}

pub fn run() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Build(args) => build::run(args),
        Command::Query(args) => query::run(args),
    }
}