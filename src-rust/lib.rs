use pathfinder_core::PathfinderHandler;
use shared_types::receiver::Receiver;
use shared_types::router::{Request, WireResponse, WireResponseSender};
use shared_types::storage::InMemoryStorage;
use wasm_bindgen::prelude::*;

/// WASM-specific transport that calls JavaScript callback
struct WasmTransport {
    callback: js_sys::Function,
}

impl WireResponseSender for WasmTransport {
    fn send_response(&self, wire_response: WireResponse) {
        let this = JsValue::NULL;
        if let Ok(serialized) = serde_wasm_bindgen::to_value(&wire_response) {
            let _ = self.callback.call1(&this, &serialized);
        }
    }
}

/// Main entry point for router-based requests
#[wasm_bindgen]
pub fn send_request(request_js: JsValue, response_callback: js_sys::Function) -> Result<(), JsValue> {
    let request: Request = serde_wasm_bindgen::from_value(request_js)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse request: {}", e)))?;

    let transport = Box::new(WasmTransport {
        callback: response_callback,
    });

    // Create handler with optional in-memory storage
    let handler = PathfinderHandler::new(Some(std::sync::Arc::new(InMemoryStorage::new())));

    // Create receiver for this session
    let receiver = Receiver::new("wasm-session".to_string(), handler, Some(InMemoryStorage::new()));

    // Handle the request
    receiver.handle_request(request, transport);

    Ok(())
}
