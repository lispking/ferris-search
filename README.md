# ferris-search 🦀

> A blazing-fast MCP server for multi-engine web search, written in Rust.

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

- **Multi-engine fan-out** — search across multiple engines simultaneously with a single call
- **12 search engines** — Bing, DuckDuckGo, Brave, Baidu, CSDN, Juejin, Exa, Firecrawl, Zhihu, LinuxDo, Jina, Tavily
- **7 MCP tools** — `web_search` + 6 article/content fetchers
- **No API keys required** for most engines (Brave, Exa, Firecrawl, Jina, and Tavily require API keys)
- **Single binary** — ~8 MB, no runtime dependencies
- **Proxy support** — HTTP/SOCKS5 proxy via env var

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

Domain-specific fetchers with better content extraction than the generic `fetch_web_content`.

| Tool | URL Constraint |
|------|----------------|
| `fetch_csdn_article` | must contain `csdn.net` |
| `fetch_juejin_article` | must contain `juejin.cn` and `/post/` |
| `fetch_zhihu_article` | must contain `zhihu.com` |
| `fetch_linuxdo_article` | must contain `linux.do` and `/topic/` |

## Configuration

All configuration is done via environment variables.

| Env Var | Default | Description |
|---------|---------|-------------|
| `DEFAULT_SEARCH_ENGINE` | `bing` | Engine used when `engines` param is omitted |
| `ALLOWED_SEARCH_ENGINES` | all engines | Comma-separated allow-list |
| `BRAVE_API_KEY` | — | Required for `brave` engine |
| `EXA_API_KEY` | — | Required for `exa` engine |
| `FIRECRAWL_API_KEY` | — | Required for `firecrawl` engine |
| `JINA_API_KEY` | — | Required for `jina` engine |
| `TAVILY_API_KEY` | — | Required for `tavily` engine |
| `USE_PROXY` | `false` | Enable HTTP/SOCKS5 proxy |
| `PROXY_URL` | `http://127.0.0.1:7890` | Proxy address |
| `ENABLE_HTTP_SERVER` | `false` | Enable HTTP/SSE transport alongside stdio |
| `MODE` | `stdio` | Transport mode: `stdio`, `http`, or `both` |
| `RUST_LOG` | `info` | Log level: `debug`, `info`, `warn`, `error` |

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
