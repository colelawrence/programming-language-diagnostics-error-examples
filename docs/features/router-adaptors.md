# Router Adaptors: Transport-Agnostic TypeScript Client

## Overview

The router system provides a transport-agnostic way to communicate with the pathfinder backend. You can seamlessly switch between WASM (in-browser) and WebSocket (server-side) transports using adaptors.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Application Code                      │
│              (React components, etc.)                    │
└─────────────────────┬───────────────────────────────────┘
                      │
                      │ Uses Router interface
                      ▼
          ┌───────────────────────┐
          │   createRouter()      │  ← Unified API
          │                       │
          │  - findShortestPath() │
          │  - dispose()          │
          └───────────┬───────────┘
                      │
        ┌─────────────┴─────────────┐
        │                           │
        │  Adaptor Interface        │
        │  - init()                 │
        │  - sendRequest()          │
        │  - dispose()              │
        │                           │
        └─────────────┬─────────────┘
                      │
         ┌────────────┴────────────┐
         │                         │
         ▼                         ▼
┌────────────────┐        ┌────────────────┐
│ WASM Adaptor   │        │  WS Adaptor    │
│                │        │                │
│ - In-browser   │        │ - Server-side  │
│ - No network   │        │ - Network call │
│ - WASM module  │        │ - ws:// proto  │
└────────────────┘        └────────────────┘
```

## Usage

### WASM Transport (Default)

```typescript
import { createRouter, createWasmAdaptor } from "./router";

const router = createRouter({
  adaptor: createWasmAdaptor(),
});

router.findShortestPath(params).subscribe({
  next: (result) => console.log("Path:", result.path),
  error: (error) => console.error("Error:", error),
  complete: () => console.log("Done"),
});
```

### WebSocket Transport

```typescript
import { createRouter, createWebSocketAdaptor } from "./router";

const router = createRouter({
  adaptor: createWebSocketAdaptor({
    url: "ws://localhost:10810",
    autoReconnect: true,
    reconnectDelay: 1000,
  }),
});

router.findShortestPath(params).subscribe({
  next: (result) => console.log("Path:", result.path),
  error: (error) => console.error("Error:", error),
  complete: () => console.log("Done"),
});

// Clean up when done
router.dispose();
```

## Benefits

1. **Transport Independence**: Application code doesn't need to know which transport is used
2. **Easy Testing**: Switch to WASM for unit tests, WebSocket for integration tests
3. **Flexibility**: Add new transports (HTTP, gRPC, etc.) without changing application code
4. **Type Safety**: Fully typed with TypeScript, shared types generated from Rust

## Creating Custom Adaptors

```typescript
import type { Adaptor, RequestEnum, WireResponse } from "./router/types";

export function createCustomAdaptor(): Adaptor {
  return {
    async init() {
      // Initialize your transport
    },

    sendRequest(request: RequestEnum) {
      return {
        subscribe: (observer) => {
          // Send request and call observer.next(wireResponse)
          // when responses arrive
        },
      };
    },

    dispose() {
      // Clean up resources
    },
  };
}
```

## Wire Protocol

Both adaptors use the same wire protocol matching the Rust `RequestEnum` and `ResponseEnum` types:

**Request:**
```typescript
{
  Call: [requestId, { find_shortest_path: params }]
}
```

**Response:**
```typescript
[requestId, { N: { find_shortest_path: result } }]  // Next
[requestId, { Error: "error message" }]              // Error
[requestId, { Complete: "notes" }]                   // Complete
[requestId, { Aborted: "reason" }]                   // Aborted
```

## Files

- `src/router/types.ts` - Core types and interfaces
- `src/router/router.ts` - Router factory and implementation
- `src/router/wasmAdaptor.ts` - WASM transport adaptor
- `src/router/websocketAdaptor.ts` - WebSocket transport adaptor
- `src/router/index.ts` - Public API exports
