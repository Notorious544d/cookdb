mod build_db;
mod ingredients;
mod read_db;
mod render;

use std::path::Path;

use anyhow::Context;
use cooklang::{Converter, CooklangParser, Extensions};
use rusqlite::{Connection, OpenFlags};


fn main() -> anyhow::Result<()> {
    let recipes = Path::new(r"C:\Users\SESMAIL\Documents\homelab-config\food\recipes");
    let db_path = Path::new("recipes.db");

    // Create once, reuse for every file.
    let parser = CooklangParser::new(Extensions::all(), Converter::default());

    // let recipe = build_db::load_recipe(&parser, &recipes.join("Seebian.cook"))?;

    let n = build_db::build(&parser, recipes, db_path)?;
    println!("built {} with {n} recipes", db_path.display());

    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("opening {}", db_path.display()))?;

    let mut recipe = read_db::fetch_recipe(&conn, "Seebian")?
        .context("recipe 'Seebian' not found")?;
    read_db::Scale::Factor(2.0).apply(&mut recipe, parser.converter())?;

    let page = render::RecipePage::new("Seebian", &recipe, parser.converter());
    std::fs::write("Seebian.html", page.html()?)?;

    Ok(())

    /*
    for ing in &recipe.ingredients {
        match IngredientData::load(&ingredients_dir, &ing.name)? {
            Some(data) => println!(
                "{}: density {} price/unit {:.5}",
                ing.name,
                data.meta.density,
                data.price_per_unit()?
            ),
            None => println!("{}: no data", ing.name),
        }
    }

    for w in warnings.iter() {
        eprintln!("warning: {w}");
    }

    for (i, ing) in recipe.ingredients.iter().enumerate() {
        println!("{i} {} {:?}", ing.name, ing.quantity);
    }
    */
}