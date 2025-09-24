# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DataForge is a Tauri-based desktop database client application combining a React TypeScript frontend with a Rust backend.

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

## Architecture

### Technology Stack
- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Rust with Tauri 2.0
- **Package Manager**: pnpm (via Nix)
- **Development Environment**: Nix Flakes

### Project Structure
- `/src/` - React TypeScript frontend
  - `main.tsx` - React entry point
  - `App.tsx` - Main component with Tauri integration
- `/src-tauri/` - Rust backend
  - `src/lib.rs` - Tauri commands and core logic
  - `src/main.rs` - Application entry point
  - `tauri.conf.json` - Tauri configuration

### Frontend-Backend Communication
- Frontend uses `@tauri-apps/api` to invoke Rust commands
- Example: `await invoke("greet", { name: "World" })`
- Commands are defined in `src-tauri/src/lib.rs` with `#[tauri::command]`

## Key Configuration

### Tauri Settings
- App ID: `com.hnk.dataforge`
- Dev server: `http://localhost:1420`
- Window: 800x600px default
- Bundle targets: All platforms

### TypeScript Configuration
- Target: ES2020
- Strict mode enabled
- Module: ESNext with bundler resolution

## Development Notes

- Always use `nix develop` to ensure correct environment
- The project uses Nix Flakes for reproducible builds
- Vite is configured to ignore `src-tauri` for file watching
- Frontend and backend run concurrently during development