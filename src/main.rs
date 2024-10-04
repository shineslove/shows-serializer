use color_eyre::eyre::Result;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::{env, fs};
//TODO: move db create to migration

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

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let data = fs::read_to_string("anime-offline-database.json")?;
    let output: AnimeDatabase = serde_json::from_str(data.as_str())?;
    let db_url = env::var("DATABASE_URL")?;
    dbg!(&db_url);
    let pool = SqlitePool::connect(&db_url).await?;
    //title, kind, episodes, status, season, picture, thumbnail
    let create_table = sqlx::query!(
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
    "#
    )
    .execute(&pool)
    .await?;

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
        sqlx::query!(
            r#"
            INSERT INTO anime (title, kind, episodes, status, season,year, picture, thumbnail)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
        "#,
            item.title,
            item.kind,
            item.episodes,
            item.status,
            item.season,
            item.year,
            item.picture,
            item.thumbnail
        )
        .execute(&pool)
        .await?;
    }
    dbg!(create_table);
    // println!("{:?}", output);
    Ok(())
}
