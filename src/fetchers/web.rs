use crate::utils::{
    http_client::{build_client, chrome_headers},
    url_safety::assert_public_http_url,
};
use scraper::{Html, Selector};

const DEFAULT_MAX_CHARS: usize = 30_000;
const MAX_DOWNLOAD_BYTES: usize = 2 * 1024 * 1024;

pub struct WebContent {
    pub url: String,
    pub title: String,
    pub content: String,
    pub truncated: bool,
}

fn normalize_text(s: &str) -> String {
    s.replace("\r\n", "\n")
        .replace('\u{00a0}', " ")
        .split('\n')
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .split("\n\n\n")
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn extract_main_text(html: &str) -> (String, String) {
    let doc = Html::parse_document(html);
    let title = Selector::parse("title")
        .ok()
        .and_then(|s| doc.select(&s).next())
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    // Remove noise elements
    // (scraper doesn't mutate, so we work around by collecting text from good containers)
    let containers = [
        "article",
        "main",
        "#main",
        ".main",
        ".content",
        "#content",
        ".post",
        ".article",
        ".entry-content",
        ".post-content",
        ".article-content",
    ];
    for sel_str in &containers {
        if let Ok(sel) = Selector::parse(sel_str)
            && let Some(el) = doc.select(&sel).next()
        {
            let text = el.text().collect::<String>();
            let normalized = normalize_text(&text);
            if normalized.len() > 200 {
                return (title, normalized);
            }
        }
    }

    // Fallback: body minus script/style (scraper text() skips script/style content naturally)
    if let Ok(sel) = Selector::parse("body")
        && let Some(el) = doc.select(&sel).next()
    {
        let text = el.text().collect::<String>();
        return (title, normalize_text(&text));
    }

    (title, String::new())
}

pub async fn fetch_web_content(url: &str, max_chars: Option<usize>) -> anyhow::Result<WebContent> {
    assert_public_http_url(url)?;
    let max = max_chars.unwrap_or(DEFAULT_MAX_CHARS).clamp(1_000, 200_000);
    let client = build_client()?;
    let resp = client.get(url).headers(chrome_headers()).send().await?;

    let final_url = resp.url().to_string();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    // Read up to MAX_DOWNLOAD_BYTES
    let bytes = resp.bytes().await?;
    let raw = &bytes[..bytes.len().min(MAX_DOWNLOAD_BYTES)];
    let body = String::from_utf8_lossy(raw).into_owned();

    // Markdown / plain text paths
    let is_md = url.ends_with(".md")
        || url.ends_with(".markdown")
        || url.ends_with(".mdx")
        || content_type.contains("markdown");

    let (title, mut text) = if is_md {
        let title = url.rsplit('/').next().unwrap_or(url).to_string();
        (title, normalize_text(&body))
    } else if !content_type.contains("html") && !body.to_lowercase().contains("<!doctype html") {
        (String::new(), normalize_text(&body))
    } else {
        extract_main_text(&body)
    };

    let truncated = text.len() > max;
    if truncated {
        // Truncate at a newline boundary if possible
        let cut = text[..max].rfind('\n').unwrap_or(max);
        text.truncate(cut);
    }

    Ok(WebContent {
        url: final_url,
        title,
        content: text,
        truncated,
    })
}
