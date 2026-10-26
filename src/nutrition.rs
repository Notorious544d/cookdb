use cooklang::Recipe;
use std::collections::HashMap;
use std::path::Path;

#[cfg(feature = "nutrition")]
pub use cooklang_ingredients::enrichment::EnrichedIngredients;
#[cfg(not(feature = "nutrition"))]
pub enum EnrichedIngredients {} // placeholder so the type exists

pub type Enriched = HashMap<String, EnrichedIngredients>;

#[cfg(feature = "nutrition")]
pub fn enrich(recipes: &HashMap<String, Recipe>, data: &Path) -> anyhow::Result<Option<Enriched>> {
    let mut out = HashMap::new();
    for slug in recipes.keys() {
        let ing = cooklang_ingredients::enrichment::resolve_recipe_ingredients(slug, recipes, data)?;
        out.insert(slug.clone(), ing);
    }
    println!("Enriched {} recipes", out.len());
    Ok(Some(out))
}

#[cfg(not(feature = "nutrition"))]
pub fn enrich(_: &HashMap<String, Recipe>, _: &Path) -> anyhow::Result<Option<Enriched>> {
    Ok(None)
}