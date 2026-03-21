use crate::adapters::{
	common::{AdapterStage, ExecutionResult},
	driver::AdapterState,
};
use crate::gui::{
	components::{self, PaneType},
	messages::{Message, PlotMessage},
	plot_state::PlotState,
};
use iced::{application, widget::pane_grid, window, Element, Size, Task};
use iced_code_editor::CodeEditor;
use polars::frame::DataFrame;
use std::time::Instant;

struct AppState {
	panes: pane_grid::State<PaneType>,
	dashboard: Option<pane_grid::State<PlotState>>,
	code_editor: CodeEditor,
	data_frame: DataFrame,
	status: String,
	adapter_state: AdapterState,
	code_started: Instant,
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
	let data_frame = DataFrame::default();
	let (mut panes, editor_pane) = pane_grid::State::new(PaneType::CodeEditor);
	let (data_pane, _) = panes
		.split(pane_grid::Axis::Horizontal, editor_pane, PaneType::DataTable)
		.unwrap();

	let _ = panes.split(pane_grid::Axis::Vertical, data_pane, PaneType::Dashboard);

	let mut code_editor = CodeEditor::new("", "sql");
	code_editor.set_theme(iced_code_editor::theme::from_iced_theme(
		&components::theme(),
	));
	AppState {
		panes,
		dashboard: None,
		code_editor,
		data_frame,
		status: "".to_string(),
		adapter_state: AdapterState::default(),
		code_started: Instant::now(),
		is_maximized: false,
	}
}

fn view(app_state: &AppState) -> Element<'_, Message> {
	components::main_screen(
		&app_state.panes,
		&app_state.dashboard,
		&app_state.code_editor,
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
		Message::CodeEditEvent(edit_event) => {
			return app_state
				.code_editor
				.update(&edit_event)
				.map(Message::CodeEditEvent);
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
		Message::DashboardPaneResized(pane_grid::ResizeEvent { split, ratio }) => {
			if let Some(dashboard) = &mut app_state.dashboard {
				dashboard.resize(split, ratio);
			}
		}
		Message::DashboardPaneDragged(pane_grid::DragEvent::Dropped { pane, target }) => {
			if let Some(dashboard) = &mut app_state.dashboard {
				dashboard.drop(pane, target);
			}
		}
		Message::DashboardPaneDragged(pane_grid::DragEvent::Canceled { .. }) => {}
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
				async move { AdapterState::connect(config).await },
				Message::AdapterConnected,
			);
		}
		Message::AdapterConnected(dba) => {
			app_state.adapter_state.connection = dba;
			app_state.adapter_state.stage = AdapterStage::Connected;
			app_state.status = "Adapter connected.".into();
		}
		Message::Run => match &mut app_state.adapter_state.connection {
			None => {}
			Some(db) => {
				let code = app_state.code_editor.content();
				let db = db.clone();
				app_state.status = "Code running...".into();
				app_state.code_started = Instant::now();
				return Task::perform(
					async move {
						let mut guard = db.write().await;
						guard.dispatch(&code).await
					},
					Message::RunResult,
				);
			}
		},
		Message::RunResult(er) => {
			let time_elapsed = (app_state.code_started.elapsed().as_millis() as f64) / 1000.0;
			match er {
				ExecutionResult::Affected(rows_affected) => {
					app_state.status = format!("Rows affected: {rows_affected} in {time_elapsed}s");
				}
				ExecutionResult::Batch(ver) => {
					let msg = ver
						.into_iter()
						.map(|er| format!("{:?}", er))
						.collect::<String>();
					app_state.status = format!("Batch complete {msg} in {time_elapsed}s");
				}
				ExecutionResult::CommandCompleted(msg) => {
					app_state.status = format!("Command complete {msg} in {time_elapsed}s");
				}
				ExecutionResult::Err(msg) => {
					app_state.status = format!("Error {msg} in {time_elapsed}s");
				}
				ExecutionResult::Rows(df) => {
					app_state.data_frame = df;
					app_state.status = format!("Code finished: {time_elapsed}s");
				}
				ExecutionResult::None => {
					app_state.status = format!("Noop finished: {time_elapsed}s");
				}
			}
		}
		Message::AddPlot(plot_type) => {
			let new_plot = PlotState::new(plot_type, &app_state.data_frame, 1200, 1200);
			if let Some(dashboard) = &mut app_state.dashboard {
				let last_pane = *dashboard.panes.keys().next().unwrap();
				let _ = dashboard.split(pane_grid::Axis::Horizontal, last_pane, new_plot);
			} else {
				let (dashboard, _) = pane_grid::State::new(new_plot);
				app_state.dashboard = Some(dashboard);
			}
		}
		Message::ClosePlot(pane) => {
			if let Some(dashboard) = &mut app_state.dashboard {
				if dashboard.panes.len() <= 1 {
					app_state.dashboard = None;
				} else {
					let _ = dashboard.close(pane);
				}
			}
		}
		Message::PlotEvent(pane, plot_message) => {
			if let Some(dashboard) = &mut app_state.dashboard {
				if let Some(plot_state) = dashboard.get_mut(pane) {
					match plot_message {
						PlotMessage::RefreshData => {
							plot_state.refresh(&app_state.data_frame);
						}
						_ => plot_state.update(plot_message),
					}
				}
			}
		}
		_ => {}
	}
	Task::none()
}
