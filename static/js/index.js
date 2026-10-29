import { openDb, query } from './db.js';
import { searchRecipes } from './search.js';
import { mountSiteHeader } from './site-header.js';

await mountSiteHeader();

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

function render() {
  const term = search.value.trim();
  const matches = term
    ? (() => {
        const slugs = new Set(searchRecipes(db, term).map((r) => r.slug));
        return recipes.filter((r) => slugs.has(r.slug));
      })()
    : [...recipes];
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
