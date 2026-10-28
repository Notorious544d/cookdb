use clap::Args;
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;
use crate::context::Context;

const SEARCH_SQL: &str = include_str!("../static/search.sql");

#[derive(Args)]
pub struct SearchArgs {
    #[arg(required = true, num_args = 1..)]
    terms: Vec<String>,
    /// Print slugs instead of recipe names
    #[arg(long)]
    slug: bool,
    #[arg(long)]
    db: Option<PathBuf>,
}

pub fn run(ctx: &Context, args: SearchArgs) -> anyhow::Result<()> {
    let db = args.db.unwrap_or_else(|| ctx.db_path());
    let conn = Connection::open_with_flags(&db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let pattern = format!("%{}%", args.terms.join(" "));

    let mut stmt = conn.prepare(SEARCH_SQL)?;
    let hits: Vec<(String, String)> = stmt
        .query_map([pattern], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;

    for (slug, title) in &hits {
        if args.slug {
            println!("\"{slug}\"");
        } else {
            println!("{title}");
        }
    }
    Ok(())
}