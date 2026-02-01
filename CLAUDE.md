# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DataForge is a Tauri-based desktop database client application combining a React TypeScript frontend with a Rust backend. It supports PostgreSQL, MySQL, and SQLite databases with a clean adapter pattern architecture.

## Development Commands

### Setup and Development
```bash
# Enter Nix development environment (required)
nix develop

# Install dependencies
pnpm install

# Start development server with hot reload
pnpm tauri dev

# Build for production
pnpm tauri build
```

### Frontend-only Commands
```bash
pnpm dev      # Start Vite dev server
pnpm build    # TypeScript + Vite build
pnpm preview  # Preview built app
```

### Testing and Quality
```bash
# No test commands currently configured - check package.json for updates
# No linting commands currently configured - check package.json for updates
```

## Architecture

### Technology Stack
- **Frontend**: React 19 + TypeScript + Vite + TailwindCSS + shadcn/ui
- **State Management**: Zustand 5.0
- **Backend**: Rust with Tauri 2.0
- **Database Support**: SQLx 0.8 (PostgreSQL, MySQL, SQLite)
- **Package Manager**: pnpm (via Nix)
- **Development Environment**: Nix Flakes

### Project Structure
- `/src/` - React TypeScript frontend
  - `components/` - UI components including MonacoQueryEditor, DatabaseExplorer, TableView
  - `stores/` - Zustand state management
  - `types/` - TypeScript type definitions
  - `layouts/` - Application layout components
- `/src-tauri/` - Rust backend
  - `src/lib.rs` - Tauri commands and core logic
  - `src/database/` - Database adapter pattern implementation
    - `adapter/` - Database-specific implementations (postgres.rs, mysql.rs, sqlite.rs)
    - `dialect/` - SQL dialect implementations for each database
    - `capabilities.rs` - Database feature detection
    - `templates.rs` - Common SQL operation templates
    - `connection.rs` - Connection management
    - `config.rs` - Configuration handling
  - `tauri.conf.json` - Tauri configuration
- `/docs/` - Architecture and roadmap documentation

### Frontend-Backend Communication
- Frontend uses `@tauri-apps/api/core` to invoke Rust commands
- Example: `await invoke("execute_query", { connectionId, query })`
- Commands are defined in `src-tauri/src/lib.rs` with `#[tauri::command]`
- All communication happens via IPC (no network exposure)

### Database Adapter Pattern
The backend implements a trait-based adapter pattern for multi-database support:
- Common `DatabaseAdapter` trait in `src-tauri/src/database/adapter/mod.rs`
- Specific implementations for each database type (PostgreSQL, MySQL, SQLite)
- `SqlDialect` trait for database-specific SQL generation
- `DatabaseCapabilities` for feature detection per database
- `QueryTemplates` for common SQL operation templates
- Connection profiles stored with OS keychain integration for security

## Key Dependencies

### Frontend
- Monaco Editor for SQL editing with syntax highlighting
- Radix UI components via shadcn/ui
- react-resizable-panels for layout management
- lucide-react for icons

### Backend
- SQLx for async database operations
- keyring for OS keychain integration
- aes-gcm for encryption
- sqlparser for SQL parsing and validation
- tokio for async runtime

## Security Considerations
- Passwords stored in OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- AES-GCM encryption for sensitive data
- Parameterized queries to prevent SQL injection
- No network exposure - all via Tauri IPC

## Development Notes

- Always use `nix develop` to ensure correct environment with PostgreSQL 16, MySQL 8.0, and SQLite
- The project uses Nix Flakes for reproducible builds
- Vite is configured to ignore `src-tauri` for file watching
- Frontend and backend run concurrently during development
- Current implementation includes query execution, database exploration, and connection management
- Query history, dark mode, and data editing features are planned but not yet implemented

## ブランチ戦略

### 基本ルール
- `main`: 本番環境相当の安定版
- `feature/*`: 機能開発ブランチ（Sprint単位）
- `feature/*/sub-*`: 作業単位の細分化ブランチ

### 作業フロー
1. Sprint開始時に`feature/`ブランチを作成
2. 各作業単位で`sub-`ブランチを作成
3. 小さな単位で実装・テスト・コミット
4. `sub-`ブランチを`feature`ブランチにマージ
5. Sprint完了後、`feature`ブランチを`main`にマージ

### コミットルール
- 作業単位ごとに意味のあるコミット
- プレフィックス使用：
  - `feat:` 新機能
  - `fix:` バグ修正
  - `docs:` ドキュメント
  - `refactor:` リファクタリング
  - `test:` テスト
  - `chore:` その他の変更
- 日本語コミットメッセージOK

### 例
```bash
# Sprint用のfeatureブランチ作成
git checkout -b feature/postgres-connection

# 作業単位のsub-ブランチ作成
git checkout -b feature/postgres-connection/sub-rust-deps

# 実装・コミット
git add .
git commit -m "feat: add database dependencies to Cargo.toml"

# featureブランチにマージ
git checkout feature/postgres-connection
git merge --no-ff feature/postgres-connection/sub-rust-deps

# 完了したらsub-ブランチを削除
git branch -d feature/postgres-connection/sub-rust-deps
```