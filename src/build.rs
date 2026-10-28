use cookdb_core::parse;
use cookdb_db::nutrition;
use std::collections::HashMap;
use std::time::Instant;
use crate::context::Context;

pub fn run(ctx: &Context) -> anyhow::Result<()> {
    let out_dir = ctx.output_dir();
    let db_path = ctx.db_path();
    let recipes = ctx.base_path();

    let mut recipes_by_key = HashMap::new();
    for path in parse::recipe_files(&recipes)? {
        let slug = parse::slug(&path)?.to_ascii_lowercase();
        let recipe = parse::load_recipe(&path)?;
        anyhow::ensure!(
            recipes_by_key.insert(slug.clone(), recipe).is_none(),
            "duplicate recipe slug ignoring case: '{slug}'"
        );
    }

    let enriched = nutrition::enrich(&recipes_by_key, &ctx.data_dir())?;
    let start = Instant::now();
    std::fs::create_dir_all(&out_dir)?;
    cookdb_db::build(recipes, &db_path, enriched.as_ref())?;
    let duration = start.elapsed();
    
    println!("Database created in {:.2}s", duration.as_secs_f64());
    Ok(())
}