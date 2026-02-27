use crate::gui::colors;
use iced::{
	advanced::{
		layout::{Limits, Node},
		mouse::{self, Cursor, Interaction, ScrollDelta},
		renderer::{self, Style},
		text::{self, Renderer as TextRenderer, Text},
		widget::{tree, Tree},
		Clipboard, Layout, Shell, Widget,
	},
	alignment::{Horizontal, Vertical},
	border, keyboard, Color, Element, Event,
	Length::{self, Fill},
	Pixels, Point, Rectangle, Size,
};

const ROW_HEIGHT: f32 = 28.0;
const HEADER_HEIGHT: f32 = 32.0;
const CELL_PADDING_X: f32 = 8.0;
const FONT_SIZE: f32 = 13.0;
const MIN_COL_WIDTH: f32 = 28.0;
const DEFAULT_COL_WIDTH: f32 = 80.0;
const V_SCROLLBAR_WIDTH: f32 = 12.0;
const H_SCROLLBAR_HEIGHT: f32 = 12.0;
const COL_RESIZE_GRAB_ZONE: f32 = 4.0;

pub struct Table<'a> {
	headers: &'a [String],
	columns: &'a [Vec<String>],
	total_row_count: usize,
	row_offset: usize,
	col_width: Option<f32>,
}

impl<'a> Table<'a> {
	pub fn new(
		headers: &'a [String],
		columns: &'a [Vec<String>],
		total_row_count: usize,
		row_offset: usize,
	) -> Self {
		debug_assert_eq!(headers.len(), columns.len(), "header/column count mismatch");
		Self {
			headers,
			columns,
			total_row_count,
			row_offset,
			col_width: None,
		}
	}

	pub fn col_width(mut self, width: f32) -> Self {
		self.col_width = Some(width);
		self
	}

	fn col_count(&self) -> usize {
		self.columns.len()
	}

	fn default_col_width(&self, viewport_width: f32) -> f32 {
		if let Some(w) = self.col_width {
			return w.max(DEFAULT_COL_WIDTH);
		}
		let n = self.col_count().max(1) as f32;
		(viewport_width / n).max(DEFAULT_COL_WIDTH)
	}

	fn col_widths<'s>(&self, state: &'s mut TableState, viewport_width: f32) -> &'s [f32] {
		let col_count = self.col_count();
		if state.col_widths.len() != col_count {
			let default = self.default_col_width(viewport_width);
			state.col_widths = vec![default; col_count];
		}
		&state.col_widths
	}

	fn col_widths_ref<'s>(&self, state: &'s TableState) -> &'s [f32] {
		&state.col_widths
	}

	fn total_content_width(&self, state: &TableState) -> f32 {
		state.col_widths.iter().sum()
	}

	fn total_content_height(&self) -> f32 {
		HEADER_HEIGHT + self.total_row_count as f32 * ROW_HEIGHT
	}

	fn loaded_row_count(&self) -> usize {
		self.columns.first().map_or(0, |c| c.len())
	}

	fn col_left_edges(&self, state: &TableState) -> Vec<f32> {
		let mut edges = Vec::with_capacity(state.col_widths.len());
		let mut x = 0.0f32;
		for &w in &state.col_widths {
			edges.push(x);
			x += w;
		}
		edges
	}

	fn divider_at_cursor(
		&self,
		state: &TableState,
		bounds: Rectangle,
		cursor_x: f32,
		cursor_y: f32,
	) -> Option<usize> {
		if cursor_y < bounds.y || cursor_y > bounds.y + HEADER_HEIGHT {
			return None;
		}
		let content_x = cursor_x - bounds.x + state.h_scroll_offset;
		let edges = self.col_left_edges(state);
		for (i, &left) in edges.iter().enumerate() {
			let divider_x = left + state.col_widths[i];
			if (content_x - divider_x).abs() <= COL_RESIZE_GRAB_ZONE {
				return Some(i);
			}
		}
		None
	}

	fn h_scrollbar_thumb_rect(
		&self,
		bounds: Rectangle,
		h_scroll_offset: f32,
		state: &TableState,
	) -> Rectangle {
		let total_w = self.total_content_width(state);
		let track_w = bounds.width - V_SCROLLBAR_WIDTH;
		let thumb_w = (track_w * (track_w / total_w.max(1.0))).max(20.0);
		let max_scroll = (total_w - track_w).max(0.0);
		let thumb_x = bounds.x
			+ if max_scroll > 0.0 {
				h_scroll_offset / max_scroll * (track_w - thumb_w)
			} else {
				0.0
			};
		Rectangle {
			x: thumb_x,
			y: bounds.y + bounds.height - H_SCROLLBAR_HEIGHT + 2.0,
			width: thumb_w,
			height: H_SCROLLBAR_HEIGHT - 4.0,
		}
	}

	fn v_scrollbar_thumb_rect(&self, bounds: Rectangle, v_scroll_offset: f32) -> Rectangle {
		let total_h = self.total_content_height();
		let track_h = bounds.height - HEADER_HEIGHT - H_SCROLLBAR_HEIGHT;
		let thumb_h = (track_h * (track_h / (total_h - HEADER_HEIGHT).max(1.0))).max(20.0);
		let max_scroll = (total_h - bounds.height + H_SCROLLBAR_HEIGHT).max(0.0);
		let thumb_y = bounds.y
			+ HEADER_HEIGHT
			+ if max_scroll > 0.0 {
				v_scroll_offset / max_scroll * (track_h - thumb_h)
			} else {
				0.0
			};
		Rectangle {
			x: bounds.x + bounds.width - V_SCROLLBAR_WIDTH - 2.0,
			y: thumb_y,
			width: V_SCROLLBAR_WIDTH - 4.0,
			height: thumb_h,
		}
	}
}

