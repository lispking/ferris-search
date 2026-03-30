use crate::{
    types::SearchResult,
    utils::http_client::{build_client, chrome_headers},
};
use reqwest::header::{CACHE_CONTROL, PRAGMA, UPGRADE_INSECURE_REQUESTS};
use scraper::{Html, Selector};

const BING_BASE: &str = "https://cn.bing.com/search";

fn build_url(query: &str, page: usize) -> String {
    format!(
        "{}?q={}&setlang=zh-CN&ensearch=0&first={}",
        BING_BASE,
        urlencoding::encode(query),
        1 + page * 10
    )
}

fn parse_results(html: &str, limit: usize) -> Vec<SearchResult> {
    let doc = Html::parse_document(html);
    let sel_algo = Selector::parse("#b_results li.b_algo").unwrap();
    let sel_h2a = Selector::parse("h2 a").unwrap();
    let sel_cap = Selector::parse(".b_caption p, .b_dList li, p").unwrap();
    let sel_src = Selector::parse(".b_attribution cite, cite").unwrap();

    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for el in doc.select(&sel_algo) {
        if results.len() >= limit {
            break;
        }
        let link = el.select(&sel_h2a).next();
        let url = link
            .and_then(|a| a.value().attr("href"))
            .map(|s| s.to_string())
            .unwrap_or_default();
        if url.is_empty() || seen.contains(&url) {
            continue;
        }
        if url.contains("bing.com/search") || url.contains("bing.com/ck/a") {
            continue;
        }
        let title = link
            .map(|a| a.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();
        let description = el
            .select(&sel_cap)
            .next()
            .map(|p| p.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let source = el
            .select(&sel_src)
            .next()
            .map(|c| c.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if title.is_empty() && description.is_empty() {
            continue;
        }
        seen.insert(url.clone());
        results.push(SearchResult {
            title,
            url,
            description,
            source,
            engine: "bing".into(),
        });
    }
    results
}

pub async fn search_bing(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let client = build_client()?;
    let mut all: Vec<SearchResult> = Vec::new();
    let mut page = 0usize;

    while all.len() < limit {
        let url = build_url(query, page);
        let mut headers = chrome_headers();
        headers.insert(CACHE_CONTROL, "no-cache".parse().unwrap());
        headers.insert(PRAGMA, "no-cache".parse().unwrap());
        headers.insert(UPGRADE_INSECURE_REQUESTS, "1".parse().unwrap());

        let resp = client.get(&url).headers(headers).send().await?;
        let html = resp.text().await?;
        let results = parse_results(&html, limit - all.len());
        if results.is_empty() {
            break;
        }
        all.extend(results);
        page += 1;
    }

    Ok(all)
}
