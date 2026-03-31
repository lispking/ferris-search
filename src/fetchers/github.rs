use crate::utils::http_client::build_client;

const README_FILENAMES: &[&str] = &[
    "README.md",
    "readme.md",
    "Readme.md",
    "README.rst",
    "README.txt",
    "README",
];

struct RepoInfo {
    owner: String,
    repo: String,
}

fn parse_github_url(url: &str) -> Option<RepoInfo> {
    let url = url.trim_end_matches('/');
    // strip protocol + host
    let path = if let Some(p) = url.strip_prefix("https://github.com/") {
        p
    } else if let Some(p) = url.strip_prefix("http://github.com/") {
        p
    } else if let Some(p) = url.strip_prefix("git@github.com:") {
        p
    } else {
        return None;
    };
    // strip query string, fragment, and .git suffix
    let path = path.split(['?', '#']).next().unwrap_or(path);
    let path = path.trim_end_matches(".git");
    // take only first two path segments
    let parts: Vec<&str> = path.splitn(3, '/').collect();
    if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
        return None;
    }
    Some(RepoInfo {
        owner: parts[0].to_string(),
        repo: parts[1].to_string(),
    })
}

async fn fetch_readme(owner: &str, repo: &str) -> Option<String> {
    let client = build_client().ok()?;
    for filename in README_FILENAMES {
        let raw_url = format!(
            "https://raw.githubusercontent.com/{}/{}/HEAD/{}",
            owner, repo, filename
        );
        if let Ok(resp) = client
            .get(&raw_url)
            .header("User-Agent", "ferris-search/0.1")
            .send()
            .await
            && resp.status().is_success()
            && let Ok(text) = resp.text().await
        {
            return Some(text);
        }
    }
    None
}

pub async fn fetch_github_readme(url: &str) -> anyhow::Result<String> {
    let info = parse_github_url(url)
        .ok_or_else(|| anyhow::anyhow!("Could not parse GitHub repository URL: {}", url))?;
    fetch_readme(&info.owner, &info.repo)
        .await
        .ok_or_else(|| anyhow::anyhow!("Could not fetch README for {}/{}", info.owner, info.repo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_https_url() {
        let info = parse_github_url("https://github.com/tokio-rs/tokio").unwrap();
        assert_eq!(info.owner, "tokio-rs");
        assert_eq!(info.repo, "tokio");
    }

    #[test]
    fn parse_http_url() {
        let info = parse_github_url("http://github.com/owner/repo").unwrap();
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn parse_git_ssh_url() {
        let info = parse_github_url("git@github.com:owner/repo.git").unwrap();
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn parse_strips_dot_git() {
        let info = parse_github_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn parse_strips_query_and_fragment() {
        let info = parse_github_url("https://github.com/owner/repo?tab=readme#section").unwrap();
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn parse_trailing_slash() {
        let info = parse_github_url("https://github.com/owner/repo/").unwrap();
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn parse_deeper_path_takes_owner_repo() {
        let info = parse_github_url("https://github.com/owner/repo/issues/42").unwrap();
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn parse_rejects_non_github() {
        assert!(parse_github_url("https://gitlab.com/owner/repo").is_none());
    }

    #[test]
    fn parse_rejects_incomplete() {
        assert!(parse_github_url("https://github.com/").is_none());
        assert!(parse_github_url("https://github.com/owner").is_none());
        assert!(parse_github_url("https://github.com/owner/").is_none());
    }
}