#[derive(Default)]
pub struct TableState {
	col_widths: Vec<f32>,
	resizing_col: Option<usize>,
	resize_drag_start_x: f32,
	resize_drag_start_width: f32,
	h_drag_start_offset: f32,
	h_drag_start_x: f32,
	h_dragging_scrollbar: bool,
	h_scroll_offset: f32,
	v_drag_start_offset: f32,
	v_drag_start_y: f32,
	v_dragging_scrollbar: bool,
	v_scroll_offset: f32,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Table<'_>
where
	Renderer: renderer::Renderer + TextRenderer<Font = iced::Font>,
{
	fn tag(&self) -> tree::Tag {
		tree::Tag::of::<TableState>()
	}

	fn state(&self) -> tree::State {
		tree::State::new(TableState::default())
	}

	fn size(&self) -> Size<Length> {
		Size {
			width: Fill,
			height: Fill,
		}
	}

	fn layout(&mut self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
		Node::new(limits.max())
	}

	fn mouse_interaction(
		&self,
		tree: &Tree,
		layout: Layout<'_>,
		cursor: Cursor,
		_viewport: &Rectangle,
		_renderer: &Renderer,
	) -> Interaction {
		let state = tree.state.downcast_ref::<TableState>();
		if state.resizing_col.is_some() {
			return Interaction::ResizingHorizontally;
		}
		if let Some(pos) = cursor.position() {
			let bounds = layout.bounds();
			if self
				.divider_at_cursor(state, bounds, pos.x, pos.y)
				.is_some()
			{
				return Interaction::ResizingHorizontally;
			}
		}
		Interaction::default()
	}

	fn update(
		&mut self,
		tree: &mut Tree,
		event: &Event,
		layout: Layout<'_>,
		cursor: Cursor,
		_renderer: &Renderer,
		_clipboard: &mut dyn Clipboard,
		shell: &mut Shell<'_, Message>,
		_viewport: &Rectangle,
	) {
		let bounds = layout.bounds();
		let viewport_w = bounds.width - V_SCROLLBAR_WIDTH;
		{
			let state = tree.state.downcast_mut::<TableState>();
			self.col_widths(state, viewport_w);
		}
		let state = tree.state.downcast_mut::<TableState>();
		let total_h = self.total_content_height();
		let total_w = self.total_content_width(state);
		let viewport_h = bounds.height - H_SCROLLBAR_HEIGHT;
		let max_v_scroll = (total_h - viewport_h).max(0.0);
		let max_h_scroll = (total_w - viewport_w).max(0.0);
		let v_thumb = self.v_scrollbar_thumb_rect(bounds, state.v_scroll_offset);
		let h_thumb = self.h_scrollbar_thumb_rect(bounds, state.h_scroll_offset, state);

		match event {
			Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
				if let Some(pos) = cursor.position() {
					if let Some(col_idx) = self.divider_at_cursor(state, bounds, pos.x, pos.y) {
						state.resizing_col = Some(col_idx);
						state.resize_drag_start_x = pos.x;
						state.resize_drag_start_width = state.col_widths[col_idx];
						shell.request_redraw();
						return;
					}
					if cursor.is_over(v_thumb) {
						state.v_dragging_scrollbar = true;
						state.v_drag_start_y = pos.y;
						state.v_drag_start_offset = state.v_scroll_offset;
						shell.request_redraw();
					} else if cursor.is_over(h_thumb) {
						state.h_dragging_scrollbar = true;
						state.h_drag_start_x = pos.x;
						state.h_drag_start_offset = state.h_scroll_offset;
						shell.request_redraw();
					}
				}
			}
			Event::Mouse(mouse::Event::CursorMoved { position }) => {
				if let Some(col_idx) = state.resizing_col {
					let delta = position.x - state.resize_drag_start_x;
					state.col_widths[col_idx] =
						(state.resize_drag_start_width + delta).max(MIN_COL_WIDTH);
					let new_total_w = self.total_content_width(state);
					let new_max_h = (new_total_w - viewport_w).max(0.0);
					state.h_scroll_offset = state.h_scroll_offset.min(new_max_h);
					shell.request_redraw();
				} else if state.v_dragging_scrollbar {
					let drag_delta = position.y - state.v_drag_start_y;
					let track_h = bounds.height - HEADER_HEIGHT - H_SCROLLBAR_HEIGHT;
					let thumb_h = v_thumb.height;
					let scroll_ratio = drag_delta / (track_h - thumb_h).max(1.0);
					state.v_scroll_offset = (state.v_drag_start_offset
						+ scroll_ratio * max_v_scroll)
						.clamp(0.0, max_v_scroll);
					shell.request_redraw();
				} else if state.h_dragging_scrollbar {
					let drag_delta = position.x - state.h_drag_start_x;
					let track_w = viewport_w;
					let thumb_w = h_thumb.width;
					let scroll_ratio = drag_delta / (track_w - thumb_w).max(1.0);
					state.h_scroll_offset = (state.h_drag_start_offset
						+ scroll_ratio * max_h_scroll)
						.clamp(0.0, max_h_scroll);
					shell.request_redraw();
				}
			}
			Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
				if state.resizing_col.is_some() {
					state.resizing_col = None;
					shell.request_redraw();
				} else if state.v_dragging_scrollbar || state.h_dragging_scrollbar {
					state.v_dragging_scrollbar = false;
					state.h_dragging_scrollbar = false;
					shell.request_redraw();
				}
			}
			Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(bounds) => {
				match delta {
					ScrollDelta::Lines { x, y } => {
						if x.abs() > y.abs() {
							state.h_scroll_offset = (state.h_scroll_offset - x * MIN_COL_WIDTH)
								.clamp(0.0, max_h_scroll);
						} else {
							state.v_scroll_offset =
								(state.v_scroll_offset - y * ROW_HEIGHT).clamp(0.0, max_v_scroll);
						}
					}
					ScrollDelta::Pixels { x, y } => {
						if x.abs() > y.abs() {
							state.h_scroll_offset =
								(state.h_scroll_offset - x).clamp(0.0, max_h_scroll);
						} else {
							state.v_scroll_offset =
								(state.v_scroll_offset - y).clamp(0.0, max_v_scroll);
						}
					}
				}
				shell.request_redraw();
			}
			Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) if cursor.is_over(bounds) => {
				let page_size = viewport_h - HEADER_HEIGHT;
				match key {
					keyboard::Key::Named(keyboard::key::Named::PageDown) => {
						state.v_scroll_offset =
							(state.v_scroll_offset + page_size).clamp(0.0, max_v_scroll);
					}
					keyboard::Key::Named(keyboard::key::Named::PageUp) => {
						state.v_scroll_offset =
							(state.v_scroll_offset - page_size).clamp(0.0, max_v_scroll);
					}
					keyboard::Key::Named(keyboard::key::Named::Home) => {
						state.v_scroll_offset = 0.0;
					}
					keyboard::Key::Named(keyboard::key::Named::End) => {
						state.v_scroll_offset = max_v_scroll;
					}
					keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
						state.v_scroll_offset =
							(state.v_scroll_offset + ROW_HEIGHT).clamp(0.0, max_v_scroll);
					}
					keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
						state.v_scroll_offset =
							(state.v_scroll_offset - ROW_HEIGHT).clamp(0.0, max_v_scroll);
					}
					keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
						state.h_scroll_offset =
							(state.h_scroll_offset + MIN_COL_WIDTH).clamp(0.0, max_h_scroll);
					}
					keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
						state.h_scroll_offset =
							(state.h_scroll_offset - MIN_COL_WIDTH).clamp(0.0, max_h_scroll);
					}
					_ => {}
				}
				shell.request_redraw();
			}
			_ => {}
		}
	}

	fn draw(
		&self,
		tree: &Tree,
		renderer: &mut Renderer,
		_theme: &Theme,
		_style: &Style,
		layout: Layout<'_>,
		_cursor: Cursor,
		_viewport: &Rectangle,
	) {
		let state = tree.state.downcast_ref::<TableState>();
		if state.col_widths.is_empty() {
			return;
		}
		let bounds = layout.bounds();
		let viewport_w = bounds.width - V_SCROLLBAR_WIDTH;
		let v_scroll = state.v_scroll_offset;
		let h_scroll = state.h_scroll_offset;
		renderer.fill_quad(
			renderer::Quad {
				bounds,
				border: border::rounded(4),
				..renderer::Quad::default()
			},
			colors::BG_PRIMARY,
		);
		renderer.with_layer(bounds, |renderer| {
			let header_clip = Rectangle {
				x: bounds.x,
				y: bounds.y,
				width: viewport_w,
				height: HEADER_HEIGHT,
			};
			renderer.fill_quad(
				renderer::Quad {
					bounds: header_clip,
					..renderer::Quad::default()
				},
				colors::BG_SECONDARY,
			);
			renderer.with_layer(header_clip, |renderer| {
				let col_widths = self.col_widths_ref(state);
				let mut cell_x = bounds.x - h_scroll;
				for (col_idx, header) in self.headers.iter().enumerate() {
					let col_w = col_widths[col_idx];
					if cell_x + col_w >= bounds.x && cell_x <= bounds.x + viewport_w {
						if col_idx > 0 {
							renderer.fill_quad(
								renderer::Quad {
									bounds: Rectangle {
										x: cell_x,
										y: bounds.y,
										width: 1.0,
										height: HEADER_HEIGHT,
									},
									..renderer::Quad::default()
								},
								colors::TABLE_BORDER,
							);
						}
						draw_text(
							renderer,
							header,
							Rectangle {
								x: cell_x + CELL_PADDING_X,
								y: bounds.y,
								width: col_w - CELL_PADDING_X,
								height: HEADER_HEIGHT,
							},
							colors::TABLE_TEXT_HEADER,
							true,
						);
					}
					cell_x += col_w;
				}
			});
			renderer.fill_quad(
				renderer::Quad {
					bounds: Rectangle {
						x: bounds.x,
						y: bounds.y + HEADER_HEIGHT - 1.0,
						width: viewport_w,
						height: 1.0,
					},
					..renderer::Quad::default()
				},
				colors::TABLE_BORDER,
			);
			let rows_clip = Rectangle {
				x: bounds.x,
				y: bounds.y + HEADER_HEIGHT,
				width: viewport_w,
				height: bounds.height - HEADER_HEIGHT - H_SCROLLBAR_HEIGHT,
			};
			renderer.with_layer(rows_clip, |renderer| {
				let first_visible = (v_scroll / ROW_HEIGHT).floor() as usize;
				let visible_count =
					((bounds.height - HEADER_HEIGHT) / ROW_HEIGHT).ceil() as usize + 1;
				let loaded = self.loaded_row_count();
				let col_widths = self.col_widths_ref(state);
				for row_offset in 0..=visible_count {
					let row_idx = first_visible + row_offset;
					if row_idx >= loaded {
						break;
					}
					let row_y = bounds.y + HEADER_HEIGHT + row_idx as f32 * ROW_HEIGHT - v_scroll;
					if row_y + ROW_HEIGHT < bounds.y + HEADER_HEIGHT {
						continue;
					}
					let abs_idx = self.row_offset + row_idx;
					let row_bg = if abs_idx.is_multiple_of(2) {
						colors::TABLE_ROW_EVEN
					} else {
						colors::TABLE_ROW_ODD
					};
					renderer.fill_quad(
						renderer::Quad {
							bounds: Rectangle {
								x: bounds.x,
								y: row_y,
								width: viewport_w,
								height: ROW_HEIGHT,
							},
							..renderer::Quad::default()
						},
						row_bg,
					);
					renderer.fill_quad(
						renderer::Quad {
							bounds: Rectangle {
								x: bounds.x,
								y: row_y + ROW_HEIGHT - 1.0,
								width: viewport_w,
								height: 1.0,
							},
							..renderer::Quad::default()
						},
						colors::TABLE_BORDER,
					);
					let mut cell_x = bounds.x - h_scroll;
					for (col_idx, col_data) in self.columns.iter().enumerate() {
						let col_w = col_widths[col_idx];
						if cell_x + col_w >= bounds.x && cell_x <= bounds.x + viewport_w {
							if col_idx > 0 {
								renderer.fill_quad(
									renderer::Quad {
										bounds: Rectangle {
											x: cell_x,
											y: row_y,
											width: 1.0,
											height: ROW_HEIGHT,
										},
										..renderer::Quad::default()
									},
									colors::TABLE_BORDER,
								);
							}
							if let Some(cell) = col_data.get(row_idx) {
								draw_text(
									renderer,
									cell,
									Rectangle {
										x: cell_x + CELL_PADDING_X,
										y: row_y,
										width: col_w - CELL_PADDING_X,
										height: ROW_HEIGHT,
									},
									colors::TEXT_PRIMARY,
									false,
								);
							}
						}
						cell_x += col_w;
					}
				}
			});
			let total_h = self.total_content_height();
			if total_h > bounds.height {
				let thumb = self.v_scrollbar_thumb_rect(bounds, v_scroll);
				renderer.fill_quad(
					renderer::Quad {
						bounds: thumb,
						border: border::rounded(2),
						..renderer::Quad::default()
					},
					colors::SCROLLBAR_THUMB,
				);
			}
			let total_w = self.total_content_width(state);
			if total_w > viewport_w {
				let h_thumb = self.h_scrollbar_thumb_rect(bounds, h_scroll, state);
				renderer.fill_quad(
					renderer::Quad {
						bounds: h_thumb,
						border: border::rounded(2),
						..renderer::Quad::default()
					},
					colors::SCROLLBAR_THUMB,
				);
			}
		});
	}
}

