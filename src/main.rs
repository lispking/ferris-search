mod cli;
mod config;
mod engines;
mod fetchers;
mod index;
mod tools;
mod types;
mod utils;

use std::io::IsTerminal;
use std::process;

use anyhow::Result;
use clap::Parser;
use rmcp::{ServiceExt, transport::io::stdio};
use tracing_subscriber::{EnvFilter, fmt};

use cli::{Cli, Command, OutputFormat};
use config::CONFIG;
use tools::WebSearchHandler;
use tools::helpers::{
    ALL_ENGINES, do_multi_search, do_search, format_results, resolve_engines, results_to_text,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // If no subcommand and stdin is not a TTY → auto-enter MCP mode
    // (backward-compatible with Claude Desktop / Cursor which pipe stdin)
    let command = cli.command.unwrap_or_else(|| {
        if !std::io::stdin().is_terminal() {
            Command::Mcp
        } else {
            // Show help and exit when invoked interactively without subcommand
            let _ = <Cli as clap::CommandFactory>::command().print_help();
            println!();
            process::exit(0);
        }
    });

    match command {
        Command::Mcp => run_mcp().await,
        Command::Search {
            query,
            engine,
            limit,
            format,
        } => {
            init_cli_logging();
            run_search(&query, &engine, limit, &format).await
        }
        Command::Fetch {
            url,
            max_chars,
            format,
        } => {
            init_cli_logging();
            run_fetch(&url, max_chars, &format).await
        }
        Command::ListEngines { format } => {
            run_list_engines(&format);
            Ok(())
        }
        Command::ShowConfig { format } => {
            run_show_config(&format);
            Ok(())
        }
        Command::IndexLocal {
            path,
            index_path,
            format,
        } => {
            init_cli_logging();
            run_index_local(&path, index_path.as_deref(), &format)
        }
        Command::SearchLocal {
            query,
            index_path,
            limit,
            format,
        } => {
            init_cli_logging();
            run_search_local(&query, index_path.as_deref(), limit, &format)
        }
    }
}

fn init_cli_logging() {
    fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();
}

// ─── mcp ────────────────────────────────────────────────────────────────────

async fn run_mcp() -> Result<()> {
    fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("ferris-search MCP server starting...");
    tracing::info!("Default engine: {}", CONFIG.default_search_engine);

    let service = WebSearchHandler::new()
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start stdio transport: {}", e))?;
    service.waiting().await?;

    Ok(())
}

// ─── search ─────────────────────────────────────────────────────────────────

async fn run_search(
    query: &str,
    raw_engines: &[String],
    limit: u32,
    format: &OutputFormat,
) -> Result<()> {
    let limit = limit.clamp(1, 50) as usize;
    let engines = resolve_engines(raw_engines);

    if engines.is_empty() {
        eprintln!("Error: no allowed engines specified.");
        process::exit(2);
    }

    if engines.len() == 1 {
        match do_search(&engines[0], query, limit).await {
            Ok(results) if results.is_empty() => {
                println!("No results found.");
            }
            Ok(results) => match format {
                OutputFormat::Text => println!("{}", format_results(&engines[0], &results)),
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&results)?);
                }
            },
            Err(e) => {
                eprintln!("Search failed: {}", e);
                process::exit(1);
            }
        }
        return Ok(());
    }

    let multi = do_multi_search(&engines, query, limit).await;

    for (engine, err) in &multi.errors {
        eprintln!("Warning: engine '{}' failed: {}", engine, err);
    }

    if multi.results.is_empty() {
        if multi.errors.is_empty() {
            println!("No results found.");
        } else {
            eprintln!("All engines failed.");
            process::exit(1);
        }
        return Ok(());
    }

    match format {
        OutputFormat::Text => {
            let mut total = 0usize;
            let mut output = String::new();
            for (engine, results) in &multi.results {
                total += results.len();
                output.push_str(&format!("## Results from {}\n\n", engine));
                output.push_str(&results_to_text(results));
                output.push('\n');
            }
            println!("Total results: {}\n\n{}", total, output);
        }
        OutputFormat::Json => {
            let all: Vec<_> = multi.results.into_iter().flat_map(|(_, r)| r).collect();
            println!("{}", serde_json::to_string_pretty(&all)?);
        }
    }

    Ok(())
}

