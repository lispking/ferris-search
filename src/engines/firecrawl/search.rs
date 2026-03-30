use crate::{config::CONFIG, types::SearchResult, utils::http_client::build_client};
use serde::{Deserialize, Serialize};

const FIRECRAWL_API: &str = "https://api.firecrawl.dev/v1/search";

#[derive(Serialize)]
struct FirecrawlRequest<'a> {
    query: &'a str,
    limit: usize,
}

#[derive(Deserialize)]
struct FirecrawlResult {
    url: String,
    title: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct FirecrawlResponse {
    data: Vec<FirecrawlResult>,
}

pub async fn search_firecrawl(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let api_key = CONFIG.firecrawl_api_key.as_deref().ok_or_else(|| {
        anyhow::anyhow!("FIRECRAWL_API_KEY environment variable is required for Firecrawl search")
    })?;

    let client = build_client()?;
    let req = FirecrawlRequest { query, limit };

    let resp = client
        .post(FIRECRAWL_API)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        return Err(anyhow::anyhow!("Firecrawl returned HTTP {}", status));
    }

    let data: FirecrawlResponse = resp.json().await?;
    let results = data
        .data
        .into_iter()
        .take(limit)
        .map(|r| {
            let source = r.url.split('/').nth(2).unwrap_or_default().to_string();
            SearchResult {
                title: r.title.unwrap_or_default(),
                url: r.url,
                description: r.description.unwrap_or_default(),
                source,
                engine: "firecrawl".into(),
            }
        })
        .collect();

    Ok(results)
}
