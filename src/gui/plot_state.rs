use crate::gui::messages::PlotMessage;
use crate::plot::colors;
use crate::plot::common::{Orientation, PlotKernel, PlotSettings};
use crate::plot::core::PlotType;
use crate::plot::kernels::bar::{self, BarPlotKernel};
use crate::plot::kernels::boxplot::{self, BoxPlotKernel};
use crate::plot::kernels::bubble::{self, BubblePlotKernel};
use crate::plot::kernels::candlestick::{self, CandlestickPlotKernel};
use crate::plot::kernels::fill_between::{self, FillBetweenPlotKernel};
use crate::plot::kernels::funnel::{self, FunnelPlotKernel};
use crate::plot::kernels::heatmap::{self, HeatmapPlotKernel};
use crate::plot::kernels::hexbin::{self, HexbinPlotKernel};
use crate::plot::kernels::histogram::{self, HistogramPlotKernel};
use crate::plot::kernels::line::{self, LinePlotKernel};
use crate::plot::kernels::parallel::{self, ParallelPlotKernel};
use crate::plot::kernels::pie::{self, PiePlotKernel};
use crate::plot::kernels::radar::{self, RadarPlotKernel};
use crate::plot::kernels::radial_dial::{self, RadialDialPlotKernel};
use crate::plot::kernels::scatter::{self, ScatterPlotKernel};
use crate::plot::kernels::stacked_area::{self, StackedAreaPlotKernel};
use crate::plot::kernels::stacked_bar::{self, StackedBarPlotKernel};
use crate::plot::kernels::violin::{self, ViolinPlotKernel};
use iced::{Rectangle, Size};
use polars::frame::DataFrame;
use polars::prelude::{Column, DataType};
use std::sync::Arc;

pub struct PlotState {
	pub kernel: Arc<dyn PlotKernel + Send + Sync>,
	pub hovered_info: Option<String>,
	pub current_plot_type: PlotType,
	pub kernel_plot_type: PlotType,
	pub plot_settings: PlotSettings,
	pub live_updates_enabled: bool,
	pub resize_render_suspended: bool,
	pub render_revision: u64,
	pub last_bounds: Rectangle,
	pub max_legend_rows_input: String,
	pub legend_x_input: String,
	pub legend_y_input: String,
	pub x_rotation_input: String,
	pub x_offset_input: String,
	pub bg_color_input: String,
	pub decoration_color_input: String,
	pub x_min_input: String,
	pub x_max_input: String,
	pub y_min_input: String,
	pub y_max_input: String,
	pub title_input: String,
	pub subtitle_input: String,
	pub x_label_input: String,
	pub y_label_input: String,
	pub title_offset_input: String,
	pub subtitle_offset_input: String,
	pub x_label_padding_input: String,
	pub y_label_padding_input: String,
	pub plot_padding_top_input: String,
	pub plot_padding_bottom_input: String,
	pub plot_padding_left_input: String,
	pub plot_padding_right_input: String,
	pub title_size_input: String,
	pub subtitle_size_input: String,
	pub x_label_size_input: String,
	pub y_label_size_input: String,
	pub x_tick_size_input: String,
	pub y_tick_size_input: String,
	pub legend_size_input: String,
	pub x_ticks_input: String,
	pub y_ticks_input: String,
	pub x_minor_ticks_input: String,
	pub y_minor_ticks_input: String,
	pub x_major_grid_width_input: String,
	pub y_major_grid_width_input: String,
	pub x_minor_grid_width_input: String,
	pub y_minor_grid_width_input: String,
	pub scatter_max_vector_points_input: String,
	pub scatter_downsample_target_input: String,
	pub scatter_raster_threshold_input: String,
	pub settings_open: bool,
}

