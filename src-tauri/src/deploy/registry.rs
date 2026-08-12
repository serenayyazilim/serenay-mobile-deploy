use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::process::ChildStdin;
use tokio::sync::Mutex as AsyncMutex;

/// Holds stdin handles of running processes waiting for 2FA code input.
/// Replaces the module-level global `Map` hack from the Next.js version —
/// here it lives as Tauri managed state for the lifetime of the app process.
#[derive(Default)]
pub struct DeployRegistry(pub Mutex<HashMap<String, Arc<AsyncMutex<ChildStdin>>>>);

impl DeployRegistry {
    pub fn insert(&self, id: String, stdin: ChildStdin) {
        self.0.lock().unwrap().insert(id, Arc::new(AsyncMutex::new(stdin)));
    }

    pub fn get(&self, id: &str) -> Option<Arc<AsyncMutex<ChildStdin>>> {
        self.0.lock().unwrap().get(id).cloned()
    }

    pub fn remove(&self, id: &str) {
        self.0.lock().unwrap().remove(id);
    }
}
