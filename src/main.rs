mod build_db;
mod cli;
mod nutrition;

fn main() -> anyhow::Result<()> {
    cli::run()
}