use cooklang::Recipe;
use cooklang_format::{PaperSize, cooklang_source, human, latex, markdown, schema, typst};
use std::{io, str::FromStr};

pub use cooklang_format::Style;

#[derive(Clone, Copy, Debug)]
pub enum Format {
    Human,
    Json,
    Cooklang,
    Markdown,
    Latex,
    Typst,
    Schema,
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "human" => Self::Human,
            "json" => Self::Json,
            "cooklang" | "cook" => Self::Cooklang,
            "markdown" | "md" => Self::Markdown,
            "latex" | "tex" => Self::Latex,
            "typst" | "typ" => Self::Typst,
            "schema" | "jsonld" => Self::Schema,
            other => return Err(format!("unknown format '{other}'")),
        })
    }
}

pub struct Options<'a> {
    pub title: &'a str,
    /// Display only: the recipe must already be scaled.
    pub scale: f64,
    /// `Ansi` for the terminal, `Plain` for files and the web.
    pub style: Style,
    pub pretty: bool,
    pub paper: PaperSize,
    pub margin: f64,
}

impl<'a> Options<'a> {
    pub fn new(title: &'a str, scale: f64) -> Self {
        Self { title, scale, style: Style::Plain, pretty: false, paper: PaperSize::A4, margin: 2.5 }
    }
}

pub fn render_to(recipe: &Recipe, format: Format, o: &Options, mut w: impl io::Write) -> io::Result<()> {
    let conv = crate::parse::converter();
    match format {
        Format::Human => human::print_human(recipe, o.title, o.scale, conv, o.style, &mut w),
        Format::Json if o.pretty => serde_json::to_writer_pretty(&mut w, recipe).map_err(io::Error::other),
        Format::Json => serde_json::to_writer(&mut w, recipe).map_err(io::Error::other),
        Format::Cooklang => cooklang_source::print_cooklang(recipe, &mut w),
        Format::Markdown => markdown::print_md(recipe, o.title, o.scale, conv, &mut w),
        Format::Latex => latex::print_latex(recipe, o.title, o.scale, conv, &mut w, o.paper, o.margin),
        Format::Typst => typst::print_typst(recipe, o.title, o.scale, conv, &mut w, o.paper, o.margin),
        Format::Schema => schema::print_schema(recipe, o.title, o.scale, conv, &mut w, o.pretty),
    }
}

pub fn render(recipe: &Recipe, format: Format, o: &Options) -> io::Result<String> {
    let mut buf = Vec::new();
    render_to(recipe, format, o, &mut buf)?;
    String::from_utf8(buf).map_err(io::Error::other)
}