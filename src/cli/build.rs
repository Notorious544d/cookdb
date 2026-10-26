use clap::Args;
use cooklang::{Converter, CooklangParser, Extensions};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use crate::build_db;
use crate::nutrition;

#[derive(Args)]
pub struct BuildArgs {
    /// Directory containing .cook recipes
    #[arg(default_value = "../recipes")]
    pub recipes: PathBuf,
    /// Output directory for recipes.db
    #[arg(default_value = "static")]
    pub out_dir: PathBuf,
}

pub fn run(args: BuildArgs) -> anyhow::Result<()> {
    let recipes = args.recipes;
    let out_dir = args.out_dir;
    let db_path = out_dir.join("recipes.db");

    // Create once, reuse for every file.
    let parser = CooklangParser::new(Extensions::all(), Converter::default());

    // Enrichment
    let mut recipes_by_key = HashMap::new();

    for entry in std::fs::read_dir(&recipes)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("cook") {
            continue;
        }

        let slug = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| anyhow::anyhow!("non-UTF-8 recipe filename: {}", path.display()))?
            .to_ascii_lowercase();

        let recipe = build_db::load_recipe(&parser, &path)?;
        anyhow::ensure!(
            recipes_by_key.insert(slug.clone(), recipe).is_none(),
            "duplicate recipe slug ignoring case: '{slug}'"
        );
    }

    let enriched = nutrition::enrich(&recipes_by_key, &recipes.join("db"))?;
    let start = Instant::now();
    std::fs::create_dir_all(&out_dir)?;
    build_db::build(&parser, &recipes, &db_path, enriched.as_ref())?;
    let duration = start.elapsed();
    
    println!("Database created in {:.2}s", duration.as_secs_f64());
    Ok(())
}