// ─── fetch ──────────────────────────────────────────────────────────────────

/// Check if a GitHub URL points to a repo root (owner/repo), not deeper paths.
fn is_github_repo_url(url: &str) -> bool {
    let path = if let Some(p) = url.strip_prefix("https://github.com/") {
        p
    } else if let Some(p) = url.strip_prefix("http://github.com/") {
        p
    } else {
        return false;
    };
    // Strip query string and fragment before counting segments
    let path = path.split(['?', '#']).next().unwrap_or(path);
    let path = path.trim_end_matches('/');
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    segments.len() == 2
}

async fn run_fetch(url: &str, max_chars: u32, format: &OutputFormat) -> Result<()> {
    use fetchers::{
        csdn::fetch_csdn_article, github::fetch_github_readme, juejin::fetch_juejin_article,
        linuxdo::fetch_linuxdo_article, web::fetch_web_content, zhihu::fetch_zhihu_article,
    };
    use utils::url_safety::{is_public_http_url, is_url_from_host};

    if !is_public_http_url(url) {
        eprintln!("Error: URL must be a public HTTP/HTTPS URL.");
        process::exit(2);
    }

    // Auto-detect fetcher based on URL
    let content: String = if is_github_repo_url(url) {
        fetch_github_readme(url).await?
    } else if is_url_from_host(url, "csdn.net") {
        fetch_csdn_article(url).await?
    } else if is_url_from_host(url, "juejin.cn") && url.contains("/post/") {
        fetch_juejin_article(url).await?
    } else if is_url_from_host(url, "zhihu.com") {
        fetch_zhihu_article(url).await?
    } else if is_url_from_host(url, "linux.do") && url.contains("/topic/") {
        fetch_linuxdo_article(url).await?
    } else {
        let result = fetch_web_content(url, Some(max_chars as usize)).await?;
        match format {
            OutputFormat::Text => {
                let truncated_note = if result.truncated {
                    "\n\n[Content truncated]"
                } else {
                    ""
                };
                println!(
                    "Title: {}\nURL: {}\n\n{}{}",
                    result.title, result.url, result.content, truncated_note
                );
                return Ok(());
            }
            OutputFormat::Json => {
                let obj = serde_json::json!({
                    "title": result.title,
                    "url": result.url,
                    "content": result.content,
                    "truncated": result.truncated,
                });
                println!("{}", serde_json::to_string_pretty(&obj)?);
                return Ok(());
            }
        }
    };

    match format {
        OutputFormat::Text => println!("{}", content),
        OutputFormat::Json => {
            let obj = serde_json::json!({
                "url": url,
                "content": content,
            });
            println!("{}", serde_json::to_string_pretty(&obj)?);
        }
    }

    Ok(())
}

// ─── list-engines ───────────────────────────────────────────────────────────

fn run_list_engines(format: &OutputFormat) {
    let allowed = &CONFIG.allowed_search_engines;
    let default = &CONFIG.default_search_engine;

    match format {
        OutputFormat::Text => {
            println!("Supported engines:\n");
            for &engine in ALL_ENGINES {
                let is_allowed = allowed.iter().any(|e| e == engine);
                let is_default = engine == default;
                let markers = match (is_default, is_allowed) {
                    (true, true) => " (default)",
                    (true, false) => " (default, disabled)",
                    (false, true) => " (allowed)",
                    (false, false) => " (disabled)",
                };
                println!("  {}{}", engine, markers);
            }
        }
        OutputFormat::Json => {
            let obj = serde_json::json!({
                "default_engine": default,
                "allowed_engines": allowed,
                "all_engines": ALL_ENGINES,
            });
            println!("{}", serde_json::to_string_pretty(&obj).unwrap());
        }
    }
}

