# Language Editor with Rust-Powered Diagnostics

A Monaco Editor-based code editor with real-time language diagnostics powered by Rust, demonstrating a transport-agnostic architecture where the same analysis logic runs via WASM (in-browser) or WebSocket (server-side).

## Quick Start

1. Install `mise`:
```bash
curl https://mise.run | sh
# or: brew install mise
```

2. Trust and install:
```bash
mise trust
mise install    # Installs Rust, Bun, Tilt, wasm-pack, etc.
```

3. Start development:
```bash
mise run tilt
```

That's it! Visit http://localhost:10880 to start editing with live error detection.

## Development Commands

```bash
# Start full dev environment (recommended)
mise run tilt         # or: tilt up
mise run tilt:down    # Stop all resources

# Manual development (if not using Tilt)
mise run dev          # Vite dev server only
mise run server       # WebSocket server only (pathfinder-server crate)
mise run wasm:watch   # Auto-rebuild WASM on changes

# Building
mise run build        # Production build
mise run wasm:build   # Build WASM only

# Type checking & testing
mise run typecheck    # Check TypeScript types
cargo test            # Run Rust tests
bun test              # Run TypeScript tests

# Code generation (after changing Rust types)
cd shared-types && cargo test --features codegen generate_typescript -- --ignored
```

## Project Structure

- `editor-core/` - Language analysis logic (transport-agnostic Rust)
- `src-rust/` - Rust WASM transport layer
- `pathfinder-server/` _(legacy crate name)_ - WebSocket transport server for editor diagnostics (uses `editor-core`)
- `shared-types/` - Protocol definitions and router infrastructure
- `src/` - React/TypeScript frontend
  - `src/editor/` - Monaco Editor components and diagnostics
  - `src/router/` - Transport-agnostic router with WASM and WebSocket adaptors
- `pkg/` - Generated WASM module (auto-generated)
- `dist-types/` - Auto-generated TypeScript types from Rust
- `docs/` - Project documentation
- `pathfinder-core/` _(legacy)_ - Legacy pathfinding demo logic (kept for reference/examples)

## Features

### Editor
- **Monaco Editor** - Full VS Code editor in the browser
- **Real-time Analysis** - Debounced code analysis as you type
- **Error Highlighting** - Span-based error underlining with hover messages
- **Diagnostics Panel** - Detailed error list with line/column positions
- **Theme Support** - Light/dark mode with terminal aesthetic

### Language Diagnostics
- Syntax errors (unbalanced braces, etc.)
- Type errors (type mismatches)
- Undefined variable detection
- Duplicate definitions
- TODO comment warnings
- **Extensible** - Easy to add new error types

### Architecture
- **Transport-Agnostic** - Same analysis logic for WASM and WebSocket
- **Type-Safe** - Rust types auto-generate TypeScript types
- **Span-Ready** - Full line/column information for error highlighting
- **Observable Pattern** - Streaming responses from Rust

## How It Works

1. You type code in the Monaco Editor
2. After 500ms debounce, content is sent to Rust for analysis
3. Rust parser analyzes code and generates structured diagnostics
4. Errors with source code spans are returned
5. Editor displays inline decorations and diagnostics panel
6. All via WASM (no network) or WebSocket (server-side) - same API!
