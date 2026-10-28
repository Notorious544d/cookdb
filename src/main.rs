mod context;
mod build;
mod query;
mod recipe;
mod search;

use clap::{Parser, Subcommand};
use context::Context;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cookdb", version, about)]
struct Cli {
    /// Recipe repository root
    #[arg(long, global = true, env = "COOKDB_BASE_PATH", default_value = ".")]
    base_path: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build the recipe database
    Build,
    /// Run a read-only SQL query against the database
    Query(query::QueryArgs),
    /// Parse a recipe and print it in another format
    Recipe(recipe::RecipeArgs),
    /// Search for recipes
    Search(search::SearchArgs),
}

pub fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let ctx = Context::new(&cli.base_path)?;
    match cli.command {
        Command::Build => build::run(&ctx),
        Command::Query(args) => query::run(&ctx, args),
        Command::Recipe(args) => recipe::run(&ctx, args),
        Command::Search(args) => search::run(&ctx, args),
    }
}