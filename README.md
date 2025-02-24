# zola (né Gutenberg)

[![Build Status](https://dev.azure.com/getzola/zola/_apis/build/status/getzola.zola?branchName=master)](https://dev.azure.com/getzola/zola/_build/latest?definitionId=1&branchName=master)
![GitHub all releases](https://img.shields.io/github/downloads/getzola/zola/total)

A fast static site generator in a single binary with everything built-in.

To find out more see the [Zola Documentation](https://www.getzola.org/documentation/getting-started/overview/), look
in the [docs/content](docs/content) folder of this repository or visit the [Zola community forum](https://zola.discourse.group).

This tool and its template engine [tera](https://keats.github.io/tera/) were born from an intense dislike of the (insane) Golang template engine and therefore of
Hugo that I was using before for 6+ sites.

# Fork information

This is a custom fork of Zola, with some additional features and changes. The main goal of this fork was to fit my own specific needs for my [blog](https://blog.cstef.dev), but then I figured they could be useful to others 😄.

I have opened a few [pull requests](https://github.com/getzola/zola/pulls?q=is%3Apr+author%3Acestef) containing some of the changes I made
to the upstream repository, but they have not all been merged yet. I will keep this fork up-to-date with the upstream repository, but I will not be actively maintaining it.

## Changes


| Feature                                                                         | Branch                                                                               | PR                                                                                   | Status        |
| ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------ | ------------- |
| Math typesetting with [Typst](https://typst.app) and [KaTeX](https://katex.org) | [feature/math-rendering](https://github.com/cestef/zola/tree/feature/math-rendering) | [#2791](https://github.com/getzola/zola/pull/2791)                                   | 🔄 In progress |
| `data-copy` fence setting                                                       | [feature/copy](https://github.com/cestef/zola/tree/feature/copy)                     | [#2805](https://github.com/getzola/zola/pull/2805)                                   | ✅ Merged      |
| `include=<filename>` fence setting                                              | [feature/include-code](https://github.com/cestef/zola/tree/feature/include-code)     | [#2797](https://github.com/getzola/zola/pull/2797)                                   | 🔄 In progress |
| Obsidian-style callouts                                                         | [feature/callouts](https://github.com/cestef/zola/tree/feature/callouts)             | [`pulldown-cmark` #1013](https://github.com/pulldown-cmark/pulldown-cmark/pull/1013) | 🛑 Not opened  |
| Version variable in Tera context                                                | -                                                                                    | [@extua #2793](https://github.com/getzola/zola/pull/2793)                            | 🔄 In progress |
| [Pikchr](https://pikchr.org) support for diagrams                               | [blog](https://github.com/cestef/zola/tree/blog)                                     | -                                                                                    | 🛑 Not opened  |

<!-- - [ ] [Add support for math typesetting with Typst and KaTeX](https://github.com/getzola/zola/pull/2791) -->

# List of features

- [Single binary](https://www.getzola.org/documentation/getting-started/cli-usage/)
- [Syntax highlighting](https://www.getzola.org/documentation/content/syntax-highlighting/)
- [Sass compilation](https://www.getzola.org/documentation/content/sass/)
- Assets co-location
- [Multilingual site support](https://www.getzola.org/documentation/content/multilingual/) (Basic currently)
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
