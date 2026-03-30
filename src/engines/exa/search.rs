use crate::{config::CONFIG, types::SearchResult, utils::http_client::build_client};
use serde::{Deserialize, Serialize};

const EXA_API: &str = "https://api.exa.ai/search";

#[derive(Serialize)]
struct ExaRequest<'a> {
    query: &'a str,
    #[serde(rename = "numResults")]
    num_results: usize,
    #[serde(rename = "useAutoprompt")]
    use_autoprompt: bool,
}

#[derive(Deserialize)]
struct ExaResult {
    title: Option<String>,
    url: String,
    #[serde(rename = "publishedDate")]
    published_date: Option<String>,
    author: Option<String>,
    score: Option<f64>,
}

#[derive(Deserialize)]
struct ExaResponse {
    results: Vec<ExaResult>,
}

pub async fn search_exa(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let api_key = CONFIG.exa_api_key.as_deref().ok_or_else(|| {
        anyhow::anyhow!("EXA_API_KEY environment variable is required for Exa search")
    })?;

    let client = build_client()?;
    let req = ExaRequest {
        query,
        num_results: limit,
        use_autoprompt: true,
    };

    let resp = client
        .post(EXA_API)
        .header("x-api-key", api_key)
        .json(&req)
        .send()
        .await?;

    let data: ExaResponse = resp.json().await?;
    let results = data
        .results
        .into_iter()
        .map(|r| {
            let description = match (r.published_date.as_deref(), r.score) {
                (Some(date), Some(score)) => format!("Published: {} | Score: {:.2}", date, score),
                (Some(date), None) => format!("Published: {}", date),
                (None, Some(score)) => format!("Score: {:.2}", score),
                _ => String::new(),
            };
            SearchResult {
                title: r.title.unwrap_or_default(),
                url: r.url,
                description,
                source: r.author.unwrap_or_default(),
                engine: "exa".into(),
            }
        })
        .collect();

    Ok(results)
}
