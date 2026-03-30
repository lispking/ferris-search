use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct WebSearchParams {
    /// Search query string
    pub query: String,
    /// Max results to return per engine (default 10, max 50)
    pub limit: Option<u32>,
    /// One or more search engines to use. Supported: bing, duckduckgo, brave, baidu, csdn, juejin, exa, zhihu, linuxdo.
    /// Defaults to the server's configured default engine.
    pub engines: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct FetchUrlParams {
    /// URL to fetch
    pub url: String,
    /// Max characters of content to return (default 30000, max 200000)
    pub max_chars: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GithubReadmeParams {
    /// GitHub repository URL (e.g. https://github.com/owner/repo)
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ArticleUrlParams {
    /// URL of the article to fetch
    pub url: String,
}
