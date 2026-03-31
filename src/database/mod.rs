use rusqlite::Connection;

use crate::{
    constants::{
        CACHE_DIR, CREATE_SCHEDULES_TABLE, CREATE_TOKENS_TABLE, DB_FILENAME, SCHEDULE_TABLE_NAME,
        TOKEN_TABLE_NAME,
    },
    utils::gracefully_exit,
};

pub struct Database {
    table_name: &'static str,
}

impl Database {
    pub fn new(table_name: &'static str) -> Self {
        Self { table_name }
    }

    pub fn open_connection(&self) -> Connection {
        let data_dir = match dirs::data_dir() {
            Some(path) => path,
            None => gracefully_exit("Failed to locate a data directory for scheduled tweets."),
        };

        let cli_data_dir = data_dir.join(CACHE_DIR);
        if let Err(err) = std::fs::create_dir_all(&cli_data_dir) {
            gracefully_exit(&format!(
                "Failed to create schedule data directory '{}': {err}",
                cli_data_dir.display()
            ));
        }

        let path = cli_data_dir.join(DB_FILENAME);
        let connection = match Connection::open(path) {
            Ok(connection) => connection,
            Err(err) => gracefully_exit(&format!(
                "Failed to open the {} database: {err}",
                self.table_name
            )),
        };

        let migration_query = match self.table_name {
            SCHEDULE_TABLE_NAME => CREATE_SCHEDULES_TABLE,
            TOKEN_TABLE_NAME => CREATE_TOKENS_TABLE,
            _ => todo!(),
        };

        if let Err(err) = connection.execute(migration_query, []) {
            gracefully_exit(&format!(
                "Failed to initialize {} database schema: {err}",
                self.table_name
            ));
        }

        connection
    }
}
