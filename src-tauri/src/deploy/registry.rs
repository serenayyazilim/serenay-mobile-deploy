use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::process::ChildStdin;
use tokio::sync::Mutex as AsyncMutex;

/// 2FA kod girişi için çalışan process'lerin stdin handle'larını tutar.
/// Next.js'teki modül-seviyeli global `Map` hack'inin yerine geçer — burada
/// Tauri managed state olarak uygulama process'i boyunca yaşar.
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
