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

const KNOWN_REBINDING_DOMAINS: &[&str] = &[
    ".rbndr.us",
    ".1u.ms",
    ".rebind.it",
    ".rebind.network",
    ".nip.io",
    ".localtest.me",
];

/// # Returns
/// Return true if
/// - domain is a DNS rebinding domain
/// - domain is a private IP
fn is_dns_rebinding_domain(domain: &str) -> bool {
    if KNOWN_REBINDING_DOMAINS.iter().any(|d| domain.ends_with(*d)) {
        return true;
    }

    let Ok(ip): Result<Vec<IpAddr>, _> = dns_lookup::lookup_host(domain).map(|x| x.collect())
    else {
        return false;
    };
    ip.iter().any(|i| is_private_ip(*i))
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
            if is_dns_rebinding_domain(d) {
                return false;
            }
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
