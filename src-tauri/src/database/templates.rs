use serde::{Deserialize, Serialize};
use super::adapter::DatabaseType;

/// Query template category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateCategory {
    Table,
    Index,
    View,
    Constraint,
    Query,
    Admin,
}

/// Query template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTemplate {
    pub id: String,
    pub name: String,
    pub category: TemplateCategory,
    pub description: String,
    pub template: String,
    pub parameters: Vec<TemplateParameter>,
    pub supported_databases: Vec<DatabaseType>,
}

/// Template parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub description: String,
    pub default_value: Option<String>,
    pub required: bool,
}

/// Database-specific query templates
pub struct QueryTemplates;

impl QueryTemplates {
    /// Get all templates for a specific database type
    pub fn for_database(db_type: DatabaseType) -> Vec<QueryTemplate> {
        let mut templates = Vec::new();
        
        // Add common templates
        templates.extend(Self::common_templates(db_type));
        
        // Add database-specific templates
        match db_type {
            DatabaseType::PostgreSQL => templates.extend(Self::postgres_templates()),
            DatabaseType::MySQL => templates.extend(Self::mysql_templates()),
            DatabaseType::SQLite => templates.extend(Self::sqlite_templates()),
        }
        
        templates
    }
    
    /// Common templates adjusted for each database
    fn common_templates(db_type: DatabaseType) -> Vec<QueryTemplate> {
        vec![
            // CREATE TABLE
            QueryTemplate {
                id: "create_table".to_string(),
                name: "Create Table".to_string(),
                category: TemplateCategory::Table,
                description: "Create a new table with primary key".to_string(),
                template: match db_type {
                    DatabaseType::PostgreSQL => {
                        r#"CREATE TABLE {{schema}}.{{table_name}} (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);"#.to_string()
                    },
                    DatabaseType::MySQL => {
                        r#"CREATE TABLE {{database}}.{{table_name}} (
    id INT AUTO_INCREMENT PRIMARY KEY,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;"#.to_string()
                    },
                    DatabaseType::SQLite => {
                        r#"CREATE TABLE {{table_name}} (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);"#.to_string()
                    },
                },
                parameters: vec![
                    TemplateParameter {
                        name: "table_name".to_string(),
                        description: "Name of the table".to_string(),
                        default_value: Some("new_table".to_string()),
                        required: true,
                    },
                    if db_type == DatabaseType::PostgreSQL {
                        TemplateParameter {
                            name: "schema".to_string(),
                            description: "Schema name".to_string(),
                            default_value: Some("public".to_string()),
                            required: true,
                        }
                    } else if db_type == DatabaseType::MySQL {
                        TemplateParameter {
                            name: "database".to_string(),
                            description: "Database name".to_string(),
                            default_value: None,
                            required: true,
                        }
                    } else {
                        TemplateParameter {
                            name: "".to_string(),
                            description: "".to_string(),
                            default_value: None,
                            required: false,
                        }
                    },
                ].into_iter().filter(|p| !p.name.is_empty()).collect(),
                supported_databases: vec![db_type],
            },
            
            // CREATE INDEX
            QueryTemplate {
                id: "create_index".to_string(),
                name: "Create Index".to_string(),
                category: TemplateCategory::Index,
                description: "Create an index on table columns".to_string(),
                template: match db_type {
                    DatabaseType::PostgreSQL => {
                        r#"CREATE INDEX {{index_name}} 
ON {{schema}}.{{table_name}} ({{columns}})
WHERE {{condition}};"#.to_string()
                    },
                    DatabaseType::MySQL => {
                        r#"CREATE INDEX {{index_name}} 
ON {{table_name}} ({{columns}});"#.to_string()
                    },
                    DatabaseType::SQLite => {
                        r#"CREATE INDEX {{index_name}} 
ON {{table_name}} ({{columns}})
WHERE {{condition}};"#.to_string()
                    },
                },
                parameters: vec![
                    TemplateParameter {
                        name: "index_name".to_string(),
                        description: "Name of the index".to_string(),
                        default_value: Some("idx_table_column".to_string()),
                        required: true,
                    },
                    TemplateParameter {
                        name: "table_name".to_string(),
                        description: "Table to index".to_string(),
                        default_value: None,
                        required: true,
                    },
                    TemplateParameter {
                        name: "columns".to_string(),
                        description: "Columns to index".to_string(),
                        default_value: None,
                        required: true,
                    },
                ],
                supported_databases: vec![db_type],
            },
            
            // ADD FOREIGN KEY
            QueryTemplate {
                id: "add_foreign_key".to_string(),
                name: "Add Foreign Key".to_string(),
                category: TemplateCategory::Constraint,
                description: "Add a foreign key constraint".to_string(),
                template: match db_type {
                    DatabaseType::PostgreSQL => {
                        r#"ALTER TABLE {{table_name}}
ADD CONSTRAINT {{constraint_name}}
FOREIGN KEY ({{column}})
REFERENCES {{ref_table}}({{ref_column}})
ON DELETE CASCADE
ON UPDATE CASCADE;"#.to_string()
                    },
                    DatabaseType::MySQL => {
                        r#"ALTER TABLE {{table_name}}
ADD CONSTRAINT {{constraint_name}}
FOREIGN KEY ({{column}})
REFERENCES {{ref_table}}({{ref_column}})
ON DELETE CASCADE
ON UPDATE CASCADE;"#.to_string()
                    },
                    DatabaseType::SQLite => {
                        r#"-- SQLite requires foreign keys to be defined during table creation
-- Or recreate the table with the foreign key
-- Ensure foreign keys are enabled: PRAGMA foreign_keys = ON;"#.to_string()
                    },
                },
                parameters: vec![
                    TemplateParameter {
                        name: "table_name".to_string(),
                        description: "Table to add constraint to".to_string(),
                        default_value: None,
                        required: true,
                    },
                    TemplateParameter {
                        name: "constraint_name".to_string(),
                        description: "Name of the constraint".to_string(),
                        default_value: Some("fk_table_ref".to_string()),
                        required: true,
                    },
                ],
                supported_databases: vec![db_type],
            },
        ]
    }
    
