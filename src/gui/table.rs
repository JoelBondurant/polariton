use crate::gui::colors;
use iced::{
	advanced::{
		layout::{Limits, Node},
		mouse::{self, Cursor, ScrollDelta},
		renderer::{self, Style},
		text::{self, Renderer as TextRenderer, Text},
		widget::{tree, Tree},
		Clipboard, Layout, Shell, Widget,
	},
	alignment::{Horizontal, Vertical},
	border, Color, Element, Event, Point,
	Length::{self, Fill},
	Pixels, Rectangle, Size,
};

const ROW_HEIGHT: f32 = 28.0;
const HEADER_HEIGHT: f32 = 32.0;
const CELL_PADDING_X: f32 = 8.0;
const FONT_SIZE: f32 = 13.0;

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

	fn col_width_for(&self, total_width: f32) -> f32 {
		self.col_width.unwrap_or_else(|| {
			let n = self.col_count().max(1) as f32;
			total_width / n
		})
	}

	fn total_content_height(&self) -> f32 {
		HEADER_HEIGHT + self.total_row_count as f32 * ROW_HEIGHT
	}

	fn loaded_row_count(&self) -> usize {
		self.columns.first().map_or(0, |c| c.len())
	}
}

#[derive(Default)]
pub struct TableState {
	scroll_offset: f32,
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

	fn update(
		&mut self,
		tree: &mut Tree,
		event: &Event,
		layout: Layout<'_>,
		cursor: Cursor,
		_renderer: &Renderer,
		_clipboard: &mut dyn Clipboard,
		_shell: &mut Shell<'_, Message>,
		_viewport: &Rectangle,
	) {
		let state = tree.state.downcast_mut::<TableState>();
		let bounds = layout.bounds();
		if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event
			&& cursor.is_over(bounds)
		{
			let delta_y = match delta {
				ScrollDelta::Lines { y, .. } => y * ROW_HEIGHT,
				ScrollDelta::Pixels { y, .. } => *y,
			};
			let max_scroll = (self.total_content_height() - bounds.height).max(0.0);
			state.scroll_offset = (state.scroll_offset - delta_y).clamp(0.0, max_scroll);
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
		let bounds = layout.bounds();
		let col_w = self.col_width_for(bounds.width);
		let scroll = state.scroll_offset;
		renderer.fill_quad(
			renderer::Quad {
				bounds,
				border: border::rounded(4),
				..renderer::Quad::default()
			},
			colors::BG_PRIMARY,
		);
		renderer.with_layer(bounds, |renderer| {
			// Header
			renderer.fill_quad(
				renderer::Quad {
					bounds: Rectangle {
						x: bounds.x,
						y: bounds.y,
						width: bounds.width,
						height: HEADER_HEIGHT,
					},
					..renderer::Quad::default()
				},
				colors::BG_SECONDARY,
			);
			for (col_idx, header) in self.headers.iter().enumerate() {
				let cell_x = bounds.x + col_idx as f32 * col_w;

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
			// Header bottom border
			renderer.fill_quad(
				renderer::Quad {
					bounds: Rectangle {
						x: bounds.x,
						y: bounds.y + HEADER_HEIGHT - 1.0,
						width: bounds.width,
						height: 1.0,
					},
					..renderer::Quad::default()
				},
				colors::TABLE_BORDER,
			);
			// Rows
			let first_visible = (scroll / ROW_HEIGHT).floor() as usize;
			let visible_count =
				((bounds.height - HEADER_HEIGHT) / ROW_HEIGHT).ceil() as usize + 1;
			let loaded = self.loaded_row_count();
			for row_offset in 0..=visible_count {
				let row_idx = first_visible + row_offset;
				if row_idx >= loaded {
					break;
				}
				let row_y = bounds.y + HEADER_HEIGHT + row_idx as f32 * ROW_HEIGHT - scroll;
				if row_y + ROW_HEIGHT < bounds.y + HEADER_HEIGHT {
					continue;
				}
				// Use absolute row index for stable alternating colors across pages
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
							width: bounds.width,
							height: ROW_HEIGHT,
						},
						..renderer::Quad::default()
					},
					row_bg,
				);
				// Row bottom border
				renderer.fill_quad(
					renderer::Quad {
						bounds: Rectangle {
							x: bounds.x,
							y: row_y + ROW_HEIGHT - 1.0,
							width: bounds.width,
							height: 1.0,
						},
						..renderer::Quad::default()
					},
					colors::TABLE_BORDER,
				);
				// Draw each cell by indexing into the column, not a row vec
				for (col_idx, col_data) in self.columns.iter().enumerate() {
					let cell_x = bounds.x + col_idx as f32 * col_w;
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
			}
			// Scrollbar
			let total_h = self.total_content_height();
			if total_h > bounds.height {
				let track_h = bounds.height - HEADER_HEIGHT;
				let thumb_h = (track_h * (bounds.height / total_h)).max(20.0);
				let thumb_y = bounds.y
					+ HEADER_HEIGHT
					+ scroll / (total_h - bounds.height) * (track_h - thumb_h);
				renderer.fill_quad(
					renderer::Quad {
						bounds: Rectangle {
							x: bounds.x + bounds.width - 6.0,
							y: thumb_y,
							width: 4.0,
							height: thumb_h,
						},
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
