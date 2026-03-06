use crate::adapters::{
	common::{AdapterField, AdapterStage, DatabaseAdapter},
	parquet, sqlite,
};
use polars::prelude::{LazyFrame, PlRefPath, ScanArgsParquet};
use std::{collections::BTreeMap, path::Path, sync::Arc};
use tokio::sync::RwLock;
use tokio_rusqlite::Connection as AsyncConnection;

#[derive(Clone, Debug, Default)]
pub enum AdapterSelection {
	#[default]
	None,
	SQLite,
	Parquet,
}

#[derive(Clone, Debug, Default)]
pub enum AdapterConfiguration {
	#[default]
	None,
	SQLite {
		connection_string: String,
	},
	Parquet {
		file_path: String,
	},
}

#[derive(Default)]
pub struct AdapterState {
	pub stage: AdapterStage,
	pub selection: AdapterSelection,
	pub fields: BTreeMap<String, String>,
	pub configuration: AdapterConfiguration,
	pub connection: Option<Arc<RwLock<dyn DatabaseAdapter>>>,
}

impl AdapterState {
	pub fn configure(&mut self) {
		match &self.selection {
			AdapterSelection::None => {
				self.configuration = AdapterConfiguration::None;
				self.stage = AdapterStage::Unconfigured;
			}
			AdapterSelection::Parquet => {
				let file_path = self.fields.get("file_path").unwrap().clone();
				self.configuration = AdapterConfiguration::Parquet { file_path };
				self.stage = AdapterStage::Configured;
			}
			AdapterSelection::SQLite => {
				let connection_string = self
					.fields
					.get("connection_string")
					.unwrap_or(&"memory".to_string())
					.clone();
				self.configuration = AdapterConfiguration::SQLite { connection_string };
				self.stage = AdapterStage::Configured;
			}
		}
	}

	pub async fn connect(config: AdapterConfiguration) -> Option<Arc<RwLock<dyn DatabaseAdapter>>> {
		match config {
			AdapterConfiguration::None => None,
			AdapterConfiguration::Parquet { file_path } => {
				let file_path = Path::new(&file_path);
				let file_prefix = file_path.file_prefix().unwrap().to_str().unwrap_or("");
				let ref_path = PlRefPath::try_from_path(file_path).unwrap();
				let lf = LazyFrame::scan_parquet(ref_path, ScanArgsParquet::default()).unwrap();
				let context = polars::sql::SQLContext::new();
				context.register(file_prefix, lf);
				Some(Arc::new(RwLock::new(parquet::ParquetAdapter { context })))
			}
			AdapterConfiguration::SQLite { connection_string } => {
				if connection_string == "memory" {
					match AsyncConnection::open_in_memory().await.ok() {
						Some(aconn) => Some(Arc::new(RwLock::new(sqlite::SQLiteAdapter { aconn }))),
						None => None,
					}
				} else {
					match AsyncConnection::open(connection_string).await.ok() {
						Some(aconn) => Some(Arc::new(RwLock::new(sqlite::SQLiteAdapter { aconn }))),
						None => None,
					}
				}
			}
		}
	}
}

pub fn fields_for(selection: &AdapterSelection) -> &'static [AdapterField] {
	match selection {
		AdapterSelection::None => &[],
		AdapterSelection::SQLite => sqlite::FIELDS,
		AdapterSelection::Parquet => parquet::FIELDS,
	}
}
