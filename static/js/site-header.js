export function mountSiteHeader() {
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

  host.innerHTML = `
    <nav class="nav-card relative">
      <div class="px-3 lg:px-6 py-4">
        <div class="flex items-center justify-between flex-wrap gap-y-3">
          <a href="index.html" class="flex items-center mr-2" aria-label="Recipes home">
            <img src="images/android-chrome-192x192.png"
                 alt="Cook"
                 class="brand-mark h-8 w-8 rounded-md"
                 width="32"
                 height="32">
          </a>

          <div class="relative z-50 flex-1 min-w-0 mr-2 md:mr-4">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg class="h-5 w-5 text-faint" fill="none" stroke="currentColor"
                   viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
              </svg>
            </div>
            <label for="search-input" class="sr-only">Search recipes</label>
            <input id="search-input" type="search"
                   placeholder="Search recipes..." class="search-input">
            <div id="search-results"
                 class="absolute top-full mt-2 w-full card shadow-[var(--shadow-overlay)] hidden max-h-96 overflow-y-auto"
                 style="z-index: 9999;"></div>
          </div>

          <div class="flex items-center gap-1 lg:gap-2">
            <button id="shortcuts-help" type="button" class="icon-btn ml-2"
                    aria-label="Keyboard shortcuts" title="Keyboard shortcuts (?)">
              <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
            </button>

            <button id="theme-toggle" type="button" class="icon-btn ml-2"
                    aria-label="Toggle light/dark theme" title="Toggle light/dark theme">
              <svg class="hidden dark:block" fill="currentColor"
                   viewBox="0 0 20 20" aria-hidden="true">
                <path fill-rule="evenodd"
                      d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 100 2h1z"
                      clip-rule="evenodd"></path>
              </svg>
              <svg class="block dark:hidden" fill="currentColor"
                   viewBox="0 0 20 20" aria-hidden="true">
                <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"></path>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </nav>
  `;

  host.querySelector('#theme-toggle').addEventListener('click', window.toggleTheme);
  host.querySelector('#shortcuts-help').addEventListener('click', () => {
    window.showShortcutsHelp();
  });
}
