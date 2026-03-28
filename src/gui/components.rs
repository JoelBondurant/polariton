use crate::adapters::{
	common::{AdapterFieldType, AdapterStage},
	driver::{fields_for, AdapterSelection, AdapterState},
};
use crate::gui::messages::{Message, PlotMessage};
use crate::gui::plot_state::PlotState;
use crate::gui::{
	colors,
	menu::{MenuBar, MenuFontPolicy, MenuItem, MenuRoot, MenuState},
	table::Table,
};
use crate::persistence::{SavedConnection, SavedStatement};
use crate::plot::colors::ColorTheme;
use crate::plot::common::{GridLineStyle, PlotRenderLayer, PlotWidget, ScatterRenderMode};
use crate::plot::core::PlotType;
use iced::widget::{
	button, canvas, center, checkbox, column, container, mouse_area, opaque, pane_grid, pick_list,
	row, scrollable, space, stack, text, text_input, TextInput,
};
use iced::{
	border, font, mouse,
	theme::{Palette, Theme},
	window::Direction,
	Alignment, Background, Center, Color, Element, Fill, FillPortion, Font, Length,
};
use iced_code_editor::CodeEditor;
use polars::frame::DataFrame;

pub const BUTTON_SIZE_DEFAULT: (u32, u32) = (120, 40);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneType {
	CodeEditor,
	DataTable,
	Dashboard,
}

pub fn theme() -> Theme {
	Theme::custom(
		"BlackHole".to_string(),
		Palette {
			background: colors::BG_PRIMARY,
			danger: colors::DANGER,
			primary: colors::PRIMARY,
			success: colors::SUCCESS,
			text: colors::TEXT_PRIMARY,
			warning: colors::WARNING,
		},
	)
}

pub fn title_bar<'a>() -> Element<'a, Message> {
	let width = 34;
	let height = 30;
	let font_size = 16;
	container(
		row![
			mouse_area(container(row![
				space::horizontal(),
				space::horizontal().width(width),
				text("Polariton")
					.size(font_size)
					.font(Font {
						weight: font::Weight::Bold,
						..Default::default()
					})
					.color(colors::WHITE),
				space::horizontal()
			]))
			.on_press(Message::DragWindow),
			button(
				text("—")
					.font(Font {
						weight: font::Weight::Bold,
						..Default::default()
					})
					.size(font_size)
					.align_y(Center)
					.align_x(Center)
			)
			.width(width)
			.height(height)
			.style(|_theme: &Theme, status: button::Status| match status {
				button::Status::Hovered => button::Style {
					background: Some(Background::Color(colors::BRAND_PURPLE)),
					text_color: colors::TEXT_TITLE_BUTTON_HOVER,
					..button::Style::default()
				},
				_ => button::Style {
					background: Some(Background::Color(Color::TRANSPARENT)),
					text_color: colors::TEXT_TITLE_BUTTON,
					..button::Style::default()
				},
			})
			.on_press(Message::MinimizeWindow),
			button(
				text("□")
					.font(Font {
						weight: font::Weight::Bold,
						..Default::default()
					})
					.size(font_size)
					.align_y(Center)
					.align_x(Center)
			)
			.width(width)
			.height(height)
			.style(|_theme: &Theme, status: button::Status| match status {
				button::Status::Hovered => button::Style {
					background: Some(Background::Color(colors::BRAND_PURPLE)),
					text_color: colors::TEXT_TITLE_BUTTON_HOVER,
					..button::Style::default()
				},
				_ => button::Style {
					background: Some(Background::Color(Color::TRANSPARENT)),
					text_color: colors::TEXT_TITLE_BUTTON,
					..button::Style::default()
				},
			})
			.on_press(Message::MaximizeWindow),
			button(
				text("✕")
					.font(Font {
						weight: font::Weight::Bold,
						..Default::default()
					})
					.size(font_size)
					.align_y(Center)
					.align_x(Center)
			)
			.width(width)
			.height(height)
			.style(|_theme: &Theme, status: button::Status| match status {
				button::Status::Hovered => button::Style {
					background: Some(Background::Color(colors::BRAND_PURPLE)),
					text_color: colors::TEXT_TITLE_BUTTON_HOVER,
					..button::Style::default()
				},
				_ => button::Style {
					background: Some(Background::Color(Color::TRANSPARENT)),
					text_color: colors::TEXT_TITLE_BUTTON,
					..button::Style::default()
				},
			})
			.on_press(Message::CloseWindow),
		]
		.padding(0)
		.align_y(iced::Center),
	)
	.width(Fill)
	.height(height)
	.style(|_theme| container::Style {
		background: Some(colors::BG_SECONDARY.into()),
		border: border::Border {
			color: colors::WHITE,
			width: 0.2,
			radius: 0.0.into(),
		},
		..Default::default()
	})
	.into()
}

fn pane_title_bar<'a>(_pane_type: PaneType) -> pane_grid::TitleBar<'a, Message> {
	pane_grid::TitleBar::new(
		container(space::horizontal().width(Fill))
			.width(Fill)
			.padding(5)
			.style(|_| container::Style {
				background: Some(Background::Color(colors::BG_SECONDARY)),
				..Default::default()
			}),
	)
	.padding(2)
}

