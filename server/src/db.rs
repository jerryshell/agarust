#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct Auth {
    pub id: i64,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: i64,
    pub auth_id: i64,
    pub nickname: String,
    pub color: i64,
    pub best_score: i64,
}
