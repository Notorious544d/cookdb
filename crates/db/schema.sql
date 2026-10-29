CREATE TABLE recipes (
    slug           TEXT PRIMARY KEY,
    title          TEXT,
    cuisine        TEXT COLLATE NOCASE,
    difficulty     TEXT,
    servings_count INTEGER,
    servings_text  TEXT,
    source_name    TEXT,
    source_url     TEXT,
    prep_minutes   INTEGER,
    cook_minutes   INTEGER,
    total_minutes  INTEGER
);


CREATE TABLE recipe_tags (
    slug TEXT NOT NULL REFERENCES recipes(slug) ON DELETE CASCADE,
    tag  TEXT NOT NULL COLLATE NOCASE,
    PRIMARY KEY (slug, tag)
);
CREATE INDEX idx_recipe_tags_tag ON recipe_tags(tag);


CREATE TABLE recipe_ingredients (
    recipe_slug TEXT NOT NULL REFERENCES recipes(slug) ON DELETE CASCADE,
    ingredient_name TEXT NOT NULL COLLATE NOCASE,
    PRIMARY KEY (recipe_slug, ingredient_name)
);
CREATE INDEX idx_recipe_ingredients_name
    ON recipe_ingredients(ingredient_name);


CREATE TABLE recipe_data (
    slug TEXT PRIMARY KEY REFERENCES recipes(slug) ON DELETE CASCADE,
    json TEXT NOT NULL
);


CREATE TABLE recipe_enrichment (
    slug TEXT PRIMARY KEY COLLATE NOCASE,
    json TEXT NOT NULL
);