// ─── show-config ────────────────────────────────────────────────────────────

fn mask_key(key: &Option<String>) -> String {
    match key {
        None => "(not set)".into(),
        Some(k) if k.chars().count() <= 8 => "***".into(),
        Some(k) => {
            let first4: String = k.chars().take(4).collect();
            let last4: String = k
                .chars()
                .rev()
                .take(4)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            format!("{}...{}", first4, last4)
        }
    }
}

fn run_show_config(format: &OutputFormat) {
    let c = &*CONFIG;

    match format {
        OutputFormat::Text => {
            println!("ferris-search configuration\n");
            println!("  Default engine:     {}", c.default_search_engine);
            println!(
                "  Allowed engines:    {}",
                c.allowed_search_engines.join(", ")
            );
            println!("  Proxy enabled:      {}", c.use_proxy);
            println!("  Proxy URL:          {}", c.proxy_url);
            println!("  BRAVE_API_KEY:      {}", mask_key(&c.brave_api_key));
            println!("  EXA_API_KEY:        {}", mask_key(&c.exa_api_key));
            println!("  FIRECRAWL_API_KEY:  {}", mask_key(&c.firecrawl_api_key));
            println!("  JINA_API_KEY:       {}", mask_key(&c.jina_api_key));
            println!("  TAVILY_API_KEY:     {}", mask_key(&c.tavily_api_key));
            println!("  GITHUB_TOKEN:       {}", mask_key(&c.github_token));
            println!("  Local index path:   {}", c.local_docs_index_path);
            let effective_ext: Vec<String> = if c.local_docs_extensions.is_empty() {
                index::collector::DEFAULT_EXTENSIONS
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            } else {
                c.local_docs_extensions.clone()
            };
            println!("  Local extensions:   {}", effective_ext.join(", "));
        }
        OutputFormat::Json => {
            let obj = serde_json::json!({
                "default_search_engine": c.default_search_engine,
                "allowed_search_engines": c.allowed_search_engines,
                "use_proxy": c.use_proxy,
                "proxy_url": c.proxy_url,
                "brave_api_key": mask_key(&c.brave_api_key),
                "exa_api_key": mask_key(&c.exa_api_key),
                "firecrawl_api_key": mask_key(&c.firecrawl_api_key),
                "jina_api_key": mask_key(&c.jina_api_key),
                "tavily_api_key": mask_key(&c.tavily_api_key),
                "github_token": mask_key(&c.github_token),
                "local_docs_index_path": c.local_docs_index_path,
                "local_docs_extensions": if c.local_docs_extensions.is_empty() {
                    serde_json::json!(index::collector::DEFAULT_EXTENSIONS)
                } else {
                    serde_json::json!(c.local_docs_extensions)
                },
            });
            println!("{}", serde_json::to_string_pretty(&obj).unwrap());
        }
    }
}

// ─── index-local ────────────────────────────────────────────────────────────

