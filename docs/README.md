# Documentation Index

## Quick Links

- **[Main README](../README.md)** - Project overview and quick start
- **[AGENTS.md](../AGENTS.md)** - Commands reference and architecture guide
- **[IMPLEMENTATION_SUMMARY.md](../IMPLEMENTATION_SUMMARY.md)** - Complete implementation details

## Documentation Structure

### Architecture
- **[architecture.md](architecture/architecture.md)** - System design and transport-agnostic architecture
- **[crate-structure.md](architecture/crate-structure.md)** - Rust crate organization (legacy pathfinding docs)
- **[transport-agnostic-design.md](architecture/transport-agnostic-design.md)** - Transport abstraction patterns

### Development
- **[codegen.md](development/codegen.md)** - Type generation from Rust to TypeScript
- **[adding-routes.md](development/adding-routes.md)** - Adding new router methods
- **[adding-error-types.md](development/adding-error-types.md)** - Extending the error system
- **[testing.md](development/testing.md)** - Testing strategies

### Features
- **[editor-diagnostics.md](features/editor-diagnostics.md)** - Language diagnostics system
- **[router-adaptors.md](features/router-adaptors.md)** - Transport abstraction patterns
- **[transport-toggle.md](features/transport-toggle.md)** - Switching between WASM and WebSocket

### Proposals
- **[rpc-handles.md](proposals/rpc-handles.md)** - Design proposals and RFCs

## Getting Started

### For Developers

1. Read [Quick Start](../README.md#quick-start)
2. Review [AGENTS.md](../AGENTS.md) for commands
3. Read [Architecture](architecture/architecture.md) for system design
4. Check [Codegen Guide](development/codegen.md) for type generation

### For Contributors

1. Read [Adding Error Types](development/adding-error-types.md)
2. Review [Adding Routes](development/adding-routes.md)
3. Check [Testing Guide](development/testing.md)
4. Follow code style in [AGENTS.md](../AGENTS.md#code-style)

### For Architects

1. Review [Architecture](architecture/architecture.md)
2. Study [Transport Agnostic Design](architecture/transport-agnostic-design.md)
3. Check [Router Adaptors](features/router-adaptors.md)
4. Read [Implementation Summary](../IMPLEMENTATION_SUMMARY.md)

## Key Concepts

### Transport Agnostic
Business logic (editor-core) runs on both WASM and WebSocket with zero changes. See [architecture.md](architecture/architecture.md).

### Type Generation
Rust types automatically generate TypeScript types via codegen. See [codegen.md](development/codegen.md).

### Error Protocol
Structured errors with spans for precise highlighting. See [editor-diagnostics.md](features/editor-diagnostics.md).

### Observable Pattern
Streaming responses from Rust to TypeScript. See [router-adaptors.md](features/router-adaptors.md).

## Common Tasks

### Add a New Error Type
→ [Adding Error Types Guide](development/adding-error-types.md)

### Add a New Router Method
→ [Adding Routes Guide](development/adding-routes.md)

### Update TypeScript Types from Rust
```bash
cd shared-types
cargo test --features codegen generate_typescript -- --ignored
```

### Switch Transports
See [Transport Toggle](features/transport-toggle.md)

## Legacy Documentation

Some documentation references the original pathfinding application:
- `architecture/crate-structure.md` - Contains pathfinding examples (patterns still valid)
- Examples in `src/examples/` demonstrate pathfinding with same architecture

The architecture patterns remain identical - only the business logic changed from pathfinding to language analysis.

## Contributing

When updating documentation:
1. Keep docs in sync with code
2. Update IMPLEMENTATION_SUMMARY.md for major changes
3. Add examples and code snippets
4. Link between related docs
5. Update this index when adding new docs

