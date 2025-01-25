# zola (né Gutenberg)

[![Build Status](https://dev.azure.com/getzola/zola/_apis/build/status/getzola.zola?branchName=master)](https://dev.azure.com/getzola/zola/_build/latest?definitionId=1&branchName=master)
![GitHub all releases](https://img.shields.io/github/downloads/getzola/zola/total)

A fast static site generator in a single binary with everything built-in.

To find out more see the [Zola Documentation](https://www.getzola.org/documentation/getting-started/overview/), look
in the [docs/content](docs/content) folder of this repository or visit the [Zola community forum](https://zola.discourse.group).

This tool and its template engine [tera](https://keats.github.io/tera/) were born from an intense dislike of the (insane) Golang template engine and therefore of Hugo that I was using before for 6+ sites.

---

## About This Fork

This fork of [Zola](https://github.com/getzola/zola) introduces additional features designed to support specific use cases, including:

- **Math server-side rendering** for better performance and accessibility.
- **Diagram server-side rendering** to integrate visual elements directly into the build process.
- **Custom callouts** for enhanced content styling.
- **Caching enhancements** to improve build efficiency.
- Other tweaks specific to my workflow and preferences.

### Why a Fork?

While Zola is an excellent static site generator, my use cases required features that diverge from its core design philosophy. I considered contributing these changes back to the main repository but decided against it because:

1. **Highly Opinionated Changes:** My additions align closely with my specific requirements and might not fit Zola's broad audience or maintainers' vision.
2. **Complexity and Scope:** Some features, like server-side math rendering, add dependencies or complexity that may not align with Zola's lightweight philosophy.

### Contributions and Feedback

Although I opted to maintain this fork independently, I deeply respect the work of the Zola community. If any part of this fork aligns with the project's goals or sparks interest, I'm happy to collaborate, share code, or refine my changes for a potential pull request.

---

# List of features

- [Single binary](https://www.getzola.org/documentation/getting-started/cli-usage/)
- [Syntax highlighting](https://www.getzola.org/documentation/content/syntax-highlighting/)
- [Sass compilation](https://www.getzola.org/documentation/content/sass/)
- Assets co-location
- [Multilingual site suport](https://www.getzola.org/documentation/content/multilingual/) (Basic currently)
- [Image processing](https://www.getzola.org/documentation/content/image-processing/)
- [Themes](https://www.getzola.org/documentation/themes/overview/)
- [Shortcodes](https://www.getzola.org/documentation/content/shortcodes/)
- [Internal links](https://www.getzola.org/documentation/content/linking/)
- [External link checker](https://www.getzola.org/documentation/getting-started/cli-usage/#check)
- [Table of contents automatic generation](https://www.getzola.org/documentation/content/table-of-contents/)
- Automatic header anchors
- [Aliases](https://www.getzola.org/documentation/content/page/#front-matter)
- [Pagination](https://www.getzola.org/documentation/templates/pagination/)
- [Custom taxonomies](https://www.getzola.org/documentation/templates/taxonomies/)
- [Search with no servers or any third parties involved](https://www.getzola.org/documentation/content/search/)
- [Live reload](https://www.getzola.org/documentation/getting-started/cli-usage/#serve)
- Deploy on many platforms easily: [Netlify](https://www.getzola.org/documentation/deployment/netlify/), [Vercel](https://www.getzola.org/documentation/deployment/vercel/), [Cloudflare Pages](https://www.getzola.org/documentation/deployment/cloudflare-pages/), etc
