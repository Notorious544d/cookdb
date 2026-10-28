pub mod nutrition;

use anyhow::Context;
use cooklang::{Recipe};
use rusqlite::{Connection, Transaction, params};
use std::path::Path;
use crate::nutrition::{Enriched};

#[cfg(feature = "nutrition")]
use std::collections::HashMap;
#[cfg(feature = "nutrition")]
use crate::nutrition::EnrichedIngredients;

const SCHEMA: &str = include_str!("../schema.sql");
const SEED: &str = include_str!("../vocab.sql");

const CUISINE_EXISTS: &str = "SELECT 1 FROM cuisines WHERE name = ?1";
const TAG_EXISTS: &str = "SELECT 1 FROM tags WHERE name = ?1";

/// Rebuilds the database from every `.cook` file directly under `recipes_dir`.
pub fn build(recipes_dir: &Path, db_path: &Path, _enriched: Option<&Enriched>) -> anyhow::Result<usize> {
    let mut conn = create(db_path)?;
    let tx = conn.transaction()?;
    let mut count = 0;
    for path in cookdb_core::parse::recipe_files(recipes_dir)? {
        let slug = cookdb_core::parse::slug(&path)?;
        let recipe = cookdb_core::parse::load_recipe(&path)?;
        insert_recipe(&tx, slug, &recipe)
            .with_context(|| format!("inserting {}", path.display()))?;
        count += 1;
    }

    #[cfg(feature = "nutrition")]
    if let Some(e) = _enriched {
        insert_enriched_recipes(&tx, e)?;
        insert_enriched_ingredients(&tx, e)?;
    }

    tx.commit()?;
    Ok(count)
}

/// Creates a fresh database, replacing any existing file.
fn create(path: &Path) -> anyhow::Result<Connection> {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e).with_context(|| format!("removing {}", path.display())),
    }
    let conn = Connection::open(path).with_context(|| format!("opening {}", path.display()))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(SCHEMA).context("applying schema")?;
    conn.execute_batch(SEED).context("applying seed")?;
    Ok(conn)
}

fn ensure_known(
    tx: &Transaction,
    sql: &str,
    slug: &str,
    kind: &str,
    value: &str,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        tx.prepare_cached(sql)?.exists([value])?,
        "{slug}: unknown {kind} '{value}' (add it to vocab.sql)"
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

    // Tags from frontmatter
    for tag in m.tags().unwrap_or_default() {
        let tag = tag.trim();
        ensure_known(tx, TAG_EXISTS, slug, "tag", tag)?;
        tx.execute(
            "INSERT OR IGNORE INTO recipe_tags (slug, tag) VALUES (?1, ?2)",
            params![slug, tag],
        )?;
    }
    insert_data(tx, slug, recipe)?;
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

#[cfg(feature = "nutrition")]
fn insert_enriched_ingredients(
    tx: &Transaction,
    enriched_recipes: &HashMap<String, EnrichedIngredients>,
) -> anyhow::Result<()> {
    let mut insert = tx.prepare_cached(
        "INSERT OR IGNORE INTO recipe_ingredients (recipe_slug, ingredient_name)
         VALUES (?1, ?2)",
    )?;

    for (slug, enriched) in enriched_recipes {
        let database_slug: String = tx.query_row(
            "SELECT slug FROM recipes WHERE slug = ?1 COLLATE NOCASE",
            [slug],
            |row| row.get(0),
        )?;

        for ing in enriched {
            let name = ing.name.trim();
            if !name.is_empty() {
                insert.execute(params![database_slug, name])?;
            }
        }
    }

    Ok(())
}

#[cfg(feature = "nutrition")]
fn insert_enriched_recipes(
    tx: &Transaction,
    enriched_recipes: &HashMap<String, EnrichedIngredients>,
) -> anyhow::Result<()> {
    let mut insert =
        tx.prepare_cached("INSERT INTO recipe_enrichment (slug, json) VALUES (?1, ?2)")?;

    for (slug, enriched) in enriched_recipes {
        insert.execute(params![slug, serde_json::to_string(enriched)?])?;
    }

    Ok(())
}
