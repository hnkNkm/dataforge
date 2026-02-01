use super::SqlDialect;
use crate::database::DatabaseType;

/// PostgreSQL-specific SQL dialect implementation
#[derive(Debug, Clone)]
pub struct PostgreSQLDialect;

impl PostgreSQLDialect {
    pub fn new() -> Self {
        Self
    }
}

impl SqlDialect for PostgreSQLDialect {
    fn quote_identifier(&self, identifier: &str) -> String {
        // PostgreSQL uses double quotes for identifiers
        // We should escape any existing double quotes
        let escaped = identifier.replace('"', r#""""#);
        format!(r#""{}""#, escaped)
    }
    
    fn limit_clause(&self, limit: Option<usize>, offset: Option<usize>) -> String {
        let mut clause = String::new();
        
        if let Some(limit_val) = limit {
            clause.push_str(&format!(" LIMIT {}", limit_val));
        }
        
        if let Some(offset_val) = offset {
            clause.push_str(&format!(" OFFSET {}", offset_val));
        }
        
        clause
    }
    
    fn boolean_literal(&self, value: bool) -> String {
        // PostgreSQL accepts TRUE/FALSE, true/false, 't'/'f', '1'/'0'
        // We'll use the standard TRUE/FALSE
        if value {
            "TRUE".to_string()
        } else {
            "FALSE".to_string()
        }
    }
    
    fn current_timestamp(&self) -> &'static str {
        // PostgreSQL supports both CURRENT_TIMESTAMP and NOW()
        "CURRENT_TIMESTAMP"
    }
    
    fn auto_increment_type(&self) -> &'static str {
        // PostgreSQL uses SERIAL for auto-incrementing integers
        // For larger values, use BIGSERIAL
        "SERIAL"
    }
    
    fn string_concat(&self, left: &str, right: &str) -> String {
        // PostgreSQL uses the || operator for string concatenation
        format!("{} || {}", left, right)
    }
    
    fn case_insensitive_like(&self) -> &'static str {
        // PostgreSQL has ILIKE for case-insensitive pattern matching
        "ILIKE"
    }
    
    fn date_literal(&self, date: &str) -> String {
        // PostgreSQL uses DATE 'YYYY-MM-DD' format
        format!("DATE '{}'", date)
    }
    
    fn datetime_literal(&self, datetime: &str) -> String {
        // PostgreSQL uses TIMESTAMP 'YYYY-MM-DD HH:MM:SS' format
        format!("TIMESTAMP '{}'", datetime)
    }
    
    fn database_type(&self) -> DatabaseType {
        DatabaseType::PostgreSQL
    }
    
    fn supports_returning_clause(&self) -> bool {
        // PostgreSQL supports RETURNING clause
        true
    }
    
    fn supports_upsert(&self) -> bool {
        // PostgreSQL supports ON CONFLICT ... DO UPDATE (UPSERT)
        true
    }
    
    fn supports_schemas(&self) -> bool {
        // PostgreSQL has full schema support
        true
    }
}

impl Default for PostgreSQLDialect {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quote_identifier() {
        let dialect = PostgreSQLDialect::new();
        assert_eq!(dialect.quote_identifier("table_name"), r#""table_name""#);
        assert_eq!(dialect.quote_identifier("column"), r#""column""#);
        assert_eq!(dialect.quote_identifier(r#"table"with"quote"#), r#""table""with""quote""#);
    }
    
    #[test]
    fn test_limit_clause() {
        let dialect = PostgreSQLDialect::new();
        assert_eq!(dialect.limit_clause(Some(10), None), " LIMIT 10");
        assert_eq!(dialect.limit_clause(Some(10), Some(20)), " LIMIT 10 OFFSET 20");
        assert_eq!(dialect.limit_clause(None, Some(20)), " OFFSET 20");
        assert_eq!(dialect.limit_clause(None, None), "");
    }
    
    #[test]
    fn test_boolean_literal() {
        let dialect = PostgreSQLDialect::new();
        assert_eq!(dialect.boolean_literal(true), "TRUE");
        assert_eq!(dialect.boolean_literal(false), "FALSE");
    }
    
    #[test]
    fn test_string_concat() {
        let dialect = PostgreSQLDialect::new();
        assert_eq!(dialect.string_concat("first_name", "last_name"), "first_name || last_name");
    }
    
    #[test]
    fn test_date_literals() {
        let dialect = PostgreSQLDialect::new();
        assert_eq!(dialect.date_literal("2023-01-15"), "DATE '2023-01-15'");
        assert_eq!(dialect.datetime_literal("2023-01-15 10:30:00"), "TIMESTAMP '2023-01-15 10:30:00'");
    }
    
    #[test]
    fn test_qualified_table_name() {
        let dialect = PostgreSQLDialect::new();
        assert_eq!(
            dialect.qualified_table_name(Some("public"), "users"),
            r#""public"."users""#
        );
        assert_eq!(
            dialect.qualified_table_name(None, "users"),
            r#""users""#
        );
    }
    
    #[test]
    fn test_features() {
        let dialect = PostgreSQLDialect::new();
        assert!(dialect.supports_returning_clause());
        assert!(dialect.supports_upsert());
        assert!(dialect.supports_schemas());
    }
}