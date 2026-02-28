use iced::{
	widget::{pane_grid, text_editor},
	window,
};

#[derive(Clone)]
pub enum Message {
	CloseWindow,
	CodeAction(text_editor::Action),
	DragWindow,
	MaximizeWindow,
	MinimizeWindow,
	PaneDragged(pane_grid::DragEvent),
	PaneResized(pane_grid::ResizeEvent),
	ResizeWindow(window::Direction),
	Run,
}
