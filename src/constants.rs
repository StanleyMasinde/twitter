pub const CACHE_DIR: &str = "twitter-cli";
pub const DB_FILENAME: &str = "db.sqlite3";
pub const SCHEDULE_TABLE_NAME: &str = "scheduled_tweets";
pub const TOKEN_TABLE_NAME: &str = "access_tokens";

// Migrations
pub const CREATE_TOKENS_TABLE: &str = r#"
                CREATE TABLE IF NOT EXISTS access_tokens (
                id INTEGER PRIMARY KEY,
                account_id INTEGER UNIQUE,

                access_token TEXT NOT NULL,
                refresh_token TEXT,
                token_type TEXT NOT NULL DEFAULT 'Bearer',

                expires_at DATETIME,

                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
)"#;

pub const CREATE_SCHEDULES_TABLE: &str = r#"
            CREATE TABLE IF NOT EXISTS scheduled_tweets (
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
"#;
