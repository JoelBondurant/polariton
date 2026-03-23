use crate::plot::common::{
	AxisType, CoordinateTransformer, PlotBackend, PlotKernel, PlotLayout,
	PlotSettings, format_label, polars_type_to_axis_type,
};
use iced::advanced::mouse::Cursor;
use iced::{Color, Rectangle};
use polars::prelude::*;
use std::sync::Arc;

pub struct ScatterPlotKernel {
	pub prepared_data: Arc<ScatterPreparedData>,
}

impl PlotKernel for ScatterPlotKernel {
	fn layout(&self, settings: PlotSettings) -> PlotLayout {
		PlotLayout::Cartesian {
			x_range: (
				settings.x_min.unwrap_or(self.prepared_data.x_range.0),
				settings.x_max.unwrap_or(self.prepared_data.x_range.1),
			),
			y_range: (
				settings.y_min.unwrap_or(self.prepared_data.y_range.0),
				settings.y_max.unwrap_or(self.prepared_data.y_range.1),
			),
			x_axis_type: self.prepared_data.x_axis_type,
			y_axis_type: self.prepared_data.y_axis_type,
		}
	}

	fn plot(
		&self,
		backend: &mut dyn PlotBackend,
		_bounds: Rectangle,
		transform: &CoordinateTransformer,
		_cursor: Cursor,
		settings: PlotSettings,
	) {
		for series in &self.prepared_data.series {
			let color = settings.color_theme.get_color(series.color_t);
			backend.fill_path(
				&|builder| {
					for p in &series.points {
						let pixel_p = transform.cartesian(p[0], p[1]);
						builder.circle(pixel_p, self.prepared_data.point_size_px);
					}
				},
				color,
			);
		}
	}

	fn draw_legend(
		&self,
		backend: &mut dyn PlotBackend,
		bounds: Rectangle,
		settings: PlotSettings,
	) {
		let num_series = self.prepared_data.series.len();
		if num_series == 0 {
			return;
		}
		let max_rows = settings.max_legend_rows.max(1) as usize;
		let num_cols = num_series.div_ceil(max_rows);
		let actual_rows = num_series.min(max_rows);
		let item_height = 25.0;
		let legend_padding = 10.0;
		let swatch_size = 12.0;
		let col_width = 150.0;
		let legend_width = num_cols as f32 * col_width + legend_padding * 2.0;
		let legend_height = actual_rows as f32 * item_height + legend_padding * 2.0;
		let x = (bounds.width - legend_width) * settings.legend_x;
		let y = (bounds.height - legend_height) * settings.legend_y;
		backend.fill_rectangle(
			iced::Point::new(x, y),
			iced::Size::new(legend_width, legend_height),
			Color {
				a: 0.6,
				..settings.background_color
			},
		);
		for (i, series) in self.prepared_data.series.iter().enumerate() {
			let color = settings.color_theme.get_color(series.color_t);
			let col = i / max_rows;
			let row = i % max_rows;
			let item_x = x + legend_padding + col as f32 * col_width;
			let item_y = y + legend_padding + row as f32 * item_height;
			backend.fill_path(
				&|builder| {
					builder.circle(
						iced::Point::new(item_x + swatch_size / 2.0, item_y + item_height / 2.0),
						swatch_size / 2.0,
					);
				},
				color,
			);
			backend.fill_text(iced::widget::canvas::Text {
				content: series.name.clone(),
				position: iced::Point::new(item_x + swatch_size + 10.0, item_y + item_height / 2.0),
				color: settings.decoration_color,
				size: iced::Pixels(settings.legend_size),
				align_x: iced::alignment::Horizontal::Left.into(),
				align_y: iced::alignment::Vertical::Center,
				..Default::default()
			});
		}
	}

	fn hover(&self, transform: &CoordinateTransformer, cursor: Cursor) -> Option<String> {
		if let Some(cursor_pos) = cursor.position()
			&& let Some((x, y)) = transform.pixel_to_cartesian(cursor_pos) {
			return Some(format!("X: {}, Y: {}", 
				format_label(x, self.prepared_data.x_axis_type),
				format_label(y, self.prepared_data.y_axis_type)));
		}
		None
	}

	fn x_label(&self) -> String {
		self.prepared_data.x_label.clone()
	}

	fn y_label(&self) -> String {
		self.prepared_data.y_label.clone()
	}
}

pub struct ScatterSeries {
	pub name: String,
	pub points: Vec<[f64; 2]>,
	pub color_t: f32,
}

