# ferris-search Configuration Reference

## Environment Variables

### DEFAULT_SEARCH_ENGINE

- **Default:** `bing`
- **Values:** any of `bing`, `duckduckgo`, `brave`, `baidu`, `csdn`, `juejin`, `exa`, `firecrawl`, `zhihu`, `linuxdo`, `jina`, `tavily`, `github`, `github_code`
- **Effect:** Used when `web_search` is called without an `engines` parameter

```bash
DEFAULT_SEARCH_ENGINE=duckduckgo
```

---

### ALLOWED_SEARCH_ENGINES

- **Default:** all 14 engines
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

### BRAVE_API_KEY

- **Default:** unset
- **Effect:** Required to use the `brave` engine. Without it, `brave` calls will fail.
- **Get a key:** <https://brave.com/search/api/>

```bash
BRAVE_API_KEY=your-brave-api-key
```

---

### EXA_API_KEY

- **Default:** unset
- **Effect:** Required to use the `exa` engine. Without it, `exa` calls will fail.
- **Get a key:** <https://exa.ai>

```bash
EXA_API_KEY=exa-xxxxxxxxxxxxxxxx
```

---

### FIRECRAWL_API_KEY

- **Default:** unset
- **Effect:** Required to use the `firecrawl` engine. Without it, `firecrawl` calls will fail.
- **Get a key:** <https://firecrawl.dev>

```bash
FIRECRAWL_API_KEY=fc-xxxxxxxxxxxxxxxx
```

---

### JINA_API_KEY

- **Default:** unset
- **Effect:** Required to use the `jina` engine. Without it, `jina` calls will fail.
- **Get a key:** <https://jina.ai>

```bash
JINA_API_KEY=jina_xxxxxxxxxxxxxxxx
```

---

### TAVILY_API_KEY

- **Default:** unset
- **Effect:** Required to use the `tavily` engine. Without it, `tavily` calls will fail.
- **Get a key:** <https://tavily.com>

```bash
TAVILY_API_KEY=tvly-xxxxxxxxxxxxxxxx
```

---

### GITHUB_TOKEN

- **Default:** unset
- **Effect:** Optional. Used by `github` (repository search) and `github_code` (code search) engines.
  - Without token: anonymous requests, rate-limited to **60 req/hr**
  - With token: authenticated requests, rate-limited to **5000 req/hr**
- **Get a token:** GitHub Settings → Developer settings → Personal access tokens (no special scopes needed for public search)

```bash
GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxx
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

### LOCAL_DOCS_INDEX_PATH

- **Default:** `.ferris-index`
- **Effect:** Directory where `index-local` writes and `search-local` reads the Tantivy full-text index
- **Used by:** CLI `index-local` and `search-local` subcommands

```bash
LOCAL_DOCS_INDEX_PATH=./my-docs-index
```

---

### LOCAL_DOCS_EXTENSIONS

- **Default:** `md,markdown,txt,html,htm,pdf` (when unset, uses built-in defaults)
- **Format:** comma-separated list of file extensions (without dots)
- **Effect:** Controls which file types are collected during `index-local`

```bash
# Only index Markdown and text files
LOCAL_DOCS_EXTENSIONS=md,txt

# Include PDF
LOCAL_DOCS_EXTENSIONS=md,markdown,txt,html,htm,pdf
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