pub fn main_screen<'a>(
	panes: &'a pane_grid::State<PaneType>,
	dashboard: &'a Option<pane_grid::State<PlotState>>,
	menu_state: &'a MenuState,
	code_editor: &'a CodeEditor,
	data_frame: &'a DataFrame,
	status_msg: &'a str,
	status_error: &'a str,
	status_df_size: Option<(usize, usize)>,
	status_time_elapsed: Option<f64>,
	adapter_state: &'a AdapterState,
	saved_connections: &'a [SavedConnection],
	saved_statements: &'a [SavedStatement],
	showing_password_prompt: bool,
	password_entry: &'a str,
	password_entry_error: &'a str,
	showing_settings: bool,
	settings_new_password: &'a str,
	settings_confirm_password: &'a str,
	settings_error: &'a str,
	is_password_protected: bool,
	show_column_types: bool,
	showing_save_statement_dialog: bool,
	save_statement_name: &'a str,
	editing_statement_id: Option<i64>,
) -> Element<'a, Message> {
	let main_pane = pane_grid(panes, |_id, pane_type, _is_maximized| match pane_type {
		PaneType::CodeEditor => pane_grid::Content::new(center(
			container(code_editor.view().map(Message::CodeEditEvent))
				.padding(1)
				.style(|_| container::Style {
					border: border::Border {
						color: colors::BORDER_PRIMARY,
						width: 1.0,
						radius: 5.0.into(),
					},
					..Default::default()
				}),
		))
		.title_bar(pane_title_bar(PaneType::CodeEditor)),
		PaneType::DataTable => pane_grid::Content::new(center(
			Table::new(data_frame, 0).show_column_types(show_column_types),
		))
		.title_bar(pane_title_bar(PaneType::DataTable)),
		PaneType::Dashboard => pane_grid::Content::new(if let Some(dashboard) = dashboard {
			dashboard_view(dashboard)
		} else {
			center(text("").color(colors::TEXT_SECONDARY)).into()
		})
		.title_bar(pane_title_bar(PaneType::Dashboard)),
	})
	.width(Fill)
	.height(Fill)
	.spacing(2)
	.on_drag(Message::PaneDragged)
	.on_resize(10, Message::PaneResized);
	let main_content = container(main_pane).padding(4).width(Fill);
	let status_cell_style = |_: &Theme| container::Style {
		border: border::Border {
			color: colors::BORDER_DIM,
			width: 1.0,
			radius: 3.0.into(),
		},
		..Default::default()
	};
	let time_str = status_time_elapsed
		.map(|t| format!("{t:.3}s"))
		.unwrap_or_default();
	let size_str = status_df_size
		.map(|(r, c)| format!("{r} \u{00d7} {c}"))
		.unwrap_or_default();
	let time_box = container(center(
		text(time_str).color(colors::TEXT_SECONDARY).size(14),
	))
	.width(100)
	.height(Fill)
	.padding([0, 4])
	.style(status_cell_style);
	let size_box = container(center(
		text(size_str).color(colors::TEXT_SECONDARY).size(14),
	))
	.width(140)
	.height(Fill)
	.padding([0, 4])
	.style(status_cell_style);
	let msg_box = container(center(text(status_msg).color(colors::TEXT_STATUS).size(14)))
		.width(Fill)
		.height(Fill)
		.padding([0, 6])
		.style(status_cell_style);
	let error_box = container(center(text(status_error).color(colors::DANGER).size(14)))
		.width(Fill)
		.height(Fill)
		.padding([0, 6])
		.style(status_cell_style);
	let status_bar = container(row![time_box, size_box, msg_box, error_box].spacing(3))
		.height(28)
		.padding([2, 4])
		.width(Fill);
	let main_window = window_decorations(
		column![main_content, status_bar],
		menu_state,
		saved_connections,
		saved_statements,
	);
	let adapter_modal = adapter_view(adapter_state);
	let password_modal: Element<Message> = if showing_password_prompt {
		password_prompt_view(password_entry, password_entry_error)
	} else {
		container(text("")).into()
	};
	let settings_modal: Element<Message> = if showing_settings {
		settings_dialog_view(
			settings_new_password,
			settings_confirm_password,
			settings_error,
			is_password_protected,
			show_column_types,
		)
	} else {
		container(text("")).into()
	};
	let save_statement_modal: Element<Message> = if showing_save_statement_dialog {
		save_statement_dialog_view(save_statement_name, editing_statement_id.is_some())
	} else {
		container(text("")).into()
	};
	stack![main_window, adapter_modal, password_modal, settings_modal, save_statement_modal].into()
}

pub fn menu_bar<'a>(
	menu_state: &'a MenuState,
	saved_connections: &'a [SavedConnection],
	saved_statements: &'a [SavedStatement],
) -> Element<'a, Message> {
	Element::from(MenuBar::new(
		build_menu_roots(saved_connections, saved_statements),
		menu_state,
	)
	.font_policy(MenuFontPolicy::SystemWithFallback))
	.map(Message::Menu)
}

fn build_menu_roots(
	saved_connections: &[SavedConnection],
	saved_statements: &[SavedStatement],
) -> Vec<MenuRoot> {
	vec![
		MenuRoot {
			id: "connect".into(),
			label: "Connect".into(),
			items: vec![
				MenuItem::Action {
					id: "connect:new".into(),
					label: "New".into(),
				},
				MenuItem::Separator,
				MenuItem::Submenu {
					id: "connect:saved".into(),
					label: "Saved".into(),
					items: build_saved_connection_items(saved_connections),
				},
			],
		},
		MenuRoot {
			id: "code".into(),
			label: "Code".into(),
			items: vec![
				MenuItem::Action {
					id: "code:run".into(),
					label: "Run  (Ctrl+Enter)".into(),
				},
				MenuItem::Action {
					id: "code:save".into(),
					label: "Save...".into(),
				},
				MenuItem::Separator,
				MenuItem::Submenu {
					id: "code:saved".into(),
					label: "Saved".into(),
					items: build_saved_statement_items(saved_statements),
				},
			],
		},
		MenuRoot {
			id: "plot".into(),
			label: "Plot".into(),
			items: vec![
				MenuItem::Submenu {
					id: "plot:new".into(),
					label: "New".into(),
					items: crate::plot::core::PlotType::ALL
						.iter()
						.map(|plot_type| MenuItem::Action {
							id: format!("plot:new:{plot_type:?}"),
							label: plot_type.to_string(),
						})
						.collect(),
				},
				MenuItem::Submenu {
					id: "plot:export".into(),
					label: "Export".into(),
					items: [
						crate::gui::messages::ExportFormat::SVG,
						crate::gui::messages::ExportFormat::PNG,
						crate::gui::messages::ExportFormat::AVIF,
					]
					.into_iter()
					.map(|format| MenuItem::Action {
						id: format!("plot:export:{format}"),
						label: format.to_string(),
					})
					.collect(),
				},
			],
		},
		MenuRoot {
			id: "settings".into(),
			label: "Settings".into(),
			items: vec![MenuItem::Action {
				id: "settings:preferences".into(),
				label: "Preferences".into(),
			}],
		},
	]
}

fn build_saved_connection_items(saved_connections: &[SavedConnection]) -> Vec<MenuItem> {
	if saved_connections.is_empty() {
		return vec![MenuItem::Action {
			id: "noop".into(),
			label: "None saved".into(),
		}];
	}

	saved_connections
		.iter()
		.map(|conn| MenuItem::Submenu {
			id: format!("connect:saved:{}", conn.id),
			label: conn.name.clone(),
			items: vec![
				MenuItem::Action {
					id: format!("connect:load:{}", conn.id),
					label: "Load".into(),
				},
				MenuItem::Action {
					id: format!("connect:edit:{}", conn.id),
					label: "Edit".into(),
				},
				MenuItem::Action {
					id: format!("connect:delete:{}", conn.id),
					label: "Delete".into(),
				},
			],
		})
		.collect()
}

