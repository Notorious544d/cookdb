use anyhow::Context;
use cooklang::{Converter, Recipe};
use rusqlite::{Connection, OptionalExtension};

/// The JSON stored at build time, untouched. `None` if the slug doesn't exist.
pub fn fetch_json(conn: &Connection, slug: &str) -> anyhow::Result<Option<String>> {
    conn.query_row(
        "SELECT json FROM recipe_data WHERE slug = ?1",
        [slug],
        |row| row.get(0),
    )
    .optional()
    .with_context(|| format!("reading recipe_data for '{slug}'"))
}

/// The recipe at its base scale. `None` if the slug doesn't exist.
pub fn fetch_recipe(conn: &Connection, slug: &str) -> anyhow::Result<Option<Recipe>> {
    fetch_json(conn, slug)?
        .map(|json| serde_json::from_str(&json).with_context(|| format!("decoding '{slug}'")))
        .transpose()
}

pub enum Scale {
    Servings(u32),
    Factor(f64),
}

impl Scale {
    pub fn apply(&self, recipe: &mut Recipe, converter: &Converter) -> anyhow::Result<()> {
        match *self {
            Scale::Servings(n) => recipe.scale_to_servings(n, converter)?,
            Scale::Factor(f) => recipe.scale(f, converter),
        }
        Ok(())
    }
}