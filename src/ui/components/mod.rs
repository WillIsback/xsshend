/// Composants modulaires pour l'interface TUI
pub mod header;
pub mod progress;
pub mod logs;
pub mod status;
pub mod controls;
pub mod layout;

pub use header::Header;
pub use progress::ProgressView;
pub use logs::LogsView;
pub use status::StatusBar;
pub use controls::Controls;
pub use layout::AppLayout;
