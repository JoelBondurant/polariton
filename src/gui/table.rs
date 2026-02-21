use iced::{
	advanced::{
		layout::{Limits, Node},
		mouse::Cursor,
		renderer::{self, Style},
		widget::Tree,
		Layout, Widget,
	},
	border, Element,
	Length::{self, Fill},
	Rectangle, Size,
};

use crate::gui::colors;

pub struct Table {
	data: Vec<i32>,
}

impl Table {
	pub fn new(data: Vec<i32>) -> Self {
		Self { data }
	}
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Table
where
	Renderer: renderer::Renderer,
{
	fn size(&self) -> Size<Length> {
		Size {
			width: Fill,
			height: Fill,
		}
	}
	fn draw(
		&self,
		_tree: &Tree,
		renderer: &mut Renderer,
		_theme: &Theme,
		_style: &Style,
		layout: Layout<'_>,
		_cursor: Cursor,
		_viewport: &Rectangle,
	) {
		renderer.fill_quad(
			renderer::Quad {
				bounds: layout.bounds(),
				border: border::rounded(1),
				..renderer::Quad::default()
			},
			colors::BG_PRIMARY,
		);
	}

	fn layout(&mut self, _tree: &mut Tree, _renderer: &Renderer, _limits: &Limits) -> Node {
		Node::new(Size {
			width: 100.0,
			height: 100.0,
		})
	}
}

impl<Message, Theme, Renderer> From<Table> for Element<'_, Message, Theme, Renderer>
where
	Renderer: renderer::Renderer,
{
	fn from(table: Table) -> Self {
		Self::new(table)
	}
}
