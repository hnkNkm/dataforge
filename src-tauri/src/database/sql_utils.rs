use sqlparser::dialect::{Dialect, PostgreSqlDialect, MySqlDialect, SQLiteDialect};
use sqlparser::parser::Parser;

/// SQL文を分割して返す
pub fn split_sql_statements(sql: &str, database_type: &super::adapter::DatabaseType) -> Result<Vec<String>, String> {
    let dialect = get_dialect(database_type);

    match Parser::parse_sql(&*dialect, sql) {
        Ok(statements) => {
            // パーサーが正常に分割できた場合
            Ok(statements.iter().map(|stmt| stmt.to_string()).collect())
        }
        Err(_) => {
            // パーサーが失敗した場合は、セミコロンで分割（フォールバック）
            Ok(split_by_semicolon(sql))
        }
    }
}

/// データベースタイプに応じたDialectを取得
fn get_dialect(database_type: &super::adapter::DatabaseType) -> Box<dyn Dialect> {
    match database_type {
        super::adapter::DatabaseType::PostgreSQL => Box::new(PostgreSqlDialect {}),
        super::adapter::DatabaseType::MySQL => Box::new(MySqlDialect {}),
        super::adapter::DatabaseType::SQLite => Box::new(SQLiteDialect {}),
    }
}

/// セミコロンでSQL文を分割（フォールバック用）
fn split_by_semicolon(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut string_char = ' ';
    let mut escape_next = false;

    for ch in sql.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }

        if ch == '\\' && in_string {
            escape_next = true;
            current.push(ch);
            continue;
        }

        if !in_string && (ch == '\'' || ch == '"') {
            in_string = true;
            string_char = ch;
            current.push(ch);
        } else if in_string && ch == string_char {
            in_string = false;
            current.push(ch);
        } else if !in_string && ch == ';' {
            // セミコロンを含めて文を追加
            current.push(ch);
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                statements.push(trimmed.to_string());
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }

    // 最後の文を追加
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        statements.push(trimmed.to_string());
    }

    statements
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::adapter::DatabaseType;

    #[test]
    fn test_split_simple_statements() {
        let sql = "SELECT * FROM users; INSERT INTO users (name) VALUES ('test');";
        let statements = split_sql_statements(sql, &DatabaseType::PostgreSQL).unwrap();
        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn test_split_with_string_containing_semicolon() {
        let sql = "SELECT 'hello; world' FROM test; SELECT 2";
        let statements = split_sql_statements(sql, &DatabaseType::PostgreSQL).unwrap();
        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn test_single_statement() {
        let sql = "SELECT * FROM users WHERE id = 1";
        let statements = split_sql_statements(sql, &DatabaseType::PostgreSQL).unwrap();
        assert_eq!(statements.len(), 1);
    }
}