mod adapters;
mod core;
mod gui;
mod persistence;
mod plot;

fn main() -> gui::Result {
	let saved_window_size = tokio::runtime::Runtime::new()
		.ok()
		.and_then(|rt| rt.block_on(persistence::load_window_size()));
	gui::run(saved_window_size)
}
