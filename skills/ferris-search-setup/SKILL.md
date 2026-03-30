---
name: ferris-search-setup
description: |
  CRITICAL: Use for ferris-search installation and configuration. Triggers on:
  ferris-search install, mcp add ferris-search, setup ferris-search,
  DEFAULT_SEARCH_ENGINE, ALLOWED_SEARCH_ENGINES, BRAVE_API_KEY, EXA_API_KEY, FIRECRAWL_API_KEY, JINA_API_KEY, TAVILY_API_KEY, USE_PROXY,
  ferris-search config, ferris-search build, ferris-search Docker,
  安装ferris-search, 配置搜索引擎, MCP服务器配置, 代理设置
---

# ferris-search Setup & Configuration Skill

> **Version:** ferris-search 0.1.0 | **Last Updated:** 2026-03-30

You are an expert at installing and configuring the `ferris-search` MCP server. Help users by:
- **Setup**: Guide through build, install, and MCP registration
- **Configuration**: Explain env vars and their effects

## Documentation

Refer to the local files for detailed documentation:
- `./references/configuration.md` - All environment variables and their effects

## IMPORTANT: Documentation Completeness Check

**Before answering questions, Claude MUST:**

1. Read `./references/configuration.md`
2. If file read fails: still answer based on SKILL.md patterns

## Key Patterns

### Build & register with Claude Code
```bash
cargo build --release
claude mcp add ferris-search ./target/release/ferris-search
```

### With environment variables
```bash
claude mcp add ferris-search ./target/release/ferris-search \
  -e DEFAULT_SEARCH_ENGINE=bing \
  -e ALLOWED_SEARCH_ENGINES=bing,duckduckgo,brave
```

### Claude Desktop / Cursor (mcp-config.json)
```json
{
  "mcpServers": {
    "ferris-search": {
      "command": "/path/to/ferris-search",
      "env": {
        "DEFAULT_SEARCH_ENGINE": "bing",
        "ALLOWED_SEARCH_ENGINES": "bing,duckduckgo,brave,baidu",
        "EXA_API_KEY": "your-key-here"
      }
    }
  }
}
```

### With proxy
```bash
claude mcp add ferris-search ./target/release/ferris-search \
  -e USE_PROXY=true \
  -e PROXY_URL=http://127.0.0.1:7890
```

### Docker
```bash
docker build -t ferris-search .
docker run -e DEFAULT_SEARCH_ENGINE=bing ferris-search
```

## Configuration Reference

| Env Var | Default | Description |
|---------|---------|-------------|
| `DEFAULT_SEARCH_ENGINE` | `bing` | Engine used when `engines` param is omitted |
| `ALLOWED_SEARCH_ENGINES` | all 9 engines | Comma-separated allow-list |
| `BRAVE_API_KEY` | — | Required only for `brave` engine |
| `EXA_API_KEY` | — | Required only for `exa` engine |
| `FIRECRAWL_API_KEY` | — | Required only for `firecrawl` engine |
| `JINA_API_KEY` | — | Required only for `jina` engine |
| `TAVILY_API_KEY` | — | Required only for `tavily` engine |
| `USE_PROXY` | `false` | Enable HTTP/SOCKS5 proxy |
| `PROXY_URL` | `http://127.0.0.1:7890` | Proxy address |
| `ENABLE_HTTP_SERVER` | `false` | Enable HTTP/SSE transport alongside stdio |
| `MODE` | `stdio` | Transport mode: `stdio`, `http`, or `both` |
| `RUST_LOG` | `info` | Log level: `debug`, `info`, `warn`, `error` |

## When Writing Code

1. Always build with `--release` for production use (~8 MB binary, <10 ms startup)
2. Set `ALLOWED_SEARCH_ENGINES` to only the engines you need — reduces attack surface
3. Never commit `EXA_API_KEY`, `FIRECRAWL_API_KEY`, `JINA_API_KEY`, `TAVILY_API_KEY`, or `BRAVE_API_KEY` to source control — use env var injection
4. For Chinese content workflows, include `baidu`, `csdn`, `juejin`, `zhihu` in allow-list

## When Answering Questions

1. `claude mcp add` is the recommended path for Claude Code users
2. JSON config is needed for Claude Desktop / Cursor
3. Proxy support works for all engines including those behind GFW
4. `ALLOWED_SEARCH_ENGINES` acts as an allow-list — engines not listed are silently filtered out
