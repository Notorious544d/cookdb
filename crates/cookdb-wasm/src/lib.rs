use wasm_bindgen::prelude::*;

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