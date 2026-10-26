import { openDb, query } from './db.js';
import { mountSiteHeader } from './site-header.js';
import { quantityText } from './scale.js';

mountSiteHeader();

const el = (tag, cls, text) => {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text !== undefined) e.textContent = text;
  return e;
};

const params = new URLSearchParams(location.search);
const slug = params.get('slug');
let factor = (() => { const n = parseFloat(params.get('scale')); return n > 0 && n <= 200 ? n : 1; })();
let enrichedIngredients = null;
const selectedProducts = new Map();

const nutritionFields = [
  ['calories', 'Calories'],
  ['protein', 'Protein'],
  ['carbohydrates', 'Carbohydrates'],
  ['sugar', 'Sugar'],
  ['fat', 'Fat'],
  ['saturated_fat', 'Saturated fat'],
  ['monounsaturated_fat', 'Monounsaturated fat'],
  ['polyunsaturated_fat', 'Polyunsaturated fat'],
  ['fibre', 'Fibre'],
  ['sodium', 'Sodium'],
];

function renderNutrition(data, scale) {
  const products = document.getElementById('nutrition-products');
  const table = document.getElementById('nutrition-table');
  const values = document.getElementById('nutrition-values');
  const total = Object.fromEntries(nutritionFields.map(([key]) => [key, 0]));

  products.replaceChildren(
    ...data
      .map((ingredient, index) => ({ ingredient, index }))
      .filter(({ ingredient }) => ingredient.products.length > 1)
      .map(({ ingredient, index }) => {
        const label = el('label', 'nutrition-product-row');
        label.append(el('span', 'nutrition-product-name', ingredient.name));
        const select = el('select', 'nutrition-product-select');
        select.dataset.ingredientIndex = index;
        ingredient.products.forEach((product, productIndex) => {
          const option = el('option', '', product.name);
          option.value = productIndex;
          option.selected = (selectedProducts.get(index) ?? 0) === productIndex;
          select.append(option);
        });
        label.append(select);
        return label;
      }),
  );

  for (const [index, ingredient] of data.entries()) {
    let grams = 0;
    let complete = true;
    for (const occurrence of ingredient.occurrences) {
      if (!Number.isFinite(occurrence.quantity_g)) {
        complete = false;
      } else {
        grams += occurrence.quantity_g;
      }
    }
    if (!complete) continue;

    const product = ingredient.products[selectedProducts.get(index) ?? 0];
    if (!product?.nutrition_per_100g) {
      continue;
    }

    const multiplier = grams * scale / 100;
    for (const [key] of nutritionFields) {
      total[key] += product.nutrition_per_100g[key] * multiplier;
    }
  }

  values.replaceChildren(
    ...nutritionFields.map(([key, label]) => {
      const row = el('tr');
      const heading = el('th', '', label);
      heading.scope = 'row';
      row.append(heading, el('td', '', total[key].toFixed(2)));
      return row;
    }),
  );

  document.getElementById('nutrition-status').classList.add('hidden');
  table.classList.remove('hidden');
}

function stepEl(recipe, step, f) {
  const body = el('div', 'step-body mb-2');
  const refs = [];
  for (const item of step.items) {
    switch (item.type) {
      case 'text': body.append(item.value); break;
      case 'ingredient': {
        const ing = recipe.ingredients[item.index];
        const name = ing.alias ?? ing.name;
        body.append(el('span', 'ingredient-badge', name));
        const q = quantityText(ing.quantity, f);
        refs.push(q ? `${name}: ${q}` : name);
        break;
      }
      case 'cookware': {
        const c = recipe.cookware[item.index];
        body.append(el('span', 'cookware-badge', c.alias ?? c.name));
        break;
      }
      case 'timer':
        body.append(el('span', 'timer-badge', '⏱️ ' + quantityText(recipe.timers[item.index].quantity, f)));
        break;
      case 'inlineQuantity':
        body.append(el('span', 'font-bold text-accent-text', quantityText(recipe.inline_quantities[item.index], f)));
        break;
    }
  }
  const col = el('div', 'flex-1');
  col.append(body);
  if (refs.length) {
    const r = el('div', 'step-refs');
    refs.forEach((t) => r.append(el('span', 'inline-block mr-3', t)));
    col.append(r);
  }
  const row = el('div', 'flex gap-4');
  row.append(el('div', 'step-number', step.number), col);
  const li = el('li', 'step-box');
  li.append(row);
  return li;
}

