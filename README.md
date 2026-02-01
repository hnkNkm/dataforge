# DataForge

A powerful, cross-platform database client built with Tauri and React, designed for developers who need efficient database management tools.

## Features

### ✅ Implemented
- **Multi-Database Support**: PostgreSQL, MySQL, SQLite
- **Connection Management**: Secure profile storage with OS keychain integration
- **SQL Editor**: Monaco Editor with syntax highlighting and auto-completion
- **Data Explorer**: Browse databases, tables, and columns with metadata
- **Query Execution**: Run SQL queries with real-time results
- **Data Export**: CSV export with Excel compatibility (UTF-8 BOM)
- **Table View**: Sort, paginate, and explore data efficiently
- **Enhanced UI**: Resizable panels, tabs, and dark-ready interface

### 🚧 In Progress
- Query history and favorites
- Dark mode support
- Data editing capabilities
- Advanced import/export (JSON, Excel)

## Tech Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Rust + Tauri 2.0
- **Database**: SQLx 0.8 (async)
- **UI Components**: shadcn/ui + Radix UI
- **State Management**: Zustand 5.0
- **Editor**: Monaco Editor
- **Package Manager**: pnpm (via Nix)

## Development

### Prerequisites
- Nix with flakes enabled
- OR manually install: Node.js 20+, Rust, PostgreSQL, MySQL, SQLite

### Setup
```bash
# Enter development environment
nix develop

# Install dependencies
pnpm install

# Start development server
pnpm tauri dev

# Build for production
pnpm tauri build
```

## Architecture

DataForge uses a clean adapter pattern for database abstraction:

```
Frontend (React)
    ↓
IPC (Tauri Commands)
    ↓
Backend (Rust)
    ↓
Database Adapter Interface
    ↓
Specific Adapters (PostgreSQL, MySQL, SQLite)
```

## Security

- Passwords stored in OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- AES-GCM encryption for sensitive data
- Parameterized queries to prevent SQL injection
- No network exposure - all communication via Tauri IPC

## Known Issues

- [#6](https://github.com/hnkNkm/dataforge/issues/6): React state management with MCP forms
- [#7](https://github.com/hnkNkm/dataforge/issues/7): Dropdown menu interaction issues

## Roadmap

See [docs/ROADMAP.md](docs/ROADMAP.md) for detailed development timeline.

### Recent Milestones
- ✅ Sprint 1-5: Core foundation and SQL editor
- ✅ Sprint 6: Data visualization and export
- ✅ Sprint 7-8: Database explorer enhancements
- 🚧 Sprint 9-10: Extended database support
- 📋 Sprint 11+: Query management and UI polish

## Contributing

This project is in active development. Issues and pull requests are welcome!

## License

MIT