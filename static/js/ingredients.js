import { openDb, query } from './db.js';
import { mountSiteHeader } from './site-header.js';

mountSiteHeader();

const search = document.getElementById('search-input');
const count = document.getElementById('ingredient-count');
const list = document.getElementById('ingredient-list');
const heading = document.getElementById('products-heading');
const hint = document.getElementById('product-hint');
const comparison = document.getElementById('comparison-scroll');
const productRows = document.getElementById('product-rows');

search.placeholder = 'Search ingredients...';
search.setAttribute('aria-label', 'Search ingredients');

const db = await openDb();
const ingredientsByKey = new Map();

for (const { json } of query(db, 'SELECT json FROM recipe_enrichment')) {
  const enrichment = JSON.parse(json);
if (!Array.isArray(enrichment)) {
    throw new Error('Invalid recipe enrichment data: expected an ingredients array');
}

for (const ingredient of enrichment) {
    if (typeof ingredient.name !== 'string' || !Array.isArray(ingredient.products)) {
      throw new Error('Invalid enriched ingredient data');
    }

    const key = ingredient.name.toLocaleLowerCase();
    let entry = ingredientsByKey.get(key);

    if (!entry) {
      entry = { name: ingredient.name, products: new Map() };
      ingredientsByKey.set(key, entry);
    }

    for (const product of ingredient.products) {
      if (typeof product.name !== 'string') {
        throw new Error(`Invalid product data for ingredient "${ingredient.name}"`);
      }

      const productKey = product.name.toLocaleLowerCase();
      const existing = entry.products.get(productKey);

      if (existing) {
        existing.cost_per_100g ??= product.cost_per_100g;
        existing.nutrition_per_100g ??= product.nutrition_per_100g;
      } else {
        entry.products.set(productKey, product);
      }
    }
  }
}

const collator = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: 'base',
});

const ingredients = [...ingredientsByKey.values()]
  .map((ingredient) => ({
    ...ingredient,
    products: [...ingredient.products.values()],
  }))
  .sort((a, b) => collator.compare(a.name, b.name));

let selectedKey = null;

function formatValue(value) {
  return Number.isFinite(value) ? value.toFixed(1) : '—';
}

const productColumns = document.getElementById('product-columns');

function renderProducts() {
  const ingredient = ingredientsByKey.get(selectedKey);

  productColumns.replaceChildren();
  productRows.replaceChildren();

  if (!ingredient) {
    heading.textContent = 'Select an ingredient';
    hint.textContent = 'Choose an ingredient to compare its products.';
    comparison.classList.add('hidden');
    return;
  }

  heading.textContent = ingredient.name;
  hint.textContent = 'Nutrition and cost are shown per 100 g.';

  const products = [...ingredient.products.values()];

  if (products.length === 0) {
    hint.textContent = 'No product data is available for this ingredient.';
    comparison.classList.add('hidden');
    return;
  }

  comparison.classList.remove('hidden');

  const headerRow = document.createElement('tr');
  const detailHeading = document.createElement('th');
  detailHeading.scope = 'col';
  detailHeading.textContent = 'Details';
  headerRow.append(detailHeading);

  for (const product of products) {
    const cell = document.createElement('th');
    cell.scope = 'col';
    cell.textContent = product.name;
    headerRow.append(cell);
  }
  productColumns.append(headerRow);

  const rows = [
    ['Cost / 100 g', (product) => product.cost_per_100g],
    ['Calories', (product) => product.nutrition_per_100g?.calories],
    ['Protein', (product) => product.nutrition_per_100g?.protein],
    ['Carbohydrates', (product) => product.nutrition_per_100g?.carbohydrates],
    ['Fat', (product) => product.nutrition_per_100g?.fat],
    ['Sodium', (product) => product.nutrition_per_100g?.sodium],
  ];

  for (const [label, getValue] of rows) {
    const row = document.createElement('tr');
    const rowHeading = document.createElement('th');
    rowHeading.scope = 'row';
    rowHeading.textContent = label;
    row.append(rowHeading);

    for (const product of products) {
      const cell = document.createElement('td');
      cell.textContent = formatValue(getValue(product));
      row.append(cell);
    }

    productRows.append(row);
  }
}

function renderIngredients() {
  const term = search.value.trim().toLocaleLowerCase();
  const matches = ingredients.filter((ingredient) =>
    ingredient.name.toLocaleLowerCase().includes(term),
  );

  count.textContent = `${matches.length} ingredient${matches.length === 1 ? '' : 's'}`;
  list.replaceChildren();

  if (matches.length === 0) {
    list.textContent = 'No ingredients found.';
    return;
  }

  for (const ingredient of matches) {
    const key = ingredient.name.toLocaleLowerCase();
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'ingredient-option';
    button.textContent = ingredient.name;
    button.setAttribute('aria-pressed', String(key === selectedKey));
    button.addEventListener('click', () => {
      selectedKey = key;
      renderIngredients();
      renderProducts();
    });
    list.append(button);
  }
}

search.addEventListener('input', renderIngredients);

renderIngredients();
renderProducts();
