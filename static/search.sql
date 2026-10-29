WITH category(ingredient, category) AS (VALUES
    ('cod',     'fish'),
    ('salmon',  'fish'),
    ('shrimp',  'prawns'),
    ('chicken', 'meat'),
    ('lamb',    'meat')
),

ingredient_terms(slug, term) AS (
    SELECT recipe_slug, ingredient_name FROM recipe_ingredients
    UNION
    SELECT ri.recipe_slug, c.category
    FROM recipe_ingredients ri
    JOIN category c ON c.ingredient = ri.ingredient_name COLLATE NOCASE
    UNION
    SELECT recipe_slug, 'vegetarian'
    FROM recipe_ingredients
    WHERE recipe_slug NOT IN (
        SELECT ri.recipe_slug
        FROM recipe_ingredients ri
        JOIN category c ON c.ingredient = ri.ingredient_name COLLATE NOCASE
        WHERE c.category IN ('meat', 'fish', 'prawns')
    )
)

SELECT r.slug, COALESCE(NULLIF(r.title, ''), r.slug) AS title
FROM recipes r
WHERE COALESCE(NULLIF(r.title, ''), r.slug) LIKE ?1
   OR r.cuisine LIKE ?1
   OR EXISTS (SELECT 1 FROM ingredient_terms t WHERE t.slug = r.slug AND t.term LIKE ?1)
   OR EXISTS (SELECT 1 FROM recipe_tags g WHERE g.slug = r.slug AND g.tag LIKE ?1)
ORDER BY title