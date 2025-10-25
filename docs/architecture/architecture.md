# Language Editor - Transport-Agnostic Architecture

## Overview

This project demonstrates a fully transport-agnostic architecture where language analysis logic runs seamlessly via WASM (in-browser) or WebSocket (server-side) with identical APIs on both Rust and TypeScript sides.

## Quick Start

```bash
# Full development environment
mise run tilt

# Or individually:
mise run server  # WebSocket server on :10810
mise run dev     # Vite dev server on :10880
```

## Architecture Highlights

### Rust: Four-Crate Design

```
editor-core/         ← Language analysis logic (transport-independent)
pathfinder-core/     ← Legacy pathfinding (kept for reference)
shared-types/        ← Protocol & infrastructure
pathfinder-server/   ← WebSocket transport (serves both handlers)
src-rust/            ← WASM transport
```

**Key Benefits:**
- Same `EditorHandler` for WASM and WebSocket
- `CallHandler` trait separates logic from transport
- `WireResponseSender` trait abstracts communication
- Type generation ensures TypeScript/Rust sync

### TypeScript: Adaptor Pattern

```typescript
// WASM (in-browser)
const router = createRouter({
  adaptor: wasmAdaptor
});

// WebSocket (server)
const router = createRouter({
  adaptor: createWebSocketAdaptor({
    url: "ws://localhost:10810"
  })
});

// Same API for both!
router.analyze_code({ content: "..." }).subscribe({
  next: (result) => displayDiagnostics(result),
  error: (err) => console.error(err),
});
```

## Project Structure

```
editor-project/
├── editor-core/              # Language analysis (Rust)
│   ├── src/handler.rs        # Implements CallHandler
│   └── src/lib.rs            # Analysis algorithms
│
├── shared-types/             # Protocol definitions
│   ├── src/lib.rs            # Types with #[protocol]
│   ├── src/router/           # Router infrastructure
│   ├── src/context.rs        # Request context
│   └── src/storage.rs        # State management
│
├── pathfinder-server/        # WebSocket server
│   ├── src/main.rs           # Tokio async server
│   └── src/transport.rs      # WebSocketSender
│
├── src-rust/                 # WASM transport
│   └── lib.rs                # WASM bindings
│
├── src/editor/               # Editor components
│   ├── EditorComponent.tsx   # Monaco Editor wrapper
│   └── useEditorDiagnostics.ts # Analysis hook
│
├── src/router/               # TypeScript router
│   ├── types.ts              # Interfaces
│   ├── router.ts             # Factory
│   ├── wasmAdaptor.ts        # WASM transport
│   └── websocketAdaptor.ts   # WS transport
│
├── src/                      # React app
│   ├── routes/index.tsx      # Main editor route
│   └── examples/             # Transport examples
│
└── dist-types/               # Generated TypeScript types
```

## Key Files

### Documentation
- **README.md** - Quick start and overview
- **AGENTS.md** - Development commands and architecture
- **IMPLEMENTATION_SUMMARY.md** - Full implementation details
- **docs/architecture/** - Architecture documentation
- **docs/development/** - Development guides
- **docs/features/** - Feature documentation

### Configuration
- **mise.toml** - Task runner configuration
- **Tiltfile** - Development environment setup
- **Cargo.toml** - Rust workspace configuration
- **tsconfig.json** - TypeScript configuration

## Development Workflow

### Commands

```bash
# Development
mise run dev           # Vite only
mise run server        # WebSocket server
mise run tilt          # Full environment

# Building
mise run wasm:build    # Production WASM
mise run build         # Full production build

# Testing
vitest run             # Run all tests
cargo test             # Rust tests
cargo check --workspace  # Check all Rust

# Type Checking
mise run typecheck     # TypeScript
tsgo --build .         # Direct TypeScript check

# Code Generation
cd shared-types
cargo test --features codegen generate_typescript -- --ignored
```

### Adding a New Error Type

1. **Define in shared-types:**
   ```rust
   // shared-types/src/lib.rs
   #[protocol("wasm")]
   pub enum ErrorKind {
       SyntaxError { message: String },
       TypeError { expected: String, found: String },
       YourNewError { details: String }, // ← Add here
   }
   ```

2. **Implement in editor-core:**
   ```rust
   // editor-core/src/handler.rs
   fn analyze_content(content: &str) -> DiagnosticResponse {
       // Add detection logic for your new error
   }
   ```

3. **Generate types:**
   ```bash
   cd shared-types
   cargo test --features codegen generate_typescript -- --ignored
   ```

4. **Use in TypeScript:**
   ```typescript
   // Automatically available in generated types
   if (error.error_kind.type === 'YourNewError') {
       console.log(error.error_kind.details);
   }
   ```

## Design Patterns

### Transport Abstraction
- **Rust:** `WireResponseSender` trait
- **TypeScript:** `Adaptor` interface  
- **Result:** Business logic never knows about transport

### Observable Pattern
- **Streaming:** Methods can emit multiple values
- **Error Handling:** Explicit error channel
- **Completion:** Signals end of stream

### Session Management
- **Receiver:** One per connection/session
- **Context:** Per-request metadata
- **Storage:** Optional state persistence

## Performance

### WASM (In-Browser)
- **Latency:** ~1-5ms
- **Network:** None
- **Best for:** Interactive editing, real-time feedback, offline

### WebSocket (Server)
- **Latency:** ~10-50ms (network dependent)
- **Network:** Yes
- **Best for:** Heavy analysis, shared state, collaboration

## Security Considerations

- WASM runs in browser sandbox
- WebSocket server should use TLS (wss://)
- Validate all inputs in CallHandler
- Rate limit WebSocket connections
- Consider authentication/authorization

## Extending

### Add New Language Feature

1. Extend `ErrorKind` in `shared-types/src/lib.rs`
2. Add detection in `editor-core/src/handler.rs`
3. Run codegen to update TypeScript types
4. UI automatically gets new error types!

### Add HTTP REST Transport

1. Create `editor-http/` crate
2. Implement `WireResponseSender` for HTTP responses
3. Use same `EditorHandler`!

### Add Real Parser

Replace simple lexical analysis in `editor-core/src/handler.rs` with:
- Tree-sitter for production parsing
- Custom parser for your language
- Integration with existing compiler

## FAQ

**Q: Why separate core from transports?**  
A: Enables code reuse, testing, and easy transport addition.

**Q: Why Observable pattern instead of async/await?**  
A: Supports streaming responses, better for long-running operations.

**Q: Can I use both transports simultaneously?**  
A: Yes! Create two routers with different adaptors.

**Q: How do I add syntax highlighting?**  
A: Configure Monaco Editor's language definitions in the editor component.

**Q: Why Monaco Editor instead of CodeMirror?**  
A: Monaco is the actual VS Code editor - full-featured and well-maintained.

## Links

- **Main Editor:** http://localhost:10880
- **WebSocket:** ws://localhost:10810
- **Tilt UI:** http://localhost:10350

## Contributing

1. Check **AGENTS.md** for commands
2. Run `cargo check --workspace` before committing
3. Run `tsgo --build .` for type checking
4. Test with both WASM and WebSocket transports
5. Update docs if adding features
