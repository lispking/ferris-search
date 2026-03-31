use std::net::IpAddr;
use url::Url;

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            o[0] == 10
                || o[0] == 127
                || o[0] == 0
                || (o[0] == 100 && o[1] >= 64 && o[1] <= 127)
                || (o[0] == 169 && o[1] == 254)
                || (o[0] == 172 && o[1] >= 16 && o[1] <= 31)
                || (o[0] == 192 && o[1] == 168)
                || (o[0] == 198 && (o[1] == 18 || o[1] == 19))
        }
        IpAddr::V6(v6) => {
            let s = v6.segments();
            v6.is_loopback()
                || v6.is_unspecified()
                || (s[0] & 0xfe00) == 0xfc00  // fc00::/7 ULA
                || (s[0] & 0xffc0) == 0xfe80  // fe80::/10 link-local
                || v6.to_ipv4().map(|v4| {
                    let o = v4.octets();
                    o[0] == 10
                        || o[0] == 127
                        || (o[0] == 172 && o[1] >= 16 && o[1] <= 31)
                        || (o[0] == 192 && o[1] == 168)
                }).unwrap_or(false)
        }
    }
}

pub fn is_public_http_url(raw: &str) -> bool {
    let Ok(url) = Url::parse(raw) else {
        return false;
    };
    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return false;
    }
    let host = match url.host() {
        Some(h) => h,
        None => return false,
    };
    match host {
        url::Host::Ipv4(ip) => !is_private_ip(IpAddr::V4(ip)),
        url::Host::Ipv6(ip) => !is_private_ip(IpAddr::V6(ip)),
        url::Host::Domain(d) => {
            // reject numeric-only hostnames (integer IPv4 literals)
            if d.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
            // reject localhost variants
            if d == "localhost" || d.ends_with(".localhost") {
                return false;
            }
            true
        }
    }
}

pub fn assert_public_http_url(url: &str) -> anyhow::Result<()> {
    if is_public_http_url(url) {
        Ok(())
    } else {
        anyhow::bail!("URL is not a public HTTP/HTTPS URL: {}", url)
    }
}

/// Check if a URL's host matches the given domain (or is a subdomain of it).
pub fn is_url_from_host(raw: &str, domain: &str) -> bool {
    let Ok(url) = Url::parse(raw) else {
        return false;
    };
    match url.host_str() {
        Some(host) => host == domain || host.ends_with(&format!(".{}", domain)),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_urls_accepted() {
        assert!(is_public_http_url("https://example.com"));
        assert!(is_public_http_url("http://example.com/path"));
        assert!(is_public_http_url("https://github.com/owner/repo"));
    }

    #[test]
    fn non_http_rejected() {
        assert!(!is_public_http_url("ftp://example.com"));
        assert!(!is_public_http_url("file:///etc/passwd"));
        assert!(!is_public_http_url("javascript:alert(1)"));
    }

    #[test]
    fn private_ips_rejected() {
        assert!(!is_public_http_url("http://127.0.0.1"));
        assert!(!is_public_http_url("http://10.0.0.1/path"));
        assert!(!is_public_http_url("http://192.168.1.1"));
        assert!(!is_public_http_url("http://172.16.0.1"));
    }

    #[test]
    fn localhost_rejected() {
        assert!(!is_public_http_url("http://localhost"));
        assert!(!is_public_http_url("http://localhost:8080"));
        assert!(!is_public_http_url("http://foo.localhost"));
    }

    #[test]
    fn invalid_urls_rejected() {
        assert!(!is_public_http_url("not a url"));
        assert!(!is_public_http_url(""));
    }

    #[test]
    fn is_url_from_host_exact_match() {
        assert!(is_url_from_host(
            "https://github.com/owner/repo",
            "github.com"
        ));
        assert!(is_url_from_host(
            "https://blog.csdn.net/article",
            "csdn.net"
        ));
        assert!(is_url_from_host(
            "https://www.zhihu.com/question/1",
            "zhihu.com"
        ));
    }

    #[test]
    fn is_url_from_host_rejects_spoofed() {
        assert!(!is_url_from_host(
            "http://evil.com/github.com",
            "github.com"
        ));
        assert!(!is_url_from_host(
            "http://169.254.169.254/csdn.net",
            "csdn.net"
        ));
        assert!(!is_url_from_host("http://evil.com/zhihu.com", "zhihu.com"));
        assert!(!is_url_from_host("not a url", "github.com"));
    }
}
