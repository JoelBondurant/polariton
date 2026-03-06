use crate::adapters::{
	common::{DatabaseAdapter, ExecutionResult},
	driver::AdapterSelection,
};
use iced::{
	widget::{pane_grid, text_editor},
	window,
};
use std::sync::Arc;

#[derive(Clone)]
pub enum Message {
	AdapterConfigurationChanged(String, String),
	AdapterConfigurationSubmitted,
	AdapterConnected(Option<Arc<dyn DatabaseAdapter>>),
	AdapterSelected(AdapterSelection),
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
	RunResult(ExecutionResult),
}
