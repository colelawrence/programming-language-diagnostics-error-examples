# WASM Pathfinder - Transport-Agnostic Architecture

## Overview

This project demonstrates a fully transport-agnostic architecture where the same business logic runs seamlessly via WASM (in-browser) or WebSocket (server-side) with identical APIs on both Rust and TypeScript sides.

## Quick Start

```bash
# Full development environment
mise run tilt

# Or individually:
mise run server  # WebSocket server on :10810
mise run dev     # Vite dev server on :10880
```

## Architecture Highlights

### Rust: Three-Crate Design

```
pathfinder-core/     ← Pure business logic (transport-independent)
shared-types/        ← Protocol & infrastructure
pathfinder-server/   ← WebSocket transport
src-rust/            ← WASM transport
```

**Key Benefits:**
- Same `PathfinderHandler` for WASM and WebSocket
- `CallHandler` trait separates logic from transport
- `WireResponseSender` trait abstracts communication
- Type generation ensures TypeScript/Rust sync

### TypeScript: Adaptor Pattern

```typescript
// WASM (in-browser)
const router = createRouter({
  adaptor: createWasmAdaptor()
});

// WebSocket (server)
const router = createRouter({
  adaptor: createWebSocketAdaptor({
    url: "ws://localhost:10810"
  })
});

// Same API for both!
router.findShortestPath(params).subscribe({
  next: (result) => console.log(result.path),
  error: (err) => console.error(err),
});
```

## Project Structure

```
wasm-starter/
├── pathfinder-core/          # Business logic (Rust)
│   ├── src/handler.rs        # Implements CallHandler
│   └── src/lib.rs            # Pathfinding algorithms
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
├── src/router/               # TypeScript router
│   ├── types.ts              # Interfaces
│   ├── router.ts             # Factory
│   ├── wasmAdaptor.ts        # WASM transport
│   └── websocketAdaptor.ts   # WS transport
│
├── src/                      # React app
│   ├── pathfinder.tsx        # Main UI (uses WASM)
│   └── examples/             # Transport examples
│
└── dist-types/               # Generated TypeScript types
```

## Key Files

### Documentation
- **AGENTS.md** - Development commands and architecture overview
- **docs/features/router-adaptors.md** - Adaptor pattern documentation
- **docs/development/codegen.md** - Type generation guide
- **docs/development/shared-types.md** - Adding routes and methods

### Configuration
- **mise.toml** - Task runner configuration
- **Tiltfile** - Development environment setup
- **Cargo.toml** - Rust workspace configuration
- **tsconfig.json** - TypeScript configuration

### Tests
- **src/wasm.test.ts** - Legacy WASM API tests
- **src/wasmRouter.test.ts** - Router integration tests  
- **src/router/router.test.ts** - Router unit tests
- **src/router/websocket.test.ts** - WebSocket integration tests

### Manual Testing
- **test-ws-client.html** - Basic WebSocket test
- **test-ws-both-transports.html** - Side-by-side comparison

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
```

### Adding a New Method

1. **Define in shared-types:**
   ```rust
   // shared-types/src/lib.rs
   #[protocol("wasm")]
   #[codegen(fn = "new_method() -> NewResult")]
   pub struct NewMethodParams { /* ... */ }
   ```

2. **Implement in core:**
   ```rust
   // pathfinder-core/src/handler.rs
   fn call_new_method(&self, ctx: &Context, params: NewMethodParams, observer: ObserverImpl<NewResult>) {
       // Implementation
   }
   ```

3. **Generate types:**
   ```bash
   cd shared-types
   cargo test --features codegen generate_typescript -- --ignored
   ```

4. **Use in TypeScript:**
   ```typescript
   router.newMethod(params).subscribe({
     next: (result) => console.log(result),
   });
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
- **Best for:** Interactive UIs, offline apps

### WebSocket (Server)
- **Latency:** ~10-50ms (network dependent)
- **Network:** Yes
- **Best for:** Heavy computation, shared state

## Security Considerations

- WASM runs in browser sandbox
- WebSocket server should use TLS (wss://)
- Validate all inputs in CallHandler
- Rate limit WebSocket connections
- Consider authentication/authorization

## Extending

### Add HTTP REST Transport

1. Create `pathfinder-http/` crate
2. Implement `WireResponseSender` for HTTP responses
3. Use same `PathfinderHandler`!

### Add gRPC Transport

1. Create `pathfinder-grpc/` crate
2. Define proto files from shared-types
3. Implement gRPC service with `PathfinderHandler`

### Add New Pathfinding Algorithm

1. Add to `pathfinder-core/src/lib.rs`
2. Add method to `CallHandler` impl
3. Define types in `shared-types/`
4. Generate TypeScript types
5. Use from any transport!

## FAQ

**Q: Why separate core from transports?**  
A: Enables code reuse, testing, and easy transport addition.

**Q: Why Observable pattern instead of async/await?**  
A: Supports streaming responses, better for long-running operations.

**Q: Can I use both transports simultaneously?**  
A: Yes! Create two routers with different adaptors.

**Q: How do I debug WebSocket issues?**  
A: Use `test-ws-client.html` or browser DevTools Network tab.

**Q: Why Tilt for development?**  
A: Manages multiple services (WASM rebuild, Vite, WebSocket server) in one UI.

## Links

- **Main App:** http://localhost:10880
- **WebSocket:** ws://localhost:10810
- **Tilt UI:** http://localhost:10350

## Contributing

1. Check **AGENTS.md** for commands
2. Run `cargo check --workspace` before committing
3. Run `tsgo --build .` for type checking
4. Test manually with `test-ws-both-transports.html`
5. Update docs if adding features

## License

[Your License Here]
