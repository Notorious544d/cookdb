INSERT INTO cuisines (name) VALUES
    ('Burmese'),
    ('Chinese'),
    ('Indian'),
    ('Italian'),
    ('Japanese'),
    ('Mexican'),
    ('Moroccan'),
    ('Thai'),
    ('Turkish');

INSERT INTO tags (name) VALUES
    ('lamb'),
    ('rice'),
    ('vegetarian'),
    ('prawns'),
    ('fish'),
    ('chicken'),
    ('beef'),
    ('noodles'),
    ('naan'),
    ('pasta'),
    ('roti'),
    ('salad'),
    ('soup'),

    -- Meal types
    ('Breakfast'),
    ('Brunch'),
    ('Lunch'),
    ('Dinner'),

    ('Dessert');

INSERT INTO ingredient_tag_map (ingredient, tag) VALUES
    ('cod', 'fish'),
    ('salmon', 'fish'),
    ('shrimp', 'prawns');
