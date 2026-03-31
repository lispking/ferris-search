use crate::{
    config::CONFIG,
    engines::{
        baidu::search_baidu,
        bing::search_bing,
        brave::search_brave,
        csdn::search_csdn,
        duckduckgo::search_duckduckgo,
        exa::search_exa,
        firecrawl::search_firecrawl,
        github::{search_github_code, search_github_repos},
        jina::search_jina,
        juejin::search_juejin,
        linuxdo::search_linuxdo,
        tavily::search_tavily,
        zhihu::search_zhihu,
    },
    types::SearchResult,
};

pub fn normalize_engine(s: &str) -> String {
    let cleaned = s.trim().to_lowercase();
    match cleaned.as_str() {
        "duckduckgo" | "duck duck go" | "ddg" => "duckduckgo".into(),
        "bing" | "microsoft bing" => "bing".into(),
        "brave" | "brave search" => "brave".into(),
        "baidu" | "百度" => "baidu".into(),
        "csdn" => "csdn".into(),
        "juejin" | "掘金" => "juejin".into(),
        "exa" => "exa".into(),
        "zhihu" | "知乎" => "zhihu".into(),
        "linuxdo" | "linux.do" => "linuxdo".into(),
        "jina" | "jina.ai" => "jina".into(),
        "tavily" => "tavily".into(),
        "firecrawl" => "firecrawl".into(),
        "github" | "github repos" | "github repo" => "github".into(),
        "github_code" | "github code" => "github_code".into(),
        _ => cleaned,
    }
}

