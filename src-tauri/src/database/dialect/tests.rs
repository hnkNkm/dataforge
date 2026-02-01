// Comprehensive tests for SQL dialect implementations

#[cfg(test)]
mod dialect_tests {
    use crate::database::dialect::{SqlDialect, PostgreSQLDialect, MySQLDialect, SQLiteDialect};
    use crate::database::DatabaseType;
    
    #[test]
    fn test_all_dialects_quote_identifier() {
        let test_cases = vec![
            ("simple", r#""simple""#, "`simple`", r#""simple""#),
            ("with space", r#""with space""#, "`with space`", r#""with space""#),
            ("123numeric", r#""123numeric""#, "`123numeric`", r#""123numeric""#),
        ];
        
        let pg = PostgreSQLDialect::new();
        let mysql = MySQLDialect::new();
        let sqlite = SQLiteDialect::new();
        
        for (input, expected_pg, expected_mysql, expected_sqlite) in test_cases {
            assert_eq!(pg.quote_identifier(input), expected_pg, "PostgreSQL failed for {}", input);
            assert_eq!(mysql.quote_identifier(input), expected_mysql, "MySQL failed for {}", input);
            assert_eq!(sqlite.quote_identifier(input), expected_sqlite, "SQLite failed for {}", input);
        }
    }
    
    #[test]
    fn test_all_dialects_limit_clause() {
        let pg = PostgreSQLDialect::new();
        let mysql = MySQLDialect::new();
        let sqlite = SQLiteDialect::new();
        
        // Test LIMIT only
        assert_eq!(pg.limit_clause(Some(10), None), " LIMIT 10");
        assert_eq!(mysql.limit_clause(Some(10), None), " LIMIT 10");
        assert_eq!(sqlite.limit_clause(Some(10), None), " LIMIT 10");
        
        // Test LIMIT with OFFSET
        assert_eq!(pg.limit_clause(Some(10), Some(20)), " LIMIT 10 OFFSET 20");
        assert_eq!(mysql.limit_clause(Some(10), Some(20)), " LIMIT 10 OFFSET 20");
        assert_eq!(sqlite.limit_clause(Some(10), Some(20)), " LIMIT 10 OFFSET 20");
        
        // Test OFFSET only (databases handle this differently)
        assert_eq!(pg.limit_clause(None, Some(20)), " OFFSET 20");
        assert_eq!(mysql.limit_clause(None, Some(20)), " LIMIT 18446744073709551615 OFFSET 20");
        assert_eq!(sqlite.limit_clause(None, Some(20)), " LIMIT -1 OFFSET 20");
    }
    
    #[test]
    fn test_all_dialects_boolean_literal() {
        let pg = PostgreSQLDialect::new();
        let mysql = MySQLDialect::new();
        let sqlite = SQLiteDialect::new();
        
        // PostgreSQL uses TRUE/FALSE
        assert_eq!(pg.boolean_literal(true), "TRUE");
        assert_eq!(pg.boolean_literal(false), "FALSE");
        
        // MySQL uses 1/0
        assert_eq!(mysql.boolean_literal(true), "1");
        assert_eq!(mysql.boolean_literal(false), "0");
        
        // SQLite uses 1/0
        assert_eq!(sqlite.boolean_literal(true), "1");
        assert_eq!(sqlite.boolean_literal(false), "0");
    }
    
    #[test]
    fn test_all_dialects_current_timestamp() {
        let pg = PostgreSQLDialect::new();
        let mysql = MySQLDialect::new();
        let sqlite = SQLiteDialect::new();
        
        assert_eq!(pg.current_timestamp(), "CURRENT_TIMESTAMP");
        assert_eq!(mysql.current_timestamp(), "CURRENT_TIMESTAMP");
        assert_eq!(sqlite.current_timestamp(), "CURRENT_TIMESTAMP");
    }
    
    #[test]
    fn test_all_dialects_auto_increment() {
        let pg = PostgreSQLDialect::new();
        let mysql = MySQLDialect::new();
        let sqlite = SQLiteDialect::new();
        
        assert_eq!(pg.auto_increment_type(), "SERIAL");
        assert_eq!(mysql.auto_increment_type(), "INT AUTO_INCREMENT");
        assert_eq!(sqlite.auto_increment_type(), "INTEGER PRIMARY KEY AUTOINCREMENT");
    }
    
    #[test]
    fn test_all_dialects_string_concat() {
        let pg = PostgreSQLDialect::new();
        let mysql = MySQLDialect::new();
        let sqlite = SQLiteDialect::new();
        
        // PostgreSQL and SQLite use ||
        assert_eq!(pg.string_concat("a", "b"), "a || b");
        assert_eq!(sqlite.string_concat("a", "b"), "a || b");
        
        // MySQL uses CONCAT()
        assert_eq!(mysql.string_concat("a", "b"), "CONCAT(a, b)");
    }
    
    #[test]
    fn test_all_dialects_case_insensitive_like() {
        let pg = PostgreSQLDialect::new();
        let mysql = MySQLDialect::new();
        let sqlite = SQLiteDialect::new();
        
        assert_eq!(pg.case_insensitive_like(), "ILIKE");
        assert_eq!(mysql.case_insensitive_like(), "LIKE");
        assert_eq!(sqlite.case_insensitive_like(), "LIKE");
    }
    
    #[test]
    fn test_all_dialects_date_literals() {
        let pg = PostgreSQLDialect::new();
        let mysql = MySQLDialect::new();
        let sqlite = SQLiteDialect::new();
        
        let date = "2023-12-25";
        let datetime = "2023-12-25 10:30:45";
        
        // PostgreSQL uses DATE and TIMESTAMP prefixes
        assert_eq!(pg.date_literal(date), "DATE '2023-12-25'");
        assert_eq!(pg.datetime_literal(datetime), "TIMESTAMP '2023-12-25 10:30:45'");
        
        // MySQL and SQLite use quoted strings
        assert_eq!(mysql.date_literal(date), "'2023-12-25'");
        assert_eq!(mysql.datetime_literal(datetime), "'2023-12-25 10:30:45'");
        
        assert_eq!(sqlite.date_literal(date), "'2023-12-25'");
        assert_eq!(sqlite.datetime_literal(datetime), "'2023-12-25 10:30:45'");
    }
    
    #[test]
    fn test_all_dialects_database_type() {
        let pg = PostgreSQLDialect::new();
        let mysql = MySQLDialect::new();
        let sqlite = SQLiteDialect::new();
        
        assert_eq!(pg.database_type(), DatabaseType::PostgreSQL);
        assert_eq!(mysql.database_type(), DatabaseType::MySQL);
        assert_eq!(sqlite.database_type(), DatabaseType::SQLite);
    }
    
    #[test]
    fn test_all_dialects_features() {
        let pg = PostgreSQLDialect::new();
        let mysql = MySQLDialect::new();
        let sqlite = SQLiteDialect::new();
        
        // RETURNING clause support
        assert!(pg.supports_returning_clause());
        assert!(!mysql.supports_returning_clause());
        assert!(sqlite.supports_returning_clause());
        
        // UPSERT support
        assert!(pg.supports_upsert());
        assert!(mysql.supports_upsert());
        assert!(sqlite.supports_upsert());
        
        // Schema support
        assert!(pg.supports_schemas());
        assert!(mysql.supports_schemas());
        assert!(!sqlite.supports_schemas());
    }
    
    #[test]
    fn test_qualified_table_names() {
        let pg = PostgreSQLDialect::new();
        let mysql = MySQLDialect::new();
        let sqlite = SQLiteDialect::new();
        
        // With schema
        assert_eq!(
            pg.qualified_table_name(Some("public"), "users"),
            r#""public"."users""#
        );
        assert_eq!(
            mysql.qualified_table_name(Some("mydb"), "users"),
            "`mydb`.`users`"
        );
        // SQLite doesn't support schemas, so it ignores the schema
        assert_eq!(
            sqlite.qualified_table_name(Some("main"), "users"),
            r#""users""#
        );
        
        // Without schema
        assert_eq!(pg.qualified_table_name(None, "users"), r#""users""#);
        assert_eq!(mysql.qualified_table_name(None, "users"), "`users`");
        assert_eq!(sqlite.qualified_table_name(None, "users"), r#""users""#);
    }
    
    #[test]
    fn test_null_checks() {
        let pg = PostgreSQLDialect::new();
        
        assert_eq!(pg.is_null("column_name"), "column_name IS NULL");
        assert_eq!(pg.is_not_null("column_name"), "column_name IS NOT NULL");
    }
    
    #[test]
    fn test_cast_syntax() {
        let pg = PostgreSQLDialect::new();
        let mysql = MySQLDialect::new();
        let sqlite = SQLiteDialect::new();
        
        // All databases support standard CAST syntax
        assert_eq!(pg.cast("column", "INTEGER"), "CAST(column AS INTEGER)");
        assert_eq!(mysql.cast("column", "UNSIGNED"), "CAST(column AS UNSIGNED)");
        assert_eq!(sqlite.cast("column", "TEXT"), "CAST(column AS TEXT)");
    }
    
    #[test]
    fn test_escaping_special_characters() {
        let pg = PostgreSQLDialect::new();
        let mysql = MySQLDialect::new();
        let sqlite = SQLiteDialect::new();
        
        // Test escaping quotes in identifiers
        assert_eq!(
            pg.quote_identifier(r#"table"with"quotes"#),
            r#""table""with""quotes""#
        );
        assert_eq!(
            mysql.quote_identifier("table`with`backticks"),
            "`table``with``backticks`"
        );
        assert_eq!(
            sqlite.quote_identifier(r#"table"with"quotes"#),
            r#""table""with""quotes""#
        );
    }
}