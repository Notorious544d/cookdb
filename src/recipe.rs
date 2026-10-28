use clap::Args;
use cookdb_core::render::{Format, Options, Style, render_to};
use cookdb_core::parse;
use crate::context::Context;

#[derive(Args)]
pub struct RecipeArgs {
    recipe: String,
    /// Scaling factor (overridden by the `:N` suffix)
    #[arg(short, long, default_value_t = 1.0)]
    scale: f64,
    /// Output format
    #[arg(short, long, default_value = "human")]
    format: Format,
    /// Pretty-print JSON and schema output
    #[arg(long)]
    pretty: bool,
}

pub fn run(ctx: &Context, args: RecipeArgs) -> anyhow::Result<()> {
    let (name, scale) = match args.recipe.rsplit_once(':') {
        Some((p, n)) if n.parse::<f64>().is_ok() => (p, n.parse()?),
        _ => (args.recipe.as_str(), args.scale),
    };
    let path = ctx.recipe_path(name);

    let mut recipe = parse::load_recipe(&path)?;
    recipe.scale(scale, parse::converter());

    let title = path.file_stem().and_then(|s| s.to_str()).unwrap_or("recipe");
    let mut opts = Options::new(title, scale);
    opts.style = Style::Ansi;
    opts.pretty = args.pretty;
    render_to(&recipe, args.format, &opts, std::io::stdout().lock())?;
    Ok(())
}