---
name: ferris-search-cli
description: |
  CRITICAL: Use for ferris-search CLI usage. Triggers on:
  ferris-search search, ferris-search fetch, ferris-search list-engines,
  ferris-search show-config, ferris-search mcp, ferris-search CLI,
  ferris-search index-local, ferris-search search-local, local document index,
  local full-text search, 本地索引, 本地文档搜索, 全文索引, 本地检索,
  terminal search, command line search, shell search, CLI搜索,
  命令行搜索, 终端搜索, ferris-search命令行, ferris-search终端
---

# ferris-search CLI Skill

> **Version:** ferris-search 0.1.0 | **Last Updated:** 2026-03-31

Help users run `ferris-search` from the terminal. For complete parameter details, read `./references/cli-reference.md`.

## Subcommands

### search — Web search

```bash
# Default engine
ferris-search search "rust async runtime"

# Specify engines (comma-separated or repeated -e)
ferris-search search "rust async" -e bing,duckduckgo
ferris-search search "rust async" -e bing -e brave

# Limit + JSON
ferris-search search "tokio tutorial" -e bing --limit 3 --format json
```

### fetch — Extract web content

```bash
# Auto-detects fetcher by URL domain
ferris-search fetch https://github.com/tokio-rs/tokio
ferris-search fetch https://example.com --max-chars 5000
ferris-search fetch https://example.com --format json
```

Specialized domains: `github.com` (repo root → README), `csdn.net`, `juejin.cn/post/`, `zhihu.com`, `linux.do/topic/`. All others use generic web extraction.

### list-engines — Show available engines

```bash
ferris-search list-engines
ferris-search list-engines --format json
```

### show-config — Show effective configuration

```bash
ferris-search show-config
ferris-search show-config --format json
```

API keys are masked in output (first 4 + last 4 chars shown).

### index-local — Build local full-text index

```bash
# Index a directory of documents
ferris-search index-local --path ./docs

# Index multiple paths
ferris-search index-local --path ./docs --path ./notes --path ./papers

# Custom index location
ferris-search index-local --path ./docs --index-path ./my-index

# JSON output
ferris-search index-local --path ./docs --format json
```

Supported file types (configurable via `LOCAL_DOCS_EXTENSIONS`): `.md`, `.markdown`, `.txt`, `.html`, `.htm`, `.pdf`

PDF support is text-layer only. Image-only or scanned PDFs without embedded text are not OCRed.

### search-local — Search the local document index

```bash
# Search indexed documents
ferris-search search-local "async runtime"

# With limit and custom index path
ferris-search search-local "error handling" --limit 5 --index-path ./my-index

# JSON output
ferris-search search-local "design patterns" --format json
```

Results include title, file path, file type, relevance score, and a content snippet.

### mcp — Start MCP server

```bash
ferris-search mcp
```

> When stdin is piped (e.g. by Claude Desktop), `ferris-search` auto-enters MCP mode without needing `mcp`.
>
> MCP is still supported. The CLI was added alongside MCP, not as a replacement for it.

## Key Behaviors

- **Local indexing**: Powered by [Tantivy](https://github.com/quickwit-oss/tantivy), supports full-text search across local Markdown, TXT, HTML, and text-based PDF files
- **PDF limitation**: No OCR is performed; scanned or image-only PDFs may index as empty or near-empty content
- **Output format**: `--format text` (default, human-readable) or `--format json` (machine-parseable)
- **Exit codes**: 0 = success, 1 = search/fetch failure, 2 = parameter error
- **Errors to stderr**: Warnings and failures go to stderr; results go to stdout
- **Engine resolution**: CLI `--engine` overrides env `DEFAULT_SEARCH_ENGINE`; engines are filtered by `ALLOWED_SEARCH_ENGINES`
- **TTY detection**: No subcommand + interactive terminal → prints help; piped stdin → MCP mode
- **MCP status**: MCP is active and supported; only the old `MODE` / `ENABLE_HTTP_SERVER` transport toggles were removed

## Scripting Patterns

### Pipe JSON to jq

```bash
ferris-search search "rust" -e bing --format json | jq '.[].url'
```

### Multi-engine with error checking

```bash
ferris-search search "query" -e bing,brave --format json 2>errors.log
if [ $? -ne 0 ]; then cat errors.log; fi
```
