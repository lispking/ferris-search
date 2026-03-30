use crate::{config::CONFIG, types::SearchResult, utils::http_client::build_client};
use serde::{Deserialize, Serialize};

const TAVILY_API: &str = "https://api.tavily.com/search";

#[derive(Serialize)]
struct TavilyRequest<'a> {
    query: &'a str,
    max_results: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_answer: Option<bool>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: Option<String>,
    url: String,
    content: Option<String>,
    score: Option<f64>,
}

#[derive(Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

pub async fn search_tavily(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let api_key = CONFIG.tavily_api_key.as_deref().ok_or_else(|| {
        anyhow::anyhow!("TAVILY_API_KEY environment variable is required for Tavily search")
    })?;

    let client = build_client()?;
    let req = TavilyRequest {
        query,
        max_results: limit,
        include_answer: None,
    };

    let resp = client
        .post(TAVILY_API)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req)
        .send()
        .await?;

    let data: TavilyResponse = resp.json().await?;
    let results = data
        .results
        .into_iter()
        .take(limit)
        .map(|r| {
            let description = match (r.content.as_deref(), r.score) {
                (Some(c), Some(s)) => format!("{} | Score: {:.2}", c, s),
                (Some(c), None) => c.to_string(),
                (None, Some(s)) => format!("Score: {:.2}", s),
                _ => String::new(),
            };
            SearchResult {
                title: r.title.unwrap_or_default(),
                url: r.url.clone(),
                description,
                source: r.url
                    .split('/')
                    .nth(2)
                    .unwrap_or_default()
                    .to_string(),
                engine: "tavily".into(),
            }
        })
        .collect();

    Ok(results)
}
