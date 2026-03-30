use crate::{
    types::SearchResult,
    utils::http_client::{build_client, chrome_headers},
};
use reqwest::header::{HeaderName, HeaderValue};
use scraper::{Html, Selector};

const BRAVE_BASE: &str = "https://search.brave.com/search";

fn parse_results(html: &str, limit: usize) -> Vec<SearchResult> {
    let doc = Html::parse_document(html);
    // Brave uses data-type="web" on result items
    let sel_item = Selector::parse("[data-type='web']").unwrap();
    let sel_title = Selector::parse("span.title, .snippet-title, h3").unwrap();
    let sel_url = Selector::parse("a").unwrap();
    let sel_desc = Selector::parse(".snippet-description, .snippet p").unwrap();
    let sel_src = Selector::parse(".netloc, .site-name").unwrap();

    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for el in doc.select(&sel_item) {
        if results.len() >= limit {
            break;
        }
        let link = el.select(&sel_url).next();
        let url = link
            .and_then(|a| a.value().attr("href"))
            .map(|s| s.to_string())
            .unwrap_or_default();
        if url.is_empty() || !url.starts_with("http") || seen.contains(&url) {
            continue;
        }
        let title = el
            .select(&sel_title)
            .next()
            .map(|s| s.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let description = el
            .select(&sel_desc)
            .next()
            .map(|s| s.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let source = el
            .select(&sel_src)
            .next()
            .map(|s| s.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| {
                url::Url::parse(&url)
                    .ok()
                    .and_then(|u| u.host_str().map(|h| h.to_string()))
                    .unwrap_or_default()
            });
        if title.is_empty() {
            continue;
        }
        seen.insert(url.clone());
        results.push(SearchResult {
            title,
            url,
            description,
            source,
            engine: "brave".into(),
        });
    }
    results
}

pub async fn search_brave(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let client = build_client()?;
    let mut all: Vec<SearchResult> = Vec::new();
    let mut offset = 0usize;

    while all.len() < limit {
        let url = format!(
            "{}?q={}&source=web&offset={}",
            BRAVE_BASE,
            urlencoding::encode(query),
            offset
        );
        let mut headers = chrome_headers();
        headers.insert(
            HeaderName::from_static("referer"),
            HeaderValue::from_static("https://search.brave.com/"),
        );
        headers.insert(
            HeaderName::from_static("sec-ch-ua"),
            HeaderValue::from_static(
                "\"Chromium\";v=\"133\", \"Not(A:Brand\";v=\"99\", \"Google Chrome\";v=\"133\"",
            ),
        );
        headers.insert(
            HeaderName::from_static("sec-ch-ua-mobile"),
            HeaderValue::from_static("?0"),
        );
        headers.insert(
            HeaderName::from_static("sec-ch-ua-platform"),
            HeaderValue::from_static("\"Windows\""),
        );
        headers.insert(
            HeaderName::from_static("sec-fetch-dest"),
            HeaderValue::from_static("document"),
        );
        headers.insert(
            HeaderName::from_static("sec-fetch-mode"),
            HeaderValue::from_static("navigate"),
        );
        headers.insert(
            HeaderName::from_static("sec-fetch-site"),
            HeaderValue::from_static("same-origin"),
        );
        headers.insert(
            HeaderName::from_static("sec-fetch-user"),
            HeaderValue::from_static("?1"),
        );
        headers.insert(
            HeaderName::from_static("upgrade-insecure-requests"),
            HeaderValue::from_static("1"),
        );

        let resp = client.get(&url).headers(headers).send().await?;
        let html = resp.text().await?;
        let results = parse_results(&html, limit - all.len());
        if results.is_empty() {
            break;
        }
        all.extend(results);
        offset += 10;
    }
    all.truncate(limit);
    Ok(all)
}
