use crate::adapters::{common::AdapterStage, driver::AdapterState};
use crate::gui::{
	components::{self, PaneType},
	messages::Message,
};
use iced::{
	application,
	widget::{pane_grid, text_editor},
	window, Element, Size, Task,
};
use polars::frame::column::Column;
use polars::frame::DataFrame;
use polars::prelude::NamedFrom;
use polars::series::Series;

struct AppState {
	panes: pane_grid::State<PaneType>,
	code: text_editor::Content,
	data_frame: DataFrame,
	status: String,
	adapter_state: AdapterState,
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
			.collect::<Vec<String>>();
		data.push(col);
	}
	let height = data.get(0).unwrap_or(&Default::default()).len();
	let series_vec = header
		.into_iter()
		.zip(data.into_iter())
		.map(|(name, col)| Column::from(Series::new(name.into(), col)))
		.collect::<Vec<Column>>();
	let data_frame = DataFrame::new(height, series_vec).unwrap_or(Default::default());
	let (mut panes, editor_pane) = pane_grid::State::new(PaneType::CodeEditor);
	let _ = panes.split(
		pane_grid::Axis::Horizontal,
		editor_pane,
		PaneType::DataTable,
	);
	AppState {
		panes,
		code: text_editor::Content::new(),
		data_frame,
		status: "".to_string(),
		adapter_state: AdapterState::default(),
		is_maximized: false,
	}
}

fn view(app_state: &AppState) -> Element<'_, Message> {
	components::main_screen(
		&app_state.panes,
		&app_state.code,
		&app_state.data_frame,
		&app_state.status,
		&app_state.adapter_state,
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
		Message::Connect => match app_state.adapter_state.stage {
			AdapterStage::None => {
				app_state.adapter_state.stage = AdapterStage::Unselected;
				app_state.status = "Select an adapter.".into();
			}
			_ => {
				app_state.adapter_state.stage = AdapterStage::None;
				app_state.status = "No adapter selected.".into();
			}
		},
		Message::AdapterSelected(adapter_selection) => {
			app_state.adapter_state.stage = AdapterStage::Unconfigured;
			app_state.status = format!("Adapter selected: {:?}", adapter_selection);
			app_state.adapter_state.selection = adapter_selection;
		}
		Message::AdapterConfigurationChanged(key, value) => {
			app_state.adapter_state.fields.insert(key, value);
			app_state.status = "Configure adapter.".into();
		}
		Message::AdapterConfigurationSubmitted => {
			app_state.adapter_state.configure();
			app_state.status = "Adapter configured.".into();
			app_state.adapter_state.stage = AdapterStage::Configured;
			let config = app_state.adapter_state.configuration.clone();
			return Task::perform(
				async move { AdapterState::establish_connection(config).await },
				Message::AdapterConnected,
			);
		}
		Message::AdapterConnected(dba) => {
			app_state.adapter_state.connection = dba;
			app_state.adapter_state.stage = AdapterStage::Connected;
			app_state.status = "Adapter connected.".into();
		}
		Message::Run => match &app_state.adapter_state.connection {
			None => {}
			Some(db) => {
				let code = app_state.code.text().clone();
				let db = db.clone();
				app_state.status = "Code running...".into();
				return Task::perform(
					async move { db.execute(&code).await.unwrap() },
					Message::DataTable,
				);
			}
		},
		Message::DataTable(df) => {
			println!("DataFrame: {:?}", df);
			app_state.status = "Code finished.".into();
		}
		_ => {}
	}
	Task::none()
}
