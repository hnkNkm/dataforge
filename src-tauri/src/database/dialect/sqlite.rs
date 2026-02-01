use super::SqlDialect;
use crate::database::DatabaseType;

/// SQLite-specific SQL dialect implementation
#[derive(Debug, Clone)]
pub struct SQLiteDialect;

impl SQLiteDialect {
    pub fn new() -> Self {
        Self
    }
}

impl SqlDialect for SQLiteDialect {
    fn quote_identifier(&self, identifier: &str) -> String {
        // SQLite uses double quotes for identifiers (like PostgreSQL)
        // But also accepts backticks (MySQL-style) and square brackets (SQL Server style)
        // We'll use double quotes for standard SQL compliance
        let escaped = identifier.replace('"', r#""""#);
        format!(r#""{}""#, escaped)
    }
    
    fn limit_clause(&self, limit: Option<usize>, offset: Option<usize>) -> String {
        let mut clause = String::new();
        
        if let Some(limit_val) = limit {
            clause.push_str(&format!(" LIMIT {}", limit_val));
        }
        
        if let Some(offset_val) = offset {
            if limit.is_none() {
                // SQLite allows LIMIT -1 OFFSET n for unlimited rows with offset
                clause.push_str(&format!(" LIMIT -1 OFFSET {}", offset_val));
            } else {
                clause.push_str(&format!(" OFFSET {}", offset_val));
            }
        }
        
        clause
    }
    
    fn boolean_literal(&self, value: bool) -> String {
        // SQLite stores booleans as integers (0 or 1)
        if value {
            "1".to_string()
        } else {
            "0".to_string()
        }
    }
    
    fn current_timestamp(&self) -> &'static str {
        // SQLite uses CURRENT_TIMESTAMP
        "CURRENT_TIMESTAMP"
    }
    
    fn auto_increment_type(&self) -> &'static str {
        // SQLite uses INTEGER PRIMARY KEY AUTOINCREMENT
        // Note: Just INTEGER PRIMARY KEY also auto-increments,
        // but AUTOINCREMENT ensures values are never reused
        "INTEGER PRIMARY KEY AUTOINCREMENT"
    }
    
    fn string_concat(&self, left: &str, right: &str) -> String {
        // SQLite uses the || operator for string concatenation
        format!("{} || {}", left, right)
    }
    
    fn case_insensitive_like(&self) -> &'static str {
        // SQLite LIKE is case-insensitive by default for ASCII characters
        // For full Unicode support, use LIKE with COLLATE NOCASE
        "LIKE"
    }
    
    fn date_literal(&self, date: &str) -> String {
        // SQLite stores dates as strings, numbers, or NULL
        // String format should be 'YYYY-MM-DD'
        format!("'{}'", date)
    }
    
    fn datetime_literal(&self, datetime: &str) -> String {
        // SQLite stores datetime as strings
        // String format should be 'YYYY-MM-DD HH:MM:SS'
        format!("'{}'", datetime)
    }
    
    fn database_type(&self) -> DatabaseType {
        DatabaseType::SQLite
    }
    
    fn supports_returning_clause(&self) -> bool {
        // SQLite 3.35.0+ supports RETURNING clause
        // We'll assume a recent version
        true
    }
    
    fn supports_upsert(&self) -> bool {
        // SQLite supports ON CONFLICT ... DO UPDATE (UPSERT)
        true
    }
    
    fn supports_schemas(&self) -> bool {
        // SQLite has limited schema support
        // It has a concept of attached databases with schemas, but not like PostgreSQL/MySQL
        false
    }
}

impl Default for SQLiteDialect {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quote_identifier() {
        let dialect = SQLiteDialect::new();
        assert_eq!(dialect.quote_identifier("table_name"), r#""table_name""#);
        assert_eq!(dialect.quote_identifier("column"), r#""column""#);
        assert_eq!(dialect.quote_identifier(r#"table"with"quote"#), r#""table""with""quote""#);
    }
    
    #[test]
    fn test_limit_clause() {
        let dialect = SQLiteDialect::new();
        assert_eq!(dialect.limit_clause(Some(10), None), " LIMIT 10");
        assert_eq!(dialect.limit_clause(Some(10), Some(20)), " LIMIT 10 OFFSET 20");
        // SQLite uses LIMIT -1 for unlimited with offset
        assert_eq!(dialect.limit_clause(None, Some(20)), " LIMIT -1 OFFSET 20");
        assert_eq!(dialect.limit_clause(None, None), "");
    }
    
    #[test]
    fn test_boolean_literal() {
        let dialect = SQLiteDialect::new();
        assert_eq!(dialect.boolean_literal(true), "1");
        assert_eq!(dialect.boolean_literal(false), "0");
    }
    
    #[test]
    fn test_string_concat() {
        let dialect = SQLiteDialect::new();
        assert_eq!(dialect.string_concat("first_name", "last_name"), "first_name || last_name");
    }
    
    #[test]
    fn test_date_literals() {
        let dialect = SQLiteDialect::new();
        assert_eq!(dialect.date_literal("2023-01-15"), "'2023-01-15'");
        assert_eq!(dialect.datetime_literal("2023-01-15 10:30:00"), "'2023-01-15 10:30:00'");
    }
    
    #[test]
    fn test_qualified_table_name() {
        let dialect = SQLiteDialect::new();
        // SQLite doesn't really support schemas
        assert_eq!(
            dialect.qualified_table_name(Some("main"), "users"),
            r#""users""#
        );
        assert_eq!(
            dialect.qualified_table_name(None, "users"),
            r#""users""#
        );
    }
    
    #[test]
    fn test_features() {
        let dialect = SQLiteDialect::new();
        assert!(dialect.supports_returning_clause());
        assert!(dialect.supports_upsert());
        assert!(!dialect.supports_schemas());
    }
}