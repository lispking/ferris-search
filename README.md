# ferris-search 🦀

> A blazing-fast multi-engine web search CLI & MCP server, written in Rust.

## Why ferris-search?

Claude Code's built-in web search works great in ideal network conditions — but in practice, many developers run into environments where it's unreliable or unavailable: corporate networks, restricted regions, air-gapped setups, or simply spotty connectivity.

While looking for a workaround, I came across [open-webSearch](https://github.com/Aas-ee/open-webSearch), a Node.js MCP server that routes search queries through multiple engines. It solved the problem well. But I have a thing for Rust — and spinning up a Node.js runtime just to proxy a few HTTP requests felt heavier than it needed to be.

So I rewrote the same idea in Rust:

- **No Node.js runtime** — single self-contained binary, ~8 MB
- **Lower latency** — Rust async I/O, concurrent fan-out across engines
- **Smaller footprint** — negligible memory usage
- **Proxy support** — HTTP/SOCKS5 proxy via env var, for networks that need it

If Claude Code's search isn't working in your environment, this is for you.

## Enterprise & Internal Use

ferris-search is also a good foundation for **enterprise internal search** scenarios. Since it's a standard MCP server written in Rust, you can fork it and add custom search engines that connect to your internal knowledge bases — Confluence, Notion, internal wikis, code repositories, or proprietary document stores.

Some ideas:

- Add an engine that searches your internal Elasticsearch or OpenSearch cluster
- Integrate with your company's Confluence or GitLab search API
- Connect to a private RAG (Retrieval-Augmented Generation) service
- Route queries to different backends based on query language or topic

With Claude Code as the AI layer and ferris-search as the search backbone, your team gets a local AI coding assistant that can actually find and reference internal documentation — without sending anything to external search engines.

## Features

- **CLI + MCP dual mode** — use directly in the terminal or as an MCP server for Claude Desktop / Cursor / Claude Code
- **Multi-engine fan-out** — search across multiple engines simultaneously with a single call
- **14 search engines** — Bing, DuckDuckGo, Brave, Baidu, CSDN, Juejin, Exa, Firecrawl, Zhihu, LinuxDo, Jina, Tavily, GitHub (repo search), GitHub Code (code search)
- **7 MCP tools** — `web_search` + 6 article/content fetchers
- **No API keys required** for most engines (Brave, Exa, Firecrawl, Jina, and Tavily require API keys)
- **Single binary** — ~8 MB, no runtime dependencies
- **Proxy support** — HTTP/SOCKS5 proxy via env var
- **Text & JSON output** — `--format text` (default) or `--format json` for scripting
- **Local document indexing** — full-text search over local Markdown, Text, HTML, and PDF files via Tantivy

## Quick Install

### Linux / macOS (one-liner)

```bash
curl -fsSL https://raw.githubusercontent.com/lispking/ferris-search/main/install.sh | bash
```

Or from a local clone:

```bash
bash install.sh
```

The script will:

1. Build and install the binary via `cargo install`
2. Register the MCP server with Claude Code (`claude mcp add -s user`) if the CLI is found
3. Install Claude Code skills for ferris-search

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/lispking/ferris-search/main/install.ps1 | iex
```

Or from a local clone:

```powershell
.\install.ps1
```

> **Prerequisite:** [Rust](https://www.rust-lang.org/tools/install) must be installed before running either script.

---

## Quick Start

### Install from source

```bash
cargo install --path .
```

This installs the `ferris-search` binary to `~/.cargo/bin/ferris-search`. Make sure `~/.cargo/bin` is in your `PATH`.

To find the installed binary path:

```bash
which ferris-search
# or
echo "$(cargo home 2>/dev/null || echo $HOME/.cargo)/bin/ferris-search"
```

### CLI Usage

Once installed, you can use ferris-search directly from the terminal:

**Search the web:**

```bash
# Single engine (uses default engine from env, or bing)
ferris-search search "rust async runtime"

# Specify engine(s)
ferris-search search "rust async runtime" --engine bing
ferris-search search "rust async runtime" --engine bing,duckduckgo
ferris-search search "rust async runtime" -e bing -e duckduckgo

# Limit results and output JSON
ferris-search search "rust async runtime" -e bing --limit 3 --format json
```

**Fetch web content:**

```bash
# Auto-detects the best fetcher based on URL
ferris-search fetch https://github.com/nickel-org/nickel.rs
ferris-search fetch https://example.com --max-chars 5000
ferris-search fetch https://example.com --format json
```

Supported domains with specialized extraction: `github.com` (README), `csdn.net`, `juejin.cn`, `zhihu.com`, `linux.do`.

**List engines & show config:**

```bash
ferris-search list-engines
ferris-search list-engines --format json

ferris-search show-config
ferris-search show-config --format json
```

**Start MCP server explicitly:**

```bash
ferris-search mcp
```

**Index and search local documents:**

```bash
# Build a full-text index from a directory (supports md, txt, html, pdf)
ferris-search index-local --path ./docs
ferris-search index-local --path ./docs --path ./notes

# Search the local index
ferris-search search-local "async runtime"
ferris-search search-local "async runtime" --limit 5 --format json

# Use a custom index directory
ferris-search index-local --path ./docs --index-path ./my-index
ferris-search search-local "query" --index-path ./my-index
```

> **PDF note:** PDF indexing extracts text using the PDF's text layer. Image-only or scanned PDFs without embedded text are not OCRed and will usually produce poor or empty results.
> **Note:** When stdin is not a TTY (e.g., piped by Claude Desktop), ferris-search automatically enters MCP mode — no configuration change needed.
>
> **Clarification:** MCP is not deprecated in ferris-search. The project now supports both CLI and MCP. What changed is that the old transport-selection env vars (`MODE`, `ENABLE_HTTP_SERVER`) were removed in favor of stdio MCP mode plus automatic TTY detection.

### Claude Desktop / Cursor configuration

```json
{
  "mcpServers": {
    "ferris-search": {
      "command": "/Users/<you>/.cargo/bin/ferris-search",
      "env": {
        "DEFAULT_SEARCH_ENGINE": "bing"
      }
    }
  }
}
```

Replace the path with the output of `which ferris-search`.

> The binary auto-detects piped stdin and enters MCP mode, so existing configurations continue to work without adding the `mcp` subcommand.

### Claude Code (claude mcp add)

Add for the current project only:

```bash
claude mcp add ferris-search $(which ferris-search)
```

Add globally for all projects (`-s user`):

```bash
claude mcp add -s user ferris-search $(which ferris-search)
```

With environment variables:

```bash
claude mcp add -s user ferris-search $(which ferris-search) \
  -e DEFAULT_SEARCH_ENGINE=bing \
  -e ALLOWED_SEARCH_ENGINES=bing,duckduckgo,brave
```

> `-s user` registers the server in your user-level config (`~/.claude.json`) so it is available across all projects, not just the current one.

### Behind the GFW (proxy setup)

```bash
claude mcp add -s user ferris-search $(which ferris-search) \
  -e USE_PROXY=true \
  -e PROXY_URL=http://127.0.0.1:7890 \
  -e DEFAULT_SEARCH_ENGINE=bing
```

## MCP Tools

### `web_search`

Search the web using one or more engines simultaneously.

```json
{
  "query": "rust async runtime",
  "engines": ["bing", "duckduckgo"],
  "limit": 10
}
```

| Parameter | Type     | Default        | Description                              |
| --------- | -------- | -------------- | ---------------------------------------- |
| `query`   | string   | required       | Search query                             |
| `engines` | string[] | server default | Engines to search (fan-out if multiple)  |
| `limit`   | number   | 10             | Max results per engine (1–50)            |

Supported engines: `bing`, `duckduckgo`, `brave`, `baidu`, `csdn`, `juejin`, `exa`, `firecrawl`, `zhihu`, `linuxdo`, `jina`, `tavily`, `github`, `github_code`

### `fetch_web_content`

Fetch and extract text content from any public URL.

| Parameter   | Type   | Default  | Description              |
| ----------- | ------ | -------- | ------------------------ |
| `url`       | string | required | Public HTTP/HTTPS URL    |
| `max_chars` | number | 30000    | Max characters to return |

### `fetch_github_readme`

Fetch the README from a GitHub repository.

| Parameter | Type   | Description           |
| --------- | ------ | --------------------- |
| `url`     | string | GitHub repository URL |

### `fetch_csdn_article` / `fetch_juejin_article` / `fetch_zhihu_article` / `fetch_linuxdo_article`

Domain-specific fetchers with better content extraction than the generic `fetch_web_content`.

| Tool                    | URL Constraint                        |
| ----------------------- | ------------------------------------- |
| `fetch_csdn_article`    | must contain `csdn.net`               |
| `fetch_juejin_article`  | must contain `juejin.cn` and `/post/` |
| `fetch_zhihu_article`   | must contain `zhihu.com`              |
| `fetch_linuxdo_article` | must contain `linux.do` and `/topic/` |

## Configuration

All configuration is done via environment variables.

| Env Var                  | Default                        | Description                                                                               |
| ------------------------ | ------------------------------ | ----------------------------------------------------------------------------------------- |
| `DEFAULT_SEARCH_ENGINE`  | `bing`                         | Engine used when `engines` param is omitted                                               |
| `ALLOWED_SEARCH_ENGINES` | all engines                    | Comma-separated allow-list                                                                |
| `BRAVE_API_KEY`          | —                              | Required for `brave` engine                                                               |
| `EXA_API_KEY`            | —                              | Required for `exa` engine                                                                 |
| `FIRECRAWL_API_KEY`      | —                              | Required for `firecrawl` engine                                                           |
| `JINA_API_KEY`           | —                              | Required for `jina` engine                                                                |
| `TAVILY_API_KEY`         | —                              | Required for `tavily` engine                                                              |
| `GITHUB_TOKEN`           | —                              | Optional for `github` / `github_code` engines (raises rate limit from 60 to 5000 req/hr)  |
| `USE_PROXY`              | `false`                        | Enable HTTP/SOCKS5 proxy                                                                  |
| `PROXY_URL`              | `http://127.0.0.1:7890`        | Proxy address                                                                             |
| `RUST_LOG`               | `info`                         | Log level: `debug`, `info`, `warn`, `error`                                               |
| `LOCAL_DOCS_INDEX_PATH`  | `.ferris-index`                | Directory for the local document index                                                    |
| `LOCAL_DOCS_EXTENSIONS`  | `md,markdown,txt,html,htm,pdf` | Comma-separated file extensions to index                                                  |

### Common configurations

**Privacy-focused:**

```bash
claude mcp add -s user ferris-search $(which ferris-search) \
  -e DEFAULT_SEARCH_ENGINE=duckduckgo \
  -e ALLOWED_SEARCH_ENGINES=duckduckgo,brave
```

**Chinese developer workflow:**

```bash
claude mcp add -s user ferris-search $(which ferris-search) \
  -e DEFAULT_SEARCH_ENGINE=bing \
  -e ALLOWED_SEARCH_ENGINES=bing,baidu,csdn,juejin,zhihu
```

**With Exa AI search:**

```bash
claude mcp add -s user ferris-search $(which ferris-search) \
  -e DEFAULT_SEARCH_ENGINE=exa \
  -e EXA_API_KEY=exa-xxxx \
  -e ALLOWED_SEARCH_ENGINES=exa,bing,duckduckgo
```

## Acknowledgements

Inspired by [open-webSearch](https://github.com/Aas-ee/open-webSearch) — this project reimplements the same idea in Rust for performance and portability.

## License

Apache-2.0
