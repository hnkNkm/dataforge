// Example usage of SQL Dialect system in DataForge
// This file demonstrates how to use the dialect system in practice

use crate::database::{create_adapter, create_dialect, DatabaseType, ConnectionParams};
use crate::database::dialect::SqlDialect;

/// Example function showing how to use dialects with adapters
pub async fn example_dialect_usage() -> Result<(), Box<dyn std::error::Error>> {
    // Create an adapter for PostgreSQL
    let mut pg_adapter = create_adapter(DatabaseType::PostgreSQL)?;
    
    // Get the dialect from the adapter
    let dialect = pg_adapter.get_dialect();
    
    // Use dialect for SQL generation
    let table = dialect.quote_identifier("users");
    let column = dialect.quote_identifier("email");
    
    // Build a query with dialect-specific syntax
    let query = format!(
        "SELECT * FROM {} WHERE {} {} 'john%'{}",
        table,
        column,
        dialect.case_insensitive_like(),
        dialect.limit_clause(Some(10), Some(20))
    );
    
    println!("Generated PostgreSQL query: {}", query);
    // Output: SELECT * FROM "users" WHERE "email" ILIKE 'john%' LIMIT 10 OFFSET 20
    
    // Example with MySQL
    let mysql_dialect = create_dialect(DatabaseType::MySQL);
    let table = mysql_dialect.quote_identifier("products");
    let name_col = mysql_dialect.quote_identifier("name");
    let desc_col = mysql_dialect.quote_identifier("description");
    
    // Build a MySQL-specific query
    let mysql_query = format!(
        "SELECT *, {} as full_text FROM {} WHERE active = {}{}",
        mysql_dialect.string_concat(&name_col, &desc_col),
        table,
        mysql_dialect.boolean_literal(true),
        mysql_dialect.limit_clause(Some(5), None)
    );
    
    println!("Generated MySQL query: {}", mysql_query);
    // Output: SELECT *, CONCAT(`name`, `description`) as full_text FROM `products` WHERE active = 1 LIMIT 5
    
    // Example with SQLite
    let sqlite_dialect = create_dialect(DatabaseType::SQLite);
    
    // Build an insert query with auto-increment
    let create_table = format!(
        "CREATE TABLE {} (
            id {},
            created_at {} DEFAULT {},
            name TEXT NOT NULL
        )",
        sqlite_dialect.quote_identifier("logs"),
        sqlite_dialect.auto_increment_type(),
        "TIMESTAMP",
        sqlite_dialect.current_timestamp()
    );
    
    println!("Generated SQLite DDL: {}", create_table);
    // Output: CREATE TABLE "logs" (
    //     id INTEGER PRIMARY KEY AUTOINCREMENT,
    //     created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    //     name TEXT NOT NULL
    // )
    
    Ok(())
}

/// Example function showing dialect-aware query building
pub fn build_paginated_query(
    dialect: &dyn SqlDialect,
    table: &str,
    page: usize,
    page_size: usize,
) -> String {
    let offset = (page - 1) * page_size;
    
    format!(
        "SELECT * FROM {}{}",
        dialect.quote_identifier(table),
        dialect.limit_clause(Some(page_size), Some(offset))
    )
}

/// Example function for cross-database date queries
pub fn build_date_range_query(
    dialect: &dyn SqlDialect,
    table: &str,
    date_column: &str,
    start_date: &str,
    end_date: &str,
) -> String {
    format!(
        "SELECT * FROM {} WHERE {} BETWEEN {} AND {}",
        dialect.quote_identifier(table),
        dialect.quote_identifier(date_column),
        dialect.date_literal(start_date),
        dialect.date_literal(end_date)
    )
}

/// Example function showing schema-aware queries
pub fn build_schema_query(
    dialect: &dyn SqlDialect,
    schema: Option<&str>,
    table: &str,
) -> String {
    let qualified_table = dialect.qualified_table_name(schema, table);
    
    format!("SELECT COUNT(*) FROM {}", qualified_table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::dialect::{PostgreSQLDialect, MySQLDialect, SQLiteDialect};
    
    #[test]
    fn test_paginated_query_postgres() {
        let dialect = PostgreSQLDialect::new();
        let query = build_paginated_query(&dialect, "users", 3, 10);
        assert_eq!(query, r#"SELECT * FROM "users" LIMIT 10 OFFSET 20"#);
    }
    
    #[test]
    fn test_paginated_query_mysql() {
        let dialect = MySQLDialect::new();
        let query = build_paginated_query(&dialect, "users", 3, 10);
        assert_eq!(query, "SELECT * FROM `users` LIMIT 10 OFFSET 20");
    }
    
    #[test]
    fn test_date_range_query() {
        let pg_dialect = PostgreSQLDialect::new();
        let query = build_date_range_query(
            &pg_dialect,
            "orders",
            "order_date",
            "2023-01-01",
            "2023-12-31"
        );
        assert_eq!(
            query,
            r#"SELECT * FROM "orders" WHERE "order_date" BETWEEN DATE '2023-01-01' AND DATE '2023-12-31'"#
        );
    }
    
    #[test]
    fn test_schema_query() {
        let pg_dialect = PostgreSQLDialect::new();
        let query = build_schema_query(&pg_dialect, Some("public"), "users");
        assert_eq!(query, r#"SELECT COUNT(*) FROM "public"."users""#);
        
        let sqlite_dialect = SQLiteDialect::new();
        let query = build_schema_query(&sqlite_dialect, Some("main"), "users");
        // SQLite doesn't support schemas, so it ignores the schema parameter
        assert_eq!(query, r#"SELECT COUNT(*) FROM "users""#);
    }
}