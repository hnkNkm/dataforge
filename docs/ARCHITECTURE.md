# DataForge 技術アーキテクチャ

## システム構成

```
┌─────────────────────────────────────────────────────┐
│                    Frontend (UI)                     │
│                 React + TypeScript                   │
│              Vite + TailwindCSS + Zustand          │
└─────────────────────────────────────────────────────┘
                           ↕
                    IPC (Tauri API)
                           ↕
┌─────────────────────────────────────────────────────┐
│                   Backend (Core)                     │
│                       Rust                           │
│         Tauri + SQLx + tokio + serde               │
└─────────────────────────────────────────────────────┘
                           ↕
┌─────────────────────────────────────────────────────┐
│                    Databases                         │
│     PostgreSQL | MySQL | SQLite | MongoDB | etc     │
└─────────────────────────────────────────────────────┘
```

## レイヤーアーキテクチャ

### 1. プレゼンテーション層 (Frontend)

#### 技術スタック
- **Framework**: React 19
- **Language**: TypeScript 5.x
- **Build Tool**: Vite
- **Styling**: TailwindCSS + shadcn/ui
- **State Management**: Zustand
- **Router**: TanStack Router
- **Data Fetching**: TanStack Query
- **Form Handling**: React Hook Form + Zod
- **Icons**: Lucide React
- **Code Editor**: Monaco Editor

#### ディレクトリ構造
```
src/
├── components/          # UIコンポーネント
│   ├── common/         # 共通コンポーネント
│   ├── editor/         # SQLエディタ関連
│   ├── explorer/       # DBエクスプローラー
│   ├── viewer/         # データビューア
│   └── ui/            # 基本UIコンポーネント
├── pages/             # ページコンポーネント
├── hooks/             # カスタムフック
├── stores/            # Zustand stores
├── services/          # Tauri API呼び出し
├── types/             # TypeScript型定義
├── utils/             # ユーティリティ関数
└── styles/            # グローバルスタイル
```

### 2. ビジネスロジック層 (Backend)

#### 技術スタック
- **Language**: Rust
- **Framework**: Tauri 2.0
- **Async Runtime**: tokio
- **Database Drivers**:
  - PostgreSQL/MySQL: sqlx
  - SQLite: sqlx with sqlite feature
  - MongoDB: mongodb driver
- **Serialization**: serde + serde_json
- **Error Handling**: thiserror + anyhow
- **Logging**: tracing + tracing-subscriber
- **Encryption**: ring (for password encryption)

#### モジュール構成
```
src-tauri/src/
├── commands/          # Tauriコマンド
│   ├── connection.rs  # 接続管理
│   ├── query.rs      # クエリ実行
│   ├── metadata.rs   # メタデータ取得
│   └── export.rs     # エクスポート機能
├── database/         # データベースアダプター
│   ├── adapter.rs    # 共通インターフェース
│   ├── postgres.rs   # PostgreSQL実装
│   ├── mysql.rs      # MySQL実装
│   ├── sqlite.rs     # SQLite実装
│   └── pool.rs       # コネクションプール
├── models/           # データモデル
├── services/         # ビジネスロジック
├── utils/            # ユーティリティ
└── error.rs          # エラー定義
```

### 3. データアクセス層

#### データベースアダプターパターン
```rust
trait DatabaseAdapter {
    async fn connect(&self, config: ConnectionConfig) -> Result<Connection>;
    async fn execute(&self, query: &str) -> Result<QueryResult>;
    async fn get_metadata(&self) -> Result<DatabaseMetadata>;
    // ...
}
```

#### コネクションプール
- 各データベースタイプごとに専用プール
- 最大接続数管理
- アイドルタイムアウト
- 自動再接続

## セキュリティアーキテクチャ

### 認証情報管理
```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Frontend   │────▶│   Tauri API  │────▶│  OS Keychain │
│              │     │              │     │   (keyring)  │
└──────────────┘     └──────────────┘     └──────────────┘
```

- パスワードはOS標準のキーチェーンに保存
  - macOS: Keychain
  - Linux: Secret Service API
  - Windows: Windows Credential Manager
- 通信は全てIPC経由（ネットワーク非公開）
- SQLインジェクション対策（パラメータバインディング）

## パフォーマンス最適化

### フロントエンド
- **コード分割**: 動的インポート
- **仮想スクロール**: 大量データ表示
- **メモ化**: React.memo, useMemo
- **Web Worker**: 重い処理の分離

### バックエンド
- **非同期処理**: tokio による並行実行
- **ストリーミング**: 大量データの段階的取得
- **キャッシング**:
  - メタデータキャッシュ
  - クエリ結果キャッシュ（LRU）
- **バッチ処理**: 複数クエリの効率的実行

## データフロー

### クエリ実行フロー
```
1. User Input (SQL)
     ↓
2. Frontend Validation
     ↓
3. IPC Call (invoke)
     ↓
4. Command Handler
     ↓
5. Database Adapter
     ↓
6. Connection Pool
     ↓
7. Database Execute
     ↓
8. Result Streaming
     ↓
9. Data Transformation
     ↓
10. IPC Response
     ↓
11. UI Update
```

### 状態管理フロー
```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  UI Events  │────▶│   Zustand   │────▶│  React UI   │
└─────────────┘     │    Store    │     └─────────────┘
                    └─────────────┘
                           ↕
                    ┌─────────────┐
                    │  Tauri IPC  │
                    └─────────────┘
```

## エラーハンドリング

### エラー階層
1. **UI層**: ユーザーフレンドリーなメッセージ
2. **API層**: 構造化エラーレスポンス
3. **ビジネス層**: ドメイン固有エラー
4. **データ層**: データベースエラー

### エラーリカバリ
- 自動リトライ（接続エラー）
- フォールバック処理
- エラーログ記録
- ユーザー通知

## 開発環境

### 必要ツール
- Node.js 20+
- Rust 1.75+
- pnpm 8+
- Nix (開発環境管理)

### ビルドパイプライン
```
Development:
  pnpm tauri dev
    ├── Vite Dev Server (Frontend)
    └── Cargo Watch (Backend)

Production:
  pnpm tauri build
    ├── Vite Build → dist/
    └── Cargo Build → target/release/
         └── Bundle → installers/
```

## テスト戦略

### ユニットテスト
- Frontend: Vitest
- Backend: Rust built-in test

### 統合テスト
- Tauri API テスト
- データベース接続テスト

### E2Eテスト
- WebDriver (Tauri Driver)
- 主要ユースケースの自動テスト