fn run_index_local(
    paths: &[String],
    index_path: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let idx_dir = index_path.unwrap_or(&CONFIG.local_docs_index_path);
    let extensions = CONFIG.local_docs_extensions.clone();

    let (docs, errors) = index::collector::collect_documents(paths, &extensions);

    for err in &errors {
        eprintln!("Warning: {}", err);
    }

    if docs.is_empty() {
        eprintln!("No documents found to index.");
        process::exit(1);
    }

    let mut indexer = index::indexer::Indexer::new(idx_dir, true)?;
    let mut indexed = 0usize;
    let mut index_errors = 0usize;
    for doc in &docs {
        match indexer.add_document(doc) {
            Ok(()) => indexed += 1,
            Err(e) => {
                eprintln!("Warning: failed to index {}: {}", doc.path, e);
                index_errors += 1;
            }
        }
    }
    indexer.commit()?;

    let total_errors = errors.len() + index_errors;
    let summary = serde_json::json!({
        "indexed": indexed,
        "errors": total_errors,
        "index_path": idx_dir,
    });

    match format {
        OutputFormat::Text => {
            println!(
                "Indexed {} documents into {}\nWarnings: {}",
                indexed, idx_dir, total_errors
            );
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
    }

    if indexed == 0 {
        eprintln!("Error: all documents failed to index.");
        process::exit(1);
    }

    Ok(())
}

// ─── search-local ───────────────────────────────────────────────────────────

fn run_search_local(
    query: &str,
    index_path: Option<&str>,
    limit: u32,
    format: &OutputFormat,
) -> Result<()> {
    let idx_dir = index_path.unwrap_or(&CONFIG.local_docs_index_path);
    let limit = limit.clamp(1, 50) as usize;

    let searcher = index::searcher::Searcher::new(idx_dir)?;
    let results = searcher.search(query, limit)?;

    if results.is_empty() {
        println!("No results found.");
        return Ok(());
    }

    match format {
        OutputFormat::Text => {
            println!("Found {} results:\n", results.len());
            for (i, r) in results.iter().enumerate() {
                println!(
                    "{}. **{}**\nPath: {}\nType: {} | Score: {:.4}\nSnippet: {}\n",
                    i + 1,
                    r.title,
                    r.path,
                    r.file_type,
                    r.score,
                    r.snippet
                );
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── is_github_repo_url ─────────────────────────────────────────────

    #[test]
    fn github_repo_url_basic() {
        assert!(is_github_repo_url("https://github.com/tokio-rs/tokio"));
        assert!(is_github_repo_url("http://github.com/tokio-rs/tokio"));
    }

    #[test]
    fn github_repo_url_trailing_slash() {
        assert!(is_github_repo_url("https://github.com/tokio-rs/tokio/"));
    }

    #[test]
    fn github_repo_url_query_and_fragment() {
        assert!(is_github_repo_url(
            "https://github.com/tokio-rs/tokio?tab=readme"
        ));
        assert!(is_github_repo_url(
            "https://github.com/tokio-rs/tokio#readme"
        ));
        assert!(is_github_repo_url(
            "https://github.com/tokio-rs/tokio?tab=readme#section"
        ));
    }

    #[test]
    fn github_repo_url_deeper_paths_rejected() {
        assert!(!is_github_repo_url(
            "https://github.com/tokio-rs/tokio/issues"
        ));
        assert!(!is_github_repo_url(
            "https://github.com/tokio-rs/tokio/blob/main/README.md"
        ));
        assert!(!is_github_repo_url(
            "https://github.com/tokio-rs/tokio/pull/123"
        ));
    }

    #[test]
    fn github_repo_url_non_github() {
        assert!(!is_github_repo_url("https://gitlab.com/owner/repo"));
        assert!(!is_github_repo_url("https://example.com/path"));
        assert!(!is_github_repo_url("not a url"));
    }

    #[test]
    fn github_repo_url_owner_only_rejected() {
        assert!(!is_github_repo_url("https://github.com/tokio-rs"));
        assert!(!is_github_repo_url("https://github.com/tokio-rs/"));
    }

    // ─── mask_key ───────────────────────────────────────────────────────

    #[test]
    fn mask_key_none() {
        assert_eq!(mask_key(&None), "(not set)");
    }

    #[test]
    fn mask_key_short() {
        assert_eq!(mask_key(&Some("abc".into())), "***");
        assert_eq!(mask_key(&Some("12345678".into())), "***");
    }

    #[test]
    fn mask_key_long() {
        assert_eq!(mask_key(&Some("abcdefghij".into())), "abcd...ghij");
        assert_eq!(mask_key(&Some("exa-1234567890".into())), "exa-...7890");
    }
}
