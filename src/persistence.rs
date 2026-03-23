use tokio_rusqlite::Connection;

#[derive(Clone, Debug)]
pub struct SavedConnection {
	pub id: i64,
	pub name: String,
	pub adapter_type: String,
	pub config_value: String,
}

fn db_path() -> std::path::PathBuf {
	let mut pth = std::env::home_dir().unwrap_or_default();
	pth.push(".config/polariton/");
	std::fs::create_dir_all(&pth).ok();
	pth.push("settings.sqlite");
	pth
}

async fn open() -> Connection {
	let conn = Connection::open(db_path()).await.unwrap();
	conn.call(|db| {
		db.execute_batch(
			"CREATE TABLE IF NOT EXISTS connections (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				name TEXT NOT NULL,
				adapter_type TEXT NOT NULL,
				config_value TEXT NOT NULL
			)",
		)?;
		Ok::<(), tokio_rusqlite::rusqlite::Error>(())
	})
	.await
	.unwrap();
	conn
}

pub async fn load() -> Vec<SavedConnection> {
	let conn = open().await;
	conn.call(|db| {
		let mut stmt =
			db.prepare("SELECT id, name, adapter_type, config_value FROM connections ORDER BY id")?;
		let rows = stmt
			.query_map([], |row| {
				Ok(SavedConnection {
					id: row.get(0)?,
					name: row.get(1)?,
					adapter_type: row.get(2)?,
					config_value: row.get(3)?,
				})
			})?
			.filter_map(|r| r.ok())
			.collect();
		Ok::<Vec<SavedConnection>, tokio_rusqlite::rusqlite::Error>(rows)
	})
	.await
	.unwrap_or_default()
}

pub async fn delete(id: i64) -> Vec<SavedConnection> {
	let conn = open().await;
	conn.call(move |db| {
		db.execute("DELETE FROM connections WHERE id = ?1", [id])?;
		Ok::<(), tokio_rusqlite::rusqlite::Error>(())
	})
	.await
	.ok();
	load().await
}

pub async fn update(id: i64, name: String, config_value: String) -> Vec<SavedConnection> {
	let conn = open().await;
	conn.call(move |db| {
		db.execute(
			"UPDATE connections SET name = ?1, config_value = ?2 WHERE id = ?3",
			(name.as_str(), config_value.as_str(), id),
		)?;
		Ok::<(), tokio_rusqlite::rusqlite::Error>(())
	})
	.await
	.ok();
	load().await
}

pub async fn save(
	name: String,
	adapter_type: String,
	config_value: String,
) -> Vec<SavedConnection> {
	let conn = open().await;
	conn.call(move |db| {
		db.execute(
			"INSERT INTO connections (name, adapter_type, config_value) VALUES (?1, ?2, ?3)",
			(name.as_str(), adapter_type.as_str(), config_value.as_str()),
		)?;
		Ok::<(), tokio_rusqlite::rusqlite::Error>(())
	})
	.await
	.ok();
	conn.call(|db| {
		let mut stmt =
			db.prepare("SELECT id, name, adapter_type, config_value FROM connections ORDER BY id")?;
		let rows = stmt
			.query_map([], |row| {
				Ok(SavedConnection {
					id: row.get(0)?,
					name: row.get(1)?,
					adapter_type: row.get(2)?,
					config_value: row.get(3)?,
				})
			})?
			.filter_map(|r| r.ok())
			.collect();
		Ok::<Vec<SavedConnection>, tokio_rusqlite::rusqlite::Error>(rows)
	})
	.await
	.unwrap_or_default()
}
