pub mod postgres;
pub mod mysql;
pub mod sqlite;

pub use postgres::PostgreSQLDialect;
pub use mysql::MySQLDialect;
pub use sqlite::SQLiteDialect;

use crate::database::DatabaseType;

/// SQL dialect trait for database-specific SQL generation
pub trait SqlDialect: Send + Sync {
    /// Quote an identifier (table or column name) according to database rules
    /// 
    /// # Examples
    /// - PostgreSQL: "table_name" -> "table_name"
    /// - MySQL: `table_name` -> `table_name`
    /// - SQLite: "table_name" -> "table_name"
    fn quote_identifier(&self, identifier: &str) -> String;
    
    /// Generate a LIMIT/OFFSET clause
    /// 
    /// # Examples
    /// - PostgreSQL/SQLite: "LIMIT 10 OFFSET 20"
    /// - MySQL: "LIMIT 20, 10" or "LIMIT 10 OFFSET 20"
    fn limit_clause(&self, limit: Option<usize>, offset: Option<usize>) -> String;
    
    /// Format a boolean literal
    /// 
    /// # Examples
    /// - PostgreSQL: true/false or TRUE/FALSE
    /// - MySQL: 1/0 or TRUE/FALSE
    /// - SQLite: 1/0
    fn boolean_literal(&self, value: bool) -> String;
    
    /// Get the current timestamp syntax
    /// 
    /// # Examples
    /// - PostgreSQL: CURRENT_TIMESTAMP or NOW()
    /// - MySQL: CURRENT_TIMESTAMP or NOW()
    /// - SQLite: CURRENT_TIMESTAMP
    fn current_timestamp(&self) -> &'static str;
    
    /// Get the auto-increment column type definition
    /// 
    /// # Examples
    /// - PostgreSQL: "SERIAL" or "BIGSERIAL"
    /// - MySQL: "INT AUTO_INCREMENT"
    /// - SQLite: "INTEGER PRIMARY KEY AUTOINCREMENT"
    fn auto_increment_type(&self) -> &'static str;
    
    /// Get the string concatenation operator or function
    /// 
    /// # Examples
    /// - PostgreSQL: "||" operator
    /// - MySQL: CONCAT() function
    /// - SQLite: "||" operator
    fn string_concat(&self, left: &str, right: &str) -> String;
    
    /// Get the case-insensitive LIKE operator
    /// 
    /// # Examples
    /// - PostgreSQL: ILIKE
    /// - MySQL: LIKE (case-insensitive by default)
    /// - SQLite: LIKE (case-insensitive by default)
    fn case_insensitive_like(&self) -> &'static str;
    
    /// Format a date/datetime literal
    /// 
    /// # Examples
    /// - PostgreSQL: DATE '2023-01-01', TIMESTAMP '2023-01-01 12:00:00'
    /// - MySQL: '2023-01-01', '2023-01-01 12:00:00'
    /// - SQLite: '2023-01-01', '2023-01-01 12:00:00'
    fn date_literal(&self, date: &str) -> String;
    fn datetime_literal(&self, datetime: &str) -> String;
    
    /// Get the database type this dialect is for
    fn database_type(&self) -> DatabaseType;
    
    /// Check if the database supports a specific feature
    fn supports_returning_clause(&self) -> bool;
    fn supports_upsert(&self) -> bool;
    fn supports_schemas(&self) -> bool;
    
    /// Build a fully qualified table name with optional schema
    fn qualified_table_name(&self, schema: Option<&str>, table: &str) -> String {
        match schema {
            Some(s) if self.supports_schemas() => {
                format!("{}.{}", self.quote_identifier(s), self.quote_identifier(table))
            }
            _ => self.quote_identifier(table),
        }
    }
    
    /// Get the appropriate NULL check syntax
    fn is_null(&self, column: &str) -> String {
        format!("{} IS NULL", column)
    }
    
    fn is_not_null(&self, column: &str) -> String {
        format!("{} IS NOT NULL", column)
    }
    
    /// Get the CAST syntax
    fn cast(&self, expression: &str, data_type: &str) -> String {
        format!("CAST({} AS {})", expression, data_type)
    }
}

/// Factory function to create appropriate dialect
pub fn create_dialect(database_type: DatabaseType) -> Box<dyn SqlDialect> {
    match database_type {
        DatabaseType::PostgreSQL => Box::new(PostgreSQLDialect::new()),
        DatabaseType::MySQL => Box::new(MySQLDialect::new()),
        DatabaseType::SQLite => Box::new(SQLiteDialect::new()),
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod basic_tests {
    use super::*;
    
    #[test]
    fn test_dialect_creation() {
        let pg_dialect = create_dialect(DatabaseType::PostgreSQL);
        assert_eq!(pg_dialect.database_type(), DatabaseType::PostgreSQL);
        
        let mysql_dialect = create_dialect(DatabaseType::MySQL);
        assert_eq!(mysql_dialect.database_type(), DatabaseType::MySQL);
        
        let sqlite_dialect = create_dialect(DatabaseType::SQLite);
        assert_eq!(sqlite_dialect.database_type(), DatabaseType::SQLite);
    }
}