impl PlotState {
	pub fn with_kernel(
		plot_type: PlotType,
		kernel: Arc<dyn PlotKernel + Send + Sync>,
		width: u32,
		height: u32,
	) -> Self {
		let plot_settings = PlotSettings::default();
		Self {
			kernel,
			hovered_info: None,
			current_plot_type: plot_type,
			kernel_plot_type: plot_type,
			live_updates_enabled: true,
			resize_render_suspended: false,
			last_bounds: Rectangle::with_size(Size::new(width as f32, height as f32)),
			render_revision: 0,
			bg_color_input: colors::color_to_hex(plot_settings.background_color),
			decoration_color_input: colors::color_to_hex(plot_settings.decoration_color),
			x_min_input: String::new(),
			x_max_input: String::new(),
			y_min_input: String::new(),
			y_max_input: String::new(),
			title_input: String::new(),
			subtitle_input: String::new(),
			x_label_input: String::new(),
			y_label_input: String::new(),
			title_offset_input: plot_settings.title_offset.to_string(),
			subtitle_offset_input: plot_settings.subtitle_offset.to_string(),
			x_label_padding_input: plot_settings.x_label_padding.to_string(),
			y_label_padding_input: plot_settings.y_label_padding.to_string(),
			plot_padding_top_input: plot_settings.plot_padding_top.to_string(),
			plot_padding_bottom_input: plot_settings.plot_padding_bottom.to_string(),
			plot_padding_left_input: plot_settings.plot_padding_left.to_string(),
			plot_padding_right_input: plot_settings.plot_padding_right.to_string(),
			title_size_input: plot_settings.title_size.to_string(),
			subtitle_size_input: plot_settings.subtitle_size.to_string(),
			x_label_size_input: plot_settings.x_label_size.to_string(),
			y_label_size_input: plot_settings.y_label_size.to_string(),
			x_tick_size_input: plot_settings.x_tick_size.to_string(),
			y_tick_size_input: plot_settings.y_tick_size.to_string(),
			legend_size_input: plot_settings.legend_size.to_string(),
			x_ticks_input: plot_settings.x_ticks.to_string(),
			y_ticks_input: plot_settings.y_ticks.to_string(),
			x_minor_ticks_input: plot_settings.x_minor_ticks.to_string(),
			y_minor_ticks_input: plot_settings.y_minor_ticks.to_string(),
			x_major_grid_width_input: plot_settings.x_major_grid_width.to_string(),
			y_major_grid_width_input: plot_settings.y_major_grid_width.to_string(),
			x_minor_grid_width_input: plot_settings.x_minor_grid_width.to_string(),
			y_minor_grid_width_input: plot_settings.y_minor_grid_width.to_string(),
			scatter_max_vector_points_input: plot_settings.scatter_max_vector_points.to_string(),
			scatter_downsample_target_input: plot_settings
				.scatter_downsample_target
				.to_string(),
			scatter_raster_threshold_input: plot_settings.scatter_raster_threshold.to_string(),
			plot_settings: plot_settings.clone(),
			max_legend_rows_input: plot_settings.max_legend_rows.to_string(),
			legend_x_input: plot_settings.legend_x.to_string(),
			legend_y_input: plot_settings.legend_y.to_string(),
			x_rotation_input: plot_settings.x_label_rotation.to_string(),
			x_offset_input: plot_settings.x_label_offset.to_string(),
			settings_open: false,
		}
	}

	pub fn set_kernel(&mut self, plot_type: PlotType, kernel: Arc<dyn PlotKernel + Send + Sync>) {
		self.kernel = kernel;
		self.current_plot_type = plot_type;
		self.kernel_plot_type = plot_type;
		self.hovered_info = None;
		self.render_revision = self.render_revision.wrapping_add(1);
	}

