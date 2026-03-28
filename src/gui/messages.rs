use super::menu;
use crate::editor::EditorMsg;
use crate::adapters::{
	common::{DatabaseAdapter, ExecutionResult},
	driver::AdapterSelection,
};
use crate::persistence::{PrivateDb, SavedConnection, SavedStatement};
use crate::plot::colors::ColorTheme;
use crate::plot::common::{GridLineStyle, ScatterRenderMode};
use crate::plot::common::PlotKernel;
use crate::plot::core::PlotType;
use iced::{widget::pane_grid, window};
use iced::{Color, Rectangle};
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
	SVG,
	PNG,
	AVIF,
}

impl std::fmt::Display for ExportFormat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ExportFormat::SVG => write!(f, "SVG"),
			ExportFormat::PNG => write!(f, "PNG"),
			ExportFormat::AVIF => write!(f, "AVIF"),
		}
	}
}

#[derive(Clone)]
pub enum Message {
	AdapterConfigurationChanged(String, String),
	AdapterConfigurationSubmitted,
	AdapterConnected(Option<Arc<RwLock<dyn DatabaseAdapter>>>),
	AdapterSelected(AdapterSelection),
	AddPlot(PlotType),
	AddPlotReady(PlotType, Arc<dyn PlotKernel + Send + Sync>),
	ClosePlot(pane_grid::Pane),
	CloseSaveStatementDialog,
	CloseSettings,
	CloseWindow,
	CodeEditEvent(EditorMsg),
	Connect,
	ConnectionNameChanged(String),
	ConnectionSaved(Vec<SavedConnection>),
	DashboardPaneDragged(pane_grid::DragEvent),
	DashboardPaneResized(pane_grid::ResizeEvent),
	DeleteConnection(i64),
	DeleteStatement(i64),
	DoCloseWindow,
	DragWindow,
	EditConnection(i64),
	EditStatement(i64),
	Export(ExportFormat),
	ExportDone(usize, ExportFormat),
	ExportWithWindowSize(ExportFormat, Option<iced::Size>),
	LoadSavedConnection(i64),
	LoadSavedStatement(i64),
	MaximizeWindow,
	Menu(menu::MenuMessage),
	MinimizeWindow,
	OpenSaveStatementDialog,
	OpenSettings,
	PaneDragged(pane_grid::DragEvent),
	PaneResized(pane_grid::ResizeEvent),
	PasswordDecryptFailed,
	PasswordEntryChanged(String),
	PasswordEntrySubmit,
	PlotEvent(pane_grid::Pane, PlotMessage),
	RefreshPlotReady(pane_grid::Pane, PlotType, Arc<dyn PlotKernel + Send + Sync>),
	ResizePlotsSettled,
	PrivateDbError(String),
	PrivateDbReady(PrivateDb),
	PrivateDbRekeyed(PrivateDb),
	ResizeWindow(window::Direction),
	Run,
	RunResult(ExecutionResult),
	SaveConnection,
	SaveStatement,
	SaveStatementNameChanged(String),
	SaveWindowSizeAndClose(iced::Size),
	SavedConnectionsLoaded(Vec<SavedConnection>),
	SavedStatementsLoaded(Vec<SavedStatement>),
	SettingsApplyPassword,
	SettingsConfirmPasswordChanged(String),
	SettingsNewPasswordChanged(String),
	SettingsPasswordSaved,
	SettingsRemovePassword,
	ShowColumnTypesSaved,
	StatementSaved(Vec<SavedStatement>),
	ToggleShowColumnTypes(bool),
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum PlotMessage {
	ChangeBackgroundColor(Color),
	ChangeBackgroundHex(String),
	ChangeColorTheme(ColorTheme),
	ChangeDecorationColor(Color),
	ChangeDecorationHex(String),
	ChangePlotType(PlotType),
	CloseSettings,
	ApplySettings,
	RefreshData,
	SetLegendSize(f32),
	SetLegendX(f32),
	SetLegendY(f32),
	SetMaxLegendRows(u32),
	SetScatterDownsampleTarget(u32),
	SetScatterMaxVectorPoints(u32),
	SetScatterRasterThreshold(u32),
	SetScatterRenderMode(ScatterRenderMode),
	SetPlotPaddingBottom(f32),
	SetPlotPaddingLeft(f32),
	SetPlotPaddingRight(f32),
	SetPlotPaddingTop(f32),
	SetSubtitle(Option<String>),
	SetSubtitleOffset(f32),
	SetSubtitleSize(f32),
	SetTitle(Option<String>),
	SetTitleOffset(f32),
	SetTitleSize(f32),
	SetXLabel(Option<String>),
	SetXLabelPadding(f32),
	SetXLabelSize(f32),
	SetXMajorGridStyle(GridLineStyle),
	SetXMajorGridWidth(f32),
	SetXMax(Option<f64>),
	SetXMin(Option<f64>),
	SetXMinorGridStyle(GridLineStyle),
	SetXMinorGridWidth(f32),
	SetXMinorTicks(u32),
	SetXOffset(f32),
	SetXRotation(f32),
	SetXTickSize(f32),
	SetXTicks(u32),
	SetYLabel(Option<String>),
	SetYLabelPadding(f32),
	SetYLabelSize(f32),
	SetYMajorGridStyle(GridLineStyle),
	SetYMajorGridWidth(f32),
	SetYMax(Option<f64>),
	SetYMin(Option<f64>),
	SetYMinorGridStyle(GridLineStyle),
	SetYMinorGridWidth(f32),
	SetYMinorTicks(u32),
	SetYTickSize(f32),
	SetYTicks(u32),
	ToggleLiveUpdates(bool),
	ToggleSettings,
	ToggleXMajorGrid(bool),
	ToggleXMinorGrid(bool),
	ToggleXMinorTicks(bool),
	ToggleYMajorGrid(bool),
	ToggleYMinorGrid(bool),
	ToggleYMinorTicks(bool),
	UpdateBounds(Rectangle),
	UpdateHover(Option<String>),
}
