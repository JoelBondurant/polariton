use crate::adapters::{
	common::{DatabaseAdapter, ExecutionResult},
	driver::AdapterSelection,
};
use crate::plot::colors::ColorTheme;
use crate::plot::common::GridLineStyle;
use crate::plot::core::PlotType;
use iced::{widget::pane_grid, window};
use iced::{Color, Rectangle};
use iced_code_editor::Message as EditorMessage;
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
	SVG,
	PNG,
}

impl std::fmt::Display for ExportFormat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ExportFormat::SVG => write!(f, "SVG"),
			ExportFormat::PNG => write!(f, "PNG"),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotAction {
	Add(PlotType),
	Export(ExportFormat),
}

impl std::fmt::Display for PlotAction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PlotAction::Add(pt) => write!(f, "New > {}", pt),
			PlotAction::Export(fmt) => write!(f, "Export > {}", fmt),
		}
	}
}

impl PlotAction {
	pub const ALL: [PlotAction; 22] = [
		PlotAction::Add(PlotType::Bar),
		PlotAction::Add(PlotType::BoxPlot),
		PlotAction::Add(PlotType::Bubble),
		PlotAction::Add(PlotType::Candlestick),
		PlotAction::Add(PlotType::FillBetween),
		PlotAction::Add(PlotType::Funnel),
		PlotAction::Add(PlotType::Heatmap),
		PlotAction::Add(PlotType::Hexbin),
		PlotAction::Add(PlotType::Histogram),
		PlotAction::Add(PlotType::HorizontalBar),
		PlotAction::Add(PlotType::HorizontalStackedBar),
		PlotAction::Add(PlotType::Line),
		PlotAction::Add(PlotType::Parallel),
		PlotAction::Add(PlotType::Pie),
		PlotAction::Add(PlotType::Radar),
		PlotAction::Add(PlotType::RadialDial),
		PlotAction::Add(PlotType::Scatter),
		PlotAction::Add(PlotType::StackedArea),
		PlotAction::Add(PlotType::StackedBar),
		PlotAction::Add(PlotType::Violin),
		PlotAction::Export(ExportFormat::SVG),
		PlotAction::Export(ExportFormat::PNG),
	];
}

#[derive(Clone)]
pub enum Message {
	AdapterConfigurationChanged(String, String),
	AdapterConfigurationSubmitted,
	AdapterConnected(Option<Arc<RwLock<dyn DatabaseAdapter>>>),
	AdapterSelected(AdapterSelection),
	CloseWindow,
	CodeEditEvent(EditorMessage),
	Connect,
	DragWindow,
	MaximizeWindow,
	MinimizeWindow,
	PaneDragged(pane_grid::DragEvent),
	PaneResized(pane_grid::ResizeEvent),
	DashboardPaneDragged(pane_grid::DragEvent),
	DashboardPaneResized(pane_grid::ResizeEvent),
	#[allow(dead_code)]
	ResizeWindow(window::Direction),
	Run,
	RunResult(ExecutionResult),
	AddPlot(PlotType),
	PlotEvent(pane_grid::Pane, PlotMessage),
	ClosePlot(pane_grid::Pane),
	Export(ExportFormat),
	ExportWithWindowSize(ExportFormat, Option<iced::Size>),
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum PlotMessage {
	RefreshData,
	UpdateHover(Option<String>),
	UpdateBounds(Rectangle),
	ChangePlotType(PlotType),
	SetMaxLegendRows(u32),
	SetLegendX(f32),
	SetLegendY(f32),
	SetXRotation(f32),
	SetXOffset(f32),
	ChangeColorTheme(ColorTheme),
	ChangeBackgroundColor(Color),
	ChangeBackgroundHex(String),
	ChangeDecorationColor(Color),
	ChangeDecorationHex(String),
	SetXMin(Option<f64>),
	SetXMax(Option<f64>),
	SetYMin(Option<f64>),
	SetYMax(Option<f64>),
	SetTitle(Option<String>),
	SetSubtitle(Option<String>),
	SetXLabel(Option<String>),
	SetYLabel(Option<String>),
	SetTitleOffset(f32),
	SetSubtitleOffset(f32),
	SetXLabelPadding(f32),
	SetYLabelPadding(f32),
	SetPlotPaddingTop(f32),
	SetPlotPaddingBottom(f32),
	SetPlotPaddingLeft(f32),
	SetPlotPaddingRight(f32),
	SetTitleSize(f32),
	SetSubtitleSize(f32),
	SetXLabelSize(f32),
	SetYLabelSize(f32),
	SetXTickSize(f32),
	SetYTickSize(f32),
	SetLegendSize(f32),
	SetXTicks(u32),
	SetYTicks(u32),
	SetXMinorTicks(u32),
	SetYMinorTicks(u32),
	ToggleXMinorTicks(bool),
	ToggleYMinorTicks(bool),
	ToggleXMajorGrid(bool),
	ToggleYMajorGrid(bool),
	ToggleXMinorGrid(bool),
	ToggleYMinorGrid(bool),
	SetXMajorGridWidth(f32),
	SetYMajorGridWidth(f32),
	SetXMinorGridWidth(f32),
	SetYMinorGridWidth(f32),
	SetXMajorGridStyle(GridLineStyle),
	SetYMajorGridStyle(GridLineStyle),
	SetXMinorGridStyle(GridLineStyle),
	SetYMinorGridStyle(GridLineStyle),
	ToggleSettings,
	CloseSettings,
}