	pub fn update(&mut self, message: PlotMessage) {
		let was_live_updates_enabled = self.live_updates_enabled;
		let manual_apply = matches!(&message, PlotMessage::ApplySettings);
		let requires_redraw = !matches!(
			&message,
			PlotMessage::UpdateHover(_)
				| PlotMessage::UpdateBounds(_)
				| PlotMessage::ToggleSettings
				| PlotMessage::CloseSettings
				| PlotMessage::RefreshData
				| PlotMessage::ApplySettings
		);
		match message {
			PlotMessage::ApplySettings => {}
			PlotMessage::RefreshData => {}
			PlotMessage::UpdateHover(hover) => {
				self.hovered_info = hover;
			}
			PlotMessage::UpdateBounds(bounds) => {
				self.last_bounds = bounds;
			}
			PlotMessage::ChangePlotType(new_type) => {
				if new_type != self.current_plot_type {
					self.current_plot_type = new_type;
				}
			}
			PlotMessage::SetMaxLegendRows(rows) => {
				self.plot_settings.max_legend_rows = rows;
				self.max_legend_rows_input = rows.to_string();
			}
			PlotMessage::SetScatterRenderMode(mode) => {
				self.plot_settings.scatter_render_mode = mode;
			}
			PlotMessage::SetScatterMaxVectorPoints(points) => {
				self.plot_settings.scatter_max_vector_points = points.max(1);
				self.scatter_max_vector_points_input = points.to_string();
			}
			PlotMessage::SetScatterDownsampleTarget(points) => {
				self.plot_settings.scatter_downsample_target = points.max(1);
				self.scatter_downsample_target_input = points.to_string();
			}
			PlotMessage::SetScatterRasterThreshold(points) => {
				self.plot_settings.scatter_raster_threshold = points.max(1);
				self.scatter_raster_threshold_input = points.to_string();
			}
			PlotMessage::SetLegendX(x) => {
				self.plot_settings.legend_x = x.clamp(0.0, 1.0);
				self.legend_x_input = x.to_string();
			}
			PlotMessage::SetLegendY(y) => {
				self.plot_settings.legend_y = y.clamp(0.0, 1.0);
				self.legend_y_input = y.to_string();
			}
			PlotMessage::SetXRotation(deg) => {
				self.plot_settings.x_label_rotation = deg;
				self.x_rotation_input = deg.to_string();
			}
			PlotMessage::SetXOffset(offset) => {
				self.plot_settings.x_label_offset = offset;
				self.x_offset_input = offset.to_string();
			}
			PlotMessage::ChangeColorTheme(theme) => {
				self.plot_settings.color_theme = theme;
			}
			PlotMessage::ChangeBackgroundColor(color) => {
				self.plot_settings.background_color = color;
				self.bg_color_input = colors::color_to_hex(color);
				self.plot_settings.decoration_color = colors::contrast_color(color);
				self.decoration_color_input =
					colors::color_to_hex(self.plot_settings.decoration_color);
			}
			PlotMessage::ChangeBackgroundHex(hex) => {
				self.bg_color_input = hex.clone();
				if let Some(color) = colors::hex_to_color(&hex) {
					self.plot_settings.background_color = color;
					self.plot_settings.decoration_color = colors::contrast_color(color);
					self.decoration_color_input =
						colors::color_to_hex(self.plot_settings.decoration_color);
				}
			}
			PlotMessage::ChangeDecorationColor(color) => {
				self.plot_settings.decoration_color = color;
				self.decoration_color_input = colors::color_to_hex(color);
			}
			PlotMessage::ChangeDecorationHex(hex) => {
				self.decoration_color_input = hex.clone();
				if let Some(color) = colors::hex_to_color(&hex) {
					self.plot_settings.decoration_color = color;
				}
			}
			PlotMessage::SetXMin(val) => {
				self.plot_settings.x_min = val;
				self.x_min_input = val.map(|v| v.to_string()).unwrap_or_default();
			}
			PlotMessage::SetXMax(val) => {
				self.plot_settings.x_max = val;
				self.x_max_input = val.map(|v| v.to_string()).unwrap_or_default();
			}
			PlotMessage::SetYMin(val) => {
				self.plot_settings.y_min = val;
				self.y_min_input = val.map(|v| v.to_string()).unwrap_or_default();
			}
			PlotMessage::SetYMax(val) => {
				self.plot_settings.y_max = val;
				self.y_max_input = val.map(|v| v.to_string()).unwrap_or_default();
			}
			PlotMessage::SetTitle(val) => {
				self.plot_settings.title = val.as_ref().map(|s| Arc::new(s.clone()));
				self.title_input = val.unwrap_or_default();
			}
			PlotMessage::SetSubtitle(val) => {
				self.plot_settings.subtitle = val.as_ref().map(|s| Arc::new(s.clone()));
				self.subtitle_input = val.unwrap_or_default();
			}
			PlotMessage::SetXLabel(val) => {
				self.plot_settings.x_label = val.as_ref().map(|s| Arc::new(s.clone()));
				self.x_label_input = val.unwrap_or_default();
			}
			PlotMessage::SetYLabel(val) => {
				self.plot_settings.y_label = val.as_ref().map(|s| Arc::new(s.clone()));
				self.y_label_input = val.unwrap_or_default();
			}
			PlotMessage::SetTitleOffset(val) => {
				self.plot_settings.title_offset = val;
				self.title_offset_input = val.to_string();
			}
			PlotMessage::SetSubtitleOffset(val) => {
				self.plot_settings.subtitle_offset = val;
				self.subtitle_offset_input = val.to_string();
			}
			PlotMessage::SetXLabelPadding(val) => {
				self.plot_settings.x_label_padding = val;
				self.x_label_padding_input = val.to_string();
			}
			PlotMessage::SetYLabelPadding(val) => {
				self.plot_settings.y_label_padding = val;
				self.y_label_padding_input = val.to_string();
			}
			PlotMessage::SetPlotPaddingTop(val) => {
				self.plot_settings.plot_padding_top = val;
				self.plot_padding_top_input = val.to_string();
			}
			PlotMessage::SetPlotPaddingBottom(val) => {
				self.plot_settings.plot_padding_bottom = val;
				self.plot_padding_bottom_input = val.to_string();
			}
			PlotMessage::SetPlotPaddingLeft(val) => {
				self.plot_settings.plot_padding_left = val;
				self.plot_padding_left_input = val.to_string();
			}
			PlotMessage::SetPlotPaddingRight(val) => {
				self.plot_settings.plot_padding_right = val;
				self.plot_padding_right_input = val.to_string();
			}
			PlotMessage::SetTitleSize(val) => {
				self.plot_settings.title_size = val;
				self.title_size_input = val.to_string();
			}
			PlotMessage::SetSubtitleSize(val) => {
				self.plot_settings.subtitle_size = val;
				self.subtitle_size_input = val.to_string();
			}
			PlotMessage::SetXLabelSize(val) => {
				self.plot_settings.x_label_size = val;
				self.x_label_size_input = val.to_string();
			}
			PlotMessage::SetYLabelSize(val) => {
				self.plot_settings.y_label_size = val;
				self.y_label_size_input = val.to_string();
			}
			PlotMessage::SetXTickSize(val) => {
				self.plot_settings.x_tick_size = val;
				self.x_tick_size_input = val.to_string();
			}
			PlotMessage::SetYTickSize(val) => {
				self.plot_settings.y_tick_size = val;
				self.y_tick_size_input = val.to_string();
			}
			PlotMessage::SetLegendSize(val) => {
				self.plot_settings.legend_size = val;
				self.legend_size_input = val.to_string();
			}
			PlotMessage::SetXTicks(val) => {
				self.plot_settings.x_ticks = val;
				self.x_ticks_input = val.to_string();
			}
			PlotMessage::SetYTicks(val) => {
				self.plot_settings.y_ticks = val;
				self.y_ticks_input = val.to_string();
			}
			PlotMessage::ToggleLiveUpdates(val) => {
				self.live_updates_enabled = val;
			}
			PlotMessage::SetXMinorTicks(val) => {
				self.plot_settings.x_minor_ticks = val;
				self.x_minor_ticks_input = val.to_string();
			}
			PlotMessage::SetYMinorTicks(val) => {
				self.plot_settings.y_minor_ticks = val;
				self.y_minor_ticks_input = val.to_string();
			}
			PlotMessage::ToggleXMinorTicks(val) => {
				self.plot_settings.show_x_minor_ticks = val;
			}
			PlotMessage::ToggleYMinorTicks(val) => {
				self.plot_settings.show_y_minor_ticks = val;
			}
			PlotMessage::ToggleXMajorGrid(val) => {
				self.plot_settings.show_x_major_grid = val;
			}
			PlotMessage::ToggleYMajorGrid(val) => {
				self.plot_settings.show_y_major_grid = val;
			}
			PlotMessage::ToggleXMinorGrid(val) => {
				self.plot_settings.show_x_minor_grid = val;
			}
			PlotMessage::ToggleYMinorGrid(val) => {
				self.plot_settings.show_y_minor_grid = val;
			}
			PlotMessage::SetXMajorGridWidth(val) => {
				self.plot_settings.x_major_grid_width = val;
				self.x_major_grid_width_input = val.to_string();
			}
			PlotMessage::SetYMajorGridWidth(val) => {
				self.plot_settings.y_major_grid_width = val;
				self.y_major_grid_width_input = val.to_string();
			}
			PlotMessage::SetXMinorGridWidth(val) => {
				self.plot_settings.x_minor_grid_width = val;
				self.x_minor_grid_width_input = val.to_string();
			}
			PlotMessage::SetYMinorGridWidth(val) => {
				self.plot_settings.y_minor_grid_width = val;
				self.y_minor_grid_width_input = val.to_string();
			}
			PlotMessage::SetXMajorGridStyle(val) => {
				self.plot_settings.x_major_grid_style = val;
			}
			PlotMessage::SetYMajorGridStyle(val) => {
				self.plot_settings.y_major_grid_style = val;
			}
			PlotMessage::SetXMinorGridStyle(val) => {
				self.plot_settings.x_minor_grid_style = val;
			}
			PlotMessage::SetYMinorGridStyle(val) => {
				self.plot_settings.y_minor_grid_style = val;
			}
			PlotMessage::ToggleSettings => {
				self.settings_open = !self.settings_open;
			}
			PlotMessage::CloseSettings => {
				self.settings_open = false;
			}
		}
		if manual_apply || (requires_redraw && self.live_updates_enabled) {
			self.render_revision = self.render_revision.wrapping_add(1);
		}
		if !was_live_updates_enabled && self.live_updates_enabled {
			self.render_revision = self.render_revision.wrapping_add(1);
		}
	}
}

