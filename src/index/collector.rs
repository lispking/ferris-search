use crate::index::indexer::LocalDocument;
use anyhow::{Context, Result};
use scraper::{Html, Selector};
use std::path::Path;
use walkdir::WalkDir;

/// Default allowed extensions for local document indexing.
pub const DEFAULT_EXTENSIONS: &[&str] = &["md", "markdown", "txt", "html", "htm", "pdf"];

/// Collected file ready for indexing.
struct CollectedFile {
    path: String,
    file_type: String,
    content: String,
    /// Raw source for title extraction (e.g. original HTML before text extraction).
    raw_title_source: Option<String>,
}

/// Recursively scan directories and collect documents.
/// Returns (documents, errors) — errors are per-file warnings, not fatal.
pub fn collect_documents(
    paths: &[String],
    extensions: &[String],
) -> (Vec<LocalDocument>, Vec<String>) {
    let mut docs = Vec::new();
    let mut errors = Vec::new();

    let ext_set: Vec<String> = if extensions.is_empty() {
        DEFAULT_EXTENSIONS.iter().map(|s| s.to_string()).collect()
    } else {
        extensions.to_vec()
    };

    for root in paths {
        let root_path = Path::new(root);
        if !root_path.exists() {
            errors.push(format!("Path does not exist: {}", root));
            continue;
        }

        if root_path.is_file() {
            match process_file(root_path, &ext_set) {
                Ok(Some(f)) => docs.push(file_to_document(f)),
                Ok(None) => {} // skipped (extension not allowed)
                Err(e) => errors.push(format!("{}: {}", root, e)),
            }
            continue;
        }

        for entry in WalkDir::new(root).follow_links(true).into_iter() {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    errors.push(format!("Walk error: {}", e));
                    continue;
                }
            };

            if !entry.file_type().is_file() {
                continue;
            }

            match process_file(entry.path(), &ext_set) {
                Ok(Some(f)) => docs.push(file_to_document(f)),
                Ok(None) => {}
                Err(e) => errors.push(format!("{}: {}", entry.path().display(), e)),
            }
        }
    }

    (docs, errors)
}

fn process_file(path: &Path, allowed_ext: &[String]) -> Result<Option<CollectedFile>> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !allowed_ext.iter().any(|a| a == &ext) {
        return Ok(None);
    }

    let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let abs_path = abs_path.to_string_lossy();
    // Strip Windows verbatim path prefix for cleaner output.
    // `\\?\UNC\server\share` → `\\server\share` (UNC path)
    // `\\?\C:\dir` → `C:\dir` (regular path)
    let abs_path = if let Some(unc) = abs_path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{}", unc)
    } else {
        abs_path
            .strip_prefix(r"\\?\")
            .unwrap_or(&abs_path)
            .to_string()
    };

    let content = match ext.as_str() {
        "md" | "markdown" | "txt" => {
            std::fs::read_to_string(path).context("Failed to read file")?
        }
        "html" | "htm" => {
            let raw = std::fs::read_to_string(path).context("Failed to read HTML file")?;
            let text = extract_html_text(&raw);
            return Ok(Some(CollectedFile {
                path: abs_path,
                file_type: ext,
                content: text,
                raw_title_source: Some(raw),
            }));
        }
        "pdf" => extract_pdf_text(path)?,
        _ => return Ok(None),
    };

    Ok(Some(CollectedFile {
        path: abs_path,
        file_type: ext,
        content,
        raw_title_source: None,
    }))
}

fn file_to_document(f: CollectedFile) -> LocalDocument {
    let title_source = f.raw_title_source.as_deref().unwrap_or(&f.content);
    let title = extract_title(title_source, &f.file_type, &f.path);
    LocalDocument {
        title,
        body: f.content,
        path: f.path,
        file_type: f.file_type,
    }
}

/// Extract title based on file type.
/// Priority: MD first heading > HTML title/h1 > filename.
fn extract_title(content: &str, file_type: &str, path: &str) -> String {
    match file_type {
        "md" | "markdown" => {
            // Find first ATX heading (# Title)
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(heading) = trimmed.strip_prefix('#') {
                    let heading = heading.trim_start_matches('#').trim();
                    if !heading.is_empty() {
                        return heading.to_string();
                    }
                }
            }
        }
        "html" | "htm" => {
            let doc = Html::parse_document(content);
            // Try <title>
            if let Ok(sel) = Selector::parse("title") {
                if let Some(el) = doc.select(&sel).next() {
                    let t = el.text().collect::<String>().trim().to_string();
                    if !t.is_empty() {
                        return t;
                    }
                }
            }
            // Try <h1>
            if let Ok(sel) = Selector::parse("h1") {
                if let Some(el) = doc.select(&sel).next() {
                    let t = el.text().collect::<String>().trim().to_string();
                    if !t.is_empty() {
                        return t;
                    }
                }
            }
        }
        _ => {}
    }

    // Fallback: filename without extension
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string()
}

/// Extract text from HTML, reusing the same approach as fetchers/web.rs.
fn extract_html_text(html: &str) -> String {
    let doc = Html::parse_document(html);

    let containers = [
        "article",
        "main",
        "#main",
        ".main",
        ".content",
        "#content",
        ".post",
        ".article",
        ".entry-content",
        ".post-content",
        ".article-content",
    ];

    for sel_str in &containers {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&sel).next() {
                let text = el.text().collect::<String>();
                let normalized = normalize_text(&text);
                if normalized.len() > 200 {
                    return normalized;
                }
            }
        }
    }

    // Fallback: body text
    if let Ok(sel) = Selector::parse("body") {
        if let Some(el) = doc.select(&sel).next() {
            return normalize_text(&el.text().collect::<String>());
        }
    }

    normalize_text(&doc.root_element().text().collect::<String>())
}

