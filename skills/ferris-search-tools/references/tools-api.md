# ferris-search Tools API Reference

## web_search

Search the web using one or more engines. Supports concurrent fan-out.

### Parameters

| Parameter | Type | Required | Default | Constraints |
|-----------|------|----------|---------|-------------|
| `query` | string | yes | — | Any search query |
| `engines` | string[] | no | `DEFAULT_SEARCH_ENGINE` env var | Must be array, see engine list below |
| `limit` | number | no | 10 | 1–50 (clamped) |

### Engine Names & Aliases

| Canonical | Aliases |
|-----------|---------|
| `bing` | `microsoft bing` |
| `duckduckgo` | `ddg`, `duck duck go` |
| `brave` | `brave search` |
| `baidu` | `百度` |
| `csdn` | — |
| `juejin` | `掘金` |
| `exa` | — (requires `EXA_API_KEY`) |
| `zhihu` | `知乎` |
| `linuxdo` | `linux.do` |

### Output Format

Single engine:
```
Engine: bing
Total: 10

1. **Result Title**
URL: https://...
Source: bing
Description: ...
```

Multi-engine:
```
Total results: 25

## Results from bing

1. **...**
...

## Results from duckduckgo
...
```

---

## fetch_web_content

Fetch and extract text from any public URL using HTML scraping.

### Parameters

| Parameter | Type | Required | Default | Constraints |
|-----------|------|----------|---------|-------------|
| `url` | string | yes | — | Must be public HTTP/HTTPS |
| `max_chars` | number | no | 30000 | max 200000 |

### Output Format

```
Title: Page Title
URL: https://...

{extracted text content}

[Content truncated]   ← only if truncated
```

### URL Safety Rules

- Must start with `http://` or `https://`
- Must not be a private/internal IP (10.x, 192.168.x, 127.x, etc.)
- Must not be `localhost`

---

## fetch_github_readme

Fetch README from a GitHub repository via the GitHub raw content API.

### Parameters

| Parameter | Type | Required | Constraints |
|-----------|------|----------|-------------|
| `url` | string | yes | Must be a `github.com` URL |

### Supported URL Formats

```
https://github.com/owner/repo
https://github.com/owner/repo/tree/branch
```

### Output

Raw README content (markdown text).

---

## fetch_csdn_article

Fetch full article from CSDN blog.

### Parameters

| Parameter | Type | Required | Constraints |
|-----------|------|----------|-------------|
| `url` | string | yes | Must contain `csdn.net` |

---

## fetch_juejin_article

Fetch full article from Juejin.

### Parameters

| Parameter | Type | Required | Constraints |
|-----------|------|----------|-------------|
| `url` | string | yes | Must contain `juejin.cn` AND `/post/` |

---

## fetch_zhihu_article

Fetch full article from Zhihu.

### Parameters

| Parameter | Type | Required | Constraints |
|-----------|------|----------|-------------|
| `url` | string | yes | Must contain `zhihu.com` |

---

## fetch_linuxdo_article

Fetch full topic from linux.do forum.

### Parameters

| Parameter | Type | Required | Constraints |
|-----------|------|----------|-------------|
| `url` | string | yes | Must contain `linux.do` AND `/topic/` |
