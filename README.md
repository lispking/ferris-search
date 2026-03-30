# ferris-search 🦀

A blazing-fast MCP (Model Context Protocol) server for multi-engine web search, written in Rust.

Inspired by [open-webSearch](https://github.com/Aas-ee/open-webSearch) — this project reimplements the same idea with a focus on performance: lower latency, smaller memory footprint, and a single self-contained binary with no Node.js runtime required.

## Features

- **Multi-engine fan-out** — search across multiple engines simultaneously with a single call
- **9 search engines** — Bing, DuckDuckGo, Brave, Baidu, CSDN, Juejin, Exa, Zhihu, LinuxDo
- **7 MCP tools** — search + 6 article/content fetchers
- **No API keys required** for most engines (Exa requires one)
- **Single binary** — ~8 MB, no runtime dependencies
- **Proxy support** — HTTP/SOCKS proxy via env var

## Quick Start

### Build from source

```bash
cargo build --release
# binary: target/release/ferris-search
```

### Claude Desktop / Cursor configuration

```json
{
  "mcpServers": {
    "ferris-search": {
      "command": "/path/to/ferris-search",
      "env": {
        "DEFAULT_SEARCH_ENGINE": "bing"
      }
    }
  }
}
```

### Claude Code (claude mcp add)

```bash
claude mcp add ferris-search /path/to/ferris-search
```

With environment variables:

```bash
claude mcp add ferris-search /path/to/ferris-search \
  -e DEFAULT_SEARCH_ENGINE=bing \
  -e ALLOWED_SEARCH_ENGINES=bing,duckduckgo,brave
```

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

Supported engines: `bing`, `duckduckgo`, `brave`, `baidu`, `csdn`, `juejin`, `exa`, `zhihu`, `linuxdo`

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
| `EXA_API_KEY` | — | Required when using the `exa` engine |
| `MODE` | `stdio` | Transport mode: `stdio` |
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |

## License

Licensed under the [Apache License 2.0](LICENSE).
