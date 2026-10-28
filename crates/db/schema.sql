CREATE TABLE cuisines (name TEXT PRIMARY KEY COLLATE NOCASE);
CREATE TABLE tags     (name TEXT PRIMARY KEY COLLATE NOCASE);


CREATE TABLE recipes (
    slug    TEXT PRIMARY KEY,
    title   TEXT,
    cuisine TEXT REFERENCES cuisines(name)
);


CREATE TABLE recipe_tags (
    slug TEXT NOT NULL REFERENCES recipes(slug) ON DELETE CASCADE,
    tag  TEXT NOT NULL COLLATE NOCASE REFERENCES tags(name),
    PRIMARY KEY (slug, tag)
);
CREATE INDEX idx_recipe_tags_tag ON recipe_tags(tag);


CREATE TABLE ingredient_categories (
    ingredient TEXT NOT NULL COLLATE NOCASE,
    category   TEXT NOT NULL COLLATE NOCASE,
    PRIMARY KEY (ingredient, category)
);


CREATE TABLE recipe_ingredients (
    recipe_slug TEXT NOT NULL REFERENCES recipes(slug) ON DELETE CASCADE,
    ingredient_name TEXT NOT NULL COLLATE NOCASE,
    PRIMARY KEY (recipe_slug, ingredient_name)
);
CREATE INDEX idx_recipe_ingredients_name
    ON recipe_ingredients(ingredient_name);


CREATE VIEW recipe_ingredient_terms AS
    SELECT recipe_slug AS slug, ingredient_name AS term
    FROM recipe_ingredients
    UNION
    SELECT ri.recipe_slug, c.category
    FROM recipe_ingredients ri
    JOIN ingredient_categories c ON c.ingredient = ri.ingredient_name;



CREATE TABLE recipe_data (
    slug TEXT PRIMARY KEY REFERENCES recipes(slug) ON DELETE CASCADE,
    json TEXT NOT NULL 
);


CREATE TABLE recipe_enrichment (
    slug TEXT PRIMARY KEY COLLATE NOCASE,
    json TEXT NOT NULL 
);
