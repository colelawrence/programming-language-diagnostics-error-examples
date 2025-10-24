# RPC Handles Feature

## Problem
Monolithic router with flat endpoints becomes unwieldy at scale. Hundreds of methods on single router interface.

## Solution: Handle-Based Hierarchy

### Client API
```ts
const { graph_id } = await router.create_graph(...).first();
const graphHandle = router.Graph(graph_id);
await graphHandle.get_shortest_distance({ ... }).first();
await graphHandle.add_points({ ... }).complete();
```

### Architecture
- Router methods return IDs (e.g., `create_graph()` → `graph_id`)
- Factory methods create handle instances: `router.Graph(graph_id)` → typed `GraphHandle`
- Handles have scoped methods: `graphHandle.get_shortest_distance()`, `.add_points()`
- Internal: handle methods call `router.send_request(handle_id, method, params)`

### Codegen Generates
- **Rust**: Separate trait per handle type (`GraphHandler`, `NodeHandler`, etc.)
- **TypeScript**: Handle classes with typed methods
- **Router**: Dispatch logic routes by `(handle_type, handle_id, method)` to correct trait impl

### Benefits
- Namespaced API: related operations grouped
- Stateful objects: handles encapsulate server-side state
- Scalable: no flat list of hundreds of endpoints
- Composable: traits stay focused, combine at router level

---

## Implementation Considerations

### Protocol Tags
Extend codegen to detect protocol attribute on enums:
- `#[protocol("router")]` - flat router methods (current behavior)
- `#[protocol("graph-handle")]`, `#[protocol("node-handle")]` - handle methods with implicit `id: string` parameter

Generators branch on tag pattern `/^(.+)-handle$/` to extract handle type.

### Generator Changes

**generateRustRouter.ts**:
- Group enums by protocol tag
- Generate separate trait per handle type: `GraphHandle`, `NodeHandle`
- Handle trait methods get `id: HandleId` as first parameter
- Dispatch enum gains `Handle { handle_type, id, method, payload }` variant
- Router dispatch matches on `(handle_type, method)` tuple

**generateTypescript.ts**:
- Detect protocol tags from declaration attrs
- Generate handle classes with methods: `GraphHandle.get_shortest_distance(params)`
- Generate factory methods on router: `router.Graph(id: string): GraphHandle`
- Handle methods internally call `transport.callHandle(handleType, id, method, params)`

### Rust Codegen Goal (`router_goal.rs`)
```rust
pub type HandleId = String;

// Flat router trait (protocol = "router")
pub trait Router {
    fn create_graph(&mut self, params: CreateGraphParams) 
        -> Result<CreateGraphResult, RouterError>;
}

// Per-handle traits (protocol = "graph-handle")
pub trait GraphHandle {
    fn get_shortest_distance(
        &mut self, 
        id: HandleId, 
        params: GetShortestDistanceParams
    ) -> Result<GetShortestDistanceResult, RouterError>;
    
    fn add_points(
        &mut self, 
        id: HandleId, 
        params: AddPointsParams
    ) -> Result<AddPointsResult, RouterError>;
}

// Unified dispatch envelope
pub enum RpcRequest {
    Router { method: &'static str, payload: Vec<u8> },
    Handle { 
        handle: &'static str,  // "graph" | "node"
        id: HandleId, 
        method: &'static str, 
        payload: Vec<u8> 
    },
}

// Dispatcher routes by (protocol, method[, id])
pub trait RouterDispatch: Router + GraphHandle {
    fn dispatch(&mut self, req: RpcRequest) -> RpcResponse {
        match req {
            RpcRequest::Router { method, payload } => { /* ... */ }
            RpcRequest::Handle { handle, id, method, payload } => {
                match (handle, method) {
                    ("graph", "get_shortest_distance") => { /* ... */ }
                    ("graph", "add_points") => { /* ... */ }
                    _ => RpcResponse::Err(/* unknown */)
                }
            }
        }
    }
}
```

### TypeScript Codegen Goal
```ts
// Transport abstraction
interface Transport {
  callRouter<TReq, TRes>(method: string, params: TReq): Observable<TRes>;
  callHandle<TReq, TRes>(handle: string, id: string, method: string, params: TReq): Observable<TRes>;
}

// Generated router with handle factories
export class Router {
  constructor(private t: Transport) {}

  // protocol("router") methods
  create_graph(params: CreateGraphParams): Observable<CreateGraphResult> {
    return this.t.callRouter("create_graph", params);
  }

  // Handle factory methods
  Graph(id: string): GraphHandle {
    return new GraphHandle(this.t, id);
  }
}

// protocol("graph-handle") class
export class GraphHandle {
  constructor(private t: Transport, private id: string) {}

  get_shortest_distance(params: GetShortestDistanceParams): Observable<GetShortestDistanceResult> {
    return this.t.callHandle("graph", this.id, "get_shortest_distance", params);
  }

  add_points(params: AddPointsParams): Observable<AddPointsResult> {
    return this.t.callHandle("graph", this.id, "add_points", params);
  }
}
```

### Key Implementation Points
- **Input schema**: Ensure codegen input JSON exposes enum-level `protocol` attribute
- **Grouping**: Group declarations by protocol tag before generation
- **Naming**: Handle type from tag (e.g., `graph-handle` → `GraphHandle`)
- **ID parameter**: Always `HandleId`/`string`, not part of params payload
- **Dispatch**: Central router dispatch remains, just adds handle routing path
- **Back-compat**: Re-export new classes from index.ts to avoid breaking changes

### Risks & Guardrails
- Input JSON must expose protocol attrs (fail-fast if missing)
- Name collisions: reserve PascalCase for handles, snake_case for methods
- ID type consistency: canonicalize to string/HandleId across languages
- Back-compat: ensure re-exports from index.ts
