use askama::Template;
use cooklang::{Content, Converter, Item, Recipe, Step};

pub enum StepItem {
    Text(String),
    Ingredient(String),
    Cookware(String),
    Timer(String),
    Quantity(String),
}

pub struct StepView {
    pub number: u32,
    pub items: Vec<StepItem>,
    pub refs: Vec<String>, // "name: amount" line shown under the step, like cookcli
}

pub enum SectionItem {
    Step(StepView),
    Note(String),
}

pub struct SectionView {
    pub name: Option<String>,
    pub items: Vec<SectionItem>,
}

pub struct IngredientView {
    pub name: String,
    pub quantity: String,
}

#[derive(Template)]
#[template(path = "recipe.html")]
pub struct RecipePage {
    title: String,
    tags: Vec<String>,
    cuisine: Option<String>,
    ingredients: Vec<IngredientView>,
    cookware: Vec<String>,
    sections: Vec<SectionView>,
}

impl StepView {
    fn new(recipe: &Recipe, step: &Step) -> Self {
        let mut items = Vec::new();
        let mut refs = Vec::new();
        for item in &step.items {
            match item {
                Item::Text { value } => items.push(StepItem::Text(value.clone())),
                Item::Ingredient { index } => {
                    let ing = &recipe.ingredients[*index];
                    let name = ing.display_name().into_owned();
                    refs.push(match &ing.quantity {
                        Some(q) => format!("{name}: {q}"),
                        None => name.clone(),
                    });
                    items.push(StepItem::Ingredient(name));
                }
                Item::Cookware { index } => {
                    items.push(StepItem::Cookware(recipe.cookware[*index].display_name().to_string()))
                }
                Item::Timer { index } => {
                    if let Some(q) = &recipe.timers[*index].quantity {
                        items.push(StepItem::Timer(q.to_string()));
                    }
                }
                Item::InlineQuantity { index } => {
                    items.push(StepItem::Quantity(recipe.inline_quantities[*index].to_string()))
                }
            }
        }
        Self { number: step.number, items, refs }
    }
}

impl RecipePage {
    pub fn new(title: &str, recipe: &Recipe, converter: &Converter) -> Self {
        let ingredients = recipe
            .group_ingredients(converter)
            .into_iter()
            .map(|g| IngredientView {
                name: g.ingredient.display_name().into_owned(),
                quantity: g.quantity.to_string(),
            })
            .collect();
        let cookware = recipe
            .group_cookware(converter)
            .into_iter()
            .map(|c| c.cookware.display_name().to_string())
            .collect();
        let sections = recipe
            .sections
            .iter()
            .map(|s| SectionView {
                name: s.name.clone(),
                items: s
                    .content
                    .iter()
                    .map(|c| match c {
                        Content::Step(step) => SectionItem::Step(StepView::new(recipe, step)),
                        Content::Text(t) => SectionItem::Note(t.clone()),
                    })
                    .collect(),
            })
            .collect();
        Self {
            title: title.to_string(),
            tags: recipe.metadata.tags().unwrap_or_default().into_iter().map(|t| t.into_owned()).collect(),
            cuisine: recipe.metadata.get("cuisine").and_then(|v| v.as_str()).map(str::to_string),
            ingredients,
            cookware,
            sections,
        }
    }

    pub fn html(&self) -> anyhow::Result<String> {
        Ok(self.render()?)
    }
}