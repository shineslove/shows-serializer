```rust
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{Connection, SqliteConnection};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // Define the database URL
    let db_url = "sqlite:///path/to/your/database.db";

    // Create connection options
    let options = SqliteConnectOptions::from_str(db_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .foreign_keys(true);

    // Establish a connection with the specified options
    let conn = SqliteConnection::connect_with(&options).await?;

    // Verify that WAL mode is enabled
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode;")
        .fetch_one(&conn)
        .await?;

    println!("Current journal mode: {}", journal_mode);

    // Close the connection
    conn.close().await?;

    Ok(())
}

```
