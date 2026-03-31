use tantivy::schema::{FAST, INDEXED, STORED, STRING, Schema, TEXT};

/// Create the Tantivy schema for local document indexing.
///
/// Fields:
/// - `title`: document title (full-text indexed + stored)
/// - `body`: document body (full-text indexed + stored)
/// - `path`: file path (string + stored)
/// - `file_type`: extension (md/txt/html/pdf) (string + stored + FAST)
/// - `indexed_at`: index timestamp (date + indexed + stored + FAST)
pub fn create_schema() -> Schema {
    let mut builder = Schema::builder();

    builder.add_text_field("title", TEXT | STORED);
    builder.add_text_field("body", TEXT | STORED);
    builder.add_text_field("path", STRING | STORED);
    builder.add_text_field("file_type", STRING | STORED | FAST);
    builder.add_date_field("indexed_at", INDEXED | STORED | FAST);

    builder.build()
}

pub mod fields {
    pub const TITLE: &str = "title";
    pub const BODY: &str = "body";
    pub const PATH: &str = "path";
    pub const FILE_TYPE: &str = "file_type";
    pub const INDEXED_AT: &str = "indexed_at";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_has_all_fields() {
        let schema = create_schema();
        assert!(schema.get_field(fields::TITLE).is_ok());
        assert!(schema.get_field(fields::BODY).is_ok());
        assert!(schema.get_field(fields::PATH).is_ok());
        assert!(schema.get_field(fields::FILE_TYPE).is_ok());
        assert!(schema.get_field(fields::INDEXED_AT).is_ok());
    }

    #[test]
    fn test_schema_field_count() {
        let schema = create_schema();
        // Schema should have exactly 5 fields
        assert_eq!(schema.fields().count(), 5);
    }
}
