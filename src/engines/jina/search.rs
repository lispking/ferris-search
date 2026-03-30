use crate::{config::CONFIG, types::SearchResult, utils::http_client::build_client};
use serde::Deserialize;

const JINA_SEARCH_URL: &str = "https://s.jina.ai/";

#[derive(Deserialize)]
struct JinaResult {
    title: Option<String>,
    url: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct JinaResponse {
    data: Vec<JinaResult>,
}

pub async fn search_jina(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let api_key = CONFIG.jina_api_key.as_deref().ok_or_else(|| {
        anyhow::anyhow!("JINA_API_KEY environment variable is required for Jina search")
    })?;

    let client = build_client()?;
    let url = format!("{}{}", JINA_SEARCH_URL, urlencoding::encode(query));

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Accept", "application/json")
        .header("X-Retain-Images", "none")
        .send()
        .await?;

    let data: JinaResponse = resp.json().await?;
    let results = data
        .data
        .into_iter()
        .take(limit)
        .map(|r| SearchResult {
            title: r.title.unwrap_or_default(),
            url: r.url,
            description: r.description.unwrap_or_default(),
            source: "s.jina.ai".into(),
            engine: "jina".into(),
        })
        .collect();

    Ok(results)
}
