# Code Generation Architecture

This document describes the code generation system that keeps TypeScript types synchronized with Rust definitions.

## Overview

The project uses [`derive-codegen`](https://crates.io/crates/derive-codegen) to automatically generate TypeScript types from Rust code. This ensures type safety across the WASM boundary and eliminates drift between Rust and TypeScript.

## What Gets Generated

### 1. Data Types (`dist-types/index.ts`)

Rust structs marked with `#[protocol("wasm")]` are converted to TypeScript interfaces:

```rust
// shared-types/src/lib.rs
#[protocol("wasm")]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

↓ generates ↓

```typescript
// dist-types/index.ts
export interface Point {
  x: number;
  y: number;
}
```

**Type Mapping:**
- Rust primitives → TypeScript primitives (`f64`/`i32`/etc → `number`, `String` → `string`, `bool` → `boolean`)
- `Vec<T>` → `T[]`
- `Option<T>` → `T | null`
- Structs → Interfaces with the same name

### 2. Router Wire Protocol Types (`dist-types/router.gen.ts`)

Functions defined with `#[codegen(fn = "...")]` generate wire protocol types:

```rust
// shared-types/src/lib.rs
#[protocol("wasm")]
#[codegen(fn = "find_shortest_path() -> PathResult")]
pub struct ShortestPathParams {
    pub points: Vec<Point>,
    pub edges: Vec<Edge>,
    pub start_idx: usize,
    pub end_idx: usize,
}
```

↓ generates ↓

```typescript
// dist-types/router.gen.ts
export type CallGen = {
  find_shortest_path: ShortestPathParams;  // ← snake_case for wire protocol
};

export type ResponseNextGen = {
  find_shortest_path: PathResult;
};
```

**Key Points:**
- Wire keys use `snake_case` to match Rust/serde conventions
- Only types are generated, not implementations
- Used by `src/router/types.ts` to define `RequestEnum` and `ResponseEnum`

## What Does NOT Get Generated

The code generator intentionally **does not** generate:

❌ **Router implementation** - Lives in `src/router/router.ts`
- Handles initialization (`adaptor.init()`)
- Manages request ID counter
- Provides error/complete handling
- Returns `EnhancedObservable` with convenience methods

❌ **Adaptor interface** - Defined in `src/router/types.ts`
- Transport-specific (WASM vs WebSocket)
- Requires async initialization
- Returns per-request observables

❌ **Observable interfaces** - Defined in `src/router/types.ts`
- Custom signature with `next`/`error`/`complete`
- Enhanced version with `first()`/`all()`/`each()` helpers

**Rationale:** The handwritten router already matches our wire protocol, handles initialization properly, and provides the enhanced observable pattern. Generating runtime code would introduce mismatches and complexity.

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│ Rust Source of Truth                                    │
│ shared-types/src/lib.rs                                 │
│ - #[protocol("wasm")] → data types                      │
│ - #[codegen(fn = "...")] → router functions            │
└────────────────┬────────────────────────────────────────┘
                 │
                 │ derive-codegen + generateTypescript.ts
                 ↓
┌─────────────────────────────────────────────────────────┐
│ Generated Types (read-only)                             │
│ dist-types/                                             │
│ - index.ts → interfaces (Point, Edge, PathResult, etc)  │
│ - router.gen.ts → CallGen, ResponseNextGen             │
└────────────────┬────────────────────────────────────────┘
                 │
                 │ imported by
                 ↓
┌─────────────────────────────────────────────────────────┐
│ Wire Protocol Layer                                     │
│ src/router/types.ts                                     │
│ - RequestEnum = Abort | Call                           │
│ - ResponseEnum = Aborted | Error | Complete | N       │
│ - Uses CallGen/ResponseNextGen from generated code    │
└────────────────┬────────────────────────────────────────┘
                 │
                 │ used by
                 ↓
┌─────────────────────────────────────────────────────────┐
│ Router Implementation (handwritten)                     │
│ src/router/router.ts                                    │
│ - createRouter(adaptor) → Router                       │
│ - Numeric request IDs, init/dispose lifecycle         │
│ - Returns EnhancedObservable                           │
└────────────────┬────────────────────────────────────────┘
                 │
                 │ used by
                 ↓
┌─────────────────────────────────────────────────────────┐
│ Transport Adaptors                                      │
│ src/router/wasmAdaptor.ts                              │
│ src/router/websocketAdaptor.ts                         │
└────────────────┬────────────────────────────────────────┘
                 │
                 │ used by
                 ↓
┌─────────────────────────────────────────────────────────┐
│ Application Code                                        │
│ src/pathfinder.tsx, examples/, etc.                    │
└─────────────────────────────────────────────────────────┘
```

## Generator Implementation

### Location
- **Generator script:** `generators/generateTypescript.ts`
- **Invoked by:** `shared-types/src/lib.rs` test `generate_typescript`

### How It Works

1. **Input:** JSON from `derive-codegen` containing:
   ```typescript
   {
     declarations: [
       {
         id: "Point",
         container_kind: { Struct: { fields: [...] } },
         codegen_attrs?: { fn: ["find_shortest_path() -> PathResult"] }
       }
     ]
   }
   ```

2. **Processing:**
   - `generateTypeScriptInterface()` - converts structs to TS interfaces
   - `parseRouterFunctions()` - extracts `#[codegen(fn = "...")]` using regex: `/^([\w$]+)\(\) -> ([\w$]+|\(\))$/`
   - `generateRouterCode()` - builds CallGen/ResponseNextGen types

3. **Output:** JSON with file paths and source:
   ```typescript
   {
     files: [
       { path: "index.ts", source: "..." },
       { path: "router.gen.ts", source: "..." }
     ]
   }
   ```

4. **derive-codegen** writes files to `dist-types/`

### Key Functions

- **`formatToTypeScript(format)`** - Maps Rust types to TS types
  - Handles primitives, sequences, options, named types
  - Recursive for nested structures

- **`parseRouterFunctions(declarations)`** - Extracts router metadata
  - Parses `#[codegen(fn = "...")]` attributes
  - Returns `{ key, inputType, responseType }[]`

- **`generateRouterCode(fns)`** - Emits router types only
  - CallGen with snake_case keys
  - ResponseNextGen with snake_case keys
  - Clear documentation about separation of concerns

## Running Code Generation

### Manual
```bash
cd shared-types
cargo test --features codegen generate_typescript -- --ignored
```

### Via mise
```bash
mise run codegen:typescript
```

### During Build
The build process (`mise run build`) includes codegen as a prerequisite.

## Adding New Endpoints

### 1. Define Types in Rust

```rust
// shared-types/src/lib.rs

#[protocol("wasm")]
pub struct MyResult {
    pub value: f64,
    pub status: String,
}

#[protocol("wasm")]
#[codegen(fn = "my_function() -> MyResult")]
pub struct MyParams {
    pub input: Vec<f64>,
}
```

### 2. Implement Handler in Rust

```rust
// pathfinder-core/src/lib.rs or similar
impl CallHandler for MyHandler {
    fn my_function(
        &self,
        _ctx: &Context,
        params: MyParams,
        tx: ObserverImpl<MyResult>,
    ) {
        // Implementation
        tx.next(MyResult { value: 42.0, status: "ok".to_string() });
        tx.complete("done");
    }
}
```

### 3. Regenerate Types

```bash
cargo test --features codegen generate_typescript -- --ignored
```

This updates:
- `dist-types/index.ts` with `MyParams` and `MyResult` interfaces
- `dist-types/router.gen.ts` with `CallGen` and `ResponseNextGen` entries for `my_function`

### 4. Update TypeScript Router

```typescript
// src/router/router.ts
return {
  // ... existing methods
  
  myFunction(params: MyParams) {
    const obs: Observable<MyResult> = {
      subscribe: (observer) => {
        ensureInitialized()
          .then(() => {
            requestId++;
            const request: RequestEnum = {
              Call: [requestId, { my_function: params }], // ← snake_case wire key
            };

            adaptor.sendRequest(request).subscribe({
              next: (response) => {
                const [, responseEnum] = response;
                if ("N" in responseEnum && "my_function" in responseEnum.N) {
                  observer.next?.(responseEnum.N.my_function);
                }
              },
              error: observer.error,
              complete: observer.complete,
            });
          })
          .catch((error) => {
            observer.error?.(
              error instanceof Error ? error.message : String(error),
            );
          });
      },
    };
    return enhanceObservable(obs);
  },
};
```

### 5. Update Router Interface

```typescript
// src/router/types.ts
export interface Router {
  findShortestPath(params: ShortestPathParams): EnhancedObservable<PathResult>;
  myFunction(params: MyParams): EnhancedObservable<MyResult>; // ← add this
  dispose(): void;
}
```

## Best Practices

### ✅ Do

- Always regenerate types after changing Rust definitions
- Keep wire protocol keys snake_case in generated code
- Use camelCase for TypeScript method names in the router
- Document complex type mappings with Rust doc comments
- Run typecheck after regeneration: `mise run typecheck`

### ❌ Don't

- Manually edit files in `dist-types/` - they'll be overwritten
- Mix snake_case and camelCase in the same layer
- Generate runtime implementations - keep handwritten router
- Assume codegen runs automatically - always run explicitly after Rust changes

## Troubleshooting

### Types not updating after Rust changes

```bash
# Force regeneration
cd shared-types
cargo test --features codegen generate_typescript -- --ignored

# Verify output
cat ../dist-types/index.ts
```

### Type errors in router.ts

1. Check that wire keys match generated `CallGen` / `ResponseNextGen`:
   ```typescript
   // Must use snake_case key from CallGen
   Call: [requestId, { find_shortest_path: params }]
   ```

2. Verify imports:
   ```typescript
   import type { CallGen, ResponseNextGen } from "../../dist-types/router.gen";
   ```

### Regex parse errors

The generator expects the exact format: `function_name() -> ReturnType`

Valid:
- ✅ `find_shortest_path() -> PathResult`
- ✅ `get_config() -> ()`

Invalid:
- ❌ `find_shortest_path(input: String) -> PathResult` (no params in signature)
- ❌ `find_shortest_path()->PathResult` (missing spaces)
- ❌ `find_shortest_path() -> Vec<PathResult>` (generic return not supported yet)

## Future Improvements

### Potential Enhancements

1. **Generate Router Skeleton** - Emit commented method stubs in router.ts for new functions
2. **Support Generic Returns** - Handle `Result<T, E>`, `Vec<T>`, `Option<T>` in return types
3. **Streaming Indicators** - Mark functions that return multiple values vs single
4. **Deprecation Warnings** - Generate `@deprecated` JSDoc from Rust attributes
5. **Validation Helpers** - Generate runtime validators for request params

### Not Planned

- Full router implementation generation (handwritten provides better control)
- Adaptor code generation (transport-specific logic too varied)
- Automatic method name conversion (explicit mapping is clearer)

## Related Documentation

- [AGENTS.md](../AGENTS.md) - Commands and architecture overview
- [dist-types/README.md](../dist-types/README.md) - Generated types reference
- [shared-types/README.md](../shared-types/README.md) - Rust protocol definitions
