import { openDb, query } from './db.js';
import { mountRecipeSearch } from './search.js';
import { mountSiteHeader } from './site-header.js';
import init, { ScalableRecipe } from '../wasm/cookdb_wasm.js';

await mountSiteHeader();

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

function stepEl(recipe, step, t) {
  const body = el('div', 'step-body mb-2');
  const refs = [];
  for (const item of step.items) {
    switch (item.type) {
      case 'text': body.append(item.value); break;
      case 'ingredient': {
        const ing = recipe.ingredients[item.index];
        const name = ing.alias ?? ing.name;
        body.append(el('span', 'ingredient-badge', name));
        const amount = t.ingredients[item.index];
        refs.push(amount ? `${name}: ${amount}` : name);
        break;
      }
      case 'cookware': {
        const c = recipe.cookware[item.index];
        body.append(el('span', 'cookware-badge', c.alias ?? c.name));
        break;
      }
      case 'timer':
        body.append(el('span', 'timer-badge', '⏱️ ' + t.timers[item.index]));
        break;
      case 'inlineQuantity':
        body.append(el('span', 'font-bold text-accent-text', t.inline[item.index]));
        break;
    }
  }
  const col = el('div', 'flex-1');
  col.append(body);
  if (refs.length) {
    const r = el('div', 'step-refs');
    refs.forEach((text) => r.append(el('span', 'inline-block mr-3', text)));
    col.append(r);
  }
  const row = el('div', 'flex gap-4');
  row.append(el('div', 'step-number', step.number), col);
  const li = el('li', 'step-box');
  li.append(row);
  return li;
}

function render(recipe, f) {
  const t = JSON.parse(scalable.scaled_text(f, 'original'));
  updateScaleNotice(f);

  const cookware = recipe.cookware.filter((c) => c.relation.type === 'definition');
  document.getElementById('cookware-card').hidden = cookware.length === 0;
  document.getElementById('cookware').replaceChildren(
    ...cookware.map((c) => {
      const li = el('li', 'ingredient-row');
      li.append(el('span', 'font-medium', c.alias ?? c.name));
      return li;
    }),
  );
  
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
        li.append(name, el('span', 'row-value ml-2', t.ingredients[position]));
        return li;
      }),
  );
  document.getElementById('steps').replaceChildren(
    ...recipe.sections.flatMap((s) =>
      s.content.map((c) => {
        if (c.type === 'step') return stepEl(recipe, c.value, t);
        const li = el('li', 'recipe-note mb-3');
        li.append(el('span', '', '📝'), el('p', 'm-0', c.value));
        return li;
      }),
    ),
  );
  document.getElementById('scale').value = Number(((baseServings ?? 1) * f).toFixed(2));
  history.replaceState(null, '', `?slug=${encodeURIComponent(slug)}&scale=${f}`);
}

// Added
const minutes = (m) =>
  m >= 60 ? `${Math.floor(m / 60)}h${m % 60 ? ` ${m % 60}m` : ''}` : `${m} min`;

const hostname = (url) => {
  try { return new URL(url).hostname.replace(/^www\./, ''); } catch { return url; }
};

function pill(kind, ...content) {
  const span = el('span', `metadata-pill metadata-${kind}`);
  span.append(...content);
  return span;
}

function sourcePill({ source_name: name, source_url: url }) {
  let href = null;
  try {
    const u = new URL(url);
    if (u.protocol === 'http:' || u.protocol === 'https:') href = u.href;
  } catch { /* no URL, or not a valid one */ }
  if (!href) return pill('source', `📖 ${name ?? url}`);
  const link = el('a', 'text-info hover:underline ml-1', name ?? hostname(href));
  link.href = href;
  link.target = '_blank';
  link.rel = 'noopener noreferrer';
  return pill('source', '📖', link);
}

function metadataPills(row, extra) {
  const pills = [];
  if (row.cuisine) pills.push(pill('cuisine', `🌍 ${row.cuisine}`));

  if (row.servings_count != null) {
    pills.push(pill('servings', `👥 ${row.servings_count} servings`));
  } else if (row.servings_text) {
    pills.push(pill('servings', `👥 ${row.servings_text}`));
  }
  scaleNoticeEl = el('a', 'metadata-pill metadata-original');
  scaleNoticeEl.href = `recipe.html?slug=${encodeURIComponent(slug)}`;
  scaleNoticeEl.title =
    'Quantities are scaled from the original recipe. Cooking times, pan sizes and seasoning may need adjusting. Select to go back to the original.';
  pills.push(scaleNoticeEl);

  if (row.prep_minutes != null || row.cook_minutes != null) {
    if (row.prep_minutes != null) pills.push(pill('prep', `⏱️ Prep: ${minutes(row.prep_minutes)}`));
    if (row.cook_minutes != null) pills.push(pill('cook', `🔥 Cook: ${minutes(row.cook_minutes)}`));
  } else if (row.total_minutes != null) {
    pills.push(pill('time', `⏱️ ${minutes(row.total_minutes)}`));
  }
  if (row.difficulty) pills.push(pill('difficulty', `📊 ${row.difficulty}`));
  for (const [key, icon] of [['course', '🍽️'], ['diet', '🥗'], ['author', '👤']]) {
    if (typeof extra[key] === 'string') pills.push(pill(key, `${icon} ${extra[key]}`));
  }
  if (row.source_name || row.source_url) pills.push(sourcePill(row));
  return pills;
}

let scaleNoticeEl;

function updateScaleNotice(f) {
  scaleNoticeEl.style.display = f === 1 ? 'none' : '';
  scaleNoticeEl.textContent = baseServings != null
    ? `📖 Written for ${baseServings} ${baseServings === 1 ? 'serving' : 'servings'}`
    : `📖 ×${Number(f.toFixed(2))} of the original recipe`;
}

const db = await openDb();
await init();

let linkedRecipeSlugs = new Map();
let scalable;
mountRecipeSearch(db);

const [row] = query(
  db,
  `SELECT d.json, r.*
   FROM recipe_data d JOIN recipes r ON r.slug = d.slug
   WHERE d.slug = ?`,
  [slug],
);
const baseServings = row?.servings_count ?? null;
if (!row) {
  document.body.textContent = 'Recipe not found';
} else {
  const recipe = JSON.parse(row.json);
  scalable = new ScalableRecipe(row.json);
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
  const tags = document.getElementById('tags');
  const recipeTags = query(
    db,
    'SELECT tag FROM recipe_tags WHERE slug = ? ORDER BY tag',
    [slug],
  );
  for (const { tag } of recipeTags) {
      tags.append(el('span', 'tag', `#${tag}`));
  }

  const recipeMetadata = recipe.metadata?.map ?? {};
  document.getElementById('metadata').append(...metadataPills(row, recipeMetadata));
  document.getElementById('title').textContent = recipeMetadata.title ?? slug;
  document.getElementById('breadcrumb-title').textContent = recipeMetadata.title ?? slug;
  
  if (baseServings != null) {
    document.querySelector('label[for="scale"]').textContent = 'Servings:';
  }
  const set = (n) => { if (n > 0 && n <= 200) { factor = n; render(recipe, factor); } };
  const unit = baseServings ?? 1; // one stepper unit: a serving, or ×1
  const move = (delta) => set(Math.max(0.5, factor * unit + delta) / unit);
  document.getElementById('minus').onclick = () => move(-0.5);
  document.getElementById('plus').onclick = () => move(0.5);
  document.getElementById('scale').onchange = (e) => set(parseFloat(e.target.value) / unit);

  render(recipe, factor);
}
