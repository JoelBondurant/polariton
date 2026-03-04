use async_trait::async_trait;
use polars::prelude::DataFrame;

#[derive(Clone, Debug, Default)]
pub enum AdapterStage {
	#[default]
	None,
	Unselected,
	Unconfigured,
	Configured,
	Connected,
}

#[derive(Clone, Debug)]
pub enum AdapterFieldType {
	Text,
}

#[derive(Clone, Debug)]
pub struct AdapterField {
	pub key: &'static str,
	pub value: &'static str,
	pub field_type: &'static AdapterFieldType,
	pub is_secure: bool,
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
pub trait DatabaseAdapter: Send + Sync + 'static {
	async fn execute(&self, query: &str) -> Result<DataFrame, BoxError>;
}
