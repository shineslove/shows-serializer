use anyhow::Ok;
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePool},
    Connection, SqliteConnection,
};
use std::{fs, rc::Rc, str::FromStr};

#[derive(Debug, Serialize, Deserialize)]
struct AnimeDatabase {
    license: License,
    repository: String,
    #[serde(rename = "lastUpdate")]
    last_update: String,
    data: Vec<AnimeData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct License {
    name: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnimeData {
    sources: Vec<String>,
    title: String,
    #[serde(rename = "type")]
    anime_type: String,
    episodes: i32,
    status: String,
    #[serde(rename = "animeSeason")]
    anime_season: AnimeSeason,
    picture: String,
    thumbnail: String,
    synonyms: Vec<String>,
    #[serde(rename = "relatedAnime")]
    related_anime: Vec<String>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnimeSeason {
    season: String,
    year: Option<i32>,
}

#[derive(Debug)]
struct AnimeRow {
    title: String,
    kind: String,
    episodes: i32,
    status: String,
    season: String,
    year: i32,
    picture: String,
    thumbnail: String,
}

async fn create_db() -> Result<Rc<str>, anyhow::Error> {
    let db_url = "sqlite://./anime.db";
    let options = SqliteConnectOptions::from_str(db_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .foreign_keys(true);
    let mut conn = SqliteConnection::connect_with(&options).await?;
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode;")
        .fetch_one(&mut conn)
        .await?;
    println!("current journal mode: {}", journal_mode);
    conn.close().await?;
    Ok(Rc::from(db_url))
}

async fn table_migration(pool: &SqlitePool) -> Result<(), anyhow::Error> {
    let migration = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS anime (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            kind TEXT NOT NULL,
            episodes INT NOT NULL,
            status TEXT NOT NULL,
            season TEXT NOT NULL,
            year INT NOT NULL,
            picture TEXT NOT NULL,
            thumbnail TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
    "#,
    )
    .execute(pool)
    .await?;
    dbg!(migration);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let data = fs::read_to_string("anime-offline-database.json")?;
    let output: AnimeDatabase = serde_json::from_str(data.as_str())?;
    let db_name = create_db().await?;
    println!("{}", &db_name);
    let pool = SqlitePool::connect(&db_name).await?;
    table_migration(&pool).await?;
    //title, kind, episodes, status, season, picture, thumbnail
    let items: Vec<AnimeRow> = output
        .data
        .into_iter()
        .map(|anime| AnimeRow {
            title: anime.title,
            kind: anime.anime_type,
            episodes: anime.episodes,
            status: anime.status,
            season: anime.anime_season.season,
            year: anime.anime_season.year.unwrap_or_default(),
            picture: anime.picture,
            thumbnail: anime.thumbnail,
        })
        .collect();
    for item in items {
        sqlx::query(
            r#"
            INSERT INTO anime (title, kind, episodes, status, season,year, picture, thumbnail)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
        "#,
        )
        .bind(item.title)
        .bind(item.kind)
        .bind(item.episodes)
        .bind(item.status)
        .bind(item.season)
        .bind(item.year)
        .bind(item.picture)
        .bind(item.thumbnail)
        .execute(&pool)
        .await?;
    }
    Ok(())
}
