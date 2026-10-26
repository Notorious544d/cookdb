use anyhow::Context;
use cooklang::{CooklangParser, Recipe};
use rusqlite::{Connection, Transaction, params};
use std::path::Path;
use crate::nutrition::{Enriched};

#[cfg(feature = "nutrition")]
use std::collections::HashMap;
#[cfg(feature = "nutrition")]
use crate::nutrition::EnrichedIngredients;

const SCHEMA: &str = include_str!("../schema/schema.sql");
const SEED: &str = include_str!("../schema/vocab.sql");

const CUISINE_EXISTS: &str = "SELECT 1 FROM cuisines WHERE name = ?1";
const TAG_EXISTS: &str = "SELECT 1 FROM tags WHERE name = ?1";

/// Rebuilds the database from every `.cook` file directly under `recipes_dir`.
pub fn build(
    parser: &CooklangParser,
    recipes_dir: &Path,
    db_path: &Path,
    _enriched: Option<&Enriched>,
) -> anyhow::Result<usize> {
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

    #[cfg(feature = "nutrition")]
    if let Some(e) = _enriched {
        insert_enriched_tags(&tx, e)?;
        insert_enriched_recipes(&tx, e)?;
        insert_enriched_ingredients(&tx, e)?;
    }

    tx.commit()?;
    Ok(count)
}

pub fn load_recipe(parser: &CooklangParser, path: &Path) -> anyhow::Result<Recipe> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
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
    /* Tags fron frontmatter
    for tag in m.tags().unwrap_or_default() {
        let tag = tag.trim();
        ensure_known(tx, TAG_EXISTS, slug, "tag", tag)?;
        tx.execute(
            "INSERT OR IGNORE INTO recipe_tags (slug, tag) VALUES (?1, ?2)",
            params![slug, tag],
        )?;
    }
    */
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
fn insert_enriched_tags(
    tx: &Transaction,
    enriched_recipes: &HashMap<String, EnrichedIngredients>,
) -> anyhow::Result<()> {
    const TAGS_FOR_INGREDIENT: &str = "
        SELECT name FROM tags WHERE name = ?1
        UNION
        SELECT tag FROM ingredient_tag_map WHERE ingredient = ?1
    ";

    let mut find_tags = tx.prepare_cached(TAGS_FOR_INGREDIENT)?;
    let mut insert_tag =
        tx.prepare_cached("INSERT OR IGNORE INTO recipe_tags (slug, tag) VALUES (?1, ?2)")?;

    for (slug, enriched) in enriched_recipes {
        let database_slug: String = tx.query_row(
            "SELECT slug FROM recipes WHERE slug = ?1 COLLATE NOCASE",
            [slug],
            |row| row.get(0),
        )?;

    for ingredient_name in enriched
        .iter()
        .map(|ingredient| ingredient.name.as_str())
            {
                let tags = find_tags
                    .query_map([ingredient_name.trim()], |row| row.get::<_, String>(0))?
                    .collect::<rusqlite::Result<Vec<_>>>()?;

                for tag in tags {
                    insert_tag.execute(params![database_slug, tag])?;
                }
            }
        }

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
