import { query } from './db.js';

const SEARCH_SQL = await fetch(new URL('../search.sql', import.meta.url)).then((r) => {
  if (!r.ok) throw new Error(`search.sql: HTTP ${r.status}`);
  return r.text();
});

export function searchRecipes(db, term) {
  return query(db, SEARCH_SQL, [`%${term.trim()}%`]);
}

const ITEM_CLASS =
  'search-result block px-4 py-3 hover:bg-sunk border-b border-line last:border-b-0';

export function mountRecipeSearch(db) {
  const input = document.getElementById('search-input');
  const results = document.getElementById('search-results');
  if (!input || !results) return;

  let timer;
  let selected = -1;

  const hide = () => {
    results.classList.add('hidden');
    selected = -1;
  };

  const render = (matches) => {
    results.replaceChildren();
    if (matches.length === 0) {
      const empty = document.createElement('div');
      empty.className = 'p-4 text-muted text-center';
      empty.textContent = 'No recipes found';
      results.append(empty);
    } else {
      for (const { slug, title } of matches) {
        const link = document.createElement('a');
        link.className = ITEM_CLASS;
        link.textContent = title;
        link.href = `recipe.html?slug=${encodeURIComponent(slug)}`;
        results.append(link);
      }
    }
    results.classList.remove('hidden');
  };

  input.addEventListener('input', () => {
    clearTimeout(timer);
    selected = -1;
    const term = input.value.trim();
    if (term.length < 2) return hide();

    timer = setTimeout(() => render(searchRecipes(db, term).slice(0, 20)), 150);
  });

  input.addEventListener('keydown', (event) => {
    const links = results.querySelectorAll('a');
    if (links.length === 0 || results.classList.contains('hidden')) return;

    if (event.key === 'ArrowDown') {
      event.preventDefault();
      selected = Math.min(selected + 1, links.length - 1);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      selected = Math.max(selected - 1, 0);
    } else if (event.key === 'Enter' && selected >= 0) {
      event.preventDefault();
      links[selected].click();
      return;
    } else {
      return;
    }

    links.forEach((link, i) => link.classList.toggle('search-selected', i === selected));
    links[selected].scrollIntoView({ block: 'nearest' });
  });

  document.addEventListener('click', (event) => {
    if (!input.contains(event.target) && !results.contains(event.target)) hide();
  });
}