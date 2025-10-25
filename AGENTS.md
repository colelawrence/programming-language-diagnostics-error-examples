# Agent Guide for Language Editor

## Commands
- **Dev (recommended)**: `mise run dev:watch` - Vite + auto-rebuild WASM on Rust changes (one command)
- **Dev (Tilt, best DX)**: `mise run tilt` or `tilt up` - Full dev environment with WebSocket server, WASM auto-rebuild, and type checking; `tilt down` to stop
- **Dev (manual)**: `mise run dev` - Vite only (requires manual WASM rebuild on Rust changes)
- **Server**: `mise run server` - Run WebSocket server on port 10810
- **Build**: `mise run build` or `bun run build` - Production build (WASM + Vite)
- **Type check**: `mise run typecheck` or `tsgo --build .` - Check TypeScript types
- **WASM build**: `mise run wasm:build` (prod) or `mise run wasm:dev` (dev)
- **WASM watch**: `mise run wasm:watch` - Auto-rebuild WASM on Rust changes (run alongside `mise run dev`)
- **Codegen**: Run from `shared-types/` directory: `cargo test --features codegen generate_typescript -- --ignored` - Generate TS types from Rust
- **Tests**: 
  - Rust: `cargo test` (in root or shared-types directory)
  - WASM e2e: `vitest run src/wasm.test.ts` or `bun run test:wasm`
  - All JS tests: `vitest` or `bun run test`
  - Transport comparison: Open `test-ws-both-transports.html` in browser (requires WebSocket server running)
- **Cargo check**: 
  - All: `cargo check --workspace`
  - WASM: `cargo check`
  - Server: `cargo check -p pathfinder-server`
  - Editor: `cargo check -p editor-core`
  - Pathfinder (legacy): `cargo check -p pathfinder-core`

## Architecture

### Crate Structure (Transport-Agnostic Design)
- **`editor-core/`** - Language analysis logic, no transport dependencies
  - Implements `CallHandler` trait from `shared-types`
  - Analyzes code and generates structured diagnostics
  - Works with any transport (WASM, WebSocket, HTTP, etc.)
- **`pathfinder-core/`** - Legacy pathfinding logic (kept for reference)
  - Implements `CallHandler` trait from `shared-types`
  - Uses `petgraph` for shortest path algorithms
- **`shared-types/`** - Protocol definitions and router infrastructure
  - Types with `#[protocol("wasm")]` - shared between Rust and TypeScript
  - Router types with `#[protocol("router")]` - request/response enums
  - `CallHandler` trait - business logic interface
  - `WireResponseSender` trait - transport abstraction
  - `Receiver` - session handler (instantiated per connection)
  - `Context` and `Storage` - session state management
  - Auto-generates TypeScript to `dist-types/` via codegen
- **`src-rust/`** (WASM transport) - WASM-specific transport layer
  - Implements `WireResponseSender` for WASM callbacks
  - Exports `send_request()` to JavaScript
  - Uses `editor-core::EditorHandler` for business logic
- **`pathfinder-server/`** - WebSocket transport server
  - Implements `WireResponseSender` for WebSocket connections
  - Uses `editor-core::EditorHandler` (same handler as WASM!)
  - Runs on port 10810, handles concurrent connections
  - Each connection gets its own `Receiver` instance

### TypeScript Router System (Transport-Agnostic)
- **`src/router/`** - Adaptor-based router implementation
  - `types.ts` - Core interfaces (Adaptor, Router, Observable)
  - `router.ts` - Unified router factory (`createRouter`)
  - `wasmAdaptor.ts` - In-browser WASM transport (no network)
  - `websocketAdaptor.ts` - Server-side WebSocket transport
  - Application code uses same API regardless of transport
- **Frontend**: React 19 + Vite + TanStack Router in `src/`
  - `src/editor/` - Monaco Editor components with diagnostics
  - `src/routes/index.tsx` - Main editor route using WASM adaptor
  - `src/examples/` - Transport examples and legacy pathfinding demos
- **Codegen**: `derive-codegen` auto-generates TypeScript types from Rust structs to `dist-types/`
- **Livestore**: State management with `@livestore/livestore` in `src/livestore/`
- **Build**: Vite bundles to `build/`; SPA mode; WASM built separately by wasm-pack, Vite auto-reloads on pkg/ changes
- **Testing**: End-to-end tests in `src/wasm.test.ts` verify serialization and type safety

## Code Style
- **TypeScript**: Strict mode, `verbatimModuleSyntax`, `exactOptionalPropertyTypes`; path alias `#src/*` → `src/*`
- **Module system**: ESNext with `module: "Preserve"`, no emitting; `allowImportingTsExtensions` enabled
- **Rust**: Edition 2021, serde for serialization, `wasm-bindgen` for JS interop
- **Build targets**: Chrome 115+, Firefox 120+, Safari 16+
- **Error handling**: Rust functions return `Result<JsValue, JsValue>` for WASM interop
- **Type definitions**: Use `shared-types` crate with `#[protocol("wasm")]` for types crossing WASM boundary
- **Codegen workflow**: Define types in Rust → run codegen → use generated TS types → verify with tests
- **Linting**: Biome configured to ignore `cg-types/**`, `generators/**`, and generated files

## UI Style Guidelines (Tailwind v4)
- **Design System**: Terminal-inspired interface with light/dark theme toggle defined in `src/styles.css`
- **Theme Management**: Use `useTheme` hook from `src/useTheme.ts` for theme state and toggle function
  - Theme persists to localStorage
  - Default is dark mode
  - Toggle button in header switches between light and dark
- **Typography**: System monospace fonts everywhere (`font-mono` is default)
- **Text Sizing**: Uniform base size (15px) with minimal hierarchy; use `text-label` (12px) only for graph labels
- **Color Palette**: Terminal colors that adapt to theme (semantic names automatically switch)
  - Backgrounds: `bg-background`, `bg-surface`, `bg-surface-elevated`
  - Text: `text-text`, `text-text-secondary`, `text-text-tertiary`
  - Accents: `text-primary` (green), `text-accent` (cyan), `text-warning` (yellow), `text-error` (red)
  - Borders: `border-border`, `border-border-focus`
- **Theme Implementation**: Use `data-theme="light"` attribute on `:root` to switch themes (managed by useTheme hook)
- **Monaco Editor**: Use CSS variables for theming to match the terminal aesthetic
- **Custom Colors**: Add new colors to `:root` and `:root[data-theme="light"]` in `src/styles.css` using `--color-*` namespace
- **Reference**: See `src/styles.css` for complete color definitions and design philosophy

## Editor Features
- **Monaco Editor Integration**: Full VS Code editor with TypeScript/JavaScript language support
- **Real-time Diagnostics**: Debounced code analysis (500ms) with span-based error highlighting
- **Error Protocol**: Structured errors with `ErrorKind` enum (SyntaxError, TypeError, UndefinedVariable, etc.)
- **Diagnostics Panel**: Terminal-themed panel showing all errors with line/column positions
- **Transport Switching**: Same editor works with WASM or WebSocket backend
