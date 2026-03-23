use crate::adapters::{
	common::{AdapterStage, ExecutionResult},
	driver::AdapterState,
};
use crate::gui::{
	components::{self, PaneType},
	messages::{ExportFormat, Message, PlotMessage},
	plot_state::PlotState,
};
use crate::plot::export::{PngBackend, SvgBackend};
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
	let (_data_pane, _) = panes
		.split(
			pane_grid::Axis::Horizontal,
			editor_pane,
			PaneType::DataTable,
		)
		.unwrap();
	let _ = panes
		.split(pane_grid::Axis::Vertical, editor_pane, PaneType::Dashboard)
		.unwrap();
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
		Message::PaneDragged(drag_event) => {
			if let pane_grid::DragEvent::Dropped { pane, target } = drag_event {
				app_state.panes.drop(pane, target);
			}
		}
		Message::DashboardPaneResized(pane_grid::ResizeEvent { split, ratio }) => {
			if let Some(dashboard) = &mut app_state.dashboard {
				dashboard.resize(split, ratio);
			}
		}
		Message::DashboardPaneDragged(drag_event) => {
			if let Some(dashboard) = &mut app_state.dashboard
				&& let pane_grid::DragEvent::Dropped { pane, target } = drag_event {
				dashboard.drop(pane, target);
			}
		}
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
					let rows = df.height();
					let cols = df.width();
					app_state.data_frame = df;
					app_state.status =
						format!("Code finished: {time_elapsed}s | Size: {rows} x {cols}");
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
			if let Some(dashboard) = &mut app_state.dashboard
				&& let Some(plot_state) = dashboard.get_mut(pane) {
				match plot_message {
					PlotMessage::RefreshData => {
						plot_state.refresh(&app_state.data_frame);
					}
					_ => plot_state.update(plot_message),
				}
			}
		}
		Message::Export(format) => {
			return window::latest().and_then(move |id| {
				window::size(id).map(move |size| Message::ExportWithWindowSize(format, Some(size)))
			});
		}
		Message::ExportWithWindowSize(format, window_size) => {
			if let Some(dashboard) = &app_state.dashboard {
				let size = window_size.unwrap_or(Size::new(1920.0, 1080.0));
				let main_grid_width = size.width - 20.0;
				let main_grid_height = size.height - 113.0;
				let main_grid_bounds = iced::Rectangle {
					x: 10.0,
					y: 71.0,
					width: main_grid_width,
					height: main_grid_height,
				};
				let dashboard_pane_bounds = get_pane_rects(app_state.panes.layout(), main_grid_bounds, 2.0)
					.into_iter()
					.find(|(pane, _)| app_state.panes.panes.get(pane) == Some(&PaneType::Dashboard))
					.map(|(_, rect)| rect)
					.unwrap_or(main_grid_bounds);
				for (id, rect) in get_pane_rects(dashboard.layout(), dashboard_pane_bounds, 2.0) {
					if let Some(plot_state) = dashboard.panes.get(&id) {
						let width = rect.width;
						let height = rect.height - 30.0; // Account for plot title bar
						let bounds = iced::Rectangle {
							x: 0.0,
							y: 0.0,
							width,
							height,
						};
						let padding = 20.0;
						let settings = plot_state.plot_settings.clone();
						let _padding_top = padding + settings.plot_padding_top;
						let _padding_bottom = padding + settings.plot_padding_bottom;
						let _padding_left = padding + settings.plot_padding_left;
						let _padding_right = padding + settings.plot_padding_right;
						let widget = crate::plot::common::PlotWidget {
							kernel: plot_state.kernel.as_ref(),
							title: plot_state.current_plot_type.to_string(),
							padding: 20.0,
							settings: settings.clone(),
						};
						match format {
							ExportFormat::SVG => {
								let mut backend = SvgBackend::new(width, height);
								widget.render(&mut backend, bounds);
								let svg_content = backend.finish();
								let filename = format!("plot_{:?}.svg", id);
								let _ = std::fs::write(filename, svg_content);
							}
							ExportFormat::PNG => {
								let mut backend = PngBackend::new(width as u32, height as u32);
								widget.render(&mut backend, bounds);
								let filename = format!("plot_{:?}.png", id);
								backend.save(std::path::Path::new(&filename));
							}
						}
					}
				}
				app_state.status = format!("Exported {} plots as {:?}", dashboard.panes.len(), format);
			}
		}
	}
	Task::none()
}

fn get_pane_rects(
	node: &pane_grid::Node,
	bounds: iced::Rectangle,
	spacing: f32,
) -> Vec<(pane_grid::Pane, iced::Rectangle)> {
	match node {
		pane_grid::Node::Pane(pane) => vec![(*pane, bounds)],
		pane_grid::Node::Split {
			axis,
			ratio,
			a,
			b,
			..
		} => {
			let (rect_a, rect_b) = match axis {
				pane_grid::Axis::Horizontal => {
					let height_a = (bounds.height - spacing) * ratio;
					let height_b = bounds.height - spacing - height_a;
					(
						iced::Rectangle {
							height: height_a,
							..bounds
						},
						iced::Rectangle {
							y: bounds.y + height_a + spacing,
							height: height_b,
							..bounds
						},
					)
				}
				pane_grid::Axis::Vertical => {
					let width_a = (bounds.width - spacing) * ratio;
					let width_b = bounds.width - spacing - width_a;
					(
						iced::Rectangle {
							width: width_a,
							..bounds
						},
						iced::Rectangle {
							x: bounds.x + width_a + spacing,
							width: width_b,
							..bounds
						},
					)
				}
			};
			let mut rects = get_pane_rects(a, rect_a, spacing);
			rects.extend(get_pane_rects(b, rect_b, spacing));
			rects
		}
	}
}
