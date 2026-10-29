export async function mountSiteHeader() {
  const HEADER_URL = new URL('../site-header.html', import.meta.url);
  const host = document.getElementById('site-header');
  if (!host) throw new Error('Missing #site-header placeholder');

  const applyTheme = (theme) => {
    document.documentElement.classList.toggle('dark', theme === 'dark');
    localStorage.setItem('theme', theme);
  };

  const savedTheme = localStorage.getItem('theme')
    || (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');

  document.documentElement.classList.toggle('dark', savedTheme === 'dark');

  window.toggleTheme = () => {
    applyTheme(document.documentElement.classList.contains('dark') ? 'light' : 'dark');
  };
  host.innerHTML = await (await fetch(HEADER_URL)).text();
  host.querySelector('#theme-toggle').addEventListener('click', window.toggleTheme);
  host.querySelector('#shortcuts-help').addEventListener('click', () => {
    window.showShortcutsHelp();
  });
}
