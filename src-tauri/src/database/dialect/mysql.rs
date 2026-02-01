use super::SqlDialect;
use crate::database::DatabaseType;

/// MySQL-specific SQL dialect implementation
#[derive(Debug, Clone)]
pub struct MySQLDialect;

impl MySQLDialect {
    pub fn new() -> Self {
        Self
    }
}

impl SqlDialect for MySQLDialect {
    fn quote_identifier(&self, identifier: &str) -> String {
        // MySQL uses backticks for identifiers
        // We should escape any existing backticks
        let escaped = identifier.replace('`', "``");
        format!("`{}`", escaped)
    }
    
    fn limit_clause(&self, limit: Option<usize>, offset: Option<usize>) -> String {
        match (limit, offset) {
            (Some(limit_val), Some(offset_val)) => {
                // MySQL supports both syntaxes:
                // 1. LIMIT offset, limit (old syntax)
                // 2. LIMIT limit OFFSET offset (newer, more standard)
                // We'll use the standard syntax for consistency
                format!(" LIMIT {} OFFSET {}", limit_val, offset_val)
            }
            (Some(limit_val), None) => format!(" LIMIT {}", limit_val),
            (None, Some(offset_val)) => {
                // MySQL requires a limit when using offset
                // Use a very large number as a workaround
                format!(" LIMIT 18446744073709551615 OFFSET {}", offset_val)
            }
            (None, None) => String::new(),
        }
    }
    
    fn boolean_literal(&self, value: bool) -> String {
        // MySQL treats 1/0 as boolean values
        // It also accepts TRUE/FALSE keywords
        if value {
            "1".to_string()
        } else {
            "0".to_string()
        }
    }
    
    fn current_timestamp(&self) -> &'static str {
        // MySQL supports both CURRENT_TIMESTAMP and NOW()
        "CURRENT_TIMESTAMP"
    }
    
    fn auto_increment_type(&self) -> &'static str {
        // MySQL uses AUTO_INCREMENT with an integer type
        "INT AUTO_INCREMENT"
    }
    
    fn string_concat(&self, left: &str, right: &str) -> String {
        // MySQL uses CONCAT() function for string concatenation
        format!("CONCAT({}, {})", left, right)
    }
    
    fn case_insensitive_like(&self) -> &'static str {
        // MySQL LIKE is case-insensitive by default (depending on collation)
        // For guaranteed case-insensitive matching, use LOWER() or UPPER()
        "LIKE"
    }
    
    fn date_literal(&self, date: &str) -> String {
        // MySQL uses string literals for dates
        format!("'{}'", date)
    }
    
    fn datetime_literal(&self, datetime: &str) -> String {
        // MySQL uses string literals for datetime
        format!("'{}'", datetime)
    }
    
    fn database_type(&self) -> DatabaseType {
        DatabaseType::MySQL
    }
    
    fn supports_returning_clause(&self) -> bool {
        // MySQL doesn't support RETURNING clause
        // You need to use LAST_INSERT_ID() or similar
        false
    }
    
    fn supports_upsert(&self) -> bool {
        // MySQL supports ON DUPLICATE KEY UPDATE
        true
    }
    
    fn supports_schemas(&self) -> bool {
        // MySQL has database-level organization, not schemas
        // In MySQL, "database" and "schema" are synonymous
        true
    }
}

impl Default for MySQLDialect {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quote_identifier() {
        let dialect = MySQLDialect::new();
        assert_eq!(dialect.quote_identifier("table_name"), "`table_name`");
        assert_eq!(dialect.quote_identifier("column"), "`column`");
        assert_eq!(dialect.quote_identifier("table`with`tick"), "`table``with``tick`");
    }
    
    #[test]
    fn test_limit_clause() {
        let dialect = MySQLDialect::new();
        assert_eq!(dialect.limit_clause(Some(10), None), " LIMIT 10");
        assert_eq!(dialect.limit_clause(Some(10), Some(20)), " LIMIT 10 OFFSET 20");
        // MySQL requires limit when using offset
        assert_eq!(
            dialect.limit_clause(None, Some(20)), 
            " LIMIT 18446744073709551615 OFFSET 20"
        );
        assert_eq!(dialect.limit_clause(None, None), "");
    }
    
    #[test]
    fn test_boolean_literal() {
        let dialect = MySQLDialect::new();
        assert_eq!(dialect.boolean_literal(true), "1");
        assert_eq!(dialect.boolean_literal(false), "0");
    }
    
    #[test]
    fn test_string_concat() {
        let dialect = MySQLDialect::new();
        assert_eq!(
            dialect.string_concat("first_name", "last_name"), 
            "CONCAT(first_name, last_name)"
        );
    }
    
    #[test]
    fn test_date_literals() {
        let dialect = MySQLDialect::new();
        assert_eq!(dialect.date_literal("2023-01-15"), "'2023-01-15'");
        assert_eq!(
            dialect.datetime_literal("2023-01-15 10:30:00"), 
            "'2023-01-15 10:30:00'"
        );
    }
    
    #[test]
    fn test_qualified_table_name() {
        let dialect = MySQLDialect::new();
        assert_eq!(
            dialect.qualified_table_name(Some("mydb"), "users"),
            "`mydb`.`users`"
        );
        assert_eq!(
            dialect.qualified_table_name(None, "users"),
            "`users`"
        );
    }
    
    #[test]
    fn test_features() {
        let dialect = MySQLDialect::new();
        assert!(!dialect.supports_returning_clause());
        assert!(dialect.supports_upsert());
        assert!(dialect.supports_schemas());
    }
}