use crate::{
    types::{self, DevString},
    utils::AbortSignal,
};
use chrono::Utc;
use i_cg_types_proc::protocol;
pub use router_gen::CallHandler;
use std::{
    fmt::Formatter,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, RwLock, Weak},
};

use self::router_gen::{CallGen, ResponseNextGen};

mod router_gen;

// pub trait Observer<T>: 'static + Send {
//     fn next(&self, value: T);
//     fn error(self, error: DevString);
//     fn complete(self);
// }

/// First usize is the request ID
#[protocol("router")]
pub(crate) enum RequestEnum {
    Abort(usize, DevString),
    Call(usize, CallGen),
}

#[derive(serde::Deserialize)]
#[allow(non_snake_case)]
pub struct RequestParsingTest {
    pub Call: (usize, serde_json::Value),
}

#[protocol("router")]
pub struct WireResponse(usize, ResponseEnum);

#[protocol("router")]
pub(crate) enum ResponseEnum {
    Aborted(DevString),
    Error(DevString),
    Complete(DevString),
    N(ResponseNextGen),
}

pub struct ResponseRouter<RCtx> {
    /// In order to retrieve the reply context for a given request ID.
    /// This is needed for integrating backwards with some requests such as
    /// search integrations legacy, which make many assumptions about the
    /// reply context being present (even though, it isn't strictly needed).
    mvp_rctxs: Arc<RwLock<std::collections::HashMap<usize, Weak<RCtx>>>>,
    // active_calls:
    //     Arc<RwLock<std::collections::HashMap<usize, (RCtx, crate::utils::AbortController)>>>,
    abort_controllers:
        Arc<RwLock<std::collections::HashMap<(RCtx, usize), crate::utils::AbortController>>>,
    wire_response_sender: Arc<Box<dyn crate::router::WireResponseSender<RCtx>>>,
}

impl<RCtx> Clone for ResponseRouter<RCtx> {
    fn clone(&self) -> Self {
        Self {
            mvp_rctxs: self.mvp_rctxs.clone(),
            abort_controllers: self.abort_controllers.clone(),
            wire_response_sender: self.wire_response_sender.clone(),
        }
    }
}

trait ActiveResponder: Send + Sync {
    fn mvp_request_id(&self) -> usize;
    fn respond(&self, response: ResponseEnum);
    fn get_abort_signal(&self) -> AbortSignal;
}

struct ActiveCall<RCtx> {
    request_id: usize,
    reply_context: Arc<RCtx>,
    router: ResponseRouter<RCtx>,
    abort_controller: crate::utils::AbortController,
}

impl<RCtx: Clone + Hash + PartialEq + Eq + Send + Sync + 'static> ActiveResponder
    for ActiveCall<RCtx>
{
    fn mvp_request_id(&self) -> usize {
        self.request_id
    }
    fn respond(&self, response: ResponseEnum) {
        self.router
            .respond(self.request_id, &self.reply_context, response);
    }
    fn get_abort_signal(&self) -> AbortSignal {
        self.abort_controller.signal()
    }
}

pub struct ObserverImpl<T> {
    responder: Box<dyn ActiveResponder>,
    current_time: types::At,
    _mark: PhantomData<T>,
}

impl<T> std::fmt::Debug for ObserverImpl<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObserverImpl")
            .field("req", &self.responder.mvp_request_id())
            .field("current_time", &self.current_time)
            .finish()
    }
}

#[derive(Clone)]
pub struct Emitter<T> {
    responder: Arc<Box<dyn ActiveResponder>>,
    signal: AbortSignal,
    current_time: types::At,
    _mark: PhantomData<T>,
}

#[derive(Clone)]
pub struct Completer<T> {
    responder: Arc<Box<dyn ActiveResponder>>,
    _mark: PhantomData<T>,
}

#[allow(private_bounds)]
impl<T: ToResponseNextGen> Emitter<T> {
    pub fn next(&self, value: T) {
        self.responder
            .respond(ResponseEnum::N(value.to_response_next_gen()));
    }
}
impl<T> Emitter<T> {
    /// All business logic should try to adhere to this time being the "current time".
    pub fn get_current_time(&self) -> types::At {
        self.current_time
    }
    /// Check if the observable has been aborted.
    // Consider ideas of https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html
    pub fn check_aborted(&self) -> Result<(), DevString> {
        if self.signal.is_aborted() {
            Err(DevString::new("emitter's observable was aborted"))
        } else {
            Ok(())
        }
    }
}

#[allow(private_bounds)]
impl<T: ToResponseNextGen> Completer<T> {
    pub fn error(self, error: DevString) {
        self.responder.respond(ResponseEnum::Error(error));
    }
    pub fn complete(self, notes: DevString) {
        self.responder.respond(ResponseEnum::Complete(notes));
    }
}

