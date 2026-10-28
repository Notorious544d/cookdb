SELECT r.slug, COALESCE(NULLIF(r.title, ''), r.slug) AS title
FROM recipes r
WHERE COALESCE(NULLIF(r.title, ''), r.slug) LIKE ?1
   OR r.cuisine LIKE ?1
   OR EXISTS (SELECT 1 FROM recipe_ingredient_terms t
              WHERE t.slug = r.slug AND t.term LIKE ?1)
   OR EXISTS (SELECT 1 FROM recipe_tags g
              WHERE g.slug = r.slug AND g.tag LIKE ?1)
ORDER BY title