fn normalize_text(s: &str) -> String {
    s.replace("\r\n", "\n")
        .replace('\u{00a0}', " ")
        .split('\n')
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .split("\n\n\n")
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Extract text from a PDF file.
fn extract_pdf_text(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).context("Failed to read PDF file")?;
    let text =
        pdf_extract::extract_text_from_mem(&bytes).context("Failed to extract text from PDF")?;
    Ok(normalize_text(&text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extract_title_markdown_heading() {
        let content = "# My Document\n\nSome body text.";
        assert_eq!(extract_title(content, "md", "doc.md"), "My Document");
    }

    #[test]
    fn test_extract_title_markdown_nested_heading() {
        let content = "## Second Level\n\nBody.";
        assert_eq!(extract_title(content, "md", "doc.md"), "Second Level");
    }

    #[test]
    fn test_extract_title_html_title_tag() {
        let html = "<html><head><title>Page Title</title></head><body>Text</body></html>";
        assert_eq!(extract_title(html, "html", "page.html"), "Page Title");
    }

    #[test]
    fn test_extract_title_html_h1_fallback() {
        let html = "<html><body><h1>Heading One</h1><p>Text</p></body></html>";
        assert_eq!(extract_title(html, "html", "page.html"), "Heading One");
    }

    #[test]
    fn test_extract_title_fallback_to_filename() {
        let content = "No headings here, just plain text.";
        assert_eq!(extract_title(content, "txt", "/path/to/notes.txt"), "notes");
    }

    #[test]
    fn test_extract_title_empty_heading_falls_back() {
        let content = "# \n\nBody text.";
        assert_eq!(extract_title(content, "md", "readme.md"), "readme");
    }

    #[test]
    fn test_normalize_text_removes_excess_blank_lines() {
        let input = "Line 1\n\n\nLine 2";
        let result = normalize_text(input);
        assert_eq!(result, "Line 1\n\nLine 2");
    }

    #[test]
    fn test_normalize_text_trims_trailing_spaces() {
        let input = "Hello   \nWorld  ";
        let result = normalize_text(input);
        assert_eq!(result, "Hello\nWorld");
    }

    #[test]
    fn test_normalize_text_replaces_nbsp() {
        let input = "Hello\u{00a0}World";
        let result = normalize_text(input);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_extract_html_text_from_article() {
        let html = format!(
            "<html><body><article>{}</article></body></html>",
            "Important content. ".repeat(20)
        );
        let text = extract_html_text(&html);
        assert!(text.contains("Important content."));
    }

    #[test]
    fn test_extract_html_text_body_fallback() {
        let html = "<html><body><p>Simple paragraph content here.</p></body></html>";
        let text = extract_html_text(html);
        assert!(text.contains("Simple paragraph content here."));
    }

    #[test]
    fn test_default_extensions() {
        assert!(DEFAULT_EXTENSIONS.contains(&"md"));
        assert!(DEFAULT_EXTENSIONS.contains(&"txt"));
        assert!(DEFAULT_EXTENSIONS.contains(&"html"));
        assert!(DEFAULT_EXTENSIONS.contains(&"pdf"));
        assert!(!DEFAULT_EXTENSIONS.contains(&"rs"));
    }

    #[test]
    fn test_collect_documents_with_temp_files() {
        let dir = TempDir::new().unwrap();

        fs::write(dir.path().join("readme.md"), "# Hello\n\nWorld").unwrap();
        fs::write(dir.path().join("notes.txt"), "Some notes here").unwrap();
        fs::write(dir.path().join("skip.rs"), "fn main() {}").unwrap();

        let (docs, errors) = collect_documents(&[dir.path().to_string_lossy().to_string()], &[]);

        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors);
        assert_eq!(docs.len(), 2);

        let titles: Vec<&str> = docs.iter().map(|d| d.title.as_str()).collect();
        assert!(titles.contains(&"Hello"));
        assert!(titles.contains(&"notes"));
    }

    #[test]
    fn test_collect_documents_nonexistent_path() {
        let (docs, errors) = collect_documents(&["/nonexistent/path/abc123".to_string()], &[]);

        assert!(docs.is_empty());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("does not exist"));
    }

    #[test]
    fn test_collect_documents_extension_filter() {
        let dir = TempDir::new().unwrap();

        fs::write(dir.path().join("a.md"), "# A").unwrap();
        fs::write(dir.path().join("b.txt"), "B").unwrap();
        fs::write(dir.path().join("c.html"), "<p>C</p>").unwrap();

        // Only allow txt
        let (docs, errors) = collect_documents(
            &[dir.path().to_string_lossy().to_string()],
            &["txt".to_string()],
        );

        assert!(errors.is_empty());
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].file_type, "txt");
    }

    #[test]
    fn test_collect_single_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("doc.md");
        fs::write(&file_path, "# Single\n\nContent").unwrap();

        let (docs, errors) = collect_documents(&[file_path.to_string_lossy().to_string()], &[]);

        assert!(errors.is_empty());
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].title, "Single");
    }
}
