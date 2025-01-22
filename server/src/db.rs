use anyhow::Result;
use sqlx::{query_as, sqlite::SqliteQueryResult, Pool, Sqlite};
use std::sync::Arc;

#[derive(Debug)]
pub struct Auth {
    pub id: i64,
    pub username: Arc<str>,
    pub password: Arc<str>,
}

#[derive(Debug)]
pub struct Player {
    pub id: i64,
    pub auth_id: i64,
    pub nickname: Arc<str>,
    pub color: i64,
    pub best_score: i64,
}

#[derive(Debug, Clone)]
pub struct Db {
    pub db_pool: Pool<Sqlite>,
}

impl Db {
    pub async fn new(database_url: &str) -> Result<Self> {
        let db_pool = sqlx::sqlite::SqlitePool::connect(database_url).await?;
        Ok(Self { db_pool })
    }

    pub async fn auth_get_one_by_username(&self, username: &str) -> Result<Auth> {
        query_as!(
            Auth,
            r#"SELECT * FROM auth WHERE username = ? LIMIT 1"#,
            username
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| e.into())
    }

    pub async fn player_get_one_by_auth_id(&self, auth_id: i64) -> Result<Player> {
        query_as!(
            Player,
            r#"SELECT * FROM player WHERE auth_id = ? LIMIT 1"#,
            auth_id
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| e.into())
    }

    pub async fn player_get_list(&self, limit: i64) -> Result<Vec<Player>> {
        query_as!(
            Player,
            r#"SELECT * FROM player ORDER BY best_score DESC LIMIT ?"#,
            limit,
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| e.into())
    }

    pub async fn player_update_best_score_by_id(
        &self,
        best_score: i64,
        id: i64,
    ) -> Result<SqliteQueryResult> {
        query_as!(
            db::Player,
            r#"UPDATE player SET best_score = ? WHERE id = ?"#,
            best_score,
            id,
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| e.into())
    }
}
