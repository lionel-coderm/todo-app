# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Full desktop dev (Vite dev server on port 1420 + Tauri shell)
pnpm tauri dev

# Frontend only (no desktop shell)
pnpm dev

# TypeScript type-check (also runs as part of `pnpm build`)
npx vue-tsc --noEmit

# Rust backend check
cd src-tauri && cargo check

# Rust tests
cd src-tauri && cargo test

# Production build (all platforms)
pnpm build:desktop
```

No frontend test runner is configured. No ESLint/Prettier config exists. `tsconfig.json` has strict mode with `noUnusedLocals` and `noUnusedParameters`.

## Project Overview

**Todo Studio** - A desktop todo application built with **Tauri v2 + Vue 3**. Supports two runtime-switchable storage backends: JSON file and SQLite. Product identifier: `com.todo.studio`.

## Architecture

### Dual Storage Strategy

The app uses a **Strategy pattern** for storage mutations, selectable at runtime via Settings:

- **JSON mode**: Single `todos.json` file. Frontend owns all data in Pinia store. Reads/writes via `load_app_data`/`save_app_data` (full file replace). Client-side search with fuse.js.
- **SQLite mode**: `todos.db` with r2d2 connection pool (max 8). Individual CRUD commands call specific Rust functions. Server-side SQL search.

The `StorageBackend` trait (`src-tauri/src/storage/mod.rs`) defines the interface. JSON implements `load()`/`save()` only; SQLite overrides all CRUD methods. Default trait methods return errors for unsupported operations.

### Frontend-Backend Communication

`src/services/storageService.ts` wraps all `invoke()` calls to Rust `#[tauri::command]` functions in `src-tauri/src/lib.rs`. Rust serde rename attributes handle camelCase/snake_case mapping between frontend and backend.

### Frontend Store

Single Pinia **setup store** (`src/stores/todo.ts`) using Composition API pattern. Mutation operations delegate to `TodoMutationStrategy` interface (`src/stores/todoMutationStrategy.ts`):

- `createJsonMutationStrategy` - mutates local state, then debounced `persistData()`
- `createSqliteMutationStrategy` - calls Tauri invoke, then updates local state. Uses **optimistic UI** for addTodo (negative temp ID, replaced after server response, rollback on failure)

Startup uses LocalStorage cache (`todo-studio-cache-v1`) for instant paint, then `initialize()` loads from backend.

### Rust Backend Structure

- `lib.rs` - All `#[tauri::command]` functions, Tauri builder, state management (`SqliteBackendCache`, `SettingsCache`)
- `models.rs` - Shared data models with serde rename attrs (`TodoItem`, `CategoryItem`, `CreateTodoInput`, `AppSettings`)
- `ai_report.rs` - AI report generation via OpenAI-compatible API
- `storage/` - `StorageBackend` trait + `JsonStorage` + `SqliteStorage` implementations

## Key Conventions

- All UI text and code comments are in Chinese
- Window starts hidden (`visible: false`), frontend calls `showMainWindow()` after `initialize()` + nextTick for clean first paint
- `TaskPriority` enum: High=1, Medium=2, Low=3
- SQLite schema uses migration system with `PRAGMA user_version`; handles legacy TEXT priority migration
- App version is in `src-tauri/tauri.conf.json` (`version` field) and `package.json`