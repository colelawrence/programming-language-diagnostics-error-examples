use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Clone)]
pub struct AbortController(Arc<AtomicBool>);

impl AbortController {
    pub(crate) fn new() -> AbortController {
        AbortController(Arc::new(AtomicBool::new(false)))
    }
    pub fn abort(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
    pub fn is_aborted(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
    pub fn signal(&self) -> AbortSignal {
        AbortSignal(self.0.clone())
    }
}

#[derive(Clone)]
pub struct AbortSignal(Arc<AtomicBool>);

impl AbortSignal {
    pub fn is_aborted(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}