fn draw_text<Renderer>(
	renderer: &mut Renderer,
	content: &str,
	cell_bounds: Rectangle,
	color: Color,
	is_bold: bool,
) where
	Renderer: renderer::Renderer + TextRenderer<Font = iced::Font>,
{
	let font = if is_bold {
		iced::Font {
			weight: iced::font::Weight::Bold,
			..iced::Font::DEFAULT
		}
	} else {
		iced::Font::DEFAULT
	};
	renderer.fill_text(
		Text {
			content: content.to_string(),
			bounds: cell_bounds.size(),
			size: Pixels(FONT_SIZE),
			font,
			align_x: Horizontal::Left.into(),
			align_y: Vertical::Center,
			line_height: text::LineHeight::default(),
			shaping: text::Shaping::Basic,
			wrapping: text::Wrapping::None,
		},
		Point {
			x: cell_bounds.x,
			y: cell_bounds.y + cell_bounds.height / 2.0,
		},
		color,
		cell_bounds,
	);
}

impl<'a, Message, Theme, Renderer> From<Table<'a>> for Element<'a, Message, Theme, Renderer>
where
	Renderer: renderer::Renderer + TextRenderer<Font = iced::Font>,
{
	fn from(table: Table<'a>) -> Self {
		Self::new(table)
	}
}
