use clap::Args;
use rusqlite::{Connection, OpenFlags, types::ValueRef};
use std::path::PathBuf;

#[derive(Args)]
pub struct QueryArgs {
    /// SQL to run
    pub sql: String,
    /// Path to the database
    #[arg(long, default_value = "static/recipes.db")]
    pub db: PathBuf,
}

pub fn run(args: QueryArgs) -> anyhow::Result<()> {
    let conn = Connection::open_with_flags(&args.db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut stmt = conn.prepare(&args.sql)?;
    let cols = stmt.column_count();
    println!("{}", stmt.column_names().join("\t"));

    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let cells = (0..cols)
            .map(|i| {
                Ok(match row.get_ref(i)? {
                    ValueRef::Null => "NULL".to_string(),
                    ValueRef::Integer(v) => v.to_string(),
                    ValueRef::Real(v) => v.to_string(),
                    ValueRef::Text(v) => String::from_utf8_lossy(v).into_owned(),
                    ValueRef::Blob(v) => format!("<blob {} bytes>", v.len()),
                })
            })
            .collect::<rusqlite::Result<Vec<_>>>()?;
        println!("{}", cells.join("\t"));
    }
    Ok(())
}