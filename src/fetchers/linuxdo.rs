use crate::utils::http_client::{build_client, chrome_headers};
use scraper::Html;
use serde::Deserialize;

#[derive(Deserialize)]
struct DiscoursePost {
    cooked: Option<String>,
}

#[derive(Deserialize)]
struct PostStream {
    posts: Vec<DiscoursePost>,
}

#[derive(Deserialize)]
struct TopicResponse {
    post_stream: Option<PostStream>,
    title: Option<String>,
}

pub async fn fetch_linuxdo_article(url: &str) -> anyhow::Result<String> {
    let re = regex::Regex::new(r"/topic/(\d+)").unwrap();
    let topic_id = re
        .captures(url)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| anyhow::anyhow!("Invalid URL: cannot extract topic ID from: {}", url))?;

    let api_url = format!("https://linux.do/t/{}.json", topic_id);
    let client = build_client()?;
    let resp = client
        .get(&api_url)
        .header("accept", "application/json, text/javascript, */*; q=0.01")
        .header("accept-language", "zh-CN,zh;q=0.9")
        .header("referer", "https://linux.do/search")
        .header("x-requested-with", "XMLHttpRequest")
        .headers(chrome_headers())
        .send()
        .await?;

    let data: TopicResponse = resp.json().await?;
    let cooked = data
        .post_stream
        .and_then(|ps| ps.posts.into_iter().next())
        .and_then(|p| p.cooked)
        .unwrap_or_default();

    let title = data.title.unwrap_or_default();
    let doc = Html::parse_fragment(&cooked);
    let body_text: String = doc.root_element().text().collect();
    let content = body_text.trim().to_string();

    if content.is_empty() {
        anyhow::bail!(
            "Could not extract LinuxDo article content from topic: {}",
            topic_id
        );
    }

    if title.is_empty() {
        Ok(content)
    } else {
        Ok(format!("# {}\n\n{}", title, content))
    }
}
