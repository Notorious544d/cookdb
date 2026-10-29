use cookdb_core::render::{Format, Options, render};
use cooklang::convert::System;
use cooklang::{Converter, Quantity, Recipe};
use serde_json::json;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ScalableRecipe {
    base: Recipe,
    converter: Converter,
}

fn texts<'a>(qs: impl Iterator<Item = Option<&'a Quantity>>) -> Vec<String> {
    qs.map(|q| q.map(ToString::to_string).unwrap_or_default()).collect()
}

#[wasm_bindgen]
impl ScalableRecipe {
    #[wasm_bindgen(constructor)]
    pub fn new(recipe_json: &str) -> Result<ScalableRecipe, JsError> {
        Ok(Self {
            base: serde_json::from_str(recipe_json)?,
            converter: cookdb_core::parse::converter().clone(),
        })
    }

    /// JSON of display strings for every quantity at `factor`, index-aligned with the recipe arrays.
    pub fn scaled_text(&self, factor: f64, system: &str) -> String {
        let mut recipe = self.base.clone();
        recipe.scale(factor, &self.converter);
        match system {
            "metric" => { recipe.convert(System::Metric, &self.converter); }
            "imperial" => { recipe.convert(System::Imperial, &self.converter); }
            _ => {}
        }
        json!({
            "ingredients": texts(recipe.ingredients.iter().map(|i| i.quantity.as_ref())),
            "timers": texts(recipe.timers.iter().map(|t| t.quantity.as_ref())),
            "inline": texts(recipe.inline_quantities.iter().map(Some)),
        })
        .to_string()
    }

    pub fn export(&self, title: &str, scale: f64, format: &str) -> Result<String, JsError> {
        let mut recipe = self.base.clone();
        recipe.scale(scale, &self.converter);
        let format: Format = format.parse().map_err(|e: String| JsError::new(&e))?;
        Ok(render(&recipe, format, &Options::new(title, scale))?)
    }
}

#[wasm_bindgen]
pub fn export_recipe(json: &str, title: &str, scale: f64, format: &str) -> Result<String, JsError> {
    let mut recipe: Recipe = serde_json::from_str(json)?;
    recipe.scale(scale, &Converter::default());
    let format: Format = format.parse().map_err(|e: String| JsError::new(&e))?;
    Ok(render(&recipe, format, &Options::new(title, scale))?)
}

#[wasm_bindgen]
pub fn nutrition_table(enriched_json: &str, factor: f64) -> Result<String, JsError> {
    let _ = factor;
    Ok(enriched_json.to_string()) // TODO
}