fn build_saved_statement_items(saved_statements: &[SavedStatement]) -> Vec<MenuItem> {
	if saved_statements.is_empty() {
		return vec![MenuItem::Action {
			id: "noop".into(),
			label: "None saved".into(),
		}];
	}

	saved_statements
		.iter()
		.map(|stmt| MenuItem::Submenu {
			id: format!("code:saved:{}", stmt.id),
			label: stmt.name.clone(),
			items: vec![
				MenuItem::Action {
					id: format!("code:load:{}", stmt.id),
					label: "Load".into(),
				},
				MenuItem::Action {
					id: format!("code:edit:{}", stmt.id),
					label: "Edit".into(),
				},
				MenuItem::Action {
					id: format!("code:delete:{}", stmt.id),
					label: "Delete".into(),
				},
			],
		})
		.collect()
}

fn dashboard_view<'a>(state: &'a pane_grid::State<PlotState>) -> Element<'a, Message> {
	pane_grid(state, |id, plot_state, _is_maximized| {
		pane_grid::Content::new(plot_view(id, plot_state)).title_bar(
			pane_grid::TitleBar::new(
				row![
					container(space::horizontal().width(Fill))
						.padding(5)
						.width(Fill)
						.style(|_| container::Style {
							background: Some(Background::Color(colors::BG_SECONDARY)),
							text_color: Some(colors::TEXT_PRIMARY),
							..Default::default()
						}),
					button(
						text("✕")
							.font(Font {
								weight: font::Weight::Bold,
								..Default::default()
							})
							.size(12)
							.align_y(Center)
							.align_x(Center)
					)
					.width(30)
					.height(26)
					.style(|_theme: &Theme, status: button::Status| match status {
						button::Status::Hovered => button::Style {
							background: Some(Background::Color(colors::BRAND_PURPLE)),
							text_color: colors::TEXT_TITLE_BUTTON_HOVER,
							..button::Style::default()
						},
						_ => button::Style {
							background: Some(Background::Color(Color::TRANSPARENT)),
							text_color: colors::TEXT_TITLE_BUTTON,
							..button::Style::default()
						},
					})
					.on_press(Message::ClosePlot(id))
				]
				.align_y(Center),
			)
			.padding(2),
		)
	})
	.width(Fill)
	.height(Fill)
	.spacing(2)
	.on_drag(Message::DashboardPaneDragged)
	.on_resize(10, Message::DashboardPaneResized)
	.into()
}

fn plot_view<'a>(id: pane_grid::Pane, state: &'a PlotState) -> Element<'a, Message> {
	let data_canvas: Element<PlotMessage> = canvas(PlotWidget {
		kernel: state.kernel.as_ref(),
		title: state.current_plot_type.to_string(),
		padding: 20.0,
		settings: state.plot_settings.clone(),
		render_revision: state.render_revision,
		resize_render_suspended: state.resize_render_suspended,
		layer: PlotRenderLayer::Data,
	})
	.width(Fill)
	.height(Fill)
	.into();
	let overlay_canvas: Element<PlotMessage> = canvas(PlotWidget {
		kernel: state.kernel.as_ref(),
		title: state.current_plot_type.to_string(),
		padding: 20.0,
		settings: state.plot_settings.clone(),
		render_revision: state.render_revision,
		resize_render_suspended: state.resize_render_suspended,
		layer: PlotRenderLayer::OverlayInteractive,
	})
	.width(Fill)
	.height(Fill)
	.into();
	let plot_content: Element<PlotMessage> = stack![data_canvas, overlay_canvas].into();
	let plot_content = plot_content.map(move |pm| Message::PlotEvent(id, pm));
	let mut main_stack = stack![plot_content];
	if let Some(info) = &state.hovered_info {
		main_stack = main_stack.push(
			container(text(info))
				.padding(6)
				.style(|_| container::Style {
					background: Some(iced::Background::Color(iced::Color {
						a: 0.85,
						..state.plot_settings.background_color
					})),
					border: iced::Border {
						color: iced::Color {
							a: 0.2,
							..state.plot_settings.decoration_color
						},
						width: 1.0,
						radius: 2.0.into(),
					},
					text_color: Some(state.plot_settings.decoration_color),
					..Default::default()
				}),
		);
	}
	if state.settings_open {
		let settings_panel = plot_settings_panel(id, state);
		let modal_overlay = container(opaque(
			row![space::horizontal(), settings_panel].width(Fill),
		))
		.width(Fill)
		.height(Fill)
		.style(|_| container::Style {
			background: Some(Background::Color(Color {
				a: 0.2,
				..Color::BLACK
			})),
			..Default::default()
		});
		main_stack = main_stack.push(modal_overlay);
	}
	container(main_stack.width(Fill).height(Fill))
		.style(|_| container::Style {
			background: Some(Background::Color(state.plot_settings.background_color)),
			text_color: Some(state.plot_settings.decoration_color),
			..Default::default()
		})
		.into()
}

