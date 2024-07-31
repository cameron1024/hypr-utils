use crate::args::{CliJson, StoreCmd};
use crate::Context;

use super::APP_NAME;
use std::error::Error;
use std::io::Write;
use std::path::Path;
use std::process::Command;

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
            if let Some(value) = value {
                ctx.write_json(&value)?;
                ctx.writeln()?;
            }
            Ok(())
        }
        StoreCmd::Set { key, value } => store.set(&key, &value.0),
        StoreCmd::Unset { key } => {
            let deleted = store.unset(&key)?;
            writeln!(&mut ctx.out, "{deleted}")?;

            Ok(())
        }
        StoreCmd::Cached { cmd } => {
            let output = Command::new("sh").arg("-c").arg(&cmd).output()?;
            match output.status.success() {
                true => {
                    let stdout = String::from_utf8(output.stdout).unwrap();
                    store.cache(&cmd, &stdout)?;
                    writeln!(&mut ctx.out, "{stdout}")?;
                }
                false => {
                    let Some(output) = store.read_cache(&cmd)? else {
                        panic!("no cached result for `{cmd}`");
                    };

                    writeln!(&mut ctx.out, "{output}")?;
                }
            }
            Ok(())
        }
        StoreCmd::List => {
            store.list_all(&mut ctx)?;
            Ok(())
        }
        StoreCmd::Cycle {
            key,
            values,
            reverse,
        } => {
            if values.len() == 0 {
                return Ok(());
            }

            let current_value = match store.get(&key)? {
                None => values.first().unwrap(),
                Some(value) => match values.iter().position(|CliJson(json)| json == &value) {
                    None => values.first().unwrap(),
                    Some(index) => {
                        let index = match reverse {
                            true => (index + values.len() - 1) % values.len(),
                            false => (index + 1) % values.len(),
                        };
                        values.get(index).unwrap()
                    }
                },
            };

            store.set(&key, &current_value.0)?;
            ctx.write_json(&current_value.0)?;
            ctx.writeln()?;

            Ok(())
        }
    }
}

struct Store {
    db: Connection,
}

impl Store {
    pub fn new_with_path(path: &Path) -> Result<Self, Box<dyn Error>> {
        let db = rusqlite::Connection::open(path)?;

        Ok(Self { db })
    }

    pub fn new() -> Result<Self, Box<dyn Error>> {
        let path =
            xdg::BaseDirectories::new()?.place_data_file(format!("{APP_NAME}/store.sqlite"))?;

        Self::new_with_path(&path)
    }

    fn create_data_table(&self) -> Result<(), Box<dyn Error>> {
        self.db.execute(
            "CREATE TABLE IF NOT EXISTS data (id TEXT PRIMARY KEY, json JSONB NOT NULL)",
            [],
        )?;
        Ok(())
    }

    fn create_cache_table(&self) -> Result<(), Box<dyn Error>> {
        self.db.execute(
            "CREATE TABLE IF NOT EXISTS cache (id TEXT PRIMARY KEY, output TEXT NOT NULL)",
            [],
        )?;
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Value>, Box<dyn Error>> {
        self.create_data_table()?;

        let mut statement = self.db.prepare("SELECT json FROM data WHERE id = ?1")?;
        let mut iter = statement.query_map([key], |row| row.get::<_, serde_json::Value>(0))?;
        match iter.next() {
            None => Ok(None),
            Some(value) => Ok(Some(value?)),
        }
    }

    fn set(&self, key: &str, value: &serde_json::Value) -> Result<(), Box<dyn Error>> {
        self.create_data_table()?;
        self.db
            .execute("INSERT OR REPLACE INTO data values (?1, ?2)", (key, value))?;

        Ok(())
    }

    fn unset(&self, key: &str) -> Result<bool, Box<dyn Error>> {
        self.create_data_table()?;
        let rows = self.db.execute("DELETE FROM data WHERE id = ?1", [key])?;

        match rows {
            0 => Ok(false),
            1 => Ok(true),
            other => unreachable!("how... {other}"),
        }
    }

    fn read_cache(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        self.create_cache_table()?;

        let mut statement = self.db.prepare("SELECT output FROM cache WHERE id = ?1")?;
        let mut iter = statement.query_map([key], |row| row.get::<_, String>(0))?;
        match iter.next() {
            None => Ok(None),
            Some(value) => Ok(Some(value?)),
        }
    }

    fn cache(&self, cmd: &str, output: &str) -> Result<(), Box<dyn Error>> {
        self.create_cache_table()?;
        self.db.execute(
            "INSERT OR REPLACE INTO cache values (?1, ?2)",
            (cmd, output),
        )?;

        Ok(())
    }

    fn list_all<W: Write>(&self, ctx: &mut Context<W>) -> Result<(), Box<dyn Error>> {
        let mut statement = self.db.prepare("SELECT id, json FROM data")?;
        let iter = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0).unwrap(),
                row.get::<_, Value>(1).unwrap(),
            ))
        })?;

        for result in iter {
            let (key, value) = result.unwrap();

            write!(&mut ctx.out, "{key} ")?;
            ctx.write_json(&value)?;
            ctx.writeln()?;
        }

        Ok(())

        // let statement = self.db.prepare("SELECT id, json FROM data", [])?;
    }
}
