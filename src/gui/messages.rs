use iced::{
	widget::{pane_grid, text_editor},
	window,
};

use crate::gui::components::AdapterSelection;

#[derive(Clone)]
pub enum Message {
	AdapterSelected(AdapterSelection),
	AdapterConfigurationChanged(String, String),
	AdapterConfigurationSubmitted,
	CloseWindow,
	CodeAction(text_editor::Action),
	Connect,
	DragWindow,
	MaximizeWindow,
	MinimizeWindow,
	PaneDragged(pane_grid::DragEvent),
	PaneResized(pane_grid::ResizeEvent),
	ResizeWindow(window::Direction),
	Run,
}
