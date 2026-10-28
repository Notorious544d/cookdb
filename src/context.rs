use anyhow::Context as _;
use std::path::{Path, PathBuf};

const DATA_DIR: &str = "db";
const OUTPUT_DIR: &str = "_site";
const DB_FILE: &str = "recipes.db";

pub struct Context {
    base_path: PathBuf,
}

impl Context {
    pub fn new(base_path: &Path) -> anyhow::Result<Self> {
        let base_path = std::path::absolute(base_path)
            .with_context(|| format!("resolving {}", base_path.display()))?;
        anyhow::ensure!(base_path.is_dir(), "{} is not a directory", base_path.display());
        Ok(Self { base_path })
    }

    /// The recipe repository root (where the `.cook` files live).
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Ingredient data used for nutrition enrichment.
    pub fn data_dir(&self) -> PathBuf {
        self.base_path.join(DATA_DIR)
    }

    pub fn output_dir(&self) -> PathBuf {
        self.base_path.join(OUTPUT_DIR)
    }

    pub fn db_path(&self) -> PathBuf {
        self.output_dir().join(DB_FILE)
    }

    /// `name` relative to the base path; `.cook` is added if there is no extension.
    pub fn recipe_path(&self, name: &str) -> PathBuf {
        let mut path = self.base_path.join(name);
        if path.extension().is_none() {
            path.set_extension("cook");
        }
        path
    }
}