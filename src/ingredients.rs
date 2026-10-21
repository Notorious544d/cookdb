use std::path::Path;

use anyhow::Context;
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Debug, Deserialize)]
pub struct Meta {
    #[serde(default = "one")]
    pub density: f64,
    pub unit: Option<String>,
}

fn one() -> f64 {
    1.0
}

#[derive(Debug, Deserialize)]
pub struct Product {
    pub quantity: f64,
    // store -> date -> price
    pub stores: IndexMap<String, IndexMap<String, f64>>,
}

pub type Products = IndexMap<String, Product>;

#[derive(Debug, Deserialize)]
pub struct Nutrition {
    pub sodium: f64,
}

#[derive(Debug)]
pub struct IngredientData {
    pub meta: Meta,
    pub products: Products,
    pub nutrition: Nutrition,
}

fn load_yaml<T: DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    serde_yaml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

impl IngredientData {
    /// `Ok(None)` when the ingredient has no folder under `root`.
    pub fn load(root: &Path, name: &str) -> anyhow::Result<Option<Self>> {
        // Recipe-reference ingredients are paths; never let them escape `root`.
        if name.contains(['/', '\\']) || name.contains("..") {
            return Ok(None);
        }
        let dir = root.join(name);
        if !dir.is_dir() {
            return Ok(None);
        }
        Ok(Some(Self {
            meta: load_yaml(&dir.join("meta.yml"))?,
            products: load_yaml(&dir.join("products.yml"))?,
            nutrition: load_yaml(&dir.join("nutrition.yml"))?,
        }))
    }

    pub fn price_per_unit(&self) -> anyhow::Result<f64> {
        let (_, product) = self.products.first().context("no products")?;
        let (_, prices) = product.stores.first().context("no stores")?;
        let (_, price) = prices.first().context("no prices")?;
        anyhow::ensure!(product.quantity > 0.0, "product quantity must be positive");
        Ok(price / product.quantity)
    }
}