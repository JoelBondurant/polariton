use crate::adapters::{
	common::{AdapterField, AdapterStage, DatabaseAdapter},
	sqlite,
};
use std::{collections::BTreeMap, sync::Arc};
use tokio_rusqlite::Connection as AsyncConnection;

#[derive(Clone, Debug, Default)]
pub enum AdapterSelection {
	#[default]
	None,
	SQLite,
}

#[derive(Clone, Debug, Default)]
pub enum AdapterConfiguration {
	#[default]
	None,
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
	pub connection: Option<Arc<dyn DatabaseAdapter>>,
}

impl AdapterState {
	pub fn configure(&mut self) {
		match &self.selection {
			AdapterSelection::None => {
				self.configuration = AdapterConfiguration::None;
				self.stage = AdapterStage::Unconfigured;
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

	pub async fn establish_connection(
		config: AdapterConfiguration,
	) -> Option<Arc<dyn DatabaseAdapter>> {
		match config {
			AdapterConfiguration::None => None,
			AdapterConfiguration::SQLite { connection_string } => {
				match AsyncConnection::open(connection_string).await.ok() {
					Some(aconn) => Some(Arc::new(sqlite::SQLiteAdapter { aconn })),
					None => None,
				}
			}
		}
	}
}

pub fn fields_for(selection: &AdapterSelection) -> &'static [AdapterField] {
	match selection {
		AdapterSelection::None => &[],
		AdapterSelection::SQLite => sqlite::FIELDS,
	}
}
