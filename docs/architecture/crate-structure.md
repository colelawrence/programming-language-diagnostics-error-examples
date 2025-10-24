# Crate Structure

## Overview

The project is now organized into three separate crates with clear separation of concerns:

```
wasm-starter/
├── shared-types/       # Types, traits, and infrastructure
├── pathfinder-core/    # Pure business logic
└── src-rust/           # WASM transport layer
    (wasm-pathfinder)
```

## 1. shared-types/ (Infrastructure Crate)

**Purpose**: Defines all shared types and traits for transport-agnostic communication

**Contents**:
- `src/lib.rs` - Type definitions with `#[protocol("wasm")]` macro
  - `Point`, `Edge`, `PathResult`, `ShortestPathParams`
- `src/context.rs` - Session context passed to handlers
- `src/storage.rs` - Storage trait and InMemoryStorage impl
- `src/router/` - Router infrastructure
  - `mod.rs` - Request/Response types, Observer pattern
  - `router_gen.rs` - CallHandler trait, routing logic
- `src/receiver.rs` - Session receiver coordinating requests
- `proc/` - Procedural macro for `#[protocol]`

**Dependencies**: serde, chrono, own proc macro

**Exports**:
```rust
pub trait CallHandler;
pub trait Storage;
pub trait WireResponseSender;
pub struct Context;
pub struct Receiver<H, S>;
pub struct ObserverImpl<T>;
// ... types like Point, PathResult, etc.
```

## 2. pathfinder-core/ (Business Logic Crate)

**Purpose**: Pure pathfinding algorithms, completely transport-agnostic

**Contents**:
- `src/lib.rs` - Core algorithm functions
  - `compute_shortest_path()` - Pure function
  - `euclidean_distance()` - Helper
  - `reconstruct_path()` - Helper
- `src/handler.rs` - CallHandler implementation
  - `PathfinderHandler<S: Storage>` - Handler with optional caching

**Dependencies**: shared-types, petgraph, serde_json

**Exports**:
```rust
pub fn compute_shortest_path(...) -> Result<PathResult, String>;
pub struct PathfinderHandler<S: Storage>;
```

**Key Properties**:
- ✅ No WASM dependencies
- ✅ No network dependencies  
- ✅ Pure algorithms using std only
- ✅ Testable without any transport
- ✅ Can be used in WASM, server, or CLI

## 3. src-rust/ (WASM Transport Crate)

**Purpose**: Thin WASM bindings that use pathfinder-core

**Contents**:
- `lib.rs` - WASM entry points
  - `WasmTransport` - Implements WireResponseSender for JS callbacks
  - `send_request()` - Router-based entry point

**Dependencies**: pathfinder-core, shared-types, wasm-bindgen, serde-wasm-bindgen

**Exports (via wasm-bindgen)**:
```rust
#[wasm_bindgen]
pub fn init_router(...);

#[wasm_bindgen]
pub fn send_request(request: JsValue, callback: Function);

#[wasm_bindgen]
pub fn find_shortest_path(...); // legacy

#[wasm_bindgen]
pub fn find_shortest_path_typed(...); // legacy
```

**Key Properties**:
- ✅ Minimal glue code
- ✅ Uses pathfinder-core for all logic
- ✅ WasmTransport doesn't require unsafe Send/Sync
- ✅ Instantiates Receiver with in-memory storage

## Dependency Graph

```
┌─────────────────┐
│  shared-types   │  (types, traits, router infrastructure)
└────────┬────────┘
         │
         ├──────────────┐
         │              │
┌────────▼────────┐ ┌──▼──────────────┐
│ pathfinder-core │ │  wasm-pathfinder│  (WASM bindings)
│  (algorithms)   │ │  (src-rust/)    │
└─────────────────┘ └─────────────────┘
         │              │
         └──────┬───────┘
                │
         ┌──────▼──────┐
         │ TypeScript  │
         │   App       │
         └─────────────┘
```

## Benefits of This Structure

### 1. **Separation of Concerns**
- Types ≠ Logic ≠ Transport
- Each crate has single responsibility
- Easy to reason about dependencies

### 2. **Reusability**
- `pathfinder-core` can be used in:
  - WASM (current)
  - WebSocket server (future)
  - HTTP server (future)
  - CLI tool (future)
  - Embedded in other Rust apps

### 3. **Testability**
- Core logic tested without WASM runtime
- Handler tested with mock transports
- WASM layer is thin glue (less to test)

### 4. **No Unsafe Code**
- Removed unsafe Send/Sync hacks
- WireResponseSender has no thread requirements
- Clean trait boundaries

### 5. **Future-Ready**
- Add WebSocket transport without touching core
- Add new algorithms in pathfinder-core
- Extend CallHandler with new methods

## Testing

### Rust Tests
```bash
# Test core algorithms
cargo test -p pathfinder-core

# Test shared types
cargo test -p shared-types

# Test WASM (requires wasm-pack)
wasm-pack test --node
```

### TypeScript Tests
```bash
# Test router integration
bun test src/wasmRouter.test.ts

# Test legacy API
bun test src/wasm.test.ts

# All tests
bun test
```

## Future Transports

### WebSocket Server (Example)
```rust
// In a new crate: pathfinder-server
use pathfinder_core::PathfinderHandler;
use shared_types::receiver::Receiver;

struct WebSocketTransport {
    tx: mpsc::Sender<WireResponse>,
}

impl WireResponseSender for WebSocketTransport {
    fn send_response(&self, response: WireResponse) {
        let _ = self.tx.try_send(response);
    }
}

async fn handle_connection(socket: WebSocket) {
    let handler = PathfinderHandler::new(Some(Arc::new(RedisStorage::new())));
    let receiver = Receiver::new(session_id, handler, storage);
    
    // ... handle messages using receiver
}
```

## Workspace Configuration

All crates are in a Cargo workspace:

```toml
[workspace]
members = ["shared-types", "pathfinder-core"]
resolver = "2"

[package]
name = "wasm-pathfinder"
# ... WASM crate at root
```

## Build Commands

```bash
# Check all crates
cargo check --workspace

# Build WASM
mise run wasm:build
# or
wasm-pack build --target web --out-dir pkg

# Run all tests
cargo test --workspace
bun test

# Build everything
mise run build
```

## Summary

✅ **Clean separation**: Types, Logic, Transport
✅ **All tests passing**: 4 Rust + 7 TypeScript = 11 tests
✅ **No unsafe code**: Proper trait boundaries
✅ **Transport-agnostic**: Same logic for WASM/WebSocket/HTTP
✅ **Future-ready**: Easy to add new transports or algorithms
