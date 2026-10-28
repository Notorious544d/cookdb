use anyhow::Context;
use cooklang::{Converter, CooklangParser, Extensions, Recipe};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static PARSER: LazyLock<CooklangParser> =
    LazyLock::new(|| CooklangParser::new(Extensions::all(), Converter::default()));

pub fn converter() -> &'static Converter {
    PARSER.converter()
}

pub fn parse_str(text: &str, name: &str) -> anyhow::Result<Recipe> {
    let (recipe, warnings) = PARSER
        .parse(text)
        .into_result()
        .map_err(|e| anyhow::anyhow!("parsing {name}: {e}"))?;
    for w in warnings.iter() {
        eprintln!("warning: {name}: {w}");
    }
    Ok(recipe)
}

pub fn load_recipe(path: &Path) -> anyhow::Result<Recipe> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    parse_str(&text, &path.display().to_string())
}

/// Top-level `*.cook` files in `dir`, sorted.
pub fn recipe_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("cook") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

pub fn slug(path: &Path) -> anyhow::Result<&str> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .with_context(|| format!("non-UTF-8 file name: {}", path.display()))
}