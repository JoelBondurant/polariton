use iced::{widget::text_editor, window};

#[derive(Clone)]
pub enum Message {
	CloseWindow,
	CodeAction(text_editor::Action),
	DragWindow,
	MaximizeWindow,
	MinimizeWindow,
	ResizeWindow(window::Direction),
	Run,
}
