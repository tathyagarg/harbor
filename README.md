# Harbor Browser
![Banner](.github/assets/banner-16_9.png)

![Language Badge](https://shields.arson.dev/badge/Language-Rust%2C%20Zig-red?color=#ff0000)
![Hackatime Badge](https://hackatime-badge.hackclub.com/U082L0UTJ66/harbor?color=#80ff00)
![License Badge](https://shields.arson.dev/github/license/tathyagarg/harbor?color=#00ffff)
![Lines Badge](https://shields.arson.dev/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2Ftathyagarg%2Fharbor%2Frefs%2Fheads%2Fmain%2Flines.json&query=%24.lines&label=Lines&color=%238000ff)

⭐ Star us on GitHub — it motivates us a lot!

[Harbor](https://arson.dev/harbor) is a web browser built using Rust and Zig, made to teach me the ins and outs of browser development. It's not meant to be a competitor to mainstream browsers, but rather a personal project to explore the complexities of web technologies and browser architecture.

**Every last pixel is rendered by me, and every line of code is written from scratch.**

> [!IMPORTANT]
> If you don't want to build/download the browser, that's fine! Please check out the [screenshots](#screenshots) section to see the browser in action.

## Table of Contents

- [Install](#install)
- [Screenshots](#screenshots)
  - [New Tab page](#new-tab-page)
  - [Flavorless](#flavorless)
  - [Sans Style](#sans-style)
- [Components](#components)
  - [HTTP Client](#http-client) 
  - [HTML Parser](#html-parser)
  - [CSS Parser](#css-parser)
  - [TTF Parser](#ttf-parser)
  - [JavaScript Engine](#javascript-engine)
  - [Renderer](#renderer)

### Install

#### Linux and MacOS
Run the following command in your terminal in a directory of your choice:
```bash
curl -fsSL https://raw.githubusercontent.com/tathyagarg/harbor/main/scripts/install.sh | sh
chmod +x harbor
./harbor
```

OR you can clone the repository and build it yourself:
> [!NOTE]
> You require both [Rust](https://rust-lang.org/) and [Zig](https://ziglang.org/) installed to build the browser from source.
```bash
git clone https://github.com/tathyagarg/harbor.git
cd harbor/harbor/engine
cargo run --release
```

### Screenshots

#### New Tab page

<div align="center">
  <img src="https://raw.githubusercontent.com/tathyagarg/harbor/main/.github/assets/screenshots/tab.png" alt="New Tab Page" width="80%">
  <br/>
  <em>The default (and very bare bones) new tag page</em>
</div>

#### Flavorless

<div align="center">
  <img src="https://raw.githubusercontent.com/tathyagarg/harbor/main/.github/assets/screenshots/flavorless.png" alt="Flavorless" width="80%">
  <br/>
  <em>The website for the <a href="https://flavorless.hackclub.com">flavorless</a> YSWS</em>
</div>

#### Sans Style

<div align="center">
  <img src="https://raw.githubusercontent.com/tathyagarg/harbor/main/.github/assets/screenshots/sans-style.png" alt="Sans Style" width="80%">
  <br/>
  <em>The website <a href="https://sans.style/">sans.style</a></em>
</div>

### Components
Harbor is comprised of 6 main components:

#### HTTP Client
The HTTP client is responsible for making network requests to fetch web resources. It handles the complexities of the HTTP protocol, including headers, and redirects. This component is the only one that uses external libraries (except for the renderer which uses wgpu and winit) - and that is for `TLS` support (enabling `https` support).

#### HTML Parser
The HTML parser takes the raw HTML content fetched by the HTTP client and parses it into a structured format, the DOM (Document Object Model). This allows the browser to understand the structure of the webpage and how elements relate to each other.

#### CSS Parser
The CSS parser processes the CSS stylesheets associated with the webpage. It parses the CSS rules and applies them to the corresponding HTML elements in the DOM, determining how each element should be styled. Currently only the following CSS properties are supported:
- `color`
- `background-color`
- `width`
- `position`
- `top`
- `left`
- `right`
- `bottom`
- `display`
- `font-size`
- `font-family`
- `font-weight`
- `font-style`

#### TTF Parser
The TTF (TrueType Font) parser is responsible for parsing font files to extract glyph information. This allows the browser to render text using the correct fonts specified in the CSS. This enables me to render text manually from glyphs, which is a fun challenge.

#### JavaScript Engine
The JavaScript engine executes JavaScript code embedded in web pages. WIP

#### Renderer
The renderer takes the structured data from the HTML and CSS parsers and renders it onto the screen. It uses the `wgpu` library for GPU-accelerated rendering and `winit` for window management. The renderer is responsible for drawing all visual elements of the webpage.
