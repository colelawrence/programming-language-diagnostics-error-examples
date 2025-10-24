# Adding New Router Functions

This guide shows how to add a new router function that automatically generates TypeScript types.

## Example: Adding a `validate_path` Function

### 1. Define Types in Rust

```rust
// shared-types/src/lib.rs

/// Result of path validation
#[protocol("wasm")]
pub struct ValidationResult {
    pub is_valid: bool,
    pub reason: Option<String>,
}

/// Parameters for path validation
#[protocol("wasm")]
#[codegen(fn = "validate_path() -> ValidationResult")]
pub struct ValidatePathParams {
    pub points: Vec<Point>,
    pub edges: Vec<Edge>,
}
```

### 2. Regenerate Types

```bash
cd shared-types
cargo test --features codegen generate_typescript -- --ignored
```

This automatically updates:

**`dist-types/index.ts`**
```typescript
export interface ValidationResult {
  is_valid: boolean;
  reason: string | null;
}

export interface ValidatePathParams {
  points: Point[];
  edges: Edge[];
}
```

**`dist-types/router.gen.ts`**
```typescript
export type CallGen = {
  find_shortest_path: ShortestPathParams;
  validate_path: ValidatePathParams;  // ← Added automatically!
};

export type ResponseNextGen = {
  find_shortest_path: PathResult;
  validate_path: ValidationResult;    // ← Added automatically!
};
```

### 3. Implement in Rust

```rust
// pathfinder-core/src/lib.rs

impl CallHandler for PathfinderHandler {
    fn find_shortest_path(/* ... */) { /* existing */ }
    
    fn validate_path(
        &self,
        _ctx: &Context,
        params: ValidatePathParams,
        tx: ObserverImpl<ValidationResult>,
    ) {
        // Validation logic
        let is_valid = params.points.len() >= 2;
        let reason = if is_valid {
            None
        } else {
            Some("Need at least 2 points".to_string())
        };
        
        tx.next(ValidationResult { is_valid, reason });
        tx.complete("done");
    }
}
```

### 4. Implement in TypeScript Router

**IMPORTANT:** You must manually implement the router method in `src/router/router.ts`. The Router interface is defined manually in `src/router/types.ts` and is NOT auto-generated.

```typescript
// src/router/router.ts

export function createRouter(options: CreateRouterOptions): Router {
  const { adaptor } = options;
  let requestId = 0;
  let initialized = false;

  async function ensureInitialized() { /* ... */ }

  return {
    find_shortest_path(params) { /* existing implementation */ },
    
    // Add new method implementation manually
    validate_path(params: ValidatePathParams) {
      const obs: Observable<ValidationResult> = {
        subscribe: (observer) => {
          ensureInitialized()
            .then(() => {
              requestId++;
              const request: RequestEnum = {
                Call: [requestId, { validate_path: params }], // ← snake_case wire key
              };

              adaptor.sendRequest(request).subscribe({
                next: (response) => {
                  const [, responseEnum] = response;
                  if ("N" in responseEnum && "validate_path" in responseEnum.N) {
                    observer.next?.(responseEnum.N.validate_path);
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

    dispose() { /* ... */ },
  };
}
```

You'll also need to manually add the method signature to the Router interface:

```typescript
// src/router/types.ts

export interface Router {
  find_shortest_path(params: ShortestPathParams): EnhancedObservable<PathResult>;
  validate_path(params: ValidatePathParams): EnhancedObservable<ValidationResult>; // ← Add manually!
  dispose(): void;
}
```

### 5. Use in Application

```typescript
// src/pathfinder.tsx or any component

const router = createRouter({
  adaptor: createWasmAdaptor({ wasmInit })
});

// TypeScript knows about validate_path!
router.validate_path({
  points: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
  edges: [{ from: 0, to: 1 }]
}).subscribe({
  next: (result) => {
    console.log('Valid:', result.is_valid);
    console.log('Reason:', result.reason);
  }
});
```

## What Gets Updated Automatically

When you add `#[codegen(fn = "validate_path() -> ValidationResult")]` in Rust and run codegen:

✅ **`dist-types/router.gen.ts`**
- `CallGen` gets `validate_path: ValidatePathParams`
- `ResponseNextGen` gets `validate_path: ValidationResult`

✅ **Type safety across the boundary**
- Request parameters are validated at compile time
- Response types are known statically
- No drift between Rust and TypeScript for data structures

## What You Must Do Manually

❌ **Router interface definition** (`src/router/types.ts`)
- Add the method signature to the `Router` interface
- Use camelCase for method names (optional convention, but currently snake_case)

❌ **Router implementation** (`src/router/router.ts`)
- The actual method body in `createRouter()`
- Error handling, initialization, request correlation
- This gives you full control over the implementation

❌ **Rust handler implementation**
- The `validate_path` function in your `CallHandler`
- Business logic

## Key Points

1. **Wire keys use snake_case; TypeScript method names are camelCase (or snake_case)** - The wire protocol uses `validate_path` (snake_case), but TypeScript method names can follow either convention
2. **Only types are auto-generated** - `CallGen` and `ResponseNextGen` types update automatically, but the Router interface in `src/router/types.ts` is manually maintained
3. **Manual implementation required** - You must manually add the method to both the Router interface and the `createRouter()` implementation
4. **Type safety guaranteed** - Once types are generated, TypeScript enforces correct usage of parameters and responses

## Files That Never Need Route-Specific Updates

After adding a new route, these files remain route-agnostic:

✅ `src/router/wasmAdaptor.ts` - Transport layer, route-independent
✅ `src/router/websocketAdaptor.ts` - Transport layer, route-independent
✅ `src/router/enhancedObservable.ts` - Utility helpers, route-independent

## Troubleshooting

### TypeScript Error: Property 'validate_path' does not exist on type 'Router'

**Cause:** You forgot to add the method signature to the Router interface in `src/router/types.ts`

**Fix:** Add the method signature manually to the Router interface

### TypeScript Error: Type 'X' does not satisfy the constraint 'Router'

**Cause:** You forgot to implement the method in `router.ts`

**Fix:** Add the implementation in `createRouter()` return object

### Runtime Error: "Unknown key in response"

**Cause:** Mismatch between wire key in `router.ts` and Rust function name

**Fix:** Ensure you use the exact same name as defined in Rust (always snake_case for wire keys):
```typescript
Call: [requestId, { validate_path: params }]  // ✅ Correct - matches Rust
Call: [requestId, { validatePath: params }]   // ❌ Wrong - doesn't match Rust
```

### No Type Errors but Router Missing Method

**Cause:** TypeScript cache issue

**Fix:**
```bash
# Clear TypeScript cache and rebuild
rm -rf node_modules/.vite
mise run typecheck
```
