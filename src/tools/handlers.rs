use super::helpers::{do_search, format_results, normalize_engine, results_to_text};
use super::params::{ArticleUrlParams, FetchUrlParams, GithubReadmeParams, WebSearchParams};
use crate::{
    config::CONFIG,
    fetchers::{
        csdn::fetch_csdn_article, github::fetch_github_readme, juejin::fetch_juejin_article,
        linuxdo::fetch_linuxdo_article, web::fetch_web_content, zhihu::fetch_zhihu_article,
    },
};
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};

#[derive(Debug, Clone)]
pub struct WebSearchHandler {
    tool_router: ToolRouter<Self>,
}

impl WebSearchHandler {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl WebSearchHandler {
    /// Search the web using the configured search engine
    #[tool(
        name = "web_search",
        description = "Search the web using one or more engines. engines parameter accepts an array: [\"bing\", \"duckduckgo\", ...]. Supported: bing, duckduckgo, brave, baidu, csdn, juejin, exa, firecrawl, zhihu, linuxdo, jina, tavily, github (repo search), github_code (code search)."
    )]
    pub async fn web_search(&self, p: Parameters<WebSearchParams>) -> String {
        let params = p.0;
        let limit = params.limit.unwrap_or(10).clamp(1, 50) as usize;

        // Resolve engine list
        let engines: Vec<String> = match params.engines {
            Some(list) if !list.is_empty() => list.iter().map(|e| normalize_engine(e)).collect(),
            _ => vec![CONFIG.default_search_engine.clone()],
        };

        // Filter against allowed engines
        let engines: Vec<String> = engines
            .into_iter()
            .filter(|e| CONFIG.is_engine_allowed(e))
            .collect();

        if engines.is_empty() {
            return "No allowed engines specified.".into();
        }

        if engines.len() == 1 {
            // Single engine — simple path
            match do_search(&engines[0], &params.query, limit).await {
                Ok(results) if results.is_empty() => return "No results found.".into(),
                Ok(results) => return format_results(&engines[0], &results),
                Err(e) => return format!("Search failed: {}", e),
            }
        }

        // Multi-engine fan-out
        let mut handles = Vec::new();
        for engine in engines.clone() {
            let query = params.query.clone();
            handles.push(tokio::spawn(async move {
                let res = do_search(&engine, &query, limit).await;
                (engine, res)
            }));
        }

        let mut output = String::new();
        let mut total = 0usize;
        for handle in handles {
            if let Ok((engine, Ok(results))) = handle.await
                && !results.is_empty()
            {
                total += results.len();
                output.push_str(&format!("## Results from {}\n\n", engine));
                output.push_str(&results_to_text(&results));
                output.push('\n');
            }
        }

        if output.is_empty() {
            "No results found.".into()
        } else {
            format!("Total results: {}\n\n{}", total, output)
        }
    }

    /// Fetch the content of a web page
    #[tool(
        name = "fetch_web_content",
        description = "Fetch and extract text content from any public web URL."
    )]
    pub async fn fetch_web_content_tool(&self, p: Parameters<FetchUrlParams>) -> String {
        let params = p.0;
        let max_chars = params.max_chars.map(|c| c as usize);
        match fetch_web_content(&params.url, max_chars).await {
            Ok(result) => {
                let truncated_note = if result.truncated {
                    "\n\n[Content truncated]"
                } else {
                    ""
                };
                format!(
                    "Title: {}\nURL: {}\n\n{}{}",
                    result.title, result.url, result.content, truncated_note
                )
            }
            Err(e) => format!("Failed to fetch URL: {}", e),
        }
    }

    /// Fetch a GitHub repository README
    #[tool(
        name = "fetch_github_readme",
        description = "Fetch the README content from a GitHub repository URL."
    )]
    pub async fn fetch_github_readme_tool(&self, p: Parameters<GithubReadmeParams>) -> String {
        let url = p.0.url;
        if !url.contains("github.com") {
            return "URL must be from github.com".into();
        }
        match fetch_github_readme(&url).await {
            Ok(content) => content,
            Err(e) => format!("Failed to fetch GitHub README: {}", e),
        }
    }

    /// Fetch a CSDN article
    #[tool(
        name = "fetch_csdn_article",
        description = "Fetch full article content from a CSDN (csdn.net) post URL."
    )]
    pub async fn fetch_csdn_article_tool(&self, p: Parameters<ArticleUrlParams>) -> String {
        let url = p.0.url;
        if !url.contains("csdn.net") {
            return "URL must be from csdn.net".into();
        }
        match fetch_csdn_article(&url).await {
            Ok(content) => content,
            Err(e) => format!("Failed to fetch CSDN article: {}", e),
        }
    }

    /// Fetch a Juejin article
    #[tool(
        name = "fetch_juejin_article",
        description = "Fetch full article content from a Juejin (juejin.cn) post URL."
    )]
    pub async fn fetch_juejin_article_tool(&self, p: Parameters<ArticleUrlParams>) -> String {
        let url = p.0.url;
        if !url.contains("juejin.cn") || !url.contains("/post/") {
            return "URL must be from juejin.cn and contain /post/ path".into();
        }
        match fetch_juejin_article(&url).await {
            Ok(content) => content,
            Err(e) => format!("Failed to fetch Juejin article: {}", e),
        }
    }

    /// Fetch a Zhihu article
    #[tool(
        name = "fetch_zhihu_article",
        description = "Fetch full article content from a Zhihu (zhihu.com) URL."
    )]
    pub async fn fetch_zhihu_article_tool(&self, p: Parameters<ArticleUrlParams>) -> String {
        let url = p.0.url;
        if !url.contains("zhihu.com") {
            return "URL must be from zhihu.com".into();
        }
        match fetch_zhihu_article(&url).await {
            Ok(content) => content,
            Err(e) => format!("Failed to fetch Zhihu article: {}", e),
        }
    }

    /// Fetch a LinuxDo article
    #[tool(
        name = "fetch_linuxdo_article",
        description = "Fetch full article content from a linux.do topic URL."
    )]
    pub async fn fetch_linuxdo_article_tool(&self, p: Parameters<ArticleUrlParams>) -> String {
        let url = p.0.url;
        if !url.contains("linux.do") || !url.contains("/topic/") {
            return "URL must be from linux.do and contain /topic/ path".into();
        }
        match fetch_linuxdo_article(&url).await {
            Ok(content) => content,
            Err(e) => format!("Failed to fetch LinuxDo article: {}", e),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for WebSearchHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("web-search", "2.0.0"))
    }
}