pub struct ScatterPreparedData {
	pub series: Vec<ScatterSeries>,
	pub x_range: (f64, f64),
	pub y_range: (f64, f64),
	pub x_axis_type: AxisType,
	pub y_axis_type: AxisType,
	pub point_size_px: f32,
	pub x_label: String,
	pub y_label: String,
}

pub fn prepare_scatter_data(df: &DataFrame, cat_col: &str, x_col: &str, y_col: &str, point_size_px: f32) -> ScatterPreparedData {
	let x_col_series = match df.column(x_col) {
		Ok(c) => c.cast(&DataType::Float64).unwrap_or_else(|_| Column::from(Series::new_empty(x_col.into(), &DataType::Float64))),
		Err(_) => Column::from(Series::new_empty(x_col.into(), &DataType::Float64)),
	};
	let y_col_series = match df.column(y_col) {
		Ok(c) => c.cast(&DataType::Float64).unwrap_or_else(|_| Column::from(Series::new_empty(y_col.into(), &DataType::Float64))),
		Err(_) => Column::from(Series::new_empty(y_col.into(), &DataType::Float64)),
	};
	let x_dtype = df.column(x_col).map(|c| c.dtype().clone()).unwrap_or(DataType::Float64);
	let y_dtype = df.column(y_col).map(|c| c.dtype().clone()).unwrap_or(DataType::Float64);
	let x_axis_type = polars_type_to_axis_type(&x_dtype);
	let y_axis_type = polars_type_to_axis_type(&y_dtype);
	let x_series = x_col_series.f64().unwrap();
	let y_series = y_col_series.f64().unwrap();
	let x_range = (x_series.min().unwrap_or(0.0), x_series.max().unwrap_or(1.0));
	let y_range = (y_series.min().unwrap_or(0.0), y_series.max().unwrap_or(1.0));
	let x_pad = (x_range.1 - x_range.0).max(0.001) * 0.1;
	let y_pad = (y_range.1 - y_range.0).max(0.001) * 0.1;
	let x_range = (x_range.0 - x_pad, x_range.1 + x_pad);
	let y_range = (y_range.0 - y_pad, y_range.1 + y_pad);
	if df.height() == 0 || x_col.is_empty() || y_col.is_empty() {
		return ScatterPreparedData {
			series: vec![],
			x_range,
			y_range,
			x_axis_type,
			y_axis_type,
			point_size_px,
			x_label: x_col.to_string(),
			y_label: y_col.to_string(),
		};
	}
	let partitions = if cat_col.is_empty() {
		vec![df.clone()]
	} else {
		df.partition_by([cat_col], true).unwrap_or_else(|_| vec![df.clone()])
	};
	let num_partitions = partitions.len();
	let mut series_list = Vec::with_capacity(num_partitions);
	for (i, group_df) in partitions.into_iter().enumerate() {
		let cat_name = if cat_col.is_empty() {
			"All Data".to_string()
		} else {
			let cat_val = group_df.column(cat_col).and_then(|c| c.get(0)).unwrap_or(AnyValue::Null);
			if let AnyValue::String(s) = cat_val {
				s.to_string()
			} else if cat_val.is_null() {
				"Null".to_string()
			} else {
				cat_val.to_string().replace("\"", "")
			}
		};
		let xs_col = match group_df.column(x_col) {
			Ok(c) => c.cast(&DataType::Float64).unwrap_or_else(|_| Column::from(Series::new_empty(x_col.into(), &DataType::Float64))),
			Err(_) => Column::from(Series::new_empty(x_col.into(), &DataType::Float64)),
		};
		let ys_col = match group_df.column(y_col) {
			Ok(c) => c.cast(&DataType::Float64).unwrap_or_else(|_| Column::from(Series::new_empty(y_col.into(), &DataType::Float64))),
			Err(_) => Column::from(Series::new_empty(y_col.into(), &DataType::Float64)),
		};
		let xs = xs_col.f64().unwrap();
		let ys = ys_col.f64().unwrap();
		let t = if num_partitions > 1 {
			i as f32 / (num_partitions - 1) as f32
		} else {
			0.5
		};
		let mut points = Vec::with_capacity(group_df.height());
		for j in 0..group_df.height() {
			points.push([xs.get(j).unwrap(), ys.get(j).unwrap()]);
		}
		series_list.push(ScatterSeries { name: cat_name, points, color_t: t });
	}
	ScatterPreparedData {
		series: series_list,
		x_range,
		y_range,
		x_axis_type,
		y_axis_type,
		point_size_px,
		x_label: x_col.to_string(),
		y_label: y_col.to_string(),
	}
}
