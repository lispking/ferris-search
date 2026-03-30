use crate::{
    types::SearchResult,
    utils::http_client::{build_client, chrome_headers},
};
use scraper::{Html, Selector};

const BAIDU_BASE: &str = "https://www.baidu.com/s";

fn parse_results(html: &str, limit: usize) -> Vec<SearchResult> {
    let doc = Html::parse_document(html);
    let sel_item = Selector::parse("div.c-container, div[tpl]").unwrap();
    let sel_title = Selector::parse("h3.t a, h3 a").unwrap();
    let sel_desc = Selector::parse(".c-font-normal.c-color-text, .c-abstract").unwrap();
    let sel_src = Selector::parse(".cosc-source, .c-showurl").unwrap();

    let mut results = Vec::new();
    for el in doc.select(&sel_item) {
        if results.len() >= limit {
            break;
        }
        let link = el.select(&sel_title).next();
        let url = link
            .and_then(|a| a.value().attr("href"))
            .map(|s| s.to_string())
            .unwrap_or_default();
        if url.is_empty() || !url.starts_with("http") {
            continue;
        }
        let title = link
            .map(|a| a.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let description = el
            .select(&sel_desc)
            .next()
            .map(|d| d.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let source = el
            .select(&sel_src)
            .next()
            .map(|s| s.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        if title.is_empty() {
            continue;
        }
        results.push(SearchResult {
            title,
            url,
            description,
            source,
            engine: "baidu".into(),
        });
    }
    results
}

pub async fn search_baidu(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let client = build_client()?;
    let mut all: Vec<SearchResult> = Vec::new();
    let mut pn = 0usize;

    while all.len() < limit {
        let url = format!("{}?wd={}&pn={}", BAIDU_BASE, urlencoding::encode(query), pn);
        let mut headers = chrome_headers();
        headers.insert("referer", "https://www.baidu.com/".parse().unwrap());

        let resp = client.get(&url).headers(headers).send().await?;
        let html = resp.text().await?;
        let results = parse_results(&html, limit - all.len());
        if results.is_empty() {
            break;
        }
        all.extend(results);
        pn += 10;
    }
    all.truncate(limit);
    Ok(all)
}
