use crate::{config::CONFIG, types::SearchResult, utils::http_client::build_client};
use serde::Deserialize;

const GITHUB_API_REPOS: &str = "https://api.github.com/search/repositories";
const GITHUB_API_CODE: &str = "https://api.github.com/search/code";

#[derive(Deserialize)]
struct RepoItem {
    full_name: String,
    html_url: String,
    description: Option<String>,
    stargazers_count: u64,
    language: Option<String>,
}

#[derive(Deserialize)]
struct RepoResponse {
    items: Vec<RepoItem>,
}

#[derive(Deserialize)]
struct CodeRepo {
    full_name: String,
}

#[derive(Deserialize)]
struct CodeItem {
    name: String,
    html_url: String,
    path: String,
    repository: CodeRepo,
    // score field not used
}

#[derive(Deserialize)]
struct CodeResponse {
    items: Vec<CodeItem>,
}

fn add_auth(builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    if let Some(token) = &CONFIG.github_token {
        builder.header("Authorization", format!("Bearer {}", token))
    } else {
        builder
    }
}

pub async fn search_github_repos(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let client = build_client()?;
    let req = client
        .get(GITHUB_API_REPOS)
        .header("User-Agent", "ferris-search/0.1")
        .header("Accept", "application/vnd.github+json")
        .query(&[("q", query), ("per_page", &limit.to_string())]);
    let req = add_auth(req);
    let resp = req.send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("GitHub API error {}: {}", status, body);
    }
    let data: RepoResponse = resp.json().await?;
    let results = data
        .items
        .into_iter()
        .map(|r| {
            let mut desc_parts = Vec::new();
            if let Some(desc) = &r.description
                && !desc.is_empty()
            {
                desc_parts.push(desc.clone());
            }
            desc_parts.push(format!("Stars: {}", r.stargazers_count));
            if let Some(lang) = &r.language {
                desc_parts.push(format!("Language: {}", lang));
            }
            SearchResult {
                title: r.full_name.clone(),
                url: r.html_url,
                description: desc_parts.join(" | "),
                source: r.full_name,
                engine: "github".into(),
            }
        })
        .collect();
    Ok(results)
}

pub async fn search_github_code(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let client = build_client()?;
    let req = client
        .get(GITHUB_API_CODE)
        .header("User-Agent", "ferris-search/0.1")
        .header("Accept", "application/vnd.github+json")
        .query(&[("q", query), ("per_page", &limit.to_string())]);
    let req = add_auth(req);
    let resp = req.send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("GitHub API error {}: {}", status, body);
    }
    let data: CodeResponse = resp.json().await?;
    let results = data
        .items
        .into_iter()
        .map(|r| SearchResult {
            title: format!("{} — {}", r.repository.full_name, r.name),
            url: r.html_url,
            description: format!("Path: {}", r.path),
            source: r.repository.full_name,
            engine: "github_code".into(),
        })
        .collect();
    Ok(results)
}
