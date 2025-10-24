use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Storage abstraction for persisting session data
pub trait Storage: Send + Sync {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn set(&self, key: &str, value: Vec<u8>);
    fn delete(&self, key: &str);
}

/// In-memory storage implementation (good for WASM or testing)
#[derive(Clone)]
pub struct InMemoryStorage {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage for InMemoryStorage {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.read().ok()?.get(key).cloned()
    }

    fn set(&self, key: &str, value: Vec<u8>) {
        if let Ok(mut data) = self.data.write() {
            data.insert(key.to_string(), value);
        }
    }

    fn delete(&self, key: &str) {
        if let Ok(mut data) = self.data.write() {
            data.remove(key);
        }
    }
}

/// No-op storage implementation for when storage is not needed
#[derive(Clone, Copy)]
pub struct NoStorage;

impl Storage for NoStorage {
    fn get(&self, _key: &str) -> Option<Vec<u8>> {
        None
    }

    fn set(&self, _key: &str, _value: Vec<u8>) {}

    fn delete(&self, _key: &str) {}
}
