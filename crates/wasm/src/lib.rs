use cookdb_core::render::{Format, Options, render};
use cooklang::{Converter, Recipe};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn export_recipe(json: &str, title: &str, scale: f64, format: &str) -> Result<String, JsError> {
    let mut recipe: Recipe = serde_json::from_str(json)?;
    recipe.scale(scale, &Converter::default());
    let format: Format = format.parse().map_err(|e: String| JsError::new(&e))?;
    Ok(render(&recipe, format, &Options::new(title, scale))?)
}

#[wasm_bindgen]
pub fn scale_recipe(recipe_json: &str, factor: f64) -> Result<String, JsError> {
    let _ = factor;
    Ok(recipe_json.to_string()) // TODO: deserialize cooklang::Recipe, scale, re-serialize
}

#[wasm_bindgen]
pub fn nutrition_table(enriched_json: &str, factor: f64) -> Result<String, JsError> {
    let _ = factor;
    Ok(enriched_json.to_string()) // TODO
}