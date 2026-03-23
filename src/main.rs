mod adapters;
mod core;
mod gui;
mod persistence;
mod plot;

fn main() -> gui::Result {
	gui::run()
}
