use crate::{config::CONFIG, types::SearchResult, utils::http_client::build_client};
use serde::Deserialize;

const BRAVE_API: &str = "https://api.search.brave.com/res/v1/web/search";

#[derive(Deserialize)]
struct BraveWebResult {
    title: String,
    url: String,
    description: Option<String>,
    #[serde(rename = "profile")]
    profile: Option<BraveProfile>,
}

#[derive(Deserialize)]
struct BraveProfile {
    name: Option<String>,
}

#[derive(Deserialize)]
struct BraveWebResults {
    results: Option<Vec<BraveWebResult>>,
}

#[derive(Deserialize)]
struct BraveResponse {
    web: Option<BraveWebResults>,
}

pub async fn search_brave(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let api_key = CONFIG.brave_api_key.as_deref().ok_or_else(|| {
        anyhow::anyhow!("BRAVE_API_KEY environment variable is required for Brave Search")
    })?;

    let client = build_client()?;
    let resp = client
        .get(BRAVE_API)
        .header("X-Subscription-Token", api_key)
        .header("Accept", "application/json")
        .query(&[("q", query), ("count", &limit.min(20).to_string())])
        .send()
        .await?;

    let status = resp.status();
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(anyhow::anyhow!(
            "Brave Search rate limited (429): try again later"
        ));
    }
    if !status.is_success() {
        return Err(anyhow::anyhow!("Brave Search returned HTTP {}", status));
    }

    let data: BraveResponse = resp.json().await?;
    let results = data
        .web
        .and_then(|w| w.results)
        .unwrap_or_default()
        .into_iter()
        .take(limit)
        .map(|r| {
            let source = r
                .profile
                .and_then(|p| p.name)
                .unwrap_or_else(|| r.url.split('/').nth(2).unwrap_or_default().to_string());
            SearchResult {
                title: r.title,
                url: r.url,
                description: r.description.unwrap_or_default(),
                source,
                engine: "brave".into(),
            }
        })
        .collect();

    Ok(results)
}
