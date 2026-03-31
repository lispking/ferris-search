---
name: ferris-search-tools
description: |
  CRITICAL: Use for ferris-search MCP tool usage. Triggers on:
  web_search, fetch_web_content, fetch_github_readme, fetch_csdn_article,
  fetch_juejin_article, fetch_zhihu_article, fetch_linuxdo_article,
  ferris-search tool, MCP search tool, multi-engine search,
  github search, github_code search, GitHub repository search, GitHub code search,
  搜索引擎工具, 网页抓取, 文章获取, MCP搜索, 多引擎搜索, GitHub搜索
---

# ferris-search MCP Tools Skill

> **Version:** ferris-search 0.1.0 | **Last Updated:** 2026-03-31

You are an expert at using the `ferris-search` MCP server tools. Help users by:
- **Writing MCP calls**: Generate correct tool invocations with proper parameters
- **Answering questions**: Explain which tool to use and why, troubleshoot issues

## Documentation

Refer to the local files for detailed documentation:
- `./references/tools-api.md` - Complete tool parameter reference
- `./references/engines.md` - Search engine details and aliases

## IMPORTANT: Documentation Completeness Check

**Before answering questions, Claude MUST:**

1. Read the relevant reference file(s) listed above
2. If file read fails or file is empty:
   - Inform user: "本地文档不完整，建议运行 `/sync-crate-skills ferris-search --force` 更新文档"
   - Still answer based on SKILL.md patterns + built-in knowledge
3. If reference file exists, incorporate its content into the answer

## Key Patterns

### Single-engine search
```json
{
  "tool": "web_search",
  "query": "rust tokio tutorial",
  "limit": 10
}
```

### Multi-engine fan-out
```json
{
  "tool": "web_search",
  "query": "rust async runtime",
  "engines": ["bing", "duckduckgo", "brave"],
  "limit": 5
}
```

### GitHub repository search
```json
{
  "tool": "web_search",
  "query": "tokio async runtime stars:>1000",
  "engines": ["github"],
  "limit": 10
}
```

### GitHub code search
```json
{
  "tool": "web_search",
  "query": "reqwest Client language:rust",
  "engines": ["github_code"],
  "limit": 10
}
```

### Fetch any web page
```json
{
  "tool": "fetch_web_content",
  "url": "https://doc.rust-lang.org/book/",
  "max_chars": 50000
}
```

### Fetch GitHub README
```json
{
  "tool": "fetch_github_readme",
  "url": "https://github.com/tokio-rs/tokio"
}
```

### Fetch domain-specific article
```json
// CSDN
{ "tool": "fetch_csdn_article", "url": "https://blog.csdn.net/..." }
// Juejin
{ "tool": "fetch_juejin_article", "url": "https://juejin.cn/post/..." }
// Zhihu
{ "tool": "fetch_zhihu_article", "url": "https://zhuanlan.zhihu.com/p/..." }
// LinuxDo
{ "tool": "fetch_linuxdo_article", "url": "https://linux.do/topic/..." }
```

## API Reference Table

| Tool | Required Params | Optional Params | URL Constraint |
|------|----------------|-----------------|----------------|
| `web_search` | `query` | `engines`, `limit` (1–50) | — |
| `fetch_web_content` | `url` | `max_chars` (default 30000) | public HTTP/HTTPS |
| `fetch_github_readme` | `url` | — | github.com |
| `fetch_csdn_article` | `url` | — | csdn.net |
| `fetch_juejin_article` | `url` | — | juejin.cn + /post/ |
| `fetch_zhihu_article` | `url` | — | zhihu.com |
| `fetch_linuxdo_article` | `url` | — | linux.do + /topic/ |

## Deprecated Patterns (Don't Use)

| Deprecated | Correct | Notes |
|------------|---------|-------|
| Passing engine as string `"engines": "bing"` | `"engines": ["bing"]` | Must be an array |
| `limit > 50` | `limit: 50` | Clamped to max 50 |
| Using `fetch_web_content` for CSDN/Juejin/Zhihu | Use domain-specific fetcher | Better extraction quality |

## When Writing Code

1. Use domain-specific fetchers (csdn, juejin, zhihu, linuxdo) instead of `fetch_web_content` when the URL matches
2. For research tasks, fan-out across 2–3 engines: `["bing", "duckduckgo"]` for global, add `"baidu"` for Chinese content
3. Keep `limit` ≤ 10 unless more results are explicitly needed
4. `max_chars` default (30000) is sufficient for most articles; increase only for very long documents

## When Answering Questions

1. Check which tool fits: search vs fetch — don't use `web_search` when you already have a URL
2. For Chinese content (CSDN, Juejin, Zhihu), always prefer the dedicated fetcher
3. Engine aliases are normalized: `"ddg"`, `"duck duck go"`, `"百度"` all work
4. `exa` engine requires `EXA_API_KEY` env var — warn user if they try to use it without one
5. `github` searches repositories (name, description, stars, language); `github_code` searches code files (path, blob URL)
6. `GITHUB_TOKEN` is optional for `github`/`github_code` — without it, GitHub rate-limits to 60 req/hr; with it, 5000 req/hr
