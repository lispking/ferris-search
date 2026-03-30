use crate::config::CONFIG;
use reqwest::{
    Client, ClientBuilder,
    header::{
        ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONNECTION, HeaderMap, HeaderValue, USER_AGENT,
    },
};

pub fn build_client() -> anyhow::Result<Client> {
    let mut builder = ClientBuilder::new()
        .gzip(true)
        .deflate(true)
        .brotli(true)
        .timeout(std::time::Duration::from_secs(20))
        .danger_accept_invalid_certs(false)
        .http2_adaptive_window(true);

    if let Some(proxy_url) = CONFIG.effective_proxy_url() {
        let proxy = reqwest::Proxy::all(&proxy_url)?;
        builder = builder.proxy(proxy);
    }

    Ok(builder.build()?)
}

pub fn chrome_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36"
    ));
    headers.insert(ACCEPT, HeaderValue::from_static(
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"
    ));
    headers.insert(
        ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"),
    );
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers
}
