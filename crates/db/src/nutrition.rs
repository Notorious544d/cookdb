use cooklang::Recipe;
use cooklang_ingredients::enrichment::resolve_recipe_ingredients;
use rusqlite::{Transaction, params};
use std::collections::HashMap;
use std::path::Path;

pub fn apply(tx: &Transaction, recipes: &HashMap<String, Recipe>, data: &Path) -> anyhow::Result<()> {
    let mut insert =
        tx.prepare_cached("INSERT INTO recipe_enrichment (slug, json) VALUES (?1, ?2)")?;
    for slug in recipes.keys() {
        let enriched = resolve_recipe_ingredients(slug, recipes, data)?;
        insert.execute(params![slug, serde_json::to_string(&enriched)?])?;
    }
    println!("Enriched {} recipes", recipes.len());
    Ok(())
}