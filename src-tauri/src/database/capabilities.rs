use serde::{Deserialize, Serialize};

/// Database capabilities that define what features are supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseCapabilities {
    /// Supports schemas (namespaces)
    pub schemas: bool,
    
    /// Supports views
    pub views: bool,
    
    /// Supports stored procedures
    pub stored_procedures: bool,
    
    /// Supports triggers
    pub triggers: bool,
    
    /// Supports transactions
    pub transactions: bool,
    
    /// Supports foreign keys
    pub foreign_keys: bool,
    
    /// Supports partial indexes
    pub partial_indexes: bool,
    
    /// Supports RETURNING clause
    pub returning_clause: bool,
    
    /// Supports JSON data type
    pub json_type: bool,
    
    /// Supports arrays
    pub arrays: bool,
    
    /// Supports full text search
    pub full_text_search: bool,
    
    /// Supports materialized views
    pub materialized_views: bool,
    
    /// Maximum identifier length
    pub max_identifier_length: usize,
    
    /// Maximum number of columns in a table
    pub max_columns: usize,
    
    /// Supports SSL/TLS connections
    pub ssl_support: bool,
    
    /// Supports connection pooling
    pub connection_pooling: bool,
    
    /// Supports EXPLAIN ANALYZE
    pub explain_analyze: bool,
    
    /// Supports savepoints
    pub savepoints: bool,
}

impl DatabaseCapabilities {
    /// PostgreSQL capabilities
    pub fn postgresql() -> Self {
        Self {
            schemas: true,
            views: true,
            stored_procedures: true,
            triggers: true,
            transactions: true,
            foreign_keys: true,
            partial_indexes: true,
            returning_clause: true,
            json_type: true,
            arrays: true,
            full_text_search: true,
            materialized_views: true,
            max_identifier_length: 63,
            max_columns: 1600,
            ssl_support: true,
            connection_pooling: true,
            explain_analyze: true,
            savepoints: true,
        }
    }
    
    /// MySQL capabilities
    pub fn mysql() -> Self {
        Self {
            schemas: true, // MySQL calls them databases
            views: true,
            stored_procedures: true,
            triggers: true,
            transactions: true,
            foreign_keys: true,
            partial_indexes: false,
            returning_clause: false,
            json_type: true,
            arrays: false,
            full_text_search: true,
            materialized_views: false,
            max_identifier_length: 64,
            max_columns: 4096,
            ssl_support: true,
            connection_pooling: true,
            explain_analyze: false, // Has EXPLAIN but not ANALYZE
            savepoints: true,
        }
    }
    
    /// SQLite capabilities
    pub fn sqlite() -> Self {
        Self {
            schemas: false,
            views: true,
            stored_procedures: false,
            triggers: true,
            transactions: true,
            foreign_keys: true, // Must be enabled with PRAGMA
            partial_indexes: true,
            returning_clause: true,
            json_type: true, // JSON1 extension
            arrays: false,
            full_text_search: true, // FTS5 extension
            materialized_views: false,
            max_identifier_length: 2147483647,
            max_columns: 32767,
            ssl_support: false,
            connection_pooling: false,
            explain_analyze: true, // Via EXPLAIN QUERY PLAN
            savepoints: true,
        }
    }
}

/// Query templates for different database operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTemplates {
    pub create_table: String,
    pub create_index: String,
    pub add_foreign_key: String,
    pub drop_table: String,
    pub truncate_table: String,
    pub analyze_table: String,
    pub show_create_table: Option<String>,
}

impl QueryTemplates {
    pub fn postgresql() -> Self {
        Self {
            create_table: r#"CREATE TABLE {table_name} (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)"#.to_string(),
            create_index: "CREATE INDEX {index_name} ON {table_name} ({columns})".to_string(),
            add_foreign_key: "ALTER TABLE {table} ADD CONSTRAINT {constraint_name} FOREIGN KEY ({column}) REFERENCES {ref_table}({ref_column})".to_string(),
            drop_table: "DROP TABLE IF EXISTS {table_name} CASCADE".to_string(),
            truncate_table: "TRUNCATE TABLE {table_name} RESTART IDENTITY CASCADE".to_string(),
            analyze_table: "ANALYZE {table_name}".to_string(),
            show_create_table: None,
        }
    }
    
    pub fn mysql() -> Self {
        Self {
            create_table: r#"CREATE TABLE `{table_name}` (
    `id` INT AUTO_INCREMENT PRIMARY KEY,
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#.to_string(),
            create_index: "CREATE INDEX `{index_name}` ON `{table_name}` ({columns})".to_string(),
            add_foreign_key: "ALTER TABLE `{table}` ADD CONSTRAINT `{constraint_name}` FOREIGN KEY (`{column}`) REFERENCES `{ref_table}`(`{ref_column}`)".to_string(),
            drop_table: "DROP TABLE IF EXISTS `{table_name}`".to_string(),
            truncate_table: "TRUNCATE TABLE `{table_name}`".to_string(),
            analyze_table: "ANALYZE TABLE `{table_name}`".to_string(),
            show_create_table: Some("SHOW CREATE TABLE `{table_name}`".to_string()),
        }
    }
    
    pub fn sqlite() -> Self {
        Self {
            create_table: r#"CREATE TABLE "{table_name}" (
    "id" INTEGER PRIMARY KEY AUTOINCREMENT,
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)"#.to_string(),
            create_index: "CREATE INDEX \"{index_name}\" ON \"{table_name}\" ({columns})".to_string(),
            add_foreign_key: "-- SQLite requires foreign keys to be defined at table creation".to_string(),
            drop_table: "DROP TABLE IF EXISTS \"{table_name}\"".to_string(),
            truncate_table: "DELETE FROM \"{table_name}\"; DELETE FROM sqlite_sequence WHERE name='{table_name}'".to_string(),
            analyze_table: "ANALYZE \"{table_name}\"".to_string(),
            show_create_table: None,
        }
    }
}