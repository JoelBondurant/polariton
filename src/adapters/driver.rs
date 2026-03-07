use crate::adapters::{
	common::{AdapterField, AdapterStage, DatabaseAdapter},
	parquet, postgres, sqlite,
};
use polars::prelude::{LazyFrame, PlRefPath, ScanArgsParquet};
use std::{collections::BTreeMap, path::Path, sync::Arc};
use tokio::sync::RwLock;
use tokio_rusqlite::Connection as AsyncConnection;

#[derive(Clone, Debug, Default)]
pub enum AdapterSelection {
	#[default]
	None,
	Parquet,
	Postgres,
	SQLite,
}

#[derive(Clone, Debug, Default)]
pub enum AdapterConfiguration {
	#[default]
	None,
	Parquet {
		file_path: String,
	},
	Postgres {
		connection_string: String,
	},
	SQLite {
		connection_string: String,
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
			AdapterSelection::Postgres => {
				let connection_string = self
					.fields
					.get("connection_string")
					.unwrap_or(&"postgresql://localhost".to_string())
					.clone();
				self.configuration = AdapterConfiguration::Postgres { connection_string };
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
			AdapterConfiguration::Postgres { connection_string } => {
				let (client, connection) = match tokio_postgres::connect(
					&connection_string,
					tokio_postgres::NoTls,
				)
				.await
				{
					Ok(res) => res,
					Err(_) => return None,
				};
				tokio::spawn(async move {
					if let Err(e) = connection.await {
						eprintln!("Postgres connection error: {}", e);
					}
				});
				Some(Arc::new(RwLock::new(postgres::PostgresAdapter { client })))
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
		AdapterSelection::Parquet => parquet::FIELDS,
		AdapterSelection::Postgres => postgres::FIELDS,
		AdapterSelection::SQLite => sqlite::FIELDS,
	}
}