    /// PostgreSQL-specific templates
    fn postgres_templates() -> Vec<QueryTemplate> {
        vec![
            // CREATE SCHEMA
            QueryTemplate {
                id: "pg_create_schema".to_string(),
                name: "Create Schema".to_string(),
                category: TemplateCategory::Admin,
                description: "Create a new schema".to_string(),
                template: r#"CREATE SCHEMA IF NOT EXISTS {{schema_name}}
AUTHORIZATION {{owner}};"#.to_string(),
                parameters: vec![
                    TemplateParameter {
                        name: "schema_name".to_string(),
                        description: "Name of the schema".to_string(),
                        default_value: None,
                        required: true,
                    },
                    TemplateParameter {
                        name: "owner".to_string(),
                        description: "Schema owner".to_string(),
                        default_value: Some("CURRENT_USER".to_string()),
                        required: false,
                    },
                ],
                supported_databases: vec![DatabaseType::PostgreSQL],
            },
            
            // CREATE MATERIALIZED VIEW
            QueryTemplate {
                id: "pg_create_mat_view".to_string(),
                name: "Create Materialized View".to_string(),
                category: TemplateCategory::View,
                description: "Create a materialized view".to_string(),
                template: r#"CREATE MATERIALIZED VIEW {{view_name}} AS
{{query}}
WITH DATA;"#.to_string(),
                parameters: vec![
                    TemplateParameter {
                        name: "view_name".to_string(),
                        description: "Name of the materialized view".to_string(),
                        default_value: None,
                        required: true,
                    },
                    TemplateParameter {
                        name: "query".to_string(),
                        description: "Query to materialize".to_string(),
                        default_value: None,
                        required: true,
                    },
                ],
                supported_databases: vec![DatabaseType::PostgreSQL],
            },
            
            // CREATE FUNCTION
            QueryTemplate {
                id: "pg_create_function".to_string(),
                name: "Create Function".to_string(),
                category: TemplateCategory::Admin,
                description: "Create a PL/pgSQL function".to_string(),
                template: r#"CREATE OR REPLACE FUNCTION {{function_name}}({{parameters}})
RETURNS {{return_type}} AS $$
BEGIN
    -- Function body
    RETURN {{return_value}};
END;
$$ LANGUAGE plpgsql;"#.to_string(),
                parameters: vec![
                    TemplateParameter {
                        name: "function_name".to_string(),
                        description: "Name of the function".to_string(),
                        default_value: None,
                        required: true,
                    },
                ],
                supported_databases: vec![DatabaseType::PostgreSQL],
            },
            
            // UPSERT (INSERT ON CONFLICT)
            QueryTemplate {
                id: "pg_upsert".to_string(),
                name: "Upsert (Insert or Update)".to_string(),
                category: TemplateCategory::Query,
                description: "Insert or update on conflict".to_string(),
                template: r#"INSERT INTO {{table_name}} ({{columns}})
VALUES ({{values}})
ON CONFLICT ({{conflict_column}})
DO UPDATE SET 
    {{update_columns}};"#.to_string(),
                parameters: vec![
                    TemplateParameter {
                        name: "table_name".to_string(),
                        description: "Table name".to_string(),
                        default_value: None,
                        required: true,
                    },
                ],
                supported_databases: vec![DatabaseType::PostgreSQL],
            },
        ]
    }
    
