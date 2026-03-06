use crate::adapters::{
	common::{DatabaseAdapter, ExecutionResult},
	driver::AdapterSelection,
};
use iced::{widget::pane_grid, window};
use iced_code_editor::Message as EditorMessage;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub enum Message {
	AdapterConfigurationChanged(String, String),
	AdapterConfigurationSubmitted,
	AdapterConnected(Option<Arc<RwLock<dyn DatabaseAdapter>>>),
	AdapterSelected(AdapterSelection),
	CloseWindow,
	CodeEditEvent(EditorMessage),
	Connect,
	DragWindow,
	MaximizeWindow,
	MinimizeWindow,
	PaneDragged(pane_grid::DragEvent),
	PaneResized(pane_grid::ResizeEvent),
	ResizeWindow(window::Direction),
	Run,
	RunResult(ExecutionResult),
}
