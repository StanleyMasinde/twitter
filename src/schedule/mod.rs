use std::{fmt::Display, str::FromStr};

use jiff::Timestamp;
use parse_datetime::parse_datetime;
use rusqlite::{
    Connection,
    types::{FromSql, FromSqlError, ValueRef},
};

const TABLE_NAME: &str = "scheduled_tweets";
const CACHE_DIR: &str = "twitter-cli";
const DB_FILENAME: &str = "db.sqlite3";

use crate::{twitter::tweet::TweetBody, utils::gracefully_exit};

#[derive(Debug)]
pub enum ScheduleStatus {
    Pending,
    Sent,
    Failed,
}

impl Display for ScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            ScheduleStatus::Pending => "Pending",
            ScheduleStatus::Sent => "Sent",
            ScheduleStatus::Failed => "Failed",
        };
        write!(f, "{text}")
    }
}

impl FromSql for ScheduleStatus {
    fn column_result(value: ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let status = value.as_str().map_err(|_| FromSqlError::InvalidType)?;
        match status {
            "pending" => Ok(ScheduleStatus::Pending),
            "sent" => Ok(ScheduleStatus::Sent),
            "failed" => Ok(ScheduleStatus::Failed),
            other => Err(FromSqlError::Other(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid schedule status in database: {other}"),
                )
                .into(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct ScheduledTweet {
    pub id: u32,
    pub body: String,
    pub status: ScheduleStatus,
    pub scheduled_for: String,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub sent_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct Schedule {
    tweet_body: TweetBody,
    send_time: Timestamp,
    connection: Connection,
}

impl Default for Schedule {
    fn default() -> Self {
        let connection = Self::open_connection();
        Self {
            tweet_body: Default::default(),
            send_time: Default::default(),
            connection,
        }
    }
}

impl Schedule {
    pub fn new(body: &str, time: &str) -> Self {
        let tweet_body = match TweetBody::from_str(body) {
            Ok(body) => body,
            Err(_) => gracefully_exit("Invalid tweet body."),
        };

        let zone_local_time = match parse_datetime(time) {
            Ok(parsed_time) => parsed_time,
            Err(err) => {
                gracefully_exit(&format!("Invalid scheduled time '{time}': {err}"));
            }
        };

        let send_time = zone_local_time.timestamp();
        let connection = Self::open_connection();

        Self {
            tweet_body,
            send_time,
            connection,
        }
    }

    pub fn save(self) -> bool {
        let query = format!(
            "
            INSERT INTO {TABLE_NAME} (
                body,
                scheduled_for
            ) VALUES (?1, ?2);
            ",
        );

        if let Err(err) = self
            .connection
            .execute(&query, (self.tweet_body.text, self.send_time.to_string()))
        {
            eprintln!("Failed to save scheduled tweet: {err}");
            return false;
        }

        true
    }

    pub fn all(&self) -> Vec<ScheduledTweet> {
        let query = format!("SELECT * from {TABLE_NAME}");
        self.query_tweets(&query)
    }

    pub fn clear(&self) -> usize {
        let query = format!("DELETE FROM {TABLE_NAME}");
        match self.connection.execute(&query, ()) {
            Ok(cleared_rows) => cleared_rows,
            Err(err) => {
                eprintln!("Failed to clear scheduled tweets: {err}");
                0
            }
        }
    }

    pub(crate) fn due(&self) -> Vec<ScheduledTweet> {
        let query = format!(
            "SELECT * from {TABLE_NAME} WHERE datetime('now') > datetime(scheduled_for) AND status = 'pending'"
        );
        self.query_tweets(&query)
    }

    pub(crate) fn failed(&self) -> Vec<ScheduledTweet> {
        let query = format!("SELECT * from {TABLE_NAME} WHERE status = 'failed'");
        self.query_tweets(&query)
    }

    pub(crate) fn sent(&self) -> Vec<ScheduledTweet> {
        let query = format!("SELECT * from {TABLE_NAME} WHERE status = 'sent'");
        self.query_tweets(&query)
    }

    pub(crate) fn mark_sent(&self, id: u32) {
        let query = format!(
            "UPDATE {TABLE_NAME}
             SET status = 'sent',
                 sent_at = CURRENT_TIMESTAMP,
                 last_error = NULL,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1"
        );
        if let Err(err) = self.connection.execute(&query, [id]) {
            eprintln!("Failed to mark scheduled tweet {} as sent: {err}", id);
        }
    }

    pub(crate) fn mark_failed(&self, id: u32, error_message: &str) {
        let query = format!(
            "UPDATE {TABLE_NAME}
             SET status = 'failed',
                 attempts = attempts + 1,
                 last_error = ?1,
                 sent_at = NULL,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = ?2"
        );
        if let Err(err) = self.connection.execute(&query, (error_message, id)) {
            eprintln!("Failed to mark scheduled tweet {} as failed: {err}", id);
        }
    }

    fn open_connection() -> Connection {
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
            Err(err) => gracefully_exit(&format!("Failed to open schedule database: {err}")),
        };

        if let Err(err) = connection.execute(&Self::migration_query(), []) {
            gracefully_exit(&format!(
                "Failed to initialize schedule database schema: {err}"
            ));
        }

        connection
    }

    fn migration_query() -> String {
        format!(
            "
            CREATE TABLE IF NOT EXISTS {TABLE_NAME} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                body TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending'
                     CHECK (status IN ('pending', 'sent', 'failed')),
                scheduled_for DATETIME NOT NULL,
                attempts INTEGER NOT NULL DEFAULT 0
                    CHECK (attempts >= 0),
                last_error TEXT,
                sent_at DATETIME,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                -- Ensure sent_at is set only when status = 'sent'
            CHECK (
             (status = 'sent' AND sent_at IS NOT NULL)
                OR (status <> 'sent')
                )
            );
            ",
        )
    }

    fn query_tweets(&self, query: &str) -> Vec<ScheduledTweet> {
        let mut stmt = match self.connection.prepare(query) {
            Ok(stmt) => stmt,
            Err(err) => {
                eprintln!("Failed to prepare schedule query: {err}");
                return vec![];
            }
        };

        let rows = match stmt.query_map([], |row| {
            Ok(ScheduledTweet {
                id: row.get(0)?,
                body: row.get(1)?,
                status: row.get(2)?,
                scheduled_for: row.get(3)?,
                attempts: row.get(4)?,
                last_error: row.get(5)?,
                sent_at: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        }) {
            Ok(rows) => rows,
            Err(err) => {
                eprintln!("Failed to run schedule query: {err}");
                return vec![];
            }
        };

        let mut tweets = vec![];
        for row in rows {
            match row {
                Ok(tweet) => tweets.push(tweet),
                Err(err) => eprintln!("Failed to decode scheduled tweet row: {err}"),
            }
        }

        tweets
    }
}

#[cfg(test)]
mod test {
    use std::{env, fs};

    use crate::schedule::Schedule;
    use serial_test::serial;

    fn setup_test_data_dir() {
        let base = env::temp_dir().join("twitter-cli-tests");
        let data_dir = base.join("data");
        let home_dir = base.join("home");
        let _ = fs::create_dir_all(&data_dir);
        let _ = fs::create_dir_all(&home_dir);
        unsafe {
            env::set_var("XDG_DATA_HOME", data_dir);
            env::set_var("HOME", home_dir);
        }
    }

    #[test]
    #[serial]
    fn schedule_save_tweet() {
        setup_test_data_dir();
        let body = "This is a scheduled Tweet";
        let time = "Tomorrow";
        let scheduled_tweet = Schedule::new(body, time);
        let _ = scheduled_tweet.save();
    }

    #[test]
    #[serial]
    fn schedule_get_all() {
        setup_test_data_dir();
        let schedule_instance = Schedule::default();
        let all = schedule_instance.all();
        println!("{:?}", all)
    }

    #[test]
    #[serial]
    fn schedule_filter_failed_and_sent() {
        setup_test_data_dir();
        let schedule = Schedule::default();
        let _ = schedule.clear();

        let _ = Schedule::new("failed tweet", "Tomorrow").save();
        let _ = Schedule::new("sent tweet", "Tomorrow").save();

        let all = schedule.all();
        let failed_id = all
            .iter()
            .find(|tweet| tweet.body == "failed tweet")
            .unwrap()
            .id;
        let sent_id = all
            .iter()
            .find(|tweet| tweet.body == "sent tweet")
            .unwrap()
            .id;

        schedule.mark_failed(failed_id, "network error");
        schedule.mark_sent(sent_id);

        let failed = schedule.failed();
        let sent = schedule.sent();

        assert!(failed.iter().any(|tweet| tweet.id == failed_id));
        assert!(sent.iter().any(|tweet| tweet.id == sent_id));
    }

    #[test]
    #[serial]
    fn schedule_mark_failed_tracks_error() {
        setup_test_data_dir();
        let schedule = Schedule::default();
        let _ = schedule.clear();

        let _ = Schedule::new("tweet that fails", "Tomorrow").save();
        let all = schedule.all();
        let tweet_id = all
            .iter()
            .find(|tweet| tweet.body == "tweet that fails")
            .unwrap()
            .id;

        schedule.mark_failed(tweet_id, "timeout");
        let failed = schedule.failed();
        let row = failed.iter().find(|tweet| tweet.id == tweet_id).unwrap();

        assert_eq!(row.attempts, 1);
        assert_eq!(row.last_error.as_deref(), Some("timeout"));
    }

    // #[test]
    // fn schedule_clear() {
    // let delete_instance = Schedule::default();
    // delete_instance.clear();

    // let all_instance = Schedule::default();
    // let all = all_instance.all();
    // println!("{:?}", all)
    // }
}
