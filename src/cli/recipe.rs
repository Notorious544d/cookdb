use clap::{Args, ValueEnum};
use cooklang::{Converter, CooklangParser, Extensions};
use cooklang_format::{PaperSize, Style, cooklang_source, human, latex, markdown, schema, typst};
use std::path::PathBuf;

use crate::build_db;

#[derive(Args)]
pub struct RecipeArgs {
    recipe: String,
    /// Scaling factor (overridden by the `:N` suffix)
    #[arg(short, long, default_value_t = 1.0)]
    scale: f64,
    /// Output format
    #[arg(short, long, value_enum, default_value = "human")]
    format: Format,
    /// Pretty-print JSON and schema output
    #[arg(long)]
    pretty: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Human,
    Json,
    Cooklang,
    Markdown,
    Latex,
    Typst,
    Schema,
}

pub fn run(args: RecipeArgs) -> anyhow::Result<()> {
    let (path, scale) = match args.recipe.rsplit_once(':') {
        Some((p, n)) if n.parse::<f64>().is_ok() => (p, n.parse()?),
        _ => (args.recipe.as_str(), args.scale),
    };
    let mut path = PathBuf::from(path);
    if path.extension().is_none() {
        path.set_extension("cook");
    }

    let parser = CooklangParser::new(Extensions::all(), Converter::default());
    let mut recipe = build_db::load_recipe(&parser, &path)?;
    recipe.scale(scale, parser.converter());

    let title = path.file_stem().and_then(|s| s.to_str()).unwrap_or("recipe");
    let conv = parser.converter();
    let mut out = std::io::stdout().lock();

    match args.format {
        Format::Human => human::print_human(&recipe, title, scale, conv, Style::Ansi, &mut out)?,
        Format::Json if args.pretty => serde_json::to_writer_pretty(&mut out, &recipe)?,
        Format::Json => serde_json::to_writer(&mut out, &recipe)?,
        Format::Cooklang => cooklang_source::print_cooklang(&recipe, out)?,
        Format::Markdown => markdown::print_md(&recipe, title, scale, conv, &mut out)?,
        Format::Latex => latex::print_latex(&recipe, title, scale, conv, &mut out, PaperSize::A4, 2.5)?,
        Format::Typst => typst::print_typst(&recipe, title, scale, conv, &mut out, PaperSize::A4, 2.5)?,
        Format::Schema => schema::print_schema(&recipe, title, scale, conv, &mut out, args.pretty)?,
    }
    Ok(())
}