function render(recipe, f) {
  if (enrichedIngredients) renderNutrition(enrichedIngredients, f);
  document.getElementById('ingredients').replaceChildren(
    ...recipe.ingredients
      .filter((i) => i.relation.relation.type === 'definition')
      .map((i) => {
        const name = el('div', 'flex-1 flex items-center');
        const position = recipe.ingredients.indexOf(i);

        if (i.reference) {
          const linkedSlug = linkedRecipeSlugs.get(position) ?? i.name;

          const checkbox = document.createElement('input');
          checkbox.type = 'checkbox';
          checkbox.checked = true;
          checkbox.className = 'ref-checkbox accent-[var(--accent)] mr-2 shrink-0';
          checkbox.dataset.refPath = linkedSlug;
          checkbox.title = 'Include in shopping list';

          const link = el(
            'a',
            'font-medium text-info hover:underline flex items-center gap-1',
            i.alias ?? i.name,
          );
          link.href = `recipe.html?slug=${encodeURIComponent(linkedSlug)}`;

          const icon = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
          icon.setAttribute('class', 'w-4 h-4');
          icon.setAttribute('fill', 'none');
          icon.setAttribute('stroke', 'currentColor');
          icon.setAttribute('viewBox', '0 0 24 24');
          icon.setAttribute('aria-hidden', 'true');

          const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
          path.setAttribute('stroke-linecap', 'round');
          path.setAttribute('stroke-linejoin', 'round');
          path.setAttribute('stroke-width', '2');
          path.setAttribute(
            'd',
            'M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1',
          );
          icon.append(path);
          link.prepend(icon);

          name.append(checkbox, link);
        } else {
          name.append(el('span', 'font-medium', i.alias ?? i.name));
        }

        const li = el('li', 'ingredient-row');
        li.append(name, el('span', 'row-value ml-2', quantityText(i.quantity, f)));
        return li;
      }),
  );
  document.getElementById('steps').replaceChildren(
    ...recipe.sections.flatMap((s) =>
      s.content.map((c) => {
        if (c.type === 'step') return stepEl(recipe, c.value, f);
        const li = el('li', 'recipe-note mb-3');
        li.append(el('span', '', '📝'), el('p', 'm-0', c.value));
        return li;
      }),
    ),
  );
  document.getElementById('scale').value = f;
  history.replaceState(null, '', `?slug=${encodeURIComponent(slug)}&scale=${f}`);
}

const db = await openDb();

// Added
let linkedRecipeSlugs = new Map();
const searchInput = document.getElementById('search-input');
const searchResults = document.getElementById('search-results');
let searchTimeout;

searchInput.addEventListener('input', () => {
    clearTimeout(searchTimeout);
    const term = searchInput.value.trim();

    if (term.length < 2) {
        searchResults.classList.add('hidden');
        return;
    }

    searchTimeout = setTimeout(() => {
        const like = `%${term}%`;
        const matches = query(db, `
          SELECT DISTINCT
              r.slug,
              COALESCE(NULLIF(r.title, ''), r.slug) AS title
          FROM recipes r
          LEFT JOIN recipe_tags t ON t.slug = r.slug
          LEFT JOIN recipe_ingredients ri ON ri.recipe_slug = r.slug
          LEFT JOIN recipe_data rd ON rd.slug = r.slug
          WHERE COALESCE(NULLIF(r.title, ''), r.slug) LIKE ? OR t.tag LIKE ? OR ri.ingredient_name LIKE ? OR rd.json LIKE ?
          ORDER BY title
          LIMIT 20`,
        [like, like, like, like]);

        searchResults.replaceChildren();

        if (matches.length === 0) {
            searchResults.append(el('div', 'p-4 text-muted text-center', 'No recipes found'));
        } else {
            for (const match of matches) {
                const link = el('a', 'search-result block px-4 py-3 hover:bg-sunk border-b border-line last:border-b-0', match.title);
                link.href = `recipe.html?slug=${encodeURIComponent(match.slug)}`;
                searchResults.append(link);
            }
        }

        searchResults.classList.remove('hidden');
    }, 300);
});

