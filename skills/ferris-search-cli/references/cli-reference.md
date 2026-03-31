# CLI Reference

## Table of Contents

1. [search](#search)
2. [fetch](#fetch)
3. [list-engines](#list-engines)
4. [show-config](#show-config)
5. [index-local](#index-local)
6. [search-local](#search-local)
7. [mcp](#mcp)
8. [Global Behavior](#global-behavior)
9. [Supported Engines](#supported-engines)

---

## search

Search the web using one or more engines.

```text
ferris-search search <QUERY> [OPTIONS]
```

| Parameter  | Short | Type          | Default                               | Description                                  |
| ---------- | ----- | ------------- | ------------------------------------- | -------------------------------------------- |
| `<QUERY>`  | —     | string        | *required*                            | Search query (positional)                    |
| `--engine` | `-e`  | string        | env `DEFAULT_SEARCH_ENGINE` or `bing` | Engine name(s). Comma-separated or repeated. |
| `--limit`  | `-l`  | u32           | `10`                                  | Max results per engine (1–50)                |
| `--format` | `-f`  | text / json   | `text`                                | Output format                                |

**Engine specification examples:**

- `-e bing` — single engine
- `-e bing,duckduckgo` — comma-separated
- `-e bing -e brave` — repeated flag
- omitted — uses `DEFAULT_SEARCH_ENGINE` env var (default: `bing`)

**Single-engine output (text):**

```text
Engine: bing
Total: 3

1. **Title**
URL: https://...
Source: bing
Description: ...
```

**Multi-engine output (text):**

```text
Total results: 8

## Results from bing

1. **Title** ...

## Results from brave

1. **Title** ...
```

**JSON output:** Array of `SearchResult` objects:

```json
[
  {
    "title": "...",
    "url": "...",
    "description": "...",
    "source": "...",
    "engine": "bing"
  }
]
```

**Error handling:**

- Single engine failure → exit 1, error to stderr
- Multi-engine partial failure → warnings to stderr, successful results to stdout
- Multi-engine all fail → "All engines failed." to stderr, exit 1

---

## fetch

Fetch and extract content from a URL. Auto-detects the best extraction method by domain.

```text
ferris-search fetch <URL> [OPTIONS]
```

| Parameter     | Short | Type        | Default    | Description                               |
| ------------- | ----- | ----------- | ---------- | ----------------------------------------- |
| `<URL>`       | —     | string      | *required* | URL to fetch (positional)                 |
| `--max-chars` | `-m`  | u32         | `30000`    | Max content characters (generic web only) |
| `--format`    | `-f`  | text / json | `text`     | Output format                             |

**Domain routing:**

| URL Pattern                                           | Fetcher                 | Notes                               |
| ----------------------------------------------------- | ----------------------- | ----------------------------------- |
| `github.com/<owner>/<repo>` (exactly 2 path segments) | `fetch_github_readme`   | Returns README.md content           |
| `csdn.net`                                            | `fetch_csdn_article`    | Full article extraction             |
| `juejin.cn` + `/post/` in path                        | `fetch_juejin_article`  | Full article extraction             |
| `zhihu.com`                                           | `fetch_zhihu_article`   | Full article extraction             |
| `linux.do` + `/topic/` in path                        | `fetch_linuxdo_article` | Full topic extraction               |
| Everything else                                       | `fetch_web_content`     | Generic with `max_chars` truncation |

**Important:** GitHub URLs with deeper paths (e.g. `/issues/`, `/blob/`, `/pull/`) fall through to generic web fetch, not README fetch. Query strings and fragments (`?tab=...`, `#readme`) are stripped before routing.

**Text output (generic web):**

```text
Title: Page Title
URL: https://...

Page content here...

[Content truncated]
```

**JSON output (generic web):**

```json
{
  "title": "...",
  "url": "...",
  "content": "...",
  "truncated": true
}
```

**JSON output (specialized fetchers):**

```json
{
  "url": "...",
  "content": "..."
}
```

---

## list-engines

List all 14 supported search engines with their status.

```text
ferris-search list-engines [OPTIONS]
```

| Parameter  | Short | Type        | Default | Description   |
| ---------- | ----- | ----------- | ------- | ------------- |
| `--format` | `-f`  | text / json | `text`  | Output format |

**Text output:**

```text
Supported engines:

  baidu (allowed)
  bing (default)
  brave (allowed)
  ...
  zhihu (disabled)
```

**JSON output:**

```json
{
  "default_engine": "bing",
  "allowed_engines": ["bing", "duckduckgo", "brave"],
  "all_engines": ["baidu", "bing", "brave", "csdn", "duckduckgo", "exa", "firecrawl", "github", "github_code", "jina", "juejin", "linuxdo", "tavily", "zhihu"]
}
```

---

## show-config

Show the current effective configuration. API keys are masked.

```text
ferris-search show-config [OPTIONS]
```

| Parameter  | Short | Type        | Default | Description   |
| ---------- | ----- | ----------- | ------- | ------------- |
| `--format` | `-f`  | text / json | `text`  | Output format |

**Key masking:** Keys with >8 chars show `abcd...wxyz` (first 4 + last 4). Keys ≤8 chars show `***`. Unset keys show `(not set)`.

---

## index-local

Build a full-text index from local documents. Powered by [Tantivy](https://github.com/quickwit-oss/tantivy).

```text
ferris-search index-local --path <DIR_OR_FILE> [OPTIONS]
```

| Parameter      | Short | Type        | Default                                          | Description                                         |
| -------------- | ----- | ----------- | ------------------------------------------------ | --------------------------------------------------- |
| `--path`       | `-p`  | string      | *required*                                       | Directory or file to index (can be repeated)        |
| `--index-path` | —     | string      | env `LOCAL_DOCS_INDEX_PATH` or `.ferris-index`   | Directory to store the Tantivy index                |
| `--format`     | `-f`  | text / json | `text`                                           | Output format                                       |

**Supported file types** (configurable via `LOCAL_DOCS_EXTENSIONS`):

| Extension          | Extraction Method                     |
| ------------------ | ------------------------------------- |
| `.md`, `.markdown` | Raw text                              |
| `.txt`             | Raw text                              |
| `.html`, `.htm`    | HTML → text stripping                 |
| `.pdf`             | pdf-extract (text layer only, no OCR) |

**Behavior:**

- Rebuilds the index from scratch each time (`clear = true`)
- Recursively walks directories, following symlinks
- Per-file errors are reported as warnings to stderr but do not abort the run
- Scanned or image-only PDFs without embedded text are not OCRed and may yield empty or low-quality indexed content
- If no documents are found or all fail to index, exits with code 1

**Text output:**

```text
Indexed 42 documents into .ferris-index
Warnings: 2
```

**JSON output:**

```json
{
  "indexed": 42,
  "errors": 2,
  "index_path": ".ferris-index"
}
```

---

## search-local

Search the local document index built by `index-local`.

```text
ferris-search search-local <QUERY> [OPTIONS]
```

| Parameter      | Short | Type        | Default                                          | Description                              |
| -------------- | ----- | ----------- | ------------------------------------------------ | ---------------------------------------- |
| `<QUERY>`      | —     | string      | *required*                                       | Search query (positional)                |
| `--index-path` | —     | string      | env `LOCAL_DOCS_INDEX_PATH` or `.ferris-index`   | Directory of the Tantivy index           |
| `--limit`      | `-l`  | u32         | `10`                                             | Max results (1–50)                       |
| `--format`     | `-f`  | text / json | `text`                                           | Output format                            |

**Query syntax:** Supports Tantivy query syntax — terms are OR-ed by default. Use `+term` for required, `-term` for exclusion, `"exact phrase"` for phrase match.

**Fields searched:** `title` and `body` (full-text indexed).

**Text output:**

```text
Found 3 results:

1. **Getting Started with Rust**
Path: /docs/getting-started.md
Type: md | Score: 12.3456
Snippet: Rust is a systems programming language...

2. **Error Handling**
Path: /docs/errors.md
Type: md | Score: 8.7654
Snippet: In Rust, errors are values...
```

**JSON output:** Array of `LocalSearchResult` objects:

```json
[
  {
    "title": "Getting Started with Rust",
    "path": "/docs/getting-started.md",
    "snippet": "Rust is a systems programming language...",
    "file_type": "md",
    "score": 12.3456
  }
]
```

---

## mcp

Start the MCP server using stdio transport.

```text
ferris-search mcp
```

No options. Logging level defaults to `info` (configurable via `RUST_LOG`).

---

## Global Behavior

### TTY Auto-detection

| Condition                             | Behavior                          |
| ------------------------------------- | --------------------------------- |
| No subcommand + piped stdin           | Auto-enters MCP mode              |
| No subcommand + interactive terminal  | Prints help and exits (code 0)    |
| Explicit subcommand                   | Runs that subcommand              |

### Exit Codes

- `0`: Success
- `1`: Search/fetch failure
- `2`: Parameter error, such as invalid engine or bad URL

### Logging

- CLI subcommands (`search`, `fetch`): default log level `warn` (to stderr)
- MCP mode (`mcp`): default log level `info` (to stderr)
- Override with `RUST_LOG` env var

---

## Supported Engines

| Engine        | Alias(es)                  | API Key Required               |
| ------------- | -------------------------- | ------------------------------ |
| `baidu`       | 百度                       | No                             |
| `bing`        | microsoft bing             | No                             |
| `brave`       | brave search               | Yes (`BRAVE_API_KEY`)          |
| `csdn`        | —                          | No                             |
| `duckduckgo`  | ddg, duck duck go          | No                             |
| `exa`         | —                          | Yes (`EXA_API_KEY`)            |
| `firecrawl`   | —                          | Yes (`FIRECRAWL_API_KEY`)      |
| `github`      | github repos, github repo  | No (optional `GITHUB_TOKEN`)   |
| `github_code` | github code                | No (optional `GITHUB_TOKEN`)   |
| `jina`        | jina.ai                    | Yes (`JINA_API_KEY`)           |
| `juejin`      | 掘金                       | No                             |
| `linuxdo`     | linux.do                   | No                             |
| `tavily`      | —                          | Yes (`TAVILY_API_KEY`)         |
| `zhihu`       | 知乎                       | No                             |
