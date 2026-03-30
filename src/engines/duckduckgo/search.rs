use crate::{
    types::SearchResult,
    utils::http_client::{build_client, chrome_headers},
};
use scraper::{Html, Selector};

const DDG_URL: &str = "https://html.duckduckgo.com/html/";

fn parse_html(html: &str, limit: usize) -> Vec<SearchResult> {
    let doc = Html::parse_document(html);
    let sel_result = Selector::parse(".result:not(.result--ad)").unwrap();
    let sel_title = Selector::parse(".result__a").unwrap();
    let sel_snip = Selector::parse(".result__snippet").unwrap();
    let sel_url = Selector::parse(".result__url").unwrap();

    let mut results = Vec::new();
    for el in doc.select(&sel_result) {
        if results.len() >= limit {
            break;
        }
        let link = el.select(&sel_title).next();
        let raw_href = link
            .and_then(|a| a.value().attr("href"))
            .unwrap_or_default();
        // DDG redirects via /l/?kh=-1&uddg=<encoded>
        let url = if raw_href.contains("uddg=") {
            let decoded = raw_href.split("uddg=").nth(1).unwrap_or_default();
            urlencoding::decode(decoded)
                .unwrap_or_default()
                .into_owned()
        } else {
            raw_href.to_string()
        };
        if url.is_empty() {
            continue;
        }
        let title = link
            .map(|a| a.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let description = el
            .select(&sel_snip)
            .next()
            .map(|s| s.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let source = el
            .select(&sel_url)
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
            engine: "duckduckgo".into(),
        });
    }
    results
}

pub async fn search_duckduckgo(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let client = build_client()?;
    let mut all: Vec<SearchResult> = Vec::new();

    // First page via POST
    let params = [("q", query), ("kl", "cn-zh")];
    let mut headers = chrome_headers();
    headers.insert(
        "content-type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );
    let resp = client
        .post(DDG_URL)
        .headers(headers.clone())
        .form(&params)
        .send()
        .await?;
    let html = resp.text().await?;
    let results = parse_html(&html, limit);
    all.extend(results);

    // Additional pages via GET with &s= offset
    let mut offset = 30usize;
    while all.len() < limit {
        let url = format!(
            "{}?q={}&kl=cn-zh&s={}",
            DDG_URL,
            urlencoding::encode(query),
            offset
        );
        let resp = client.get(&url).headers(headers.clone()).send().await?;
        let html = resp.text().await?;
        let page = parse_html(&html, limit - all.len());
        if page.is_empty() {
            break;
        }
        all.extend(page);
        offset += 30;
    }

    all.truncate(limit);
    Ok(all)
}
