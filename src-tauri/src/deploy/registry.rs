use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::process::ChildStdin;
use tokio::sync::Mutex as AsyncMutex;

/// Holds stdin handles of running processes waiting for 2FA code input.
/// Replaces the module-level global `Map` hack from the Next.js version —
/// here it lives as Tauri managed state for the lifetime of the app process.
#[derive(Default)]
pub struct DeployRegistry(
    pub Mutex<HashMap<String, Arc<AsyncMutex<ChildStdin>>>>,
    /// Process group of each running deploy, and whether it was stopped on purpose.
    pub Mutex<HashMap<String, (u32, bool)>>,
);

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

    pub fn set_group(&self, id: &str, pgid: u32) {
        self.1.lock().unwrap().insert(id.to_string(), (pgid, false));
    }

    /// Marks the deploy as stopped on purpose and returns its process group.
    pub fn mark_cancelled(&self, id: &str) -> Option<u32> {
        let mut groups = self.1.lock().unwrap();
        let entry = groups.get_mut(id)?;
        entry.1 = true;
        Some(entry.0)
    }

    /// Forgets the deploy's process group; true when it had been stopped on purpose.
    pub fn take_group(&self, id: &str) -> bool {
        self.1.lock().unwrap().remove(id).is_some_and(|(_, cancelled)| cancelled)
    }
}
