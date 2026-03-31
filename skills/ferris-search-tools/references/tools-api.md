# ferris-search Tools API Reference

## web_search

Search the web using one or more engines. Supports concurrent fan-out.

### web_search Parameters

| Parameter | Type     | Required | Default                         | Constraints                           |
| --------- | -------- | -------- | ------------------------------- | ------------------------------------- |
| `query`   | string   | yes      | —                               | Any search query                      |
| `engines` | string[] | no       | `DEFAULT_SEARCH_ENGINE` env var | Must be array, see engine list below  |
| `limit`   | number   | no       | 10                              | 1–50 (clamped)                        |

### Engine Names & Aliases

- `bing`: aliases `microsoft bing`
- `duckduckgo`: aliases `ddg`, `duck duck go`
- `brave`: aliases `brave search`; requires `BRAVE_API_KEY`
- `baidu`: aliases `百度`
- `csdn`: no aliases
- `juejin`: aliases `掘金`
- `exa`: no aliases; requires `EXA_API_KEY`
- `firecrawl`: no aliases; requires `FIRECRAWL_API_KEY`
- `zhihu`: aliases `知乎`
- `linuxdo`: aliases `linux.do`
- `jina`: aliases `jina.ai`; requires `JINA_API_KEY`
- `tavily`: no aliases; requires `TAVILY_API_KEY`
- `github`: aliases `github repos`, `github repo`; optional `GITHUB_TOKEN`; searches repositories
- `github_code`: aliases `github code`; optional `GITHUB_TOKEN`; searches code files

### fetch_web_content Output Format

Single engine:

```text
Engine: bing
Total: 10

1. **Result Title**
URL: https://...
Source: bing
Description: ...
```

Multi-engine:

```text
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

### fetch_web_content Parameters

| Parameter   | Type   | Required | Default | Constraints               |
| ----------- | ------ | -------- | ------- | ------------------------- |
| `url`       | string | yes      | —       | Must be public HTTP/HTTPS |
| `max_chars` | number | no       | 30000   | max 200000                |

### Output Format

```text
Title: Page Title
URL: https://...

{extracted text content}

[Content truncated]   ← only if truncated
```

### URL Safety Rules

All fetch tools enforce the following SSRF protection:

- Must start with `http://` or `https://`
- Must not be a private/internal IP (10.x, 192.168.x, 127.x, etc.)
- Must not be `localhost`

---

## fetch_github_readme

Fetch README from a GitHub repository via the GitHub raw content API.

### fetch_github_readme Parameters

| Parameter | Type   | Required | Constraints                    |
| --------- | ------ | -------- | ------------------------------ |
| `url`     | string | yes      | URL host must be `github.com`  |

### Supported URL Formats

```text
https://github.com/owner/repo
https://github.com/owner/repo/tree/branch
```

### Output

Raw README content (markdown text).

---

## fetch_csdn_article

Fetch full article from CSDN blog.

### fetch_csdn_article Parameters

| Parameter | Type   | Required | Constraints                                  |
| --------- | ------ | -------- | -------------------------------------------- |
| `url`     | string | yes      | URL host must be `csdn.net` or its subdomain |

---

## fetch_juejin_article

Fetch full article from Juejin.

### fetch_juejin_article Parameters

- `url` (`string`, required): URL host must be `juejin.cn` or its subdomain, and the path must contain `/post/`

---

## fetch_zhihu_article

Fetch full article from Zhihu.

### fetch_zhihu_article Parameters

| Parameter | Type   | Required | Constraints                                   |
| --------- | ------ | -------- | --------------------------------------------- |
| `url`     | string | yes      | URL host must be `zhihu.com` or its subdomain |

---

## fetch_linuxdo_article

Fetch full topic from linux.do forum.

### fetch_linuxdo_article Parameters

- `url` (`string`, required): URL host must be `linux.do` or its subdomain, and the path must contain `/topic/`
