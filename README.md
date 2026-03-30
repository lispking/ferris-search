# ferris-search 🦀

A blazing-fast MCP (Model Context Protocol) server for multi-engine web search, written in Rust.

Inspired by [open-webSearch](https://github.com/Aas-ee/open-webSearch) — this project reimplements the same idea with a focus on performance: lower latency, smaller memory footprint, and a single self-contained binary with no Node.js runtime required.

## Features

- **Multi-engine fan-out** — search across multiple engines simultaneously with a single call
- **12 search engines** — Bing, DuckDuckGo, Brave, Baidu, CSDN, Juejin, Exa, Firecrawl, Zhihu, LinuxDo, Jina, Tavily
- **7 MCP tools** — search + 6 article/content fetchers
- **No API keys required** for most engines (Brave, Exa, Firecrawl, Jina, and Tavily require API keys)
- **Single binary** — ~8 MB, no runtime dependencies
- **Proxy support** — HTTP/SOCKS proxy via env var

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

### Docker

```bash
docker build -t ferris-search .
docker run -e DEFAULT_SEARCH_ENGINE=bing ferris-search
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

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `query` | string | required | Search query |
| `engines` | string[] | server default | Engines to search (fan-out if multiple) |
| `limit` | number | 10 | Max results per engine (1–50) |

Supported engines: `bing`, `duckduckgo`, `brave`, `baidu`, `csdn`, `juejin`, `exa`, `firecrawl`, `zhihu`, `linuxdo`, `jina`, `tavily`

### `fetch_web_content`

Fetch and extract text content from any public URL.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `url` | string | required | Public HTTP/HTTPS URL |
| `max_chars` | number | 30000 | Max characters to return |

### `fetch_github_readme`

Fetch the README from a GitHub repository.

| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | string | GitHub repository URL |

### `fetch_csdn_article` / `fetch_juejin_article` / `fetch_zhihu_article` / `fetch_linuxdo_article`

Fetch full article content from the respective platforms.

| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | string | Article URL |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DEFAULT_SEARCH_ENGINE` | `bing` | Default engine when `engines` is not specified |
| `ALLOWED_SEARCH_ENGINES` | all | Comma-separated allowlist, e.g. `bing,duckduckgo` |
| `USE_PROXY` | `false` | Enable HTTP proxy |
| `PROXY_URL` | `http://127.0.0.1:7890` | Proxy URL (HTTP or SOCKS5) |
| `BRAVE_API_KEY` | — | Required when using the `brave` engine |
| `FIRECRAWL_API_KEY` | — | Required when using the `firecrawl` engine |
| `EXA_API_KEY` | — | Required when using the `exa` engine |
| `JINA_API_KEY` | — | Required when using the `jina` engine |
| `TAVILY_API_KEY` | — | Required when using the `tavily` engine |
| `MODE` | `stdio` | Transport mode: `stdio` |
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |

## License

Licensed under the [Apache License 2.0](LICENSE).
