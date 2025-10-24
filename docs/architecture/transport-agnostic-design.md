# Transport-Agnostic Backend Design

## Architecture Overview

This design separates three concerns:

1. **Business Logic** (`CallHandler` trait) - The actual algorithms
2. **Transport Layer** (`WireResponseSender`) - How messages get delivered
3. **Session Management** (`Receiver`) - Coordinates requests with context and storage

## Core Traits

### 1. WireResponseSender (Transport)
```rust
pub trait WireResponseSender {
    fn send_response(&self, wire_response: WireResponse);
}
```
- Minimal, synchronous interface
- Implementations enqueue to async writers
- No Send/Sync requirements (works in WASM and server)

### 2. CallHandler (Business Logic)
```rust
pub trait CallHandler {
    fn find_shortest_path(
        &self,
        ctx: &Context,
        params: ShortestPathParams,
        tx: ObserverImpl<PathResult>,
    );
}
```
- Receives Context with session info
- Uses ObserverImpl for streaming responses
- Completely transport-agnostic

### 3. Storage (Optional State)
```rust
pub trait Storage: Send + Sync {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn set(&self, key: &str, value: Vec<u8>);
    fn delete(&self, key: &str);
}
```
- Key-value interface
- Can be in-memory, localStorage, Redis, etc.

## Usage Patterns

### WASM Implementation

```rust
// Transport: Calls JavaScript callback
struct WasmTransport {
    callback: js_sys::Function,
}

impl WireResponseSender for WasmTransport {
    fn send_response(&self, wire_response: WireResponse) {
        let serialized = serde_wasm_bindgen::to_value(&wire_response).unwrap();
        let _ = self.callback.call1(&JsValue::NULL, &serialized);
    }
}

// Handler: Pure business logic with optional storage
struct PathfinderHandler {
    storage: Option<Arc<dyn Storage>>,
}

impl CallHandler for PathfinderHandler {
    fn find_shortest_path(
        &self,
        ctx: &Context,
        params: ShortestPathParams,
        tx: ObserverImpl<PathResult>,
    ) {
        // Check cache in storage
        if let Some(storage) = &self.storage {
            let cache_key = format!("path_{}_{}", params.start_idx, params.end_idx);
            if let Some(cached) = storage.get(&cache_key) {
                // Return cached result
            }
        }

        // Compute path
        let result = compute_shortest_path(params);
        
        // Send result
        tx.next(result);
        tx.complete("Path found".to_string());
    }
}

// WASM Entry Points
#[wasm_bindgen]
pub fn send_request(request_js: JsValue, callback: js_sys::Function) {
    let request: Request = serde_wasm_bindgen::from_value(request_js).unwrap();
    let transport = Box::new(WasmTransport { callback });
    
    let handler = PathfinderHandler {
        storage: Some(Arc::new(InMemoryStorage::new())),
    };
    
    let receiver = Receiver::new(
        "wasm-session".to_string(),
        handler,
        Some(InMemoryStorage::new()),
    );
    
    receiver.handle_request(request, transport);
}
```

### WebSocket Implementation (Future)

```rust
// Transport: Sends over WebSocket
struct WebSocketTransport {
    tx: tokio::sync::mpsc::Sender<WireResponse>,
}

impl WireResponseSender for WebSocketTransport {
    fn send_response(&self, wire_response: WireResponse) {
        // Non-blocking send to channel
        let _ = self.tx.try_send(wire_response);
    }
}

// Same handler implementation works!
struct PathfinderHandler {
    storage: Option<Arc<dyn Storage>>,
}

impl CallHandler for PathfinderHandler {
    // ... exact same implementation as WASM
}

// WebSocket Server
async fn handle_websocket_connection(socket: WebSocket) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    
    let handler = PathfinderHandler {
        storage: Some(Arc::new(RedisStorage::new())),
    };
    
    let session_id = Uuid::new_v4().to_string();
    let receiver = Receiver::new(
        session_id.clone(),
        handler,
        Some(RedisStorage::new()),
    );
    
    // Spawn writer task
    let mut write_half = socket;
    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            let json = serde_json::to_string(&response).unwrap();
            let _ = write_half.send(Message::Text(json)).await;
        }
    });
    
    // Read loop
    let mut read_half = socket;
    while let Some(Ok(msg)) = read_half.next().await {
        if let Message::Text(text) = msg {
            let request: Request = serde_json::from_str(&text).unwrap();
            let transport = Box::new(WebSocketTransport { tx: tx.clone() });
            receiver.handle_request(request, transport);
        }
    }
}
```

### HTTP Implementation (Future)

```rust
// Transport: HTTP response channel
struct HttpTransport {
    tx: tokio::sync::oneshot::Sender<HttpResponse>,
}

impl WireResponseSender for HttpTransport {
    fn send_response(&self, wire_response: WireResponse) {
        // For HTTP, we typically want the first complete response
        // Could buffer multiple next() calls and send on complete()
    }
}

// Same handler again!
async fn handle_http_request(req: HttpRequest) -> HttpResponse {
    let request: Request = serde_json::from_slice(req.body()).unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    let handler = PathfinderHandler {
        storage: Some(Arc::new(PostgresStorage::new())),
    };
    
    let receiver = Receiver::new(
        req.session_id(),
        handler,
        Some(PostgresStorage::new()),
    );
    
    let transport = Box::new(HttpTransport { tx });
    receiver.handle_request(request, transport);
    
    rx.await.unwrap()
}
```

## Storage Implementations

### In-Memory (WASM, Testing)
```rust
pub struct InMemoryStorage {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}
```

### Browser LocalStorage (WASM)
```rust
pub struct BrowserStorage;

impl Storage for BrowserStorage {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        let window = web_sys::window()?;
        let storage = window.local_storage().ok()??;
        let value = storage.get_item(key).ok()??;
        Some(value.into_bytes())
    }
    
    fn set(&self, key: &str, value: Vec<u8>) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let str_value = String::from_utf8_lossy(&value);
                let _ = storage.set_item(key, &str_value);
            }
        }
    }
}
```

### Redis (Server)
```rust
pub struct RedisStorage {
    client: redis::Client,
}

impl Storage for RedisStorage {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut con = self.client.get_connection().ok()?;
        con.get(key).ok()
    }
    
    fn set(&self, key: &str, value: Vec<u8>) {
        if let Ok(mut con) = self.client.get_connection() {
            let _: Result<(), _> = con.set(key, value);
        }
    }
}
```

## Benefits

1. **Write Once, Run Anywhere**: Same `CallHandler` works in WASM, WebSocket, HTTP
2. **Testable**: Mock `WireResponseSender` and `Storage` for unit tests
3. **Flexible**: Swap storage (memory → localStorage → Redis) without changing logic
4. **Type Safe**: All boundaries use generated types from `shared-types`
5. **Observable Pattern**: Support for streaming, errors, completion
6. **Session Aware**: Context passed to every handler method

## Migration Path

### Current State
- ✅ Traits defined
- ✅ `Receiver` and `Context` implemented
- ✅ `InMemoryStorage` implemented
- ⚠️  WASM still using old direct exports

### Next Steps
1. Extract `compute_shortest_path()` business logic from WASM exports
2. Implement `PathfinderHandler` using extracted logic
3. Update WASM entry points to use `Receiver`
4. Remove unsafe `Send + Sync` from transport
5. Add optional storage to handler
6. Document WebSocket implementation for future use

### Future Enhancements
- Cancellation tokens in `Context`
- Request timeouts
- Backpressure/bounded channels
- Multiple endpoints beyond `find_shortest_path`
- Middleware (auth, logging, metrics)
