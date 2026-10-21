use std::path::Path;

use anyhow::Context;
use cooklang::{CooklangParser, Recipe};
use rusqlite::{params, Connection, Transaction};

const SCHEMA: &str = include_str!("../schema/schema.sql");
const SEED: &str = include_str!("../schema/vocab.sql");

const CUISINE_EXISTS: &str = "SELECT 1 FROM cuisines WHERE name = ?1";
const TAG_EXISTS: &str = "SELECT 1 FROM tags WHERE name = ?1";

pub fn load_recipe(parser: &CooklangParser, path: &Path) -> anyhow::Result<Recipe> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let (recipe, warnings) = parser
        .parse(&text)
        .into_result()
        .map_err(|e| anyhow::anyhow!("parsing {}: {e}", path.display()))?;
    for w in warnings.iter() {
        eprintln!("warning: {}: {w}", path.display());
    }
    Ok(recipe)
}

/// Creates a fresh database, replacing any existing file.
fn create(path: &Path) -> anyhow::Result<Connection> {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e).with_context(|| format!("removing {}", path.display())),
    }
    let conn = Connection::open(path)
        .with_context(|| format!("opening {}", path.display()))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(SCHEMA).context("applying schema")?;
    conn.execute_batch(SEED).context("applying seed")?;
    Ok(conn)
}

fn ensure_known(tx: &Transaction, sql: &str, slug: &str, kind: &str, value: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        tx.prepare_cached(sql)?.exists([value])?,
        "{slug}: unknown {kind} '{value}' (add it to schema/seed.sql)"
    );
    Ok(())
}

fn insert_recipe(tx: &Transaction, slug: &str, recipe: &Recipe) -> anyhow::Result<()> {
    let m = &recipe.metadata;
    let cuisine = m.get("cuisine").and_then(|v| v.as_str());
    if let Some(c) = cuisine {
        ensure_known(tx, CUISINE_EXISTS, slug, "cuisine", c)?;
    }
    tx.execute(
        "INSERT INTO recipes (slug, title, cuisine) VALUES (?1, ?2, ?3)",
        params![slug, m.title(), cuisine],
    )?;
    for tag in m.tags().unwrap_or_default() {
        let tag = tag.trim();
        ensure_known(tx, TAG_EXISTS, slug, "tag", tag)?;
        tx.execute(
            "INSERT OR IGNORE INTO recipe_tags (slug, tag) VALUES (?1, ?2)",
            params![slug, tag],
        )?;
    }
    insert_ingredients(tx, slug, recipe)?;
    insert_data(tx, slug, recipe)?;
    Ok(())
}

fn insert_ingredients(tx: &Transaction, slug: &str, recipe: &Recipe) -> anyhow::Result<()> {
    for (i, ing) in recipe.ingredients.iter().enumerate() {
        tx.execute(
            "INSERT INTO recipe_ingredients (slug, position, name) VALUES (?1, ?2, ?3)",
            params![slug, i as i64, ing.name],
        )?;
    }
    Ok(())
}

fn insert_data(tx: &Transaction, slug: &str, recipe: &Recipe) -> anyhow::Result<()> {
    let json = serde_json::to_string(recipe).context("serialising recipe")?;
    tx.execute(
        "INSERT INTO recipe_data (slug, json) VALUES (?1, ?2)",
        params![slug, json],
    )?;
    Ok(())
}

/// Rebuilds the database from every `.cook` file directly under `recipes_dir`.
pub fn build(parser: &CooklangParser, recipes_dir: &Path, db_path: &Path) -> anyhow::Result<usize> {
    let mut conn = create(db_path)?;
    let tx = conn.transaction()?;
    let mut count = 0;
    for entry in std::fs::read_dir(recipes_dir)
        .with_context(|| format!("reading {}", recipes_dir.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("cook") {
            continue;
        }
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .context("non-UTF-8 file name")?;
        let recipe = load_recipe(parser, &path)?;
        insert_recipe(&tx, slug, &recipe)
            .with_context(|| format!("inserting {}", path.display()))?;
        count += 1;
    }
    tx.commit()?;
    Ok(count)
}