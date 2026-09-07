//! One module per page. Each is a view over [`crate::app::App`] plus whatever local state that
//! page needs (a filter, an expanded row), so no page owns server data of its own.

pub mod console;
pub mod overview;
pub mod performance;
pub mod players;
pub mod plugins;
pub mod setup;
pub mod worlds;