impl<T> ObserverImpl<T> {
    pub fn mvp_request_id(&self) -> usize {
        self.responder.mvp_request_id()
    }
    pub fn into_parts(self) -> (Emitter<T>, Completer<T>) {
        let responder = self.responder;
        let responder = Arc::new(responder);
        (
            Emitter {
                responder: responder.clone(),
                signal: responder.get_abort_signal(),
                current_time: self.current_time,
                _mark: PhantomData,
            },
            Completer {
                responder,
                _mark: PhantomData,
            },
        )
    }
}

#[doc(hidden)]
pub trait ToResponseNextGen {
    fn to_response_next_gen(self) -> ResponseNextGen;
}

#[allow(private_bounds)]
impl<T: ToResponseNextGen> ObserverImpl<T> {
    pub fn next(&self, value: T) {
        self.responder
            .respond(ResponseEnum::N(value.to_response_next_gen()));
    }
    pub fn error(self, error: DevString) {
        self.responder.respond(ResponseEnum::Error(error));
    }
    pub fn complete(self, notes: DevString) {
        self.responder.respond(ResponseEnum::Complete(notes));
    }
    pub fn get_abort_signal(&self) -> AbortSignal {
        self.responder.get_abort_signal()
    }
}

pub trait WireResponseSender<ReplyCtx>: Sync + Send + 'static {
    fn send_response(&self, reply_context: &ReplyCtx, wire_response: WireResponse);
}

#[derive(serde::Deserialize, Debug)]
#[serde(transparent)]
pub struct Request(RequestEnum);

impl<RCtx: Clone + Hash + PartialEq + Eq + Send + Sync + 'static> ResponseRouter<RCtx> {
    pub fn new(response_sender: Box<dyn WireResponseSender<RCtx>>) -> Self {
        Self {
            mvp_rctxs: Default::default(),
            abort_controllers: Default::default(),
            wire_response_sender: Arc::new(response_sender),
        }
    }

    pub fn mvp_get_reply_context_from_request_id(&self, request_id: usize) -> Option<RCtx> {
        self.mvp_rctxs
            .read()
            .expect("not poisoned")
            .get(&request_id)
            .and_then(|weak| weak.upgrade())
            .map(|a| RCtx::clone(&a))
    }

    fn respond(&self, request_id: usize, reply_context: &RCtx, response: ResponseEnum) {
        // if !controller.is_aborted() {
        (self.wire_response_sender)
            .send_response(reply_context, WireResponse(request_id, response));
    }

    pub fn send_error(&self, request_id: usize, reply_context: &RCtx, error: DevString) {
        self.respond(request_id, reply_context, ResponseEnum::Error(error));
    }

    fn create_responder(&self, request_id: usize, reply_context: RCtx) -> Box<dyn ActiveResponder> {
        let abort_controller = crate::utils::AbortController::new();
        let existing = self
            .abort_controllers
            .write()
            .expect("not poisoned")
            .insert(
                (reply_context.clone(), request_id),
                abort_controller.clone(),
            );
        let reply_context_arc = Arc::new(reply_context);
        self.mvp_rctxs
            .write()
            .expect("not poisoned")
            .insert(request_id, Arc::<RCtx>::downgrade(&reply_context_arc));
        if let Some(a) = existing {
            log::warn!("abort controller (id: {request_id}) already exists, so aborting it");
            a.abort();
        }

        Box::new(ActiveCall {
            request_id,
            reply_context: reply_context_arc,
            router: self.clone(),
            abort_controller,
        })
    }
    pub fn send_request(
        &self,
        Request(request): Request,
        reply_context: RCtx,
        handler: &dyn CallHandler,
    ) {
        match request {
            RequestEnum::Abort(id, reason) => {
                if let Some(controller) = self
                    .abort_controllers
                    .read()
                    .expect("not poisoned")
                    .get(&(reply_context.clone(), id))
                {
                    log::debug!("aborted, id: {id}, reason: {reason:?}");
                    controller.abort();
                } else {
                    log::warn!("abortable controller not found, id: {id}");
                }
            }
            RequestEnum::Call(id, call) => {
                router_gen::gen_call(self, id, call, reply_context, handler);
            }
        }
    }
    fn create_observer<T: ToResponseNextGen + Send + Sync + 'static>(
        &self,
        responder: Box<dyn ActiveResponder>,
    ) -> ObserverImpl<T> {
        ObserverImpl {
            responder,
            current_time: types::At {
                // TODO: use a reference time from the request
                UNIX_SECS: Utc::now().timestamp(),
            },
            _mark: PhantomData,
        }
    }
}
