use crate::utils::{
    http_client::{build_client, chrome_headers},
    url_safety::assert_public_http_url,
};
use scraper::{Html, Selector};

fn normalize_text(s: &str) -> String {
    s.replace("\r\n", "\n")
        .replace('\u{00a0}', " ")
        .split('\n')
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn extract_content(html: &str) -> String {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("#content_views").unwrap();
    if let Some(el) = doc.select(&sel).next() {
        let text = el.text().collect::<String>();
        return normalize_text(&text);
    }
    String::new()
}

pub async fn fetch_csdn_article(url: &str) -> anyhow::Result<String> {
    assert_public_http_url(url)?;
    let client = build_client()?;
    let resp = client
        .get(url)
        .headers(chrome_headers())
        .header("Host", "blog.csdn.net")
        .send()
        .await?;
    let html = resp.text().await?;
    let content = extract_content(&html);
    if content.is_empty() {
        anyhow::bail!("Could not extract CSDN article content from: {}", url);
    }
    Ok(content)
}
