pub mod detect;
pub mod generic_adapter;
pub mod sermobileboss_adapter;
pub mod sermobileboss_config;
pub mod types;

pub use detect::{detect_workspace_mode, get_adapter};
pub use types::{WorkspaceAdapter, WorkspaceMode, WorkspaceProject};