fn plot_settings_panel<'a>(id: pane_grid::Pane, state: &'a PlotState) -> Element<'a, Message> {
	let plot_event = move |pm| Message::PlotEvent(id, pm);
	container(
		column![
			row![
				text("Plot Settings").size(24),
				space::horizontal(),
				button("Refresh Data").on_press(plot_event(PlotMessage::RefreshData)),
				space::horizontal(),
				button("Close").on_press(plot_event(PlotMessage::CloseSettings))
			]
			.align_y(Alignment::Center),
			scrollable(
				column![
					section(
						"General",
						column![
							row![
								checkbox(state.live_updates_enabled)
									.label("Live Updates")
									.on_toggle(move |v| {
										plot_event(PlotMessage::ToggleLiveUpdates(v))
									}),
								space::horizontal(),
								button("Apply").on_press(plot_event(PlotMessage::ApplySettings)),
							]
							.align_y(Alignment::Center),
							horizontal_rule(),
							field(
								"Plot Type",
								pick_list(
									&PlotType::ALL[..],
									Some(state.current_plot_type),
									move |pt| plot_event(PlotMessage::ChangePlotType(pt))
								)
							),
							field(
								"Theme",
								pick_list(
									&ColorTheme::ALL[..],
									Some(state.plot_settings.color_theme),
									move |ct| plot_event(PlotMessage::ChangeColorTheme(ct))
								)
							),
							horizontal_rule(),
							field(
								"Scatter Mode",
								pick_list(
									&ScatterRenderMode::ALL[..],
									Some(state.plot_settings.scatter_render_mode),
									move |mode| plot_event(PlotMessage::SetScatterRenderMode(mode))
								)
							),
							field(
								"Vector Limit",
								text_input("", &state.scatter_max_vector_points_input).on_input(
									move |s| {
										if let Ok(val) = s.parse::<u32>() {
											plot_event(PlotMessage::SetScatterMaxVectorPoints(val))
										} else {
											plot_event(PlotMessage::UpdateHover(
												state.hovered_info.clone(),
											))
										}
									}
								)
							),
							field(
								"Sample Target",
								text_input("", &state.scatter_downsample_target_input).on_input(
									move |s| {
										if let Ok(val) = s.parse::<u32>() {
											plot_event(PlotMessage::SetScatterDownsampleTarget(val))
										} else {
											plot_event(PlotMessage::UpdateHover(
												state.hovered_info.clone(),
											))
										}
									}
								)
							),
							field(
								"Raster Threshold",
								text_input("", &state.scatter_raster_threshold_input).on_input(
									move |s| {
										if let Ok(val) = s.parse::<u32>() {
											plot_event(PlotMessage::SetScatterRasterThreshold(val))
										} else {
											plot_event(PlotMessage::UpdateHover(
												state.hovered_info.clone(),
											))
										}
									}
								)
							),
						]
					),
					section(
						"Titles",
						column![
							field(
								"Title",
								text_input("auto", &state.title_input).on_input(move |s| {
									if s.is_empty() {
										plot_event(PlotMessage::SetTitle(None))
									} else {
										plot_event(PlotMessage::SetTitle(Some(s)))
									}
								})
							),
							field(
								"Title Size",
								text_input("", &state.title_size_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f32>() {
										plot_event(PlotMessage::SetTitleSize(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Title Offset",
								text_input("", &state.title_offset_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f32>() {
										plot_event(PlotMessage::SetTitleOffset(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Subtitle",
								text_input("none", &state.subtitle_input).on_input(move |s| {
									if s.is_empty() {
										plot_event(PlotMessage::SetSubtitle(None))
									} else {
										plot_event(PlotMessage::SetSubtitle(Some(s)))
									}
								})
							),
							field(
								"Subtitle Size",
								text_input("", &state.subtitle_size_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f32>() {
										plot_event(PlotMessage::SetSubtitleSize(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Subtitle Offset",
								text_input("", &state.subtitle_offset_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f32>() {
										plot_event(PlotMessage::SetSubtitleOffset(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
						]
					),
					section(
						"X Axis",
						column![
							field(
								"Label",
								text_input("auto", &state.x_label_input).on_input(move |s| {
									if s.is_empty() {
										plot_event(PlotMessage::SetXLabel(None))
									} else {
										plot_event(PlotMessage::SetXLabel(Some(s)))
									}
								})
							),
							field(
								"Label Size",
								text_input("", &state.x_label_size_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f32>() {
										plot_event(PlotMessage::SetXLabelSize(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Label Padding",
								text_input("", &state.x_label_padding_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f32>() {
										plot_event(PlotMessage::SetXLabelPadding(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							horizontal_rule(),
							field(
								"Major Ticks",
								text_input("", &state.x_ticks_input).on_input(move |s| {
									if let Ok(val) = s.parse::<u32>() {
										plot_event(PlotMessage::SetXTicks(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Tick Size",
								text_input("", &state.x_tick_size_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f32>() {
										plot_event(PlotMessage::SetXTickSize(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							checkbox(state.plot_settings.show_x_minor_ticks)
								.label("Show Minor Ticks")
								.on_toggle(move |v| plot_event(PlotMessage::ToggleXMinorTicks(v))),
							field(
								"Minor Ticks",
								text_input("", &state.x_minor_ticks_input).on_input(move |s| {
									if let Ok(val) = s.parse::<u32>() {
										plot_event(PlotMessage::SetXMinorTicks(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							horizontal_rule(),
							checkbox(state.plot_settings.show_x_major_grid)
								.label("Show Major Grid")
								.on_toggle(move |v| plot_event(PlotMessage::ToggleXMajorGrid(v))),
							field(
								"Grid Width",
								text_input("", &state.x_major_grid_width_input).on_input(
									move |s| {
										if let Ok(val) = s.parse::<f32>() {
											plot_event(PlotMessage::SetXMajorGridWidth(val))
										} else {
											plot_event(PlotMessage::UpdateHover(
												state.hovered_info.clone(),
											))
										}
									}
								)
							),
							field(
								"Grid Style",
								pick_list(
									&GridLineStyle::ALL[..],
									Some(state.plot_settings.x_major_grid_style),
									move |s| plot_event(PlotMessage::SetXMajorGridStyle(s))
								)
							),
							horizontal_rule(),
							checkbox(state.plot_settings.show_x_minor_grid)
								.label("Show Minor Grid")
								.on_toggle(move |v| plot_event(PlotMessage::ToggleXMinorGrid(v))),
							field(
								"Minor Grid Width",
								text_input("", &state.x_minor_grid_width_input).on_input(
									move |s| {
										if let Ok(val) = s.parse::<f32>() {
											plot_event(PlotMessage::SetXMinorGridWidth(val))
										} else {
											plot_event(PlotMessage::UpdateHover(
												state.hovered_info.clone(),
											))
										}
									}
								)
							),
							field(
								"Minor Grid Style",
								pick_list(
									&GridLineStyle::ALL[..],
									Some(state.plot_settings.x_minor_grid_style),
									move |s| plot_event(PlotMessage::SetXMinorGridStyle(s))
								)
							),
						]
						.spacing(10)
					),
					section(
						"Y Axis",
						column![
							field(
								"Label",
								text_input("auto", &state.y_label_input).on_input(move |s| {
									if s.is_empty() {
										plot_event(PlotMessage::SetYLabel(None))
									} else {
										plot_event(PlotMessage::SetYLabel(Some(s)))
									}
								})
							),
							field(
								"Label Size",
								text_input("", &state.y_label_size_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f32>() {
										plot_event(PlotMessage::SetYLabelSize(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Label Padding",
								text_input("", &state.y_label_padding_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f32>() {
										plot_event(PlotMessage::SetYLabelPadding(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							horizontal_rule(),
							field(
								"Major Ticks",
								text_input("", &state.y_ticks_input).on_input(move |s| {
									if let Ok(val) = s.parse::<u32>() {
										plot_event(PlotMessage::SetYTicks(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Tick Size",
								text_input("", &state.y_tick_size_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f32>() {
										plot_event(PlotMessage::SetYTickSize(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							checkbox(state.plot_settings.show_y_minor_ticks)
								.label("Show Minor Ticks")
								.on_toggle(move |v| plot_event(PlotMessage::ToggleYMinorTicks(v))),
							field(
								"Minor Ticks",
								text_input("", &state.y_minor_ticks_input).on_input(move |s| {
									if let Ok(val) = s.parse::<u32>() {
										plot_event(PlotMessage::SetYMinorTicks(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							horizontal_rule(),
							checkbox(state.plot_settings.show_y_major_grid)
								.label("Show Major Grid")
								.on_toggle(move |v| plot_event(PlotMessage::ToggleYMajorGrid(v))),
							field(
								"Grid Width",
								text_input("", &state.y_major_grid_width_input).on_input(
									move |s| {
										if let Ok(val) = s.parse::<f32>() {
											plot_event(PlotMessage::SetYMajorGridWidth(val))
										} else {
											plot_event(PlotMessage::UpdateHover(
												state.hovered_info.clone(),
											))
										}
									}
								)
							),
							field(
								"Grid Style",
								pick_list(
									&GridLineStyle::ALL[..],
									Some(state.plot_settings.y_major_grid_style),
									move |s| plot_event(PlotMessage::SetYMajorGridStyle(s))
								)
							),
							horizontal_rule(),
							checkbox(state.plot_settings.show_y_minor_grid)
								.label("Show Minor Grid")
								.on_toggle(move |v| plot_event(PlotMessage::ToggleYMinorGrid(v))),
							field(
								"Minor Grid Width",
								text_input("", &state.y_minor_grid_width_input).on_input(
									move |s| {
										if let Ok(val) = s.parse::<f32>() {
											plot_event(PlotMessage::SetYMinorGridWidth(val))
										} else {
											plot_event(PlotMessage::UpdateHover(
												state.hovered_info.clone(),
											))
										}
									}
								)
							),
							field(
								"Minor Grid Style",
								pick_list(
									&GridLineStyle::ALL[..],
									Some(state.plot_settings.y_minor_grid_style),
									move |s| plot_event(PlotMessage::SetYMinorGridStyle(s))
								)
							),
						]
						.spacing(10)
					),
					section(
						"Plot Padding",
						column![
							field(
								"Top",
								text_input("", &state.plot_padding_top_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f32>() {
										plot_event(PlotMessage::SetPlotPaddingTop(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Bottom",
								text_input("", &state.plot_padding_bottom_input).on_input(
									move |s| {
										if let Ok(val) = s.parse::<f32>() {
											plot_event(PlotMessage::SetPlotPaddingBottom(val))
										} else {
											plot_event(PlotMessage::UpdateHover(
												state.hovered_info.clone(),
											))
										}
									}
								)
							),
							field(
								"Left",
								text_input("", &state.plot_padding_left_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f32>() {
										plot_event(PlotMessage::SetPlotPaddingLeft(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Right",
								text_input("", &state.plot_padding_right_input).on_input(
									move |s| {
										if let Ok(val) = s.parse::<f32>() {
											plot_event(PlotMessage::SetPlotPaddingRight(val))
										} else {
											plot_event(PlotMessage::UpdateHover(
												state.hovered_info.clone(),
											))
										}
									}
								)
							),
						]
					),
					section(
						"Colors",
						column![
							field(
								"Background",
								text_input("", &state.bg_color_input).on_input(move |s| {
									plot_event(PlotMessage::ChangeBackgroundHex(s))
								})
							),
							field(
								"Decoration",
								text_input("", &state.decoration_color_input).on_input(move |s| {
									plot_event(PlotMessage::ChangeDecorationHex(s))
								})
							),
						]
					),
					section(
						"Legend",
						column![
							field(
								"Max Rows",
								text_input("", &state.max_legend_rows_input).on_input(move |s| {
									if let Ok(rows) = s.parse::<u32>() {
										plot_event(PlotMessage::SetMaxLegendRows(rows))
									} else if s.is_empty() {
										plot_event(PlotMessage::SetMaxLegendRows(0))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Legend Size",
								text_input("", &state.legend_size_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f32>() {
										plot_event(PlotMessage::SetLegendSize(val))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"X (0-1)",
								text_input("", &state.legend_x_input).on_input(move |s| {
									if let Ok(x) = s.parse::<f32>() {
										plot_event(PlotMessage::SetLegendX(x))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Y (0-1)",
								text_input("", &state.legend_y_input).on_input(move |s| {
									if let Ok(y) = s.parse::<f32>() {
										plot_event(PlotMessage::SetLegendY(y))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
						]
					),
					section(
						"X Axis Labels",
						column![
							field(
								"Rotation",
								text_input("", &state.x_rotation_input).on_input(move |s| {
									if let Ok(deg) = s.parse::<f32>() {
										plot_event(PlotMessage::SetXRotation(deg))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Offset",
								text_input("", &state.x_offset_input).on_input(move |s| {
									if let Ok(offset) = s.parse::<f32>() {
										plot_event(PlotMessage::SetXOffset(offset))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
						]
					),
					section(
						"X Axis Range",
						column![
							field(
								"Min",
								text_input("auto", &state.x_min_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f64>() {
										plot_event(PlotMessage::SetXMin(Some(val)))
									} else if s.is_empty() {
										plot_event(PlotMessage::SetXMin(None))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Max",
								text_input("auto", &state.x_max_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f64>() {
										plot_event(PlotMessage::SetXMax(Some(val)))
									} else if s.is_empty() {
										plot_event(PlotMessage::SetXMax(None))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
						]
					),
					section(
						"Y Axis Range",
						column![
							field(
								"Min",
								text_input("auto", &state.y_min_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f64>() {
										plot_event(PlotMessage::SetYMin(Some(val)))
									} else if s.is_empty() {
										plot_event(PlotMessage::SetYMin(None))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
							field(
								"Max",
								text_input("auto", &state.y_max_input).on_input(move |s| {
									if let Ok(val) = s.parse::<f64>() {
										plot_event(PlotMessage::SetYMax(Some(val)))
									} else if s.is_empty() {
										plot_event(PlotMessage::SetYMax(None))
									} else {
										plot_event(PlotMessage::UpdateHover(
											state.hovered_info.clone(),
										))
									}
								})
							),
						]
					),
				]
				.spacing(20)
			)
			.direction(scrollable::Direction::Vertical(
				scrollable::Scrollbar::new()
					.width(4)
					.margin(2)
					.scroller_width(4)
			))
			.style(|_theme, _status| scrollable::Style {
				container: container::Style::default(),
				vertical_rail: scrollable::Rail {
					background: Some(Background::Color(Color::TRANSPARENT)),
					border: border::Border::default(),
					scroller: scrollable::Scroller {
						background: Background::Color(colors::SCROLLBAR_THUMB),
						border: border::Border {
							radius: 2.0.into(),
							..Default::default()
						},
					},
				},
				horizontal_rail: scrollable::Rail {
					background: Some(Background::Color(Color::TRANSPARENT)),
					border: border::Border::default(),
					scroller: scrollable::Scroller {
						background: Background::Color(colors::SCROLLBAR_THUMB),
						border: border::Border {
							radius: 2.0.into(),
							..Default::default()
						},
					},
				},
				gap: None,
				auto_scroll: scrollable::AutoScroll {
					background: Background::Color(Color::TRANSPARENT),
					border: border::Border::default(),
					shadow: iced::Shadow::default(),
					icon: Color::TRANSPARENT,
				},
			})
		]
		.spacing(20)
		.padding(20),
	)
	.width(400)
	.height(Fill)
	.style(move |_| container::Style {
		background: Some(Background::Color(Color {
			a: 0.95,
			..state.plot_settings.background_color
		})),
		border: border::Border {
			color: state.plot_settings.decoration_color,
			width: 1.0,
			radius: 0.0.into(),
		},
		text_color: Some(state.plot_settings.decoration_color),
		..Default::default()
	})
	.into()
}

fn section<'a>(title: &'a str, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
	column![
		text(title).size(18),
		container(content).padding(10).style(|_| container::Style {
			border: border::Border {
				color: Color {
					a: 0.2,
					r: 0.5,
					g: 0.5,
					b: 0.5
				},
				width: 1.0,
				radius: 4.0.into(),
			},
			..Default::default()
		})
	]
	.spacing(8)
	.into()
}

fn field<'a>(label: &'a str, widget: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
	row![text(label).width(Length::Fixed(160.0)), widget.into()]
		.spacing(10)
		.align_y(Alignment::Center)
		.into()
}

fn horizontal_rule<'a>() -> Element<'a, Message> {
	container(row![].width(Fill).height(1))
		.style(|_| container::Style {
			background: Some(Background::Color(Color {
				a: 0.1,
				..Color::WHITE
			})),
			..Default::default()
		})
		.into()
}

fn window_decorations<'a>(
	underlay: impl Into<Element<'a, Message>>,
	menu_state: &'a MenuState,
	saved_connections: &'a [SavedConnection],
	saved_statements: &'a [SavedStatement],
) -> Element<'a, Message> {
	let resize_thin = 6;
	let resize_thick = 60;
	let resize_area_northeast_top =
		styled_resize_area(resize_thick, resize_thin / 2, Direction::NorthEast);
	let resize_area_north = styled_resize_area(Fill, resize_thin / 2, Direction::North);
	let resize_area_northwest_top =
		styled_resize_area(resize_thick, resize_thin / 2, Direction::NorthWest);
	let resize_area_northwest_side =
		styled_resize_area(resize_thin, resize_thick, Direction::NorthWest);
	let resize_area_west = styled_resize_area(resize_thin, Fill, Direction::West);
	let resize_area_southwest_side =
		styled_resize_area(resize_thin, resize_thick, Direction::SouthWest);
	let resize_area_southwest_bottom =
		styled_resize_area(resize_thick, resize_thin, Direction::SouthWest);
	let resize_area_south = styled_resize_area(Fill, resize_thin, Direction::South);
	let resize_area_southeast_bottom =
		styled_resize_area(resize_thick, resize_thin, Direction::SouthEast);
	let resize_area_northeast_side =
		styled_resize_area(resize_thin, resize_thick, Direction::NorthEast);
	let resize_area_east = styled_resize_area(resize_thin, Fill, Direction::East);
	let resize_area_southeast_side =
		styled_resize_area(resize_thin, resize_thick, Direction::SouthEast);
	column![
		row![
			resize_area_northwest_top,
			resize_area_north,
			resize_area_northeast_top,
		],
		row![
			column![
				resize_area_northwest_side,
				resize_area_west,
				resize_area_southwest_side
			],
			column![
				row![title_bar()],
				stack![
					column![
						space::vertical().height(32),
						underlay.into(),
					],
					row![menu_bar(menu_state, saved_connections, saved_statements)],
				],
				row![
					resize_area_southwest_bottom,
					resize_area_south,
					resize_area_southeast_bottom,
				],
			],
			column![
				resize_area_northeast_side,
				resize_area_east,
				resize_area_southeast_side
			],
		]
	]
	.into()
}

pub fn styled_text_input<'a, Message: Clone + 'a>(
	default_str: &str,
	input_str: &str,
) -> TextInput<'a, Message> {
	text_input(default_str, input_str)
		.padding(10)
		.size(18)
		.style(|_theme: &Theme, status: text_input::Status| match status {
			text_input::Status::Focused { .. } => text_input::Style {
				background: Background::Color(colors::BG_INPUT_FOCUS),
				border: border::Border {
					color: colors::BORDER_ACCENT,
					width: 2.0,
					radius: 5.0.into(),
				},
				icon: colors::TEXT_SECONDARY,
				placeholder: colors::TEXT_PLACEHOLDER_HOVER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
			text_input::Status::Hovered => text_input::Style {
				background: Background::Color(colors::BG_INPUT_HOVER),
				border: border::Border {
					color: colors::BORDER_HOVER,
					width: 1.5,
					radius: 5.0.into(),
				},
				icon: colors::TEXT_SECONDARY,
				placeholder: colors::TEXT_PLACEHOLDER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
			_ => text_input::Style {
				background: Background::Color(colors::BG_INPUT),
				border: border::Border {
					color: colors::BORDER_PRIMARY,
					width: 1.0,
					radius: 5.0.into(),
				},
				icon: colors::TEXT_SECONDARY,
				placeholder: colors::TEXT_PLACEHOLDER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
		})
}

fn styled_resize_area<'a, WT: Into<Length>, HT: Into<Length>>(
	width: WT,
	height: HT,
	direction: Direction,
) -> Element<'a, Message> {
	mouse_area(
		container(space::horizontal().width(width).height(height)).style(|_| container::Style {
			background: Some(colors::BG_SECONDARY.into()),
			border: border::Border {
				color: colors::BORDER_DIM,
				width: 1.0,
				radius: 0.0.into(),
			},
			..Default::default()
		}),
	)
	.interaction(match direction {
		Direction::West | Direction::East => mouse::Interaction::ResizingHorizontally,
		Direction::North | Direction::South => mouse::Interaction::ResizingVertically,
		Direction::NorthEast | Direction::SouthWest => mouse::Interaction::ResizingDiagonallyUp,
		Direction::NorthWest | Direction::SouthEast => mouse::Interaction::ResizingDiagonallyDown,
	})
	.on_press(Message::ResizeWindow(direction))
	.into()
}

pub fn styled_button<'a, Message: Clone + 'a>(
	label: &str,
	msg: Message,
	size: (u32, u32),
) -> Element<'a, Message> {
	button(
		text(label.to_string())
			.size(18)
			.width(Fill)
			.align_x(Alignment::Center)
			.align_y(Alignment::Center)
			.font(Font {
				weight: font::Weight::Semibold,
				..Default::default()
			}),
	)
	.width(size.0)
	.height(size.1)
	.style(|theme: &Theme, status: button::Status| {
		let base = button::primary(theme, status);
		match status {
			button::Status::Hovered => button::Style {
				background: Some(Background::Color(colors::BG_BUTTON_HOVER)),
				border: border::Border {
					color: colors::BORDER_ACCENT,
					width: 2.0,
					radius: 5.0.into(),
				},
				text_color: colors::TEXT_SECONDARY,
				..base
			},
			_ => button::Style {
				background: Some(Background::Color(colors::BG_BUTTON)),
				border: border::Border {
					color: colors::BORDER_PRIMARY,
					width: 1.0,
					radius: 5.0.into(),
				},
				text_color: colors::TEXT_SECONDARY,
				..base
			},
		}
	})
	.on_press(msg)
	.into()
}

pub fn adapter_view(adapter_state: &AdapterState) -> Element<'static, Message> {
	match adapter_state.stage {
		AdapterStage::None => container(text("")).into(),
		AdapterStage::Unselected => adapter_gallery_view(),
		AdapterStage::Unconfigured => adapter_configuration_view(adapter_state),
		AdapterStage::Configured => container(text("")).into(),
		AdapterStage::Connected => container(text("")).into(),
	}
}

const MODAL_FILL_PORTION_V: u16 = 30;
const MODAL_FILL_PORTION_H: u16 = 40;

fn adapter_gallery_view() -> Element<'static, Message> {
	let dialog: Element<Message> = container(column![
		center(text("Select Adapter").size(24)),
		center(
			row![
				styled_button(
					"BigQuery",
					Message::AdapterSelected(AdapterSelection::BigQuery),
					BUTTON_SIZE_DEFAULT
				),
				styled_button(
					"MySQL",
					Message::AdapterSelected(AdapterSelection::MySQL),
					BUTTON_SIZE_DEFAULT
				),
				styled_button(
					"Parquet",
					Message::AdapterSelected(AdapterSelection::Parquet),
					BUTTON_SIZE_DEFAULT
				),
				styled_button(
					"Postgres",
					Message::AdapterSelected(AdapterSelection::Postgres),
					BUTTON_SIZE_DEFAULT
				),
				styled_button(
					"SQLite",
					Message::AdapterSelected(AdapterSelection::SQLite),
					BUTTON_SIZE_DEFAULT
				)
			]
			.spacing(20)
		)
	])
	.width(FillPortion(MODAL_FILL_PORTION_H))
	.height(FillPortion(MODAL_FILL_PORTION_V))
	.style(|_| container::Style {
		background: Some(colors::BG_MODAL.into()),
		border: border::Border {
			color: colors::BORDER_PRIMARY,
			width: 1.0,
			radius: 5.0.into(),
		},
		..Default::default()
	})
	.into();
	column![
		space::vertical().height(FillPortion((100 - MODAL_FILL_PORTION_V) / 2)),
		row![
			space::horizontal().width(FillPortion((100 - MODAL_FILL_PORTION_H) / 2)),
			dialog,
			space::horizontal().width(FillPortion((100 - MODAL_FILL_PORTION_H) / 2)),
		],
		space::vertical().height(FillPortion((100 - MODAL_FILL_PORTION_V) / 2)),
	]
	.into()
}

fn password_prompt_view<'a>(password_entry: &'a str, error: &'a str) -> Element<'a, Message> {
	let error_el: Element<Message> = if error.is_empty() {
		space::vertical().height(24).into()
	} else {
		container(text(error).color(colors::DANGER).size(14))
			.height(24)
			.into()
	};
	let dialog: Element<Message> = container(
		column![
			row![text("Enter Password").size(24), space::horizontal()].align_y(Alignment::Center),
			section(
				"Password",
				column![
					styled_text_input("Enter password", password_entry)
						.secure(true)
						.on_input(Message::PasswordEntryChanged)
						.on_submit(Message::PasswordEntrySubmit),
					error_el,
				]
				.spacing(4),
			),
			row![
				space::horizontal(),
				styled_button("Unlock", Message::PasswordEntrySubmit, BUTTON_SIZE_DEFAULT),
			]
			.align_y(Alignment::Center),
		]
		.spacing(20)
		.padding(20),
	)
	.width(Length::Fixed(480.0))
	.style(|_| container::Style {
		background: Some(colors::BG_MODAL.into()),
		border: border::Border {
			color: colors::BORDER_PRIMARY,
			width: 1.0,
			radius: 5.0.into(),
		},
		..Default::default()
	})
	.into();
	column![
		space::vertical().height(FillPortion((100 - MODAL_FILL_PORTION_V) / 2)),
		row![space::horizontal(), dialog, space::horizontal()],
		space::vertical().height(FillPortion((100 - MODAL_FILL_PORTION_V) / 2)),
	]
	.into()
}

fn save_statement_dialog_view<'a>(name: &'a str, is_editing: bool) -> Element<'a, Message> {
	let title = if is_editing { "Update Statement" } else { "Save Statement" };
	let btn_label = if is_editing { "Update" } else { "Save" };
	let dialog: Element<Message> = container(
		column![
			row![text(title).size(24), space::horizontal()].align_y(Alignment::Center),
			section(
				"Name",
				styled_text_input("Statement name", name)
					.on_input(Message::SaveStatementNameChanged)
					.on_submit(Message::SaveStatement),
			),
			row![
				space::horizontal(),
				styled_button("Cancel", Message::CloseSaveStatementDialog, (100, 40)),
				styled_button(btn_label, Message::SaveStatement, BUTTON_SIZE_DEFAULT),
			]
			.spacing(8)
			.align_y(Alignment::Center),
		]
		.spacing(20)
		.padding(20),
	)
	.width(Length::Fixed(480.0))
	.style(|_| container::Style {
		background: Some(colors::BG_MODAL.into()),
		border: border::Border {
			color: colors::BORDER_PRIMARY,
			width: 1.0,
			radius: 5.0.into(),
		},
		..Default::default()
	})
	.into();
	column![
		space::vertical().height(FillPortion((100 - MODAL_FILL_PORTION_V) / 2)),
		row![space::horizontal(), dialog, space::horizontal()],
		space::vertical().height(FillPortion((100 - MODAL_FILL_PORTION_V) / 2)),
	]
	.into()
}

fn settings_dialog_view<'a>(
	new_password: &'a str,
	confirm_password: &'a str,
	error: &'a str,
	is_password_protected: bool,
	show_column_types: bool,
) -> Element<'a, Message> {
	let error_el: Element<Message> = if error.is_empty() {
		space::vertical().height(24).into()
	} else {
		container(text(error).color(colors::DANGER).size(14))
			.height(24)
			.into()
	};
	let apply_label = if is_password_protected {
		"Change"
	} else {
		"Set"
	};
	let mut action_row = row![
		space::horizontal(),
		styled_button(
			apply_label,
			Message::SettingsApplyPassword,
			BUTTON_SIZE_DEFAULT
		),
	]
	.spacing(10)
	.align_y(Alignment::Center);
	if is_password_protected {
		action_row = action_row.push(styled_button(
			"Remove",
			Message::SettingsRemovePassword,
			BUTTON_SIZE_DEFAULT,
		));
	}
	let dialog: Element<Message> = container(
		column![
			row![
				text("Settings").size(24),
				space::horizontal(),
				styled_button("✕", Message::CloseSettings, (40, 32)),
			]
			.align_y(Alignment::Center),
			section(
				"Table Display",
				checkbox(show_column_types)
					.label("Show column types in header")
					.on_toggle(Message::ToggleShowColumnTypes),
			),
			section(
				"Security",
				column![
					styled_text_input("New password", new_password)
						.secure(true)
						.on_input(Message::SettingsNewPasswordChanged)
						.on_submit(Message::SettingsApplyPassword),
					styled_text_input("Confirm password", confirm_password)
						.secure(true)
						.on_input(Message::SettingsConfirmPasswordChanged)
						.on_submit(Message::SettingsApplyPassword),
					error_el,
				]
				.spacing(8),
			),
			action_row,
		]
		.spacing(20)
		.padding(20),
	)
	.width(Length::Fixed(520.0))
	.style(|_| container::Style {
		background: Some(colors::BG_MODAL.into()),
		border: border::Border {
			color: colors::BORDER_PRIMARY,
			width: 1.0,
			radius: 5.0.into(),
		},
		..Default::default()
	})
	.into();
	column![
		space::vertical().height(FillPortion((100 - MODAL_FILL_PORTION_V) / 2)),
		row![space::horizontal(), dialog, space::horizontal()],
		space::vertical().height(FillPortion((100 - MODAL_FILL_PORTION_V) / 2)),
	]
	.into()
}

pub fn adapter_configuration_view(adapter_state: &AdapterState) -> Element<'static, Message> {
	let name = adapter_state.name.clone();
	let adapter_label = match &adapter_state.selection {
		AdapterSelection::None => "None",
		AdapterSelection::BigQuery => "BigQuery",
		AdapterSelection::MySQL => "MySQL",
		AdapterSelection::Parquet => "Parquet",
		AdapterSelection::Postgres => "Postgres",
		AdapterSelection::SQLite => "SQLite",
	};
	let fields_section: Element<Message> = match &adapter_state.selection {
		AdapterSelection::None => text("Select an adapter to configure.").into(),
		selection => {
			let descriptors = fields_for(selection);
			let inputs = descriptors
				.iter()
				.fold(column![].spacing(8), |col, descriptor| {
					let current_value = adapter_state
						.fields
						.get(descriptor.key)
						.cloned()
						.unwrap_or_default();
					let key = descriptor.key;
					match descriptor.field_type {
						AdapterFieldType::Text => {
							let input = if descriptor.is_secure {
								styled_text_input(descriptor.value, &current_value).secure(true)
							} else {
								styled_text_input(descriptor.value, &current_value)
							};
							let input = input
								.on_input(move |val| {
									Message::AdapterConfigurationChanged(key.into(), val)
								})
								.on_submit(Message::AdapterConfigurationSubmitted);
							col.push(field(descriptor.key, input))
						}
					}
				});
			section(adapter_label, inputs)
		}
	};
	let dialog: Element<Message> = container(
		column![
			row![text("Configure Adapter").size(24), space::horizontal(),]
				.align_y(Alignment::Center),
			section(
				"Connection",
				field(
					"Name",
					styled_text_input("Required", &name)
						.on_input(Message::ConnectionNameChanged)
						.on_submit(Message::AdapterConfigurationSubmitted),
				),
			),
			fields_section,
			row![
				space::horizontal(),
				styled_button("Save", Message::SaveConnection, BUTTON_SIZE_DEFAULT),
				styled_button(
					"Connect",
					Message::AdapterConfigurationSubmitted,
					BUTTON_SIZE_DEFAULT,
				),
			]
			.spacing(10)
			.align_y(Alignment::Center),
		]
		.spacing(20)
		.padding(20),
	)
	.width(Length::Fixed(800.0))
	.style(|_| container::Style {
		background: Some(colors::BG_MODAL.into()),
		border: border::Border {
			color: colors::BORDER_PRIMARY,
			width: 1.0,
			radius: 5.0.into(),
		},
		..Default::default()
	})
	.into();
	column![
		space::vertical().height(FillPortion((100 - MODAL_FILL_PORTION_V) / 2)),
		row![space::horizontal(), dialog, space::horizontal(),],
		space::vertical().height(FillPortion((100 - MODAL_FILL_PORTION_V) / 2)),
	]
	.into()
}
