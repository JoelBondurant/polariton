use crate::gui::{components, messages::Message};
use iced::{
	application,
	widget::{pane_grid, text_editor},
	window, Element, Size, Task,
};

pub enum PaneType {
	CodeEditor,
	DataTable,
}

struct AppState {
	panes: pane_grid::State<PaneType>,
	code: text_editor::Content,
	data_tuple: (Vec<String>, Vec<Vec<String>>),
	status: String,
	is_maximized: bool,
}

pub type Result = iced::Result;

pub fn run() -> Result {
	application(new, update, view)
		.theme(components::theme())
		.title("Polariton")
		.window(window::Settings {
			decorations: false,
			maximized: false,
			min_size: Some(Size::new(1280.0, 720.0)),
			position: window::Position::Centered,
			resizable: true,
			size: Size::new(1920.0, 1080.0),
			transparent: false,
			..Default::default()
		})
		.run()
}

fn new() -> AppState {
	let alpha_repeat = 3;
	let header = (1..=alpha_repeat)
		.flat_map(|i| (b'a'..=b'z').map(move |ch| format!("{0}{0}{0}{1}", ch as char, i)))
		.collect::<Vec<String>>();
	let mut data = vec![];
	for offset in 0..26 * alpha_repeat {
		let col = (1..=1_000_000)
			.map(|nx| (nx + offset).to_string())
			.collect();
		data.push(col);
	}
	let data_tuple = (header, data);
	let (mut panes, editor_pane) = pane_grid::State::new(PaneType::CodeEditor);
	let _ = panes.split(
		pane_grid::Axis::Horizontal,
		editor_pane,
		PaneType::DataTable,
	);
	AppState {
		panes,
		code: text_editor::Content::new(),
		data_tuple,
		status: "".to_string(),
		is_maximized: false,
	}
}

fn view(app_state: &AppState) -> Element<'_, Message> {
	components::main_screen(
		&app_state.panes,
		&app_state.code,
		&app_state.data_tuple,
		&app_state.status,
	)
}

fn update(app_state: &mut AppState, message: Message) -> Task<Message> {
	match message {
		Message::CloseWindow => {
			return window::latest().and_then(window::close);
		}
		Message::DragWindow => {
			return window::latest().and_then(window::drag);
		}
		Message::CodeAction(action) => {
			app_state.code.perform(action);
		}
		Message::MaximizeWindow => {
			app_state.is_maximized = !app_state.is_maximized;
			let is_maximized = app_state.is_maximized;
			return window::latest().and_then(move |id| window::maximize(id, is_maximized));
		}
		Message::MinimizeWindow => {
			return window::latest().and_then(move |id| window::minimize(id, true));
		}
		Message::ResizeWindow(direction) => {
			return window::latest().and_then(move |id| window::drag_resize(id, direction));
		}
		Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
			app_state.panes.resize(split, ratio);
		}
		Message::PaneDragged(pane_grid::DragEvent::Dropped { pane, target }) => {
			app_state.panes.drop(pane, target);
		}
		Message::PaneDragged(pane_grid::DragEvent::Canceled { .. }) => {}
		_ => {}
	}
	Task::none()
}
