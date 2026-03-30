use crate::{types::SearchResult, utils::http_client::build_client};
use serde::Deserialize;

const CSDN_API: &str = "https://so.csdn.net/api/v3/search";

#[derive(Deserialize)]
struct CsdnItem {
    title: Option<String>,
    url_location: Option<String>,
    digest: Option<String>,
    nickname: Option<String>,
}

#[derive(Deserialize)]
struct CsdnResponse {
    result_vos: Option<Vec<CsdnItem>>,
}

pub async fn search_csdn(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let client = build_client()?;
    let mut all: Vec<SearchResult> = Vec::new();
    let mut page = 1u32;

    while all.len() < limit {
        let resp = client
            .get(CSDN_API)
            .query(&[("q", query), ("p", &page.to_string())])
            .header("User-Agent", "Apifox/1.0.0 (https://apifox.com)")
            .header("Accept", "*/*")
            .header("Host", "so.csdn.net")
            .send()
            .await?;

        let data: CsdnResponse = resp.json().await?;
        let items = match data.result_vos {
            Some(v) if !v.is_empty() => v,
            _ => break,
        };

        for item in items {
            if all.len() >= limit {
                break;
            }
            let title = item.title.unwrap_or_default();
            let url = item.url_location.unwrap_or_default();
            if url.is_empty() {
                continue;
            }
            all.push(SearchResult {
                title,
                url,
                description: item.digest.unwrap_or_default(),
                source: item.nickname.unwrap_or_default(),
                engine: "csdn".into(),
            });
        }
        page += 1;
    }

    Ok(all)
}
