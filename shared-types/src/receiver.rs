use crate::context::Context;
use crate::router::{CallHandler, Request, WireResponseSender};
use crate::storage::Storage;

/// Session receiver that handles requests for a single session/connection
/// This can be instantiated in WASM, over WebSockets, or via HTTP
pub struct Receiver<H: CallHandler, S: Storage> {
    session_id: String,
    handler: H,
    storage: Option<S>,
}

impl<H: CallHandler, S: Storage> Receiver<H, S> {
    pub fn new(session_id: String, handler: H, storage: Option<S>) -> Self {
        Self {
            session_id,
            handler,
            storage,
        }
    }

    pub fn handle_request(&self, request: Request, sender: Box<dyn WireResponseSender>) {
        // Extract request ID from the request
        let request_id = match &request.0 {
            crate::router::RequestEnum::Call(id, _) => *id,
            crate::router::RequestEnum::Abort(id, _) => *id,
        };

        // Create context for this request
        let ctx = Context::new(self.session_id.clone(), request_id);

        // Handle the request
        crate::router::handle_request(request, &ctx, &self.handler, sender);
    }

    pub fn storage(&self) -> Option<&S> {
        self.storage.as_ref()
    }
}
