# SQL Dialect System for DataForge

## Overview

The SQL dialect system provides database-specific SQL generation for PostgreSQL, MySQL, and SQLite. This ensures that DataForge generates correct SQL syntax for each database type, handling their differences in:

- Identifier quoting (double quotes vs backticks)
- Boolean literals (TRUE/FALSE vs 1/0)
- LIMIT/OFFSET syntax
- String concatenation operators
- Date/time literal formatting
- Auto-increment column definitions
- Case-insensitive pattern matching
- Schema support

## Architecture

### Core Components

1. **SqlDialect Trait** (`dialect/mod.rs`): The main trait defining all dialect operations
2. **Database-Specific Implementations**:
   - `PostgreSQLDialect` - PostgreSQL-specific SQL generation
   - `MySQLDialect` - MySQL-specific SQL generation
   - `SQLiteDialect` - SQLite-specific SQL generation

### Integration with Adapters

Each database adapter includes its dialect and exposes it via `get_dialect()`:

```rust
let adapter = create_adapter(DatabaseType::PostgreSQL)?;
let dialect = adapter.get_dialect();
```

## Usage Examples

### Basic Identifier Quoting

```rust
let dialect = create_dialect(DatabaseType::PostgreSQL);
let table = dialect.quote_identifier("users");        // "users"
let column = dialect.quote_identifier("first_name");  // "first_name"

let mysql_dialect = create_dialect(DatabaseType::MySQL);
let table = mysql_dialect.quote_identifier("users");  // `users`
```

### Pagination Queries

```rust
fn build_paginated_query(
    dialect: &dyn SqlDialect,
    table: &str,
    page: usize,
    page_size: usize
) -> String {
    let offset = (page - 1) * page_size;
    format!(
        "SELECT * FROM {}{}",
        dialect.quote_identifier(table),
        dialect.limit_clause(Some(page_size), Some(offset))
    )
}

// PostgreSQL: SELECT * FROM "users" LIMIT 10 OFFSET 20
// MySQL: SELECT * FROM `users` LIMIT 10 OFFSET 20
// SQLite: SELECT * FROM "users" LIMIT 10 OFFSET 20
```

### Boolean Values

```rust
let pg_dialect = PostgreSQLDialect::new();
let query = format!(
    "UPDATE users SET active = {} WHERE id = 1",
    pg_dialect.boolean_literal(true)  // "TRUE"
);

let mysql_dialect = MySQLDialect::new();
let query = format!(
    "UPDATE users SET active = {} WHERE id = 1",
    mysql_dialect.boolean_literal(true)  // "1"
);
```

### Date/Time Handling

```rust
let pg_dialect = PostgreSQLDialect::new();
let query = format!(
    "SELECT * FROM orders WHERE created_at > {}",
    pg_dialect.datetime_literal("2023-01-01 00:00:00")
);
// PostgreSQL: ... WHERE created_at > TIMESTAMP '2023-01-01 00:00:00'

let mysql_dialect = MySQLDialect::new();
let query = format!(
    "SELECT * FROM orders WHERE created_at > {}",
    mysql_dialect.datetime_literal("2023-01-01 00:00:00")
);
// MySQL: ... WHERE created_at > '2023-01-01 00:00:00'
```

### String Concatenation

```rust
// PostgreSQL and SQLite use ||
let pg_dialect = PostgreSQLDialect::new();
let concat = pg_dialect.string_concat("first_name", "last_name");
// Result: first_name || last_name

// MySQL uses CONCAT()
let mysql_dialect = MySQLDialect::new();
let concat = mysql_dialect.string_concat("first_name", "last_name");
// Result: CONCAT(first_name, last_name)
```

### Case-Insensitive Search

```rust
let pg_dialect = PostgreSQLDialect::new();
let query = format!(
    "SELECT * FROM users WHERE email {} 'john%'",
    pg_dialect.case_insensitive_like()  // "ILIKE"
);

let mysql_dialect = MySQLDialect::new();
let query = format!(
    "SELECT * FROM users WHERE email {} 'john%'",
    mysql_dialect.case_insensitive_like()  // "LIKE"
);
```

### Schema-Aware Queries

```rust
let pg_dialect = PostgreSQLDialect::new();
if pg_dialect.supports_schemas() {
    let table = pg_dialect.qualified_table_name(Some("public"), "users");
    // Result: "public"."users"
}

let sqlite_dialect = SQLiteDialect::new();
if !sqlite_dialect.supports_schemas() {
    // SQLite doesn't support schemas, handle accordingly
    let table = sqlite_dialect.qualified_table_name(None, "users");
    // Result: "users"
}
```

### Auto-Increment Columns

```rust
let pg_dialect = PostgreSQLDialect::new();
let ddl = format!(
    "CREATE TABLE users (id {}, name TEXT)",
    pg_dialect.auto_increment_type()  // "SERIAL"
);

let mysql_dialect = MySQLDialect::new();
let ddl = format!(
    "CREATE TABLE users (id {}, name TEXT)",
    mysql_dialect.auto_increment_type()  // "INT AUTO_INCREMENT"
);

let sqlite_dialect = SQLiteDialect::new();
let ddl = format!(
    "CREATE TABLE users (id {}, name TEXT)",
    sqlite_dialect.auto_increment_type()  // "INTEGER PRIMARY KEY AUTOINCREMENT"
);
```

## Feature Support Matrix

| Feature | PostgreSQL | MySQL | SQLite |
|---------|------------|--------|--------|
| Identifier Quotes | `"` | `` ` `` | `"` |
| Boolean Literals | TRUE/FALSE | 1/0 | 1/0 |
| String Concat | `\|\|` | CONCAT() | `\|\|` |
| Case-Insensitive LIKE | ILIKE | LIKE | LIKE |
| RETURNING Clause | ✅ | ❌ | ✅ |
| UPSERT Support | ✅ | ✅ | ✅ |
| Schema Support | ✅ | ✅ | ❌ |
| OFFSET without LIMIT | ✅ | ❌* | ❌** |

\* MySQL requires LIMIT when using OFFSET (uses max value workaround)
\*\* SQLite uses LIMIT -1 for unlimited rows with OFFSET

## Testing

The dialect system includes comprehensive tests:

```bash
# Run all dialect tests
cargo test --package dataforge --lib database::dialect

# Run specific dialect tests
cargo test --package dataforge --lib database::dialect::postgres
cargo test --package dataforge --lib database::dialect::mysql
cargo test --package dataforge --lib database::dialect::sqlite
```

## Future Enhancements

Potential additions to the dialect system:

1. **Window Functions**: ROW_NUMBER(), RANK(), etc. syntax differences
2. **JSON Operations**: Database-specific JSON query syntax
3. **Full-Text Search**: Different FTS implementations
4. **Regular Expressions**: Database-specific regex syntax
5. **Recursive CTEs**: WITH RECURSIVE variations
6. **Array Operations**: PostgreSQL arrays vs JSON arrays
7. **Pivot/Unpivot**: Cross-tab query generation
8. **Materialized Views**: CREATE MATERIALIZED VIEW syntax
9. **Partitioning**: Table partitioning syntax
10. **Custom Data Types**: ENUM, UUID, etc. handling

## Best Practices

1. **Always use the dialect** when generating SQL programmatically
2. **Check feature support** before using database-specific features
3. **Test with all databases** when adding new dialect methods
4. **Document differences** clearly in code comments
5. **Provide fallbacks** for unsupported features