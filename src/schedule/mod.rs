use std::{fmt::Display, str::FromStr};

use jiff::Timestamp;
use parse_datetime::parse_datetime;
use rusqlite::{Connection, types::FromSql};

const TABLE_NAME: &str = "scheduled_tweets";
const CACHE_DIR: &str = "twitter-cli";
const DB_FILENAME: &str = "db.sqlite3";

use crate::twitter::tweet::TweetBody;

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
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let val = match value.as_str().unwrap() {
            "pending" => ScheduleStatus::Pending,
            "sent" => ScheduleStatus::Sent,
            "failed" => ScheduleStatus::Failed,
            _ => ScheduleStatus::Pending,
        };
        Ok(val)
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
        let cli_data_dir = dirs::data_dir().unwrap().join(CACHE_DIR);
        let path = cli_data_dir.join(DB_FILENAME);
        let connection = Connection::open(path).unwrap();
        Self {
            tweet_body: Default::default(),
            send_time: Default::default(),
            connection,
        }
    }
}

impl Schedule {
    pub fn new(body: &str, time: &str) -> Self {
        let tweet_body = TweetBody::from_str(body).unwrap();
        let zone_local_time = parse_datetime(time).unwrap();
        let send_time = zone_local_time.timestamp();
        let cli_data_dir = dirs::data_dir().unwrap().join(CACHE_DIR);
        let path = cli_data_dir.join(DB_FILENAME);
        let connection = Connection::open(path).unwrap();
        let migrations_query = format!(
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
        );

        connection.execute(&migrations_query, []).unwrap();

        Self {
            tweet_body,
            send_time,
            connection,
        }
    }

    pub fn save(self) {
        let query = format!(
            "
            INSERT INTO {TABLE_NAME} (
                body,
                scheduled_for
            ) VALUES (?1, ?2);
            ",
        );
        self.connection
            .execute(&query, (self.tweet_body.text, self.send_time.to_string()))
            .unwrap();
    }

    pub fn all(self) -> Vec<ScheduledTweet> {
        let query = format!("SELECT * from {TABLE_NAME}");
        let mut stmt = self.connection.prepare(&query).unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok(ScheduledTweet {
                    id: row.get(0).unwrap(),
                    body: row.get(1).unwrap(),
                    status: row.get(2).unwrap(),
                    scheduled_for: row.get(3).unwrap(),
                    attempts: row.get(4).unwrap(),
                    last_error: row.get(5).unwrap(),
                    sent_at: row.get(6).unwrap(),
                    created_at: row.get(7).unwrap(),
                    updated_at: row.get(8).unwrap(),
                })
            })
            .unwrap();

        let mut tweets: Vec<ScheduledTweet> = vec![];

        for row in rows {
            tweets.push(row.unwrap());
        }

        tweets
    }

    pub fn clear(self) {
        let query = format!("DELETE FROM {TABLE_NAME}");
        self.connection.execute(&query, ()).unwrap();
    }

    pub(crate) fn due(&self) -> Vec<ScheduledTweet> {
        let query =
            format!("SELECT * from {TABLE_NAME} WHERE datetime('now') > datetime(scheduled_for)");

        let mut stmt = self.connection.prepare(&query).unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok(ScheduledTweet {
                    id: row.get(0).unwrap(),
                    body: row.get(1).unwrap(),
                    status: row.get(2).unwrap(),
                    scheduled_for: row.get(3).unwrap(),
                    attempts: row.get(4).unwrap(),
                    last_error: row.get(5).unwrap(),
                    sent_at: row.get(6).unwrap(),
                    created_at: row.get(7).unwrap(),
                    updated_at: row.get(8).unwrap(),
                })
            })
            .unwrap();

        let mut tweets: Vec<ScheduledTweet> = vec![];

        for row in rows {
            tweets.push(row.unwrap());
        }

        tweets
    }

    pub(crate) fn failed(&self) -> Vec<ScheduledTweet> {
        todo!()
    }

    pub(crate) fn sent(&self) -> Vec<ScheduledTweet> {
        todo!()
    }
}

#[cfg(test)]
mod test {

    use crate::schedule::Schedule;

    #[test]
    fn schedule_save_tweet() {
        let body = "This is a scheduled Tweet";
        let time = "Tomorrow";
        let scheduled_tweet = Schedule::new(body, time);
        scheduled_tweet.save();
    }

    #[test]
    fn schedule_get_all() {
        let schedule_instance = Schedule::default();
        let all = schedule_instance.all();
        println!("{:?}", all)
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
