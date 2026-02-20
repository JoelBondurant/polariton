use crate::gui::{components, messages::Message};
use iced::{application, widget::text_editor, window, Element, Size, Task};

struct AppState {
	code: text_editor::Content,
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
	AppState {
		code: text_editor::Content::new(),
		status: "".to_string(),
		is_maximized: false,
	}
}

fn view(app_state: &AppState) -> Element<'_, Message> {
	components::main_screen(&app_state.code, &app_state.status)
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
		Message::ResizeWindow(edge) => {
			return window::latest().and_then(move |id| window::drag_resize(id, edge));
		}
		_ => {}
	}
	Task::none()
}
