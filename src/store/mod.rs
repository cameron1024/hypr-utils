use crate::args::StoreCmd;
use crate::Context;

use super::APP_NAME;
use std::error::Error;
use std::io::Write;
use std::path::Path;

use rusqlite::Connection;
use serde_json::Value;

pub fn handle_cmd<W: Write>(
    mut ctx: Context<W>,
    store_cmd: StoreCmd,
) -> Result<(), Box<dyn Error>> {
    let store = Store::new()?;
    match store_cmd {
        StoreCmd::Get { key } => {
            let value = store.get(&key)?;
            writeln!(&mut ctx.out, "{value}")?;
            Ok(())
        }
        StoreCmd::Set { key, value } => store.set(&key, &value.0),
    }
}

struct Store {
    db: Connection,
}

impl Store {
    pub fn new_with_path(path: &Path) -> Result<Self, Box<dyn Error>> {
        let db = rusqlite::Connection::open(path)?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS data (id TEXT PRIMARY KEY, json JSONB NOT NULL)",
            [],
        )?;

        Ok(Self { db })
    }

    pub fn new() -> Result<Self, Box<dyn Error>> {
        let path =
            xdg::BaseDirectories::new()?.place_data_file(format!("{APP_NAME}/store.sqlite"))?;

        Self::new_with_path(&path)
    }

    pub fn get(&self, key: &str) -> Result<Value, Box<dyn Error>> {
        let mut statement = self.db.prepare("SELECT json FROM data WHERE id = ?1")?;
        let mut iter = statement.query_map([key], |row| row.get::<_, serde_json::Value>(0))?;
        match iter.next() {
            None => Ok(Value::Null),
            Some(value) => Ok(value?),
        }
    }

    pub fn set(&self, key: &str, value: &serde_json::Value) -> Result<(), Box<dyn Error>> {
        self.db
            .execute("INSERT OR REPLACE INTO data values (?1, ?2)", (key, value))?;

        Ok(())
    }
}
