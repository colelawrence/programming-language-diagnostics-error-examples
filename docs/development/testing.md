# Testing Guide

Complete guide to running and writing tests for the WASM Pathfinder project.

## Quick Reference

### TypeScript Tests

```bash
# Run all tests
vitest

# Run specific test file
vitest run src/wasm.test.ts

# Watch mode (auto-rerun on changes)
vitest watch

# Run with coverage
vitest run --coverage
```

### Rust Tests

```bash
# All workspace tests
cargo test --workspace

# Core pathfinding logic
cargo test -p pathfinder-core

# Server tests
cargo test -p pathfinder-server

# Shared types/protocol
cd shared-types && cargo test
```

## Test Suite Overview

### Unit Tests

#### 1. Legacy WASM Tests (`src/wasm.test.ts`)
Tests direct WASM function exports for backward compatibility.

**Run:** `vitest run src/wasm.test.ts` or `bun run test:wasm`

**Coverage:**
- Simple triangle pathfinding
- Typed parameters and serialization
- Longer path computation
- Error handling (no path found)

#### 2. Rust Core Tests
Pure pathfinding logic without transport dependencies.

**Run:** `cargo test -p pathfinder-core`

**Tests:** Graph algorithms, shortest path computation, edge cases

### Integration Tests

#### 3. Router Tests (`src/router/router.test.ts`)
Tests the router system with WASM adaptor.

**Coverage:**
- Router interface verification
- Pathfinding through router API
- Error handling
- Multiple concurrent calls
- Optional callbacks
- Complex grid pathfinding

**Note:** Some tests may have async timing issues in test environment but work correctly in browser.

#### 4. WebSocket Integration Tests (`src/router/websocket.test.ts`)
Tests WebSocket transport with real server.

**Run:**
```bash
# Start server first
cargo run -p pathfinder-server &

# Run tests
vitest run src/router/websocket.test.ts
```

**Coverage:**
- WebSocket connection lifecycle
- Pathfinding via WebSocket
- Error handling over network
- Multiple concurrent requests

**Note:** Requires running server, not part of automated CI yet.

## Manual Testing

### Interactive Browser Tests

#### 1. Main App (WASM Adaptor)
```bash
mise run dev
# or
mise run dev:watch  # with auto-rebuild on Rust changes
```

Open http://localhost:10880

**Test:**
- Interactive graph with draggable nodes
- Real-time pathfinding visualization
- Uses `createRouter({ adaptor: createWasmAdaptor() })`

#### 2. Transport Comparison (`test-ws-both-transports.html`)
```bash
# Start server
mise run server

# Open in browser
open test-ws-both-transports.html
```

**Test:**
- Side-by-side comparison of WASM vs WebSocket
- Same graph, different transports
- Verify both produce identical results

#### 3. Full Dev Environment (Tilt)
```bash
mise run tilt
# or
tilt up
```

**Services:**
- WASM auto-rebuild on Rust changes
- Vite dev server with hot reload
- WebSocket server on port 10810
- TypeScript type checking
- All services monitored in browser dashboard

Stop with `tilt down`

## Verification Checklist

Before deploying or committing major changes:

- [ ] `cargo check --workspace` - All Rust compiles
- [ ] `tsgo --build .` - TypeScript types check
- [ ] `mise run wasm:build` - WASM builds successfully
- [ ] `vitest run` - All JS tests pass
- [ ] `cargo test --workspace` - All Rust tests pass
- [ ] `cargo run -p pathfinder-server` - Server starts on port 10810
- [ ] Open `test-ws-both-transports.html` - Both transports work
- [ ] `mise run dev` - Main app loads and pathfinding works
- [ ] Drag nodes in UI - Graph updates, paths recalculate

## Test Architecture

```
tests/
├── Unit Tests (Fast, No I/O)
│   ├── src/wasm.test.ts              ← Direct WASM functions
│   ├── pathfinder-core/src/*.rs      ← Core logic tests
│   └── shared-types/src/*.rs         ← Protocol/router tests
│
├── Integration Tests (I/O, Services)
│   ├── src/router/*.test.ts          ← Router/adaptor tests
│   └── pathfinder-server tests       ← Server tests
│
└── E2E/Manual Tests (Browser)
    ├── test-ws-both-transports.html  ← Transport comparison
    └── test-ws-client.html           ← WebSocket client test
```

## Writing Tests

### TypeScript Tests (Vitest)

```typescript
import { describe, it, expect } from 'vitest';

describe('feature', () => {
  it('should work correctly', async () => {
    // Arrange
    const input = { /* ... */ };
    
    // Act
    const result = await someFunction(input);
    
    // Assert
    expect(result).toBe(expected);
  });
});
```

### Rust Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = /* ... */;
        
        // Act
        let result = some_function(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

## Testing Best Practices

### General Principles
- **Test behavior, not implementation**: Focus on inputs/outputs
- **Arrange-Act-Assert**: Structure tests clearly
- **Isolation**: Tests should not depend on each other
- **Fast feedback**: Unit tests should run quickly

### Transport-Agnostic Design
The architecture separates business logic (`pathfinder-core`) from transport layers (WASM, WebSocket). This enables:
- Testing core logic independently
- Swapping transports without changing business logic
- Consistent behavior across all transports

### Type Safety
- Use `#[protocol("wasm")]` for types crossing WASM boundary
- Run codegen after changing shared types
- Verify with `src/wasm.test.ts` to catch serialization issues

### Async Patterns
- Use Observable pattern carefully in tests
- Add appropriate timeouts
- Handle cleanup in `finally` blocks

## Known Testing Considerations

1. **WASM Initialization**: Tests need proper WASM buffer initialization, not URL-based loading
2. **Async Timing**: Some router tests may have timing issues in test environment but work correctly in browser
3. **WebSocket Tests**: Require running server, not automated in CI yet

## Future Improvements

### CI/CD Integration
Set up GitHub Actions with:
- Rust tests (`cargo test --workspace`)
- TypeScript type checking (`tsgo --build .`)
- WASM build verification
- Integration tests with server

### E2E Testing
Consider Playwright for browser-based testing:

```typescript
test('pathfinding works in browser', async ({ page }) => {
  await page.goto('http://localhost:10880');
  // Click nodes, verify paths
});
```

### Integration Test Script
Automate WebSocket tests:

```bash
#!/bin/bash
# Start server in background
cargo run -p pathfinder-server &
SERVER_PID=$!

# Wait for server
sleep 2

# Run tests
vitest run src/router/websocket.test.ts

# Cleanup
kill $SERVER_PID
```