pub fn results_to_text(results: &[SearchResult]) -> String {
    results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            format!(
                "{}. **{}**\nURL: {}\nSource: {}\nDescription: {}\n",
                i + 1,
                r.title,
                r.url,
                r.source,
                r.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_results(engine: &str, results: &[SearchResult]) -> String {
    format!(
        "Engine: {}\nTotal: {}\n\n{}",
        engine,
        results.len(),
        results_to_text(results)
    )
}

pub async fn do_search(
    engine: &str,
    query: &str,
    limit: usize,
) -> anyhow::Result<Vec<SearchResult>> {
    match engine {
        "bing" => search_bing(query, limit).await,
        "duckduckgo" => search_duckduckgo(query, limit).await,
        "brave" => search_brave(query, limit).await,
        "baidu" => search_baidu(query, limit).await,
        "csdn" => search_csdn(query, limit).await,
        "juejin" => search_juejin(query, limit).await,
        "exa" => search_exa(query, limit).await,
        "firecrawl" => search_firecrawl(query, limit).await,
        "zhihu" => search_zhihu(query, limit).await,
        "linuxdo" => search_linuxdo(query, limit).await,
        "jina" => search_jina(query, limit).await,
        "tavily" => search_tavily(query, limit).await,
        "github" => search_github_repos(query, limit).await,
        "github_code" => search_github_code(query, limit).await,
        other => anyhow::bail!("Unknown search engine: {}", other),
    }
}

/// All supported engine names.
pub const ALL_ENGINES: &[&str] = &[
    "baidu",
    "bing",
    "brave",
    "csdn",
    "duckduckgo",
    "exa",
    "firecrawl",
    "github",
    "github_code",
    "jina",
    "juejin",
    "linuxdo",
    "tavily",
    "zhihu",
];

/// Resolve a list of engine names: normalize, filter against allowed list, fall back to default.
pub fn resolve_engines(raw: &[String]) -> Vec<String> {
    let engines: Vec<String> = if raw.is_empty() {
        vec![CONFIG.default_search_engine.clone()]
    } else {
        raw.iter().map(|e| normalize_engine(e)).collect()
    };
    engines
        .into_iter()
        .filter(|e| CONFIG.is_engine_allowed(e))
        .collect()
}

/// Result from a multi-engine search: successes and failures.
pub struct MultiSearchResult {
    pub results: Vec<(String, Vec<SearchResult>)>,
    pub errors: Vec<(String, String)>,
}

/// Multi-engine concurrent search. Returns successes and failures separately.
pub async fn do_multi_search(engines: &[String], query: &str, limit: usize) -> MultiSearchResult {
    let mut handles = Vec::new();
    for engine in engines {
        let engine = engine.clone();
        let query = query.to_string();
        handles.push(tokio::spawn(async move {
            let res = do_search(&engine, &query, limit).await;
            (engine, res)
        }));
    }

    let mut results = Vec::new();
    let mut errors = Vec::new();
    for handle in handles {
        match handle.await {
            Ok((engine, Ok(r))) => {
                if !r.is_empty() {
                    results.push((engine, r));
                }
            }
            Ok((engine, Err(e))) => {
                errors.push((engine, e.to_string()));
            }
            Err(e) => {
                errors.push(("unknown".to_string(), e.to_string()));
            }
        }
    }
    MultiSearchResult { results, errors }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_standard_names() {
        assert_eq!(normalize_engine("bing"), "bing");
        assert_eq!(normalize_engine("duckduckgo"), "duckduckgo");
        assert_eq!(normalize_engine("brave"), "brave");
        assert_eq!(normalize_engine("github"), "github");
        assert_eq!(normalize_engine("github_code"), "github_code");
    }

    #[test]
    fn normalize_aliases() {
        assert_eq!(normalize_engine("ddg"), "duckduckgo");
        assert_eq!(normalize_engine("duck duck go"), "duckduckgo");
        assert_eq!(normalize_engine("microsoft bing"), "bing");
        assert_eq!(normalize_engine("brave search"), "brave");
        assert_eq!(normalize_engine("github repos"), "github");
        assert_eq!(normalize_engine("github code"), "github_code");
    }

    #[test]
    fn normalize_chinese_aliases() {
        assert_eq!(normalize_engine("百度"), "baidu");
        assert_eq!(normalize_engine("掘金"), "juejin");
        assert_eq!(normalize_engine("知乎"), "zhihu");
    }

    #[test]
    fn normalize_case_insensitive() {
        assert_eq!(normalize_engine("BING"), "bing");
        assert_eq!(normalize_engine("DuckDuckGo"), "duckduckgo");
        assert_eq!(normalize_engine("  Brave  "), "brave");
    }

    #[test]
    fn normalize_unknown_passthrough() {
        assert_eq!(normalize_engine("unknown_engine"), "unknown_engine");
    }

    #[test]
    fn resolve_engines_uses_default_when_empty() {
        let resolved = resolve_engines(&[]);

        if CONFIG.is_engine_allowed(&CONFIG.default_search_engine) {
            assert_eq!(resolved, vec![CONFIG.default_search_engine.clone()]);
        } else {
            assert!(resolved.is_empty());
        }
    }

    #[test]
    fn resolve_engines_normalizes_and_filters() {
        let raw = vec![
            " ddg ".to_string(),
            "github code".to_string(),
            "unknown_engine".to_string(),
        ];

        let resolved = resolve_engines(&raw);

        let mut expected = Vec::new();
        if CONFIG.is_engine_allowed("duckduckgo") {
            expected.push("duckduckgo".to_string());
        }
        if CONFIG.is_engine_allowed("github_code") {
            expected.push("github_code".to_string());
        }

        assert_eq!(resolved, expected);
        assert!(
            resolved
                .iter()
                .all(|engine| CONFIG.is_engine_allowed(engine))
        );
    }

    #[tokio::test]
    async fn do_multi_search_collects_unknown_engine_errors() {
        let engines = vec!["unknown_engine".to_string(), "another_unknown".to_string()];

        let result = do_multi_search(&engines, "rust", 3).await;

        assert!(result.results.is_empty());
        assert_eq!(result.errors.len(), 2);
        assert_eq!(result.errors[0].0, "unknown_engine");
        assert!(result.errors[0].1.contains("Unknown search engine"));
        assert_eq!(result.errors[1].0, "another_unknown");
        assert!(result.errors[1].1.contains("Unknown search engine"));
    }
}
