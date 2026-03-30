use crate::{types::SearchResult, utils::http_client::build_client};
use serde::{Deserialize, Serialize};

const JUEJIN_API: &str = "https://api.juejin.cn/search_api/v1/article/search";

#[derive(Serialize)]
struct JuejinReq<'a> {
    key_word: &'a str,
    limit: u32,
    cursor: &'a str,
    sort_type: u32,
}

#[derive(Deserialize)]
struct AuthorInfo {
    user_name: Option<String>,
}

#[derive(Deserialize)]
struct ArticleInfo {
    brief_content: Option<String>,
    view_count: Option<u64>,
    digg_count: Option<u64>,
}

#[derive(Deserialize)]
struct ResultModel {
    article_id: String,
    article_info: ArticleInfo,
    author_user_info: AuthorInfo,
}

#[derive(Deserialize)]
struct Item {
    result_model: ResultModel,
    #[allow(dead_code)]
    title: Option<String>,
}

#[derive(Deserialize)]
struct JuejinResponse {
    data: Option<Vec<Item>>,
    cursor: Option<String>,
    has_more: Option<bool>,
}

pub async fn search_juejin(query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
    let client = build_client()?;
    let mut all: Vec<SearchResult> = Vec::new();
    let mut cursor = "0".to_string();

    while all.len() < limit {
        let req = JuejinReq {
            key_word: query,
            limit: 10,
            cursor: &cursor,
            sort_type: 0,
        };
        let resp = client.post(JUEJIN_API).json(&req).send().await?;
        let data: JuejinResponse = resp.json().await?;

        let items = match data.data {
            Some(v) if !v.is_empty() => v,
            _ => break,
        };

        for item in &items {
            if all.len() >= limit {
                break;
            }
            let rm = &item.result_model;
            let ai = &rm.article_info;
            let title = item.title.clone().unwrap_or_default();
            if title.is_empty() {
                continue;
            }
            let description = format!(
                "{} | views: {} | likes: {}",
                ai.brief_content.as_deref().unwrap_or_default(),
                ai.view_count.unwrap_or(0),
                ai.digg_count.unwrap_or(0)
            );
            all.push(SearchResult {
                title,
                url: format!("https://juejin.cn/post/{}", rm.article_id),
                description,
                source: rm.author_user_info.user_name.clone().unwrap_or_default(),
                engine: "juejin".into(),
            });
        }

        if data.has_more != Some(true) || data.cursor.is_none() {
            break;
        }
        cursor = data.cursor.unwrap();
    }

    Ok(all)
}
