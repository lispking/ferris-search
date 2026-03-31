use crate::index::schema::{create_schema, fields};
use anyhow::{Context, Result};
use std::path::Path;
use tantivy::{Index, IndexWriter, TantivyDocument};

/// A document to be indexed.
pub struct LocalDocument {
    pub title: String,
    pub body: String,
    pub path: String,
    pub file_type: String,
}

pub struct Indexer {
    index: Index,
    writer: IndexWriter,
}

impl Indexer {
    /// Create or open a persistent index at the given path.
    /// If `clear` is true, delete all existing documents first (rebuild).
    pub fn new<P: AsRef<Path>>(index_path: P, clear: bool) -> Result<Self> {
        let schema = create_schema();
        let index_path = index_path.as_ref();

        let index = if index_path.join("meta.json").exists() {
            Index::open_in_dir(index_path).context("Failed to open existing index")?
        } else {
            std::fs::create_dir_all(index_path).context("Failed to create index directory")?;
            Index::create_in_dir(index_path, schema).context("Failed to create index")?
        };

        let writer = index
            .writer(50 * 1024 * 1024)
            .context("Failed to create index writer")?;

        let mut indexer = Self { index, writer };

        if clear {
            indexer.writer.delete_all_documents()?;
            indexer.writer.commit()?;
        }

        Ok(indexer)
    }

    pub fn add_document(&mut self, doc: &LocalDocument) -> Result<()> {
        let schema = self.index.schema();

        let title_field = schema.get_field(fields::TITLE)?;
        let body_field = schema.get_field(fields::BODY)?;
        let path_field = schema.get_field(fields::PATH)?;
        let file_type_field = schema.get_field(fields::FILE_TYPE)?;
        let indexed_at_field = schema.get_field(fields::INDEXED_AT)?;

        let now = tantivy::DateTime::from_timestamp_secs(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        );

        let mut tantivy_doc = TantivyDocument::new();
        tantivy_doc.add_text(title_field, &doc.title);
        tantivy_doc.add_text(body_field, &doc.body);
        tantivy_doc.add_text(path_field, &doc.path);
        tantivy_doc.add_text(file_type_field, &doc.file_type);
        tantivy_doc.add_date(indexed_at_field, now);

        self.writer.add_document(tantivy_doc)?;
        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
        self.writer.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_doc(title: &str, body: &str) -> LocalDocument {
        LocalDocument {
            title: title.to_string(),
            body: body.to_string(),
            path: format!("/tmp/{}.md", title),
            file_type: "md".to_string(),
        }
    }

    #[test]
    fn test_indexer_create_new() {
        let dir = TempDir::new().unwrap();
        let indexer = Indexer::new(dir.path(), false);
        assert!(indexer.is_ok());
    }

    #[test]
    fn test_indexer_add_and_commit() {
        let dir = TempDir::new().unwrap();
        let mut indexer = Indexer::new(dir.path(), false).unwrap();

        let doc = sample_doc("test", "Hello world content");
        assert!(indexer.add_document(&doc).is_ok());
        assert!(indexer.commit().is_ok());
    }

    #[test]
    fn test_indexer_reopen_existing() {
        let dir = TempDir::new().unwrap();

        // Create and commit
        {
            let mut indexer = Indexer::new(dir.path(), false).unwrap();
            indexer
                .add_document(&sample_doc("first", "First document"))
                .unwrap();
            indexer.commit().unwrap();
        }

        // Reopen
        let indexer = Indexer::new(dir.path(), false);
        assert!(indexer.is_ok());
    }

    #[test]
    fn test_indexer_clear_rebuilds() {
        let dir = TempDir::new().unwrap();

        // Create index with a document
        {
            let mut indexer = Indexer::new(dir.path(), false).unwrap();
            indexer
                .add_document(&sample_doc("old", "Old document"))
                .unwrap();
            indexer.commit().unwrap();
        }

        // Reopen with clear=true, add new doc
        {
            let mut indexer = Indexer::new(dir.path(), true).unwrap();
            indexer
                .add_document(&sample_doc("new", "New document"))
                .unwrap();
            indexer.commit().unwrap();
        }

        // Verify via searcher: only "new" should exist
        use crate::index::searcher::Searcher;
        let searcher = Searcher::new(dir.path()).unwrap();
        let results = searcher.search("old", 10).unwrap();
        assert!(results.is_empty(), "Old document should have been cleared");

        let results = searcher.search("new", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "new");
    }
}
