use super::super::{bing::search_bing, brave::search_brave, duckduckgo::search_duckduckgo};
use crate::{config::CONFIG, types::SearchResult};

pub async fn search_linuxdo(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let site_query = format!("site:linux.do {}", query);
    let mut results = dispatch_search(&site_query, limit).await?;

    if results.is_empty() && CONFIG.default_search_engine != "brave" {
        results = search_brave(&site_query, limit).await.unwrap_or_default();
    }

    let filtered: Vec<SearchResult> = results
        .into_iter()
        .filter(|r| {
            url::Url::parse(&r.url)
                .ok()
                .map(|u| {
                    u.host_str()
                        .map(|h| h == "linux.do" || h.ends_with(".linux.do"))
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        })
        .map(|mut r| {
            r.source = "linux.do".into();
            r
        })
        .take(limit)
        .collect();

    Ok(filtered)
}

async fn dispatch_search(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    match CONFIG.default_search_engine.as_str() {
        "duckduckgo" => search_duckduckgo(query, limit).await,
        "brave" => search_brave(query, limit).await,
        _ => search_bing(query, limit).await,
    }
}
