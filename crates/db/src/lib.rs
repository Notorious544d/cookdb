#[cfg(feature = "nutrition")]
pub mod nutrition;

use anyhow::Context;
use cooklang::{Recipe};
use cooklang::metadata::{NameAndUrl, RecipeTime, Servings};
use rusqlite::{Connection, Transaction, params};
use std::collections::HashMap;
use std::path::Path;

const SCHEMA: &str = include_str!("../schema.sql");

#[derive(Default)]
pub struct RecipeMetadata {
    pub title: Option<String>,
    pub cuisine: Option<String>,
    pub difficulty: Option<String>,
    pub servings: Option<Servings>,
    pub source: Option<NameAndUrl>,
    pub tags: Vec<String>,
    pub prep_minutes: Option<u32>,
    pub cook_minutes: Option<u32>,
    pub total_minutes: Option<u32>,
}

impl RecipeMetadata {
    pub fn from_recipe(recipe: &Recipe) -> Self {
        let m = &recipe.metadata;
        let text = |key: &str| m.get(key).and_then(|v| v.as_str()).map(|s| s.trim().to_owned());
        let (prep, cook, total) = match m.time(cookdb_core::parse::converter()) {
            Some(RecipeTime::Total(t)) => (None, None, Some(t)),
            Some(RecipeTime::Composed { prep_time, cook_time }) => {
                (prep_time, cook_time, Some(prep_time.unwrap_or(0) + cook_time.unwrap_or(0)))
            }
            None => (None, None, None),
        };
        Self {
            title: m.title().map(str::to_owned),
            cuisine: text("cuisine"),
            difficulty: text("difficulty"),
            servings: m.servings(),
            source: m.source(),
            tags: m.tags().unwrap_or_default().iter().map(|t| t.trim().to_owned()).collect(),
            prep_minutes: prep,
            cook_minutes: cook,
            total_minutes: total,
        }
    }
    fn checked(&self) -> Vec<(&'static str, Vec<&str>)> {
        vec![
            ("cuisine", self.cuisine.iter().map(String::as_str).collect()),
            ("difficulty", self.difficulty.iter().map(String::as_str).collect()),
            ("tags", self.tags.iter().map(String::as_str).collect()),
        ]
    }
    fn check(&self, slug: &str, allowed: &HashMap<String, Vec<String>>) -> anyhow::Result<()> {
    for (field, values) in self.checked() {
        let Some(list) = allowed.get(field) else { continue };
        for value in values {
            anyhow::ensure!(
                list.iter().any(|a| a.eq_ignore_ascii_case(value)),
                "{slug}: unknown {field} '{value}' (add it to [{field}] in config/cookdb.conf)"
            );
        }
    }
    Ok(())
}
}

/// Rebuilds the database from every `.cook` file directly under `recipes_dir`.
pub fn build(    
    recipes_dir: &Path,
    db_path: &Path,
    allowed: &HashMap<String, Vec<String>>,
    data_dir: &Path,
) -> anyhow::Result<usize> {
    let known: Vec<&str> = RecipeMetadata::default().checked().into_iter().map(|(name, _)| name).collect();
    for name in allowed.keys() {
        anyhow::ensure!(
            known.contains(&name.as_str()),
            "unknown section [{name}] in config/cookdb.conf (expected one of {known:?})"
        );
    }
    let mut conn = create(db_path)?;
    let tx = conn.transaction()?;
    let mut recipes = HashMap::new(); // lowercase slug -> recipe, for resolving links
    let mut slugs = Vec::new();       // (slug as written, lowercase key), in file order
    for path in cookdb_core::parse::recipe_files(recipes_dir)? {
        let slug = cookdb_core::parse::slug(&path)?;
        let key = slug.to_ascii_lowercase();
        let recipe = cookdb_core::parse::load_recipe(&path)?;
        anyhow::ensure!(
            recipes.insert(key.clone(), recipe).is_none(),
            "duplicate recipe slug ignoring case: '{slug}'"
        );
        slugs.push((slug.to_owned(), key));
    }
    for (slug, key) in &slugs {
        insert_recipe(&tx, slug, &recipes[key], allowed)
            .with_context(|| format!("inserting {slug}"))?;

        let mut names = Vec::new();
        ingredient_names(key, &recipes, &mut Vec::new(), &mut names)
            .with_context(|| format!("ingredients of {slug}"))?;
        for name in names {
            tx.execute(
                "INSERT OR IGNORE INTO recipe_ingredients (recipe_slug, ingredient_name) VALUES (?1, ?2)",
                params![slug, name],
            )?;
        }
    }
    let _ = data_dir; // only read when the `nutrition` feature is on
    #[cfg(feature = "nutrition")]
    nutrition::apply(&tx, &recipes, data_dir)?;

    tx.commit()?;
    Ok(recipes.len())
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
    Ok(conn)
}

fn insert_recipe(
    tx: &Transaction,
    slug: &str,
    recipe: &Recipe,
    allowed: &HashMap<String, Vec<String>>,
) -> anyhow::Result<()> {
    let meta = RecipeMetadata::from_recipe(recipe);
    meta.check(slug, allowed)?;
    let (servings_count, servings_text) = match &meta.servings {
        Some(Servings::Number(n)) => (Some(*n), None),
        Some(Servings::Text(t)) => (None, Some(t.as_str())),
        None => (None, None),
    };
    tx.execute(
        "INSERT INTO recipes (slug, title, cuisine, difficulty, servings_count, servings_text,
                              source_name, source_url, prep_minutes, cook_minutes, total_minutes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            slug, meta.title, meta.cuisine, meta.difficulty, servings_count, servings_text,
            meta.source.as_ref().and_then(|s| s.name()),
            meta.source.as_ref().and_then(|s| s.url()),
            meta.prep_minutes, meta.cook_minutes, meta.total_minutes,
        ],
    )?;
    for tag in &meta.tags {
        tx.execute(
            "INSERT OR IGNORE INTO recipe_tags (slug, tag) VALUES (?1, ?2)",
            params![slug, tag],
        )?;
    }
    let json = serde_json::to_string(recipe).context("serialising recipe")?;
    tx.execute(
        "INSERT INTO recipe_data (slug, json) VALUES (?1, ?2)",
        params![slug, json],
    )?;
    Ok(())
}

/// Names of every ingredient in `key`'s recipe, following links to other recipes.
fn ingredient_names<'a>(
    key: &str,
    recipes: &'a HashMap<String, Recipe>,
    stack: &mut Vec<String>,
    out: &mut Vec<&'a str>,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        !stack.iter().any(|s| s == key),
        "linked recipe cycle: {} -> {key}",
        stack.join(" -> ")
    );
    let recipe = recipes
        .get(key)
        .with_context(|| format!("linked recipe '{key}' was not found"))?;

    stack.push(key.to_owned());
    for ingredient in &recipe.ingredients {
        match &ingredient.reference {
            Some(reference) => {
                ingredient_names(&reference.name.to_ascii_lowercase(), recipes, stack, out)?
            }
            None => out.push(ingredient.name.trim()),
        }
    }
    stack.pop();
    Ok(())
}