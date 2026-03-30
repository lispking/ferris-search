# ferris-search Configuration Reference

## Environment Variables

### DEFAULT_SEARCH_ENGINE

- **Default:** `bing`
- **Values:** any of `bing`, `duckduckgo`, `brave`, `baidu`, `csdn`, `juejin`, `exa`, `zhihu`, `linuxdo`
- **Effect:** Used when `web_search` is called without an `engines` parameter

```bash
DEFAULT_SEARCH_ENGINE=duckduckgo
```

---

### ALLOWED_SEARCH_ENGINES

- **Default:** all 9 engines
- **Format:** comma-separated list (spaces around commas are trimmed)
- **Effect:** Acts as an allow-list. Any engine not in this list is silently ignored in `web_search` calls.

```bash
# Only allow privacy-friendly engines
ALLOWED_SEARCH_ENGINES=duckduckgo,brave

# Chinese content focus
ALLOWED_SEARCH_ENGINES=baidu,csdn,juejin,zhihu,bing
```

> If `DEFAULT_SEARCH_ENGINE` is not in `ALLOWED_SEARCH_ENGINES`, searches without explicit engines will return "No allowed engines specified."

---

### EXA_API_KEY

- **Default:** unset
- **Effect:** Required to use the `exa` engine. Without it, `exa` calls will fail.
- **Get a key:** https://exa.ai

```bash
EXA_API_KEY=exa-xxxxxxxxxxxxxxxx
```

---

### USE_PROXY / PROXY_URL

- **USE_PROXY default:** `false`
- **PROXY_URL default:** `http://127.0.0.1:7890`
- **Supported protocols:** HTTP proxy, SOCKS5 proxy
- **Effect:** All outbound HTTP requests (search + fetch) go through this proxy

```bash
USE_PROXY=true
PROXY_URL=http://127.0.0.1:7890

# SOCKS5
USE_PROXY=true
PROXY_URL=socks5://127.0.0.1:1080
```

---

### ENABLE_HTTP_SERVER / MODE

- **ENABLE_HTTP_SERVER default:** `false`
- **MODE default:** `stdio`
- **MODE values:** `stdio`, `http`, `both`
- **Effect:** Enables HTTP/SSE transport in addition to (or instead of) stdio

```bash
# HTTP only
ENABLE_HTTP_SERVER=true
MODE=http

# Both transports
MODE=both
```

---

### RUST_LOG

- **Default:** `info`
- **Values:** `error`, `warn`, `info`, `debug`, `trace`
- **Note:** Logs are written to stderr (not stdout), so they don't interfere with MCP stdio transport

```bash
RUST_LOG=debug  # verbose logging for troubleshooting
```

---

## Complete Example Configurations

### Minimal (stdio, default bing)
```bash
claude mcp add ferris-search ./target/release/ferris-search
```

### Privacy-focused
```bash
claude mcp add ferris-search ./target/release/ferris-search \
  -e DEFAULT_SEARCH_ENGINE=duckduckgo \
  -e ALLOWED_SEARCH_ENGINES=duckduckgo,brave
```

### Chinese developer workflow
```bash
claude mcp add ferris-search ./target/release/ferris-search \
  -e DEFAULT_SEARCH_ENGINE=bing \
  -e ALLOWED_SEARCH_ENGINES=bing,baidu,csdn,juejin,zhihu
```

### With Exa AI search
```bash
claude mcp add ferris-search ./target/release/ferris-search \
  -e DEFAULT_SEARCH_ENGINE=exa \
  -e EXA_API_KEY=exa-xxxx \
  -e ALLOWED_SEARCH_ENGINES=exa,bing,duckduckgo
```

### Behind GFW with proxy
```bash
claude mcp add ferris-search ./target/release/ferris-search \
  -e USE_PROXY=true \
  -e PROXY_URL=http://127.0.0.1:7890 \
  -e DEFAULT_SEARCH_ENGINE=bing
```
