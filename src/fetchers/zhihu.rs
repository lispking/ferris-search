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
    for sel_str in &["#content", ".RichText.ztext", "article", "main"] {
        if let Ok(sel) = Selector::parse(sel_str)
            && let Some(el) = doc.select(&sel).next()
        {
            let text = el.text().collect::<String>();
            let normalized = normalize_text(&text);
            if normalized.len() > 100 {
                return normalized;
            }
        }
    }
    String::new()
}

pub async fn fetch_zhihu_article(url: &str) -> anyhow::Result<String> {
    assert_public_http_url(url)?;
    let client = build_client()?;
    let mut headers = chrome_headers();
    headers.insert("referer", "https://www.zhihu.com/".parse().unwrap());
    let resp = client.get(url).headers(headers).send().await?;
    let html = resp.text().await?;
    let content = extract_content(&html);
    if content.is_empty() {
        anyhow::bail!("Could not extract Zhihu article content from: {}", url);
    }
    Ok(content)
}
