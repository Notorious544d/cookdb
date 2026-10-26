import { openDb, query } from './db.js';
import { mountSiteHeader } from './site-header.js';

mountSiteHeader();

const search = document.getElementById('search-input');
const list = document.getElementById('recipe-list');
const count = document.getElementById('recipe-count');
const sortField = document.getElementById('sort-field');
const sortDir = document.getElementById('sort-dir');
let direction = 1;

sortField.addEventListener('change', render);
sortDir.addEventListener('click', () => {
  direction *= -1;
  sortDir.textContent = direction === 1 ? '↑' : '↓';
  render();
});

const db = await openDb();

const recipes = query(
  db,
  `SELECT slug, COALESCE(NULLIF(title, ''), slug) AS title, cuisine
   FROM recipes
   ORDER BY title`,
);

const tagsByRecipe = new Map();
for (const { slug, tag } of query(db, 'SELECT slug, tag FROM recipe_tags ORDER BY tag')) {
  const key = slug.toLowerCase();
  if (!tagsByRecipe.has(key)) tagsByRecipe.set(key, []);
  tagsByRecipe.get(key).push(tag);
}

const ingredientsByRecipe = new Map();
for (const { recipe_slug, ingredient_name } of query(
  db,
  'SELECT recipe_slug, ingredient_name FROM recipe_ingredients ORDER BY ingredient_name',
)) {
  const key = recipe_slug.toLowerCase();
  if (!ingredientsByRecipe.has(key)) ingredientsByRecipe.set(key, []);
  ingredientsByRecipe.get(key).push(ingredient_name);
}

function render() {
  const term = search.value.trim().toLowerCase();
  const matches = recipes.filter((recipe) => {
    const key = recipe.slug.toLowerCase();
    const searchable = [
      recipe.title,
      recipe.cuisine ?? '',
      ...(tagsByRecipe.get(key) ?? []),
      ...(ingredientsByRecipe.get(key) ?? []),
    ].join(' ').toLowerCase();

    return searchable.includes(term);
  });

  const field = sortField.value;
  const collator = new Intl.Collator(undefined, {
    numeric: true,
    sensitivity: 'base',
  });

  matches.sort((a, b) => {
    const left = field === 'cuisine' ? (a.cuisine ?? '') : a.title;
    const right = field === 'cuisine' ? (b.cuisine ?? '') : b.title;
    return direction * collator.compare(left, right);
  });

  count.textContent = `${matches.length} recipe${matches.length === 1 ? '' : 's'}`;
  list.replaceChildren();

  if (matches.length === 0) {
    list.textContent = 'No recipes found.';
    return;
  }

  for (const recipe of matches) {
    const key = recipe.slug.toLowerCase();
    const card = document.createElement('a');
    card.className = 'recipe-card';
    card.href = `recipe.html?slug=${encodeURIComponent(recipe.slug)}`;

    const content = document.createElement('div');
    content.className = 'p-6';

    const icon = document.createElement('div');
    icon.className = 'recipe-card-icon';
    icon.textContent = '🍽️';
    content.append(icon);

    const titleRow = document.createElement('div');
    titleRow.className = 'flex flex-wrap items-center gap-2';

    const title = document.createElement('h2');
    title.className = 'recipe-card-title hover:text-accent-text';
    title.textContent = recipe.title;

    titleRow.append(title);
    content.append(titleRow);

    const tagList = document.createElement('div');
    tagList.className = 'mt-3 flex flex-wrap gap-2';

    for (const tag of tagsByRecipe.get(key) ?? []) {
      const badge = document.createElement('span');
      badge.className = 'tag';
      badge.textContent = tag;
      tagList.append(badge);
    }

    content.append(tagList);
    card.append(content);
    list.append(card);
  }
}

search.addEventListener('input', render);
render();
