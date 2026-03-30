use super::super::{bing::search_bing, brave::search_brave, duckduckgo::search_duckduckgo};
use crate::{config::CONFIG, types::SearchResult};

pub async fn search_zhihu(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let site_query = format!("site:zhihu.com {}", query);
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
                        .map(|h| h.ends_with("zhihu.com"))
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        })
        .map(|mut r| {
            r.source = "zhihu.com".into();
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
