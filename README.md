
# CookDB

Recipe parsing, SQLite storage, and static website generation for Cooklang recipes.

> Documentation is being adapted from the original project and is currently a work in progress.

<!--
[![Test](https://github.com/cooklang/cookcli/actions/workflows/test.yml/badge.svg)](https://github.com/cooklang/cookcli/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub release (latest by date)](https://img.shields.io/github/v/release/cooklang/CookCLI)](https://github.com/cooklang/CookCLI/releases)
[![GitHub Stars](https://img.shields.io/github/stars/cooklang/CookCLI?style=social)](https://github.com/cooklang/CookCLI)

**Command line tools for working with [Cooklang](https://cooklang.org) recipes.** Full reference: [CookCLI documentation](https://cooklang.org/cli/).

## 📋 Table of Contents

- [Placeholder](#-what-is-cookcli)

## 📖 What is CookCLI?

CookCLI provides a suite of commands to create shopping lists,
reports and maintain recipes. We've built it to be simple and useful
for automating your cooking and shopping routine with existing
UNIX command line and scripting tools. It can also function
as a webserver for your recipes, making them browsable on
any device with a web browser.

**Try the web interface live at [demo.cooklang.org](https://demo.cooklang.org)**

### ✨ Features

* 🔧 **UNIX Philosophy** - Integrate your Cooklang recipes with other tools
* 📝 **Meal Planning** - Script meal planning and shopping
* 🌐 **Web Interface** - Browse recipes from any device
* 🛒 **Smart Shopping** - Generate organized shopping lists
* 🔍 **Recipe Search** - Find recipes quickly
* 📦 **Pantry Tracking** - Manage inventory and expiration dates

Plan your meals with a menu file (`Weekly Plan.menu`):

```
---
servings: 2
---

==Saturday==

Breakfast: \
- @oats{1%cup} with @milk{1/2%cup} and @honey{1%tbsp}

Lunch: \
- @./lamb-chops{}

Dinner: \
- @./Neapolitan Pizza{}

==Sunday==

Breakfast: \
- @./Breakfast/Easy Pancakes{} with @maple syrup{2%tbsp}

Lunch: \
- @./Salads/Caprese{}

Dinner: \
- @./Risotto{}
```

Menu files let you reference recipes and inline ingredients, plan by day and meal, and generate a combined shopping list for the whole plan.

### Build from Source

You'll need Rust and Node.js installed. Then:

```bash
# Clone the repository
git clone https://github.com/Notorious544d/cookdb.git
cd CookCLI

# Install frontend dependencies
npm install

# Build CSS and JS (required for web UI; the build fails without them)
npm run build-css
npm run build-js

# Build the CLI with web UI
cargo build --release

# Binary will be at target/release/cook
```


## 📚 Documentation

Detailed documentation for each command is available in the [docs/](docs/) directory:

* [Recipes](docs/recipe.md) - placeholder

```

### Aisle Configuration (`aisle.conf`)

Organizes ingredients by store section for shopping lists. Items not in any category will appear under "Other".

```
// In the order you walk the store
[produce]
tomatoes|tomato
basil|basil leaves // fresh, not dried
garlic
onions

[dairy]
milk
cheese
yogurt
butter

[pantry]
flour
sugar
pasta
rice
olive oil

[meat]
chicken
beef
pork

[bakery]
bread
rolls
```

### Combining with UNIX Tools

CookCLI works great with pipes and standard tools:

```bash
# Find all recipes with chicken and create a shopping list
cook search chicken | xargs cook shopping-list

# Convert all recipes to Markdown
for r in *.cook; do
  cook recipe "$r" -f markdown > "docs/${r%.cook}.md"
done
```

## ⭐ Give us a star

Why not? It will help more people discover this tool and Cooklang.

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Areas where we'd love help:

* Bug fixes and testing
* Documentation improvements
* New features
* Recipe collections
* Translations
* UI/UX improvements

-->

## 👥 Contributors

Thanks to all the people who have contributed to CookDB!

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<!--
<a href="https://github.com/cooklang/CookCLI/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=cooklang/CookCLI" />
</a>
-->
<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->
<!-- ALL-CONTRIBUTORS-LIST:END -->

## 📄 License

MIT License. See [LICENSE](LICENSE) for details.

Some source files include code from [CookCLI](https://github.com/cooklang/cookcli), also under MIT license.  
Some source files include code from [cooklang-chef](https://github.com/Zheoni/cooklang-chef), also under MIT license.

## 🔗 Links

<!-- * [CookCLI documentation](https://cooklang.org/cli/) - every command, the server, and guides -->
* [Cooklang Specification](https://cooklang.org/docs/spec) - the recipe markup language
* [Cooklang Apps](https://cooklang.org/app/) - iOS and Android apps

## 💬 Support

<!-- * [Issue Tracker](https://github.com/cooklang/CookCLI/issues) - report bugs -->
* [Twitter](https://twitter.com/cooklang_org) - updates and news
* [Playground](https://cooklang.github.io/cooklang-rs/)
* [Discord server](https://discord.gg/fUVVvUzEEK), ask for help or say hello fellow cooks
* [Spec discussions](https://github.com/cooklang/spec/discussions), suggest a new idea or give your opinion on future development
* [Awesome Cooklang Recipes](https://github.com/cooklang/awesome-cooklang-recipes), find inspiration or share your recipes with the community.