    /// MySQL-specific templates
    fn mysql_templates() -> Vec<QueryTemplate> {
        vec![
            // CREATE STORED PROCEDURE
            QueryTemplate {
                id: "mysql_create_procedure".to_string(),
                name: "Create Stored Procedure".to_string(),
                category: TemplateCategory::Admin,
                description: "Create a stored procedure".to_string(),
                template: r#"DELIMITER //
CREATE PROCEDURE {{procedure_name}}({{parameters}})
BEGIN
    -- Procedure body
    SELECT * FROM {{table_name}};
END //
DELIMITER ;"#.to_string(),
                parameters: vec![
                    TemplateParameter {
                        name: "procedure_name".to_string(),
                        description: "Name of the procedure".to_string(),
                        default_value: None,
                        required: true,
                    },
                ],
                supported_databases: vec![DatabaseType::MySQL],
            },
            
            // CREATE TRIGGER
            QueryTemplate {
                id: "mysql_create_trigger".to_string(),
                name: "Create Trigger".to_string(),
                category: TemplateCategory::Admin,
                description: "Create a trigger".to_string(),
                template: r#"CREATE TRIGGER {{trigger_name}}
{{timing}} {{event}} ON {{table_name}}
FOR EACH ROW
BEGIN
    -- Trigger body
    -- NEW.column_name for new values
    -- OLD.column_name for old values
END;"#.to_string(),
                parameters: vec![
                    TemplateParameter {
                        name: "trigger_name".to_string(),
                        description: "Name of the trigger".to_string(),
                        default_value: None,
                        required: true,
                    },
                    TemplateParameter {
                        name: "timing".to_string(),
                        description: "BEFORE or AFTER".to_string(),
                        default_value: Some("BEFORE".to_string()),
                        required: true,
                    },
                    TemplateParameter {
                        name: "event".to_string(),
                        description: "INSERT, UPDATE, or DELETE".to_string(),
                        default_value: Some("INSERT".to_string()),
                        required: true,
                    },
                ],
                supported_databases: vec![DatabaseType::MySQL],
            },
            
            // PARTITION TABLE
            QueryTemplate {
                id: "mysql_partition_table".to_string(),
                name: "Create Partitioned Table".to_string(),
                category: TemplateCategory::Table,
                description: "Create a partitioned table".to_string(),
                template: r#"CREATE TABLE {{table_name}} (
    id INT NOT NULL,
    {{date_column}} DATE NOT NULL,
    -- other columns
    PRIMARY KEY (id, {{date_column}})
)
PARTITION BY RANGE (YEAR({{date_column}})) (
    PARTITION p2023 VALUES LESS THAN (2024),
    PARTITION p2024 VALUES LESS THAN (2025),
    PARTITION p_future VALUES LESS THAN MAXVALUE
);"#.to_string(),
                parameters: vec![
                    TemplateParameter {
                        name: "table_name".to_string(),
                        description: "Name of the table".to_string(),
                        default_value: None,
                        required: true,
                    },
                ],
                supported_databases: vec![DatabaseType::MySQL],
            },
        ]
    }
    
    /// SQLite-specific templates
    fn sqlite_templates() -> Vec<QueryTemplate> {
        vec![
            // CREATE VIRTUAL TABLE (FTS)
            QueryTemplate {
                id: "sqlite_create_fts".to_string(),
                name: "Create Full-Text Search Table".to_string(),
                category: TemplateCategory::Table,
                description: "Create a full-text search virtual table".to_string(),
                template: r#"CREATE VIRTUAL TABLE {{table_name}}
USING fts5({{columns}});"#.to_string(),
                parameters: vec![
                    TemplateParameter {
                        name: "table_name".to_string(),
                        description: "Name of the FTS table".to_string(),
                        default_value: None,
                        required: true,
                    },
                    TemplateParameter {
                        name: "columns".to_string(),
                        description: "Columns for FTS".to_string(),
                        default_value: Some("title, content".to_string()),
                        required: true,
                    },
                ],
                supported_databases: vec![DatabaseType::SQLite],
            },
            
            // PRAGMA SETTINGS
            QueryTemplate {
                id: "sqlite_pragma_settings".to_string(),
                name: "Configure Database Settings".to_string(),
                category: TemplateCategory::Admin,
                description: "Common SQLite PRAGMA settings".to_string(),
                template: r#"-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- Set journal mode to WAL for better concurrency
PRAGMA journal_mode = WAL;

-- Set synchronous mode
PRAGMA synchronous = NORMAL;

-- Optimize cache size (negative = KB, positive = pages)
PRAGMA cache_size = -20000;  -- 20MB

-- Set busy timeout to 5 seconds
PRAGMA busy_timeout = 5000;"#.to_string(),
                parameters: vec![],
                supported_databases: vec![DatabaseType::SQLite],
            },
            
            // CREATE TRIGGER FOR UPDATED_AT
            QueryTemplate {
                id: "sqlite_updated_at_trigger".to_string(),
                name: "Auto-update Timestamp Trigger".to_string(),
                category: TemplateCategory::Admin,
                description: "Create trigger to auto-update updated_at column".to_string(),
                template: r#"CREATE TRIGGER {{trigger_name}}
AFTER UPDATE ON {{table_name}}
FOR EACH ROW
BEGIN
    UPDATE {{table_name}}
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;"#.to_string(),
                parameters: vec![
                    TemplateParameter {
                        name: "trigger_name".to_string(),
                        description: "Name of the trigger".to_string(),
                        default_value: Some("update_timestamp".to_string()),
                        required: true,
                    },
                    TemplateParameter {
                        name: "table_name".to_string(),
                        description: "Table to add trigger to".to_string(),
                        default_value: None,
                        required: true,
                    },
                ],
                supported_databases: vec![DatabaseType::SQLite],
            },
        ]
    }
    
    /// Get data types for a specific database
    pub fn data_types(db_type: DatabaseType) -> Vec<DataTypeInfo> {
        match db_type {
            DatabaseType::PostgreSQL => Self::postgres_data_types(),
            DatabaseType::MySQL => Self::mysql_data_types(),
            DatabaseType::SQLite => Self::sqlite_data_types(),
        }
    }
    
    fn postgres_data_types() -> Vec<DataTypeInfo> {
        vec![
            // Numeric
            DataTypeInfo::new("INTEGER", "32-bit integer", "-2147483648 to 2147483647"),
            DataTypeInfo::new("BIGINT", "64-bit integer", "-9223372036854775808 to 9223372036854775807"),
            DataTypeInfo::new("SERIAL", "Auto-incrementing 32-bit integer", "1 to 2147483647"),
            DataTypeInfo::new("BIGSERIAL", "Auto-incrementing 64-bit integer", "1 to 9223372036854775807"),
            DataTypeInfo::new("DECIMAL(p,s)", "Exact numeric", "Up to 131072 digits before decimal, 16383 after"),
            DataTypeInfo::new("REAL", "32-bit floating point", "6 decimal digits precision"),
            DataTypeInfo::new("DOUBLE PRECISION", "64-bit floating point", "15 decimal digits precision"),
            
            // Text
            DataTypeInfo::new("VARCHAR(n)", "Variable-length string", "Up to n characters"),
            DataTypeInfo::new("TEXT", "Variable-length string", "Unlimited length"),
            DataTypeInfo::new("CHAR(n)", "Fixed-length string", "Exactly n characters"),
            
            // Date/Time
            DataTypeInfo::new("DATE", "Calendar date", "4713 BC to 5874897 AD"),
            DataTypeInfo::new("TIME", "Time of day", "00:00:00 to 24:00:00"),
            DataTypeInfo::new("TIMESTAMP", "Date and time", "4713 BC to 294276 AD"),
            DataTypeInfo::new("TIMESTAMPTZ", "Date and time with timezone", "4713 BC to 294276 AD"),
            DataTypeInfo::new("INTERVAL", "Time interval", "-178000000 years to 178000000 years"),
            
            // Boolean
            DataTypeInfo::new("BOOLEAN", "Logical Boolean", "true/false/null"),
            
            // Binary
            DataTypeInfo::new("BYTEA", "Binary data", "Up to 1GB"),
            
            // JSON
            DataTypeInfo::new("JSON", "JSON data", "Text storage"),
            DataTypeInfo::new("JSONB", "Binary JSON data", "Binary storage, indexable"),
            
            // Arrays
            DataTypeInfo::new("[]", "Array of any type", "e.g., INTEGER[], TEXT[]"),
            
            // UUID
            DataTypeInfo::new("UUID", "Universally Unique Identifier", "128-bit value"),
            
            // Network
            DataTypeInfo::new("INET", "IPv4 or IPv6 address", "7 or 19 bytes"),
            DataTypeInfo::new("CIDR", "IPv4 or IPv6 network", "7 or 19 bytes"),
            DataTypeInfo::new("MACADDR", "MAC address", "6 bytes"),
        ]
    }
    
    fn mysql_data_types() -> Vec<DataTypeInfo> {
        vec![
            // Numeric
            DataTypeInfo::new("TINYINT", "8-bit integer", "-128 to 127 (signed)"),
            DataTypeInfo::new("SMALLINT", "16-bit integer", "-32768 to 32767 (signed)"),
            DataTypeInfo::new("MEDIUMINT", "24-bit integer", "-8388608 to 8388607 (signed)"),
            DataTypeInfo::new("INT", "32-bit integer", "-2147483648 to 2147483647 (signed)"),
            DataTypeInfo::new("BIGINT", "64-bit integer", "-9223372036854775808 to 9223372036854775807"),
            DataTypeInfo::new("DECIMAL(M,D)", "Fixed-point number", "M digits total, D after decimal"),
            DataTypeInfo::new("FLOAT", "32-bit floating point", "Approximate numeric"),
            DataTypeInfo::new("DOUBLE", "64-bit floating point", "Approximate numeric"),
            
            // Text
            DataTypeInfo::new("VARCHAR(n)", "Variable-length string", "0 to 65535 characters"),
            DataTypeInfo::new("TEXT", "Text data", "Up to 65535 characters"),
            DataTypeInfo::new("MEDIUMTEXT", "Medium text data", "Up to 16777215 characters"),
            DataTypeInfo::new("LONGTEXT", "Long text data", "Up to 4294967295 characters"),
            DataTypeInfo::new("CHAR(n)", "Fixed-length string", "0 to 255 characters"),
            
            // Date/Time
            DataTypeInfo::new("DATE", "Date", "1000-01-01 to 9999-12-31"),
            DataTypeInfo::new("TIME", "Time", "-838:59:59 to 838:59:59"),
            DataTypeInfo::new("DATETIME", "Date and time", "1000-01-01 00:00:00 to 9999-12-31 23:59:59"),
            DataTypeInfo::new("TIMESTAMP", "Timestamp", "1970-01-01 00:00:01 UTC to 2038-01-19 03:14:07 UTC"),
            DataTypeInfo::new("YEAR", "Year", "1901 to 2155"),
            
            // Binary
            DataTypeInfo::new("BINARY(n)", "Fixed-length binary", "0 to 255 bytes"),
            DataTypeInfo::new("VARBINARY(n)", "Variable-length binary", "0 to 65535 bytes"),
            DataTypeInfo::new("BLOB", "Binary large object", "Up to 65535 bytes"),
            DataTypeInfo::new("MEDIUMBLOB", "Medium BLOB", "Up to 16777215 bytes"),
            DataTypeInfo::new("LONGBLOB", "Long BLOB", "Up to 4294967295 bytes"),
            
            // JSON
            DataTypeInfo::new("JSON", "JSON document", "Up to 1GB"),
            
            // Boolean
            DataTypeInfo::new("BOOLEAN", "Boolean", "Alias for TINYINT(1)"),
        ]
    }
    
    fn sqlite_data_types() -> Vec<DataTypeInfo> {
        vec![
            // Integer
            DataTypeInfo::new("INTEGER", "Integer", "64-bit signed integer"),
            
            // Real
            DataTypeInfo::new("REAL", "Floating point", "64-bit IEEE floating point"),
            
            // Text
            DataTypeInfo::new("TEXT", "Text string", "UTF-8, UTF-16BE or UTF-16LE"),
            
            // Blob
            DataTypeInfo::new("BLOB", "Binary data", "Binary large object"),
            
            // Numeric (affinity)
            DataTypeInfo::new("NUMERIC", "Numeric affinity", "INTEGER or REAL"),
            
            // Common type aliases
            DataTypeInfo::new("VARCHAR(n)", "Maps to TEXT", "Text affinity"),
            DataTypeInfo::new("CHAR(n)", "Maps to TEXT", "Text affinity"),
            DataTypeInfo::new("DATE", "Maps to TEXT or NUMERIC", "ISO8601 string or Julian day"),
            DataTypeInfo::new("DATETIME", "Maps to TEXT or NUMERIC", "ISO8601 string or Julian day"),
            DataTypeInfo::new("BOOLEAN", "Maps to INTEGER", "0 (false) or 1 (true)"),
        ]
    }
}

/// Data type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTypeInfo {
    pub name: String,
    pub description: String,
    pub range: String,
}

impl DataTypeInfo {
    fn new(name: &str, description: &str, range: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            range: range.to_string(),
        }
    }
}