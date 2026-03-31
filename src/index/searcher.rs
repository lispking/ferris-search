use crate::index::schema::fields;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tantivy::{
    Index, IndexReader, ReloadPolicy, collector::TopDocs, query::QueryParser, schema::Value,
};

/// A local search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSearchResult {
    pub title: String,
    pub path: String,
    pub snippet: String,
    pub file_type: String,
    pub score: f32,
}

pub struct Searcher {
    index: Index,
    reader: IndexReader,
}

impl Searcher {
    pub fn new<P: AsRef<Path>>(index_path: P) -> Result<Self> {
        let index = Index::open_in_dir(index_path).context("Failed to open index")?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .context("Failed to create index reader")?;

        Ok(Self { index, reader })
    }

    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<LocalSearchResult>> {
        let schema = self.index.schema();
        let searcher = self.reader.searcher();

        let title_field = schema.get_field(fields::TITLE)?;
        let body_field = schema.get_field(fields::BODY)?;
        let path_field = schema.get_field(fields::PATH)?;
        let file_type_field = schema.get_field(fields::FILE_TYPE)?;

        let query_parser = QueryParser::for_index(&self.index, vec![title_field, body_field]);
        let query = query_parser
            .parse_query(query_str)
            .context("Failed to parse query")?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;

            let title = doc
                .get_first(title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let body = doc
                .get_first(body_field)
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Generate snippet: first 200 chars of body
            let snippet = if body.len() > 200 {
                format!("{}...", &body[..body.floor_char_boundary(200)])
            } else {
                body.to_string()
            };

            let path = doc
                .get_first(path_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let file_type = doc
                .get_first(file_type_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            results.push(LocalSearchResult {
                title,
                path,
                snippet,
                file_type,
                score,
            });
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::indexer::{Indexer, LocalDocument};
    use tempfile::TempDir;

    fn build_index(dir: &Path, docs: Vec<(&str, &str)>) {
        let mut indexer = Indexer::new(dir, false).unwrap();
        for (title, body) in docs {
            indexer
                .add_document(&LocalDocument {
                    title: title.to_string(),
                    body: body.to_string(),
                    path: format!("/docs/{}.md", title),
                    file_type: "md".to_string(),
                })
                .unwrap();
        }
        indexer.commit().unwrap();
    }

    #[test]
    fn test_search_returns_matching_results() {
        let dir = TempDir::new().unwrap();
        build_index(
            dir.path(),
            vec![
                ("rust guide", "Rust is a systems programming language"),
                ("python guide", "Python is great for scripting"),
            ],
        );

        let searcher = Searcher::new(dir.path()).unwrap();
        let results = searcher.search("rust systems", 10).unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].title, "rust guide");
        assert_eq!(results[0].file_type, "md");
        assert!(results[0].score > 0.0);
    }

    #[test]
    fn test_search_respects_limit() {
        let dir = TempDir::new().unwrap();
        build_index(
            dir.path(),
            vec![
                ("doc1", "common keyword found here"),
                ("doc2", "common keyword also here"),
                ("doc3", "common keyword again"),
            ],
        );

        let searcher = Searcher::new(dir.path()).unwrap();
        let results = searcher.search("common keyword", 2).unwrap();
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_search_empty_index_returns_nothing() {
        let dir = TempDir::new().unwrap();
        // Create empty index
        let mut indexer = Indexer::new(dir.path(), false).unwrap();
        indexer.commit().unwrap();

        let searcher = Searcher::new(dir.path()).unwrap();
        let results = searcher.search("anything", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_no_match() {
        let dir = TempDir::new().unwrap();
        build_index(dir.path(), vec![("hello", "world is great")]);

        let searcher = Searcher::new(dir.path()).unwrap();
        let results = searcher.search("zzzznonexistent", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_result_has_snippet() {
        let dir = TempDir::new().unwrap();
        build_index(
            dir.path(),
            vec![("doc", "Short body text for testing snippets")],
        );

        let searcher = Searcher::new(dir.path()).unwrap();
        let results = searcher.search("testing snippets", 10).unwrap();

        assert!(!results.is_empty());
        assert!(!results[0].snippet.is_empty());
    }

    #[test]
    fn test_searcher_open_nonexistent_fails() {
        let result = Searcher::new("/nonexistent/index/path");
        assert!(result.is_err());
    }
}
