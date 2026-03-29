pub(crate) mod colors;
mod components;
mod menu;
pub mod messages;
pub mod plot_state;
pub mod statusbar;
mod state;
mod table;

pub use state::run;
pub use state::Result;