pub fn create_plot(
	plot_type: PlotType,
	df: &DataFrame,
	_width: u32,
	_height: u32,
) -> Arc<dyn PlotKernel + Send + Sync> {
	let _cols = df.get_column_names();
	let string_cols: Vec<&str> = df
		.columns()
		.iter()
		.filter(|c: &&Column| matches!(c.dtype(), DataType::String | DataType::Categorical(_, _)))
		.map(|c: &Column| c.name().as_str())
		.collect();
	let numeric_cols: Vec<&str> = df
		.columns()
		.iter()
		.filter(|c: &&Column| c.dtype().is_numeric())
		.map(|c: &Column| c.name().as_str())
		.collect();

	match plot_type {
		PlotType::Violin => {
			let cat = string_cols.first().copied().unwrap_or("");
			let val = numeric_cols.first().copied().unwrap_or("");
			let prepared = violin::prepare_violin_data(df, cat, val, None);
			Arc::new(ViolinPlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::Hexbin => {
			let prepared = hexbin::prepare_hexbin_data(df, 0.02);
			Arc::new(HexbinPlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::Line => {
			let cat = string_cols.first().copied().unwrap_or("");
			let x = numeric_cols.first().copied().unwrap_or("");
			let y = numeric_cols.get(1).copied().unwrap_or(x);
			let prepared = line::prepare_line_data(df, cat, x, y);
			Arc::new(LinePlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::Bar => {
			let cat = string_cols.first().copied().unwrap_or("");
			let group = string_cols.get(1).copied().unwrap_or("");
			let val = numeric_cols.first().copied().unwrap_or("");
			let prepared = bar::prepare_bar_data(df, cat, group, val);
			Arc::new(BarPlotKernel {
				prepared_data: Arc::new(prepared),
				orientation: Orientation::Vertical,
			})
		}
		PlotType::HorizontalBar => {
			let cat = string_cols.first().copied().unwrap_or("");
			let group = string_cols.get(1).copied().unwrap_or("");
			let val = numeric_cols.first().copied().unwrap_or("");
			let prepared = bar::prepare_bar_data(df, cat, group, val);
			Arc::new(BarPlotKernel {
				prepared_data: Arc::new(prepared),
				orientation: Orientation::Horizontal,
			})
		}
		PlotType::Scatter => {
			let cat = string_cols.first().copied().unwrap_or("");
			let x = numeric_cols.first().copied().unwrap_or("");
			let y = numeric_cols.get(1).copied().unwrap_or(x);
			let prepared = scatter::prepare_scatter_data(df, cat, x, y, 3.0);
			Arc::new(ScatterPlotKernel::new(Arc::new(prepared)))
		}
		PlotType::StackedBar => {
			let cat = string_cols.first().copied().unwrap_or("");
			let group = string_cols.get(1).copied().unwrap_or("");
			let val = numeric_cols.first().copied().unwrap_or("");
			let prepared = stacked_bar::prepare_stacked_bar_data(df, cat, group, val);
			Arc::new(StackedBarPlotKernel {
				prepared_data: Arc::new(prepared),
				orientation: Orientation::Vertical,
			})
		}
		PlotType::HorizontalStackedBar => {
			let cat = string_cols.first().copied().unwrap_or("");
			let group = string_cols.get(1).copied().unwrap_or("");
			let val = numeric_cols.first().copied().unwrap_or("");
			let prepared = stacked_bar::prepare_stacked_bar_data(df, cat, group, val);
			Arc::new(StackedBarPlotKernel {
				prepared_data: Arc::new(prepared),
				orientation: Orientation::Horizontal,
			})
		}
		PlotType::Pie => {
			let cat = string_cols.first().copied().unwrap_or("");
			let val = numeric_cols.first().copied().unwrap_or("");
			let prepared = pie::prepare_pie_data(df, cat, val);
			Arc::new(PiePlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::BoxPlot => {
			let cat = string_cols.first().copied().unwrap_or("");
			let val = numeric_cols.first().copied().unwrap_or("");
			let prepared = boxplot::prepare_box_plot_data(df, cat, val);
			Arc::new(BoxPlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::Bubble => {
			let x = numeric_cols.first().copied().unwrap_or("");
			let y = numeric_cols.get(1).copied().unwrap_or(x);
			let size = numeric_cols.get(2).copied().unwrap_or(y);
			let color = string_cols.first().copied().unwrap_or("");
			let label = string_cols.get(1).copied();
			let prepared = bubble::prepare_bubble_data(df, x, y, size, color, label);
			Arc::new(BubblePlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::Candlestick => {
			let x = numeric_cols.first().copied().unwrap_or("");
			let open = numeric_cols.get(1).copied().unwrap_or("");
			let high = numeric_cols.get(2).copied().unwrap_or("");
			let low = numeric_cols.get(3).copied().unwrap_or("");
			let close = numeric_cols.get(4).copied().unwrap_or("");
			let prepared = candlestick::prepare_candlestick_data(df, x, open, high, low, close);
			Arc::new(CandlestickPlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::FillBetween => {
			let x = numeric_cols.first().copied().unwrap_or("");
			let mid = numeric_cols.get(1).copied().unwrap_or("");
			let lower = numeric_cols.get(2).copied().unwrap_or("");
			let upper = numeric_cols.get(3).copied().unwrap_or("");
			let prepared = fill_between::prepare_fill_between_data(df, x, mid, lower, upper);
			Arc::new(FillBetweenPlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::Funnel => {
			let stage = string_cols.first().copied().unwrap_or("");
			let val = numeric_cols.first().copied().unwrap_or("");
			let prepared = funnel::prepare_funnel_data(df, stage, val);
			Arc::new(FunnelPlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::Heatmap => {
			let x = string_cols.first().copied().unwrap_or("");
			let y = string_cols.get(1).copied().unwrap_or("");
			let val = numeric_cols.first().copied().unwrap_or("");
			let prepared = heatmap::prepare_heatmap_data(df, x, y, val);
			Arc::new(HeatmapPlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::Histogram => {
			let val = numeric_cols.first().copied().unwrap_or("");
			let prepared = histogram::prepare_histogram_data(df, val, 50);
			Arc::new(HistogramPlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::StackedArea => {
			let cat = string_cols.first().copied().unwrap_or("");
			let x = numeric_cols.first().copied().unwrap_or("");
			let y = numeric_cols.get(1).copied().unwrap_or(x);
			let prepared = stacked_area::prepare_stacked_area_data(df, cat, x, y);
			Arc::new(StackedAreaPlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::Parallel => {
			let dims = numeric_cols
				.iter()
				.map(|s: &&str| s.to_string())
				.collect::<Vec<_>>();
			let cat = string_cols.first().copied().unwrap_or("");
			let prepared = parallel::prepare_parallel_data(df, &dims, cat);
			Arc::new(ParallelPlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::Radar => {
			let dims = numeric_cols
				.iter()
				.map(|s: &&str| s.to_string())
				.collect::<Vec<_>>();
			let cat = string_cols.first().copied().unwrap_or("");
			let prepared = radar::prepare_radar_data(df, &dims, cat);
			Arc::new(RadarPlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
		PlotType::RadialDial => {
			let cat = string_cols.first().copied().unwrap_or("");
			let val = numeric_cols.first().copied().unwrap_or("");
			let max = numeric_cols.get(1).copied().unwrap_or("");
			let prepared = radial_dial::prepare_radial_dial_data(df, cat, val, max);
			Arc::new(RadialDialPlotKernel {
				prepared_data: Arc::new(prepared),
			})
		}
	}
}
