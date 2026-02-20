use crate::gui::colors;
use crate::gui::messages::Message;
use iced::{
	advanced::text::highlighter::PlainText,
	border, font,
	theme::{Palette, Theme},
	widget::{
		button, center, column, container, mouse_area, row, space, text, text_editor, text_input,
		tooltip, TextEditor, TextInput, Tooltip,
	},
	window, Alignment, Background, Center, Color, Element, Fill, Font, Length,
};

const DEFAULT_BUTTON_SIZE: (u32, u32) = (120, 40);

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
					.color(colors::BRAND_GREEN),
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
			.style(|_theme: &Theme, status: button::Status| {
				match status {
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
				}
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
			.style(|_theme: &Theme, status: button::Status| {
				match status {
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
				}
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
			.style(|_theme: &Theme, status: button::Status| {
				match status {
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
				}
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
			color: colors::BORDER_SECONDARY,
			width: 2.0,
			radius: 0.0.into(),
		},
		..Default::default()
	})
	.into()
}

pub fn main_screen<'a>(code: &'a text_editor::Content, status: &'a str) -> Element<'a, Message> {
	let code_editor = styled_tooltip(
		styled_text_editor("code".into(), code).on_action(Message::CodeAction),
		"Code  ",
	);
	let main_content = container(center(column![code_editor,].spacing(4)))
		.padding(4)
		.width(Fill);
	let button_bar = row![
		space::horizontal(),
		styled_button("Run", Message::Run, DEFAULT_BUTTON_SIZE),
		space::horizontal(),
	]
	.padding(16)
	.align_y(Center);
	let status_bar = container(center(
		row![
			text("> ").color(colors::BRAND_PURPLE),
			text(status).color(colors::TEXT_STATUS),
			space::horizontal(),
			text(" <").color(colors::BRAND_PURPLE)
		]
		.spacing(1),
	))
	.height(30)
	.padding(1)
	.width(Fill);

	let resize_thin = 8;
	let resize_thick = 40;

	let resize_area_west = styled_resize_area(resize_thin, Fill, window::Direction::West);
	let resize_area_southwest_side =
		styled_resize_area(resize_thin, resize_thick, window::Direction::SouthWest);
	let resize_area_northwest_side =
		styled_resize_area(resize_thin, resize_thick, window::Direction::NorthWest);
	let resize_area_southwest_bottom =
		styled_resize_area(resize_thick, resize_thin, window::Direction::SouthWest);
	let resize_area_east = styled_resize_area(resize_thin, Fill, window::Direction::East);
	let resize_area_northeast_side =
		styled_resize_area(resize_thin, resize_thick, window::Direction::NorthEast);
	let resize_area_southeast_side =
		styled_resize_area(resize_thin, resize_thick, window::Direction::SouthEast);
	let resize_area_southeast_bottom =
		styled_resize_area(resize_thick, resize_thin, window::Direction::SouthEast);
	let resize_area_south = styled_resize_area(Fill, resize_thin, window::Direction::South);

	column![
		row![title_bar()],
		row![
			column![
				resize_area_northwest_side,
				resize_area_west,
				resize_area_southwest_side
			],
			column![
				main_content,
				button_bar,
				status_bar,
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

fn styled_resize_area<'a, WT: Into<Length>, LT: Into<Length>>(
	width: WT,
	height: LT,
	direction: window::Direction,
) -> Element<'a, Message> {
	mouse_area(
		container(space::horizontal().width(width).height(height)).style(|_theme| {
			container::Style {
				background: Some(colors::BG_SECONDARY.into()),
				border: border::Border {
					color: colors::BORDER_DIM,
					width: 1.0,
					radius: 0.0.into(),
				},
				..Default::default()
			}
		}),
	)
	.on_press(Message::ResizeWindow(direction))
	.into()
}

fn styled_tooltip<'a, Message>(
	underlay: impl Into<Element<'a, Message>>,
	label: &'a str,
) -> Tooltip<'a, Message>
where
	Message: 'a,
{
	tooltip(
		underlay,
		container(text(label.to_string()).color(colors::BRAND_GREEN).size(18)).padding(10),
		tooltip::Position::Right,
	)
}

fn styled_button<'a, Message: Clone + 'a>(
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

fn styled_text_input<'a, Message: Clone + 'a>(
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

fn styled_query_input<'a, Message: Clone + 'a>(
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
					color: colors::BORDER_ACCENT_QUERY,
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
					color: colors::BORDER_HOVER_QUERY,
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
					color: colors::BORDER_PRIMARY_QUERY,
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

fn styled_text_editor<'a>(
	id: String,
	content: &'a text_editor::Content,
) -> TextEditor<'a, PlainText, Message> {
	text_editor(content)
		.id(id)
		.size(18)
		.height(Fill)
		.wrapping(text::Wrapping::Word)
		.style(|_theme: &Theme, status: text_editor::Status| match status {
			text_editor::Status::Focused { .. } => text_editor::Style {
				background: Background::Color(colors::BG_INPUT_FOCUS),
				border: border::Border {
					color: colors::BORDER_ACCENT,
					width: 2.0,
					radius: 5.0.into(),
				},
				placeholder: colors::TEXT_PLACEHOLDER_HOVER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
			text_editor::Status::Hovered => text_editor::Style {
				background: Background::Color(colors::BG_INPUT_HOVER),
				border: border::Border {
					color: colors::BORDER_HOVER,
					width: 1.5,
					radius: 5.0.into(),
				},
				placeholder: colors::TEXT_PLACEHOLDER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
			_ => text_editor::Style {
				background: Background::Color(colors::BG_INPUT),
				border: border::Border {
					color: colors::BORDER_PRIMARY,
					width: 1.0,
					radius: 5.0.into(),
				},
				placeholder: colors::TEXT_PLACEHOLDER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
		})
}
