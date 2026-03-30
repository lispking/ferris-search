use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub default_search_engine: String,
    pub allowed_search_engines: Vec<String>,
    pub use_proxy: bool,
    pub proxy_url: String,
    pub exa_api_key: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let default_search_engine = env::var("DEFAULT_SEARCH_ENGINE")
            .unwrap_or_else(|_| "bing".to_string())
            .to_lowercase();

        let allowed_str = env::var("ALLOWED_SEARCH_ENGINES").unwrap_or_default();
        let allowed_search_engines: Vec<String> = if allowed_str.is_empty() {
            vec![
                "baidu".into(),
                "bing".into(),
                "linuxdo".into(),
                "csdn".into(),
                "duckduckgo".into(),
                "exa".into(),
                "brave".into(),
                "juejin".into(),
                "zhihu".into(),
            ]
        } else {
            allowed_str
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .collect()
        };

        let use_proxy = env::var("USE_PROXY")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        let proxy_url =
            env::var("PROXY_URL").unwrap_or_else(|_| "http://127.0.0.1:7890".to_string());

        let exa_api_key = env::var("EXA_API_KEY").ok();

        Self {
            default_search_engine,
            allowed_search_engines,
            use_proxy,
            proxy_url,
            exa_api_key,
        }
    }

    pub fn effective_proxy_url(&self) -> Option<String> {
        if self.use_proxy {
            Some(self.proxy_url.clone())
        } else {
            None
        }
    }

    pub fn is_engine_allowed(&self, engine: &str) -> bool {
        self.allowed_search_engines.iter().any(|e| e == engine)
    }
}

lazy_static::lazy_static! {
    pub static ref CONFIG: AppConfig = AppConfig::from_env();
}
