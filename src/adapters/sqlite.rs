use crate::adapters::common::{AdapterField, AdapterFieldType, DatabaseAdapter, ExecutionResult};
use async_trait::async_trait;
use polars::{
	datatypes::AnyValue,
	frame::{column::Column, DataFrame},
	series::Series,
};
use tokio_rusqlite::{
	rusqlite::{types::ValueRef, Connection as SyncConnection},
	Connection as AsyncConnection,
};

pub const FIELDS: &[AdapterField] = &[AdapterField {
	key: "connection_string",
	value: "memory",
	field_type: &AdapterFieldType::Text,
	is_secure: false,
}];

pub struct SQLiteAdapter {
	pub aconn: AsyncConnection,
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
impl DatabaseAdapter for SQLiteAdapter {
	async fn dispatch(&self, query: &str) -> ExecutionResult {
		let query = query.to_string();
		self.aconn
			.call(move |conn| sqlite_to_df(conn, &query))
			.await
			.map(ExecutionResult::Rows)
			.unwrap_or_else(|err| ExecutionResult::Err(err.to_string()))
	}
}

pub fn sqlite_to_df(conn: &SyncConnection, query: &str) -> Result<DataFrame, BoxError> {
	let mut stmt = conn.prepare(query)?;
	let col_names: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();
	let width = col_names.len();
	let total_rows: usize =
		conn.query_row(&format!("select count(1) from ({})", query), [], |row| {
			row.get(0)
		})?;
	let mut column_data: Vec<Vec<AnyValue>> = vec![Vec::with_capacity(total_rows); width];
	let mut rows = stmt.query([])?;
	while let Some(row) = rows.next()? {
		for (indx, bucket) in column_data.iter_mut().enumerate() {
			let val = match row.get_ref(indx)? {
				ValueRef::Null => AnyValue::Null,
				ValueRef::Integer(num) => AnyValue::Int64(num),
				ValueRef::Real(flt) => AnyValue::Float64(flt),
				ValueRef::Text(txt) => AnyValue::StringOwned(std::str::from_utf8(txt)?.into()),
				ValueRef::Blob(blb) => AnyValue::BinaryOwned(blb.to_vec()),
			};
			bucket.push(val);
		}
	}
	let polars_columns: Vec<Column> = col_names
		.into_iter()
		.enumerate()
		.map(|(indx, name)| {
			let ser = Series::from_any_values(name.into(), &column_data[indx], true).unwrap();
			Column::from(ser)
		})
		.collect();
	let df = DataFrame::new(total_rows, polars_columns)?;
	Ok(df)
}
