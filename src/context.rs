use anyhow::Context as _;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "config/cookdb.conf";
const DATA_DIR: &str = "db";
const DEFAULT_OUTPUT: &str = "_site";
const OUTPUT_SECTION: &str = "output";
const DB_FILE: &str = "recipes.db";

pub struct Context {
    pub recipes_dir: PathBuf,
    pub out_dir: PathBuf,
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub allowed: HashMap<String, Vec<String>>,
}

impl Context {
    pub fn new(base_path: &Path) -> anyhow::Result<Self> {
        let base = std::path::absolute(base_path)
            .with_context(|| format!("resolving {}", base_path.display()))?;
        anyhow::ensure!(base.is_dir(), "{} is not a directory", base.display());

        let config_path = base.join(CONFIG_FILE);
        let text = match std::fs::read_to_string(&config_path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e).with_context(|| format!("reading {}", config_path.display())),
        };
        let conf = cooklang::aisle::parse(&text)
            .with_context(|| format!("parsing {}", config_path.display()))?;

        let mut out = DEFAULT_OUTPUT;
        let mut allowed = HashMap::new();
        for category in &conf.categories {
            if category.name == OUTPUT_SECTION {
                let [dir] = category.ingredients.as_slice() else {
                    anyhow::bail!(
                        "[{OUTPUT_SECTION}] needs exactly one directory (a line starting with // is a comment)"
                    );
                };
                anyhow::ensure!(dir.names.len() == 1, "[{OUTPUT_SECTION}]: '|' is not allowed in a path");
                out = dir.names[0];
            } else {
                allowed.insert(
                    category.name.to_string(),
                    category.ingredients.iter().filter_map(|i| i.names.first()).map(|n| n.to_string()).collect(),
                );
            }
        }

        let out_dir = base.join(out);
        let db_path = out_dir.join(DB_FILE);
        let data_dir = base.join(DATA_DIR);
        Ok(Self { recipes_dir: base, out_dir, data_dir, db_path, allowed })
    }

    /// `name` relative to the recipes dir; `.cook` is added if there is no extension.
    pub fn recipe_path(&self, name: &str) -> PathBuf {
        let mut path = self.recipes_dir.join(name);
        if path.extension().is_none() {
            path.set_extension("cook");
        }
        path
    }
}