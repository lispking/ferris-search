use crate::{
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
