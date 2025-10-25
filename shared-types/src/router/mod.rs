use crate::context::Context;
use shared_types_proc::protocol;
use std::marker::PhantomData;

mod router_gen;
pub use router_gen::CallHandler;

#[protocol("router", externally_tagged)]
pub enum RequestEnum {
    Abort(usize, String),
    Call(usize, router_gen::CallGen),
}

#[protocol("router")]
pub struct WireResponse(pub usize, pub ResponseEnum);

#[protocol("router", externally_tagged)]
pub enum ResponseEnum {
    Aborted(String),
    Error(String),
    Complete(String),
    N(router_gen::ResponseNextGen),
}

#[derive(serde::Deserialize, Debug)]
#[serde(transparent)]
pub struct Request(pub RequestEnum);

/// Minimal transport abstraction for sending responses
/// Implementations should be lightweight and enqueue to async writers if needed
pub trait WireResponseSender {
    fn send_response(&self, wire_response: WireResponse);
}

/// Emitter for streaming values via Observable pattern
pub struct Emitter<T> {
    request_id: usize,
    sender: Box<dyn WireResponseSender>,
    _mark: PhantomData<T>,
}

impl<T> Emitter<T> {
    pub fn new(request_id: usize, sender: Box<dyn WireResponseSender>) -> Self {
        Self {
            request_id,
            sender,
            _mark: PhantomData,
        }
    }
}

/// Completer for finishing the Observable stream
pub struct Completer<T> {
    request_id: usize,
    sender: Box<dyn WireResponseSender>,
    _mark: PhantomData<T>,
}

impl<T> Completer<T> {
    pub fn new(request_id: usize, sender: Box<dyn WireResponseSender>) -> Self {
        Self {
            request_id,
            sender,
            _mark: PhantomData,
        }
    }
}

/// Observer combining Emitter and Completer
pub struct ObserverImpl<T> {
    request_id: usize,
    sender: Box<dyn WireResponseSender>,
    _mark: PhantomData<T>,
}

impl<T> ObserverImpl<T> {
    pub fn new(request_id: usize, sender: Box<dyn WireResponseSender>) -> Self {
        Self {
            request_id,
            sender,
            _mark: PhantomData,
        }
    }

    pub fn into_parts(self) -> (Emitter<T>, Completer<T>) {
        // Clone the sender's behavior via Arc internally
        // For now, we'll just create new instances - transport should handle ref counting
        let _sender = self.sender;
        // This is a limitation - we'd need Arc or Rc internally for true cloning
        // For now, observers should use next/complete directly without split
        panic!("into_parts not yet supported - use next() and complete() on ObserverImpl directly");
    }
}

pub trait ToResponseNextGen {
    fn to_response_next_gen(self) -> router_gen::ResponseNextGen;
}

impl<T: ToResponseNextGen> Emitter<T> {
    pub fn next(&self, value: T) {
        self.sender.send_response(WireResponse(
            self.request_id,
            ResponseEnum::N(value.to_response_next_gen()),
        ));
    }
}

impl<T: ToResponseNextGen> Completer<T> {
    pub fn error(self, error: String) {
        self.sender
            .send_response(WireResponse(self.request_id, ResponseEnum::Error(error)));
    }

    pub fn complete(self, notes: String) {
        self.sender.send_response(WireResponse(
            self.request_id,
            ResponseEnum::Complete(notes),
        ));
    }
}

impl<T: ToResponseNextGen> ObserverImpl<T> {
    pub fn next(&self, value: T) {
        self.sender.send_response(WireResponse(
            self.request_id,
            ResponseEnum::N(value.to_response_next_gen()),
        ));
    }

    pub fn error(self, error: String) {
        self.sender
            .send_response(WireResponse(self.request_id, ResponseEnum::Error(error)));
    }

    pub fn complete(self, notes: String) {
        self.sender.send_response(WireResponse(
            self.request_id,
            ResponseEnum::Complete(notes),
        ));
    }
}

pub fn handle_request(
    request: Request,
    ctx: &Context,
    handler: &dyn CallHandler,
    sender: Box<dyn WireResponseSender>,
) {
    match request.0 {
        RequestEnum::Abort(id, reason) => {
            sender.send_response(WireResponse(id, ResponseEnum::Aborted(reason)));
        }
        RequestEnum::Call(id, call) => {
            router_gen::gen_call(ctx, id, call, handler, sender);
        }
    }
}