document.addEventListener('click', (event) => {
    if (!searchInput.contains(event.target) && !searchResults.contains(event.target)) {
        searchResults.classList.add('hidden');
    }
});
// End

let selectedSearchIndex = -1;

searchInput.addEventListener('input', () => {
    selectedSearchIndex = -1;
});

searchInput.addEventListener('keydown', (event) => {
    const links = searchResults.querySelectorAll('a');
    if (links.length === 0 || searchResults.classList.contains('hidden')) return;

    if (event.key === 'ArrowDown') {
        event.preventDefault();
        selectedSearchIndex = Math.min(selectedSearchIndex + 1, links.length - 1);
    } else if (event.key === 'ArrowUp') {
        event.preventDefault();
        selectedSearchIndex = Math.max(selectedSearchIndex - 1, 0);
    } else if (event.key === 'Enter' && selectedSearchIndex >= 0) {
        event.preventDefault();
        links[selectedSearchIndex].click();
        return;
    } else {
        return;
    }

    links.forEach((link, index) => {
        link.classList.toggle('search-selected', index === selectedSearchIndex);
    });
    links[selectedSearchIndex].scrollIntoView({ block: 'nearest' });
});

const [row] = query(db, 'SELECT json FROM recipe_data WHERE slug = ?', [slug]);
if (!row) {
  document.body.textContent = 'Recipe not found';
} else {
  const recipe = JSON.parse(row.json);
  linkedRecipeSlugs = new Map();

  for (const [position, ingredient] of recipe.ingredients.entries()) {
    if (!ingredient.reference) continue;

    const referenceName = ingredient.reference.name;
    const matches = query(
      db,
      `SELECT slug
       FROM recipes
       WHERE slug = ? COLLATE NOCASE
          OR title = ? COLLATE NOCASE`,
      [referenceName, referenceName],
    );

    if (matches.length === 1) {
      linkedRecipeSlugs.set(position, matches[0].slug);
    } else if (matches.length > 1) {
      throw new Error(`Linked recipe reference '${referenceName}' is ambiguous`);
    }
  }
  const [enrichmentRow] = query(
    db,
    'SELECT json FROM recipe_enrichment WHERE slug = ? COLLATE NOCASE',
    [slug],
  );
  if (enrichmentRow) {
    enrichedIngredients = JSON.parse(enrichmentRow.json);
  } else {
    const status = document.getElementById('nutrition-status');
    status.textContent = 'Nutrition data unavailable for this recipe.';
    status.classList.remove('hidden');
  }
  document.getElementById('nutrition-products').addEventListener('change', (event) => {
    if (event.target.matches('select[data-ingredient-index]')) {
      selectedProducts.set(
        Number(event.target.dataset.ingredientIndex),
        Number(event.target.value),
      );
      renderNutrition(enrichedIngredients, factor);
    }
  });
  const recipeMetadata = recipe.metadata?.map ?? {};
  const tags = document.getElementById('tags');
  const cuisine = document.getElementById('cuisine');

  const recipeTags = query(
    db,
    'SELECT tag FROM recipe_tags WHERE slug = ? ORDER BY tag',
    [slug],
  );
  for (const { tag } of recipeTags) {
      tags.append(el('span', 'tag', `#${tag}`));
  }

  if (recipeMetadata.cuisine) {
      cuisine.append(el('span', 'metadata-pill metadata-cuisine', `🌍 ${recipeMetadata.cuisine}`));
  }

  document.getElementById('title').textContent = recipe.metadata.map.title ?? slug;
  document.getElementById('breadcrumb-title').textContent = recipe.metadata.map.title ?? slug;
  const set = (n) => { if (n > 0 && n <= 200) { factor = n; render(recipe, factor); } };
  document.getElementById('minus').onclick = () => set(Math.max(0.5, factor - 0.5));
  document.getElementById('plus').onclick = () => set(factor + 0.5);
  document.getElementById('scale').onchange = (e) => set(parseFloat(e.target.value));
  render(recipe, factor);
}
