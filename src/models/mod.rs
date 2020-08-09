use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub created: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub deleted: Option<NaiveDateTime>,
    #[serde(rename = "type")]
    pub _type: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar_url: Option<String>,
    pub last_password_request: Option<NaiveDateTime>,
    pub verified_date: Option<NaiveDateTime>,
    pub banned: Option<bool>,
}

impl From<Row> for User {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            created: row.get("created"),
            updated: row.get("updated"),
            deleted: row.get("deleted"),
            _type: row.get("type"),
            username: row.get("username"),
            email: row.get("email"),
            password: row.get("password"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            avatar_url: row.get("avatar_url"),
            last_password_request: row.get("last_password_request"),
            verified_date: row.get("verified_date"),
            banned: row.get("banned"),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Attempt {
    id: Uuid,
    created: NaiveDateTime,
    updated: Option<NaiveDateTime>,
    deleted: Option<NaiveDateTime>,
    grade: i32,
    send: bool,
    flash: Option<bool>,
    date: Option<NaiveDateTime>,
    user_id: Uuid,
}

impl From<Row> for Attempt {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            created: row.get("created"),
            updated: row.get("updated"),
            deleted: row.get("deleted"),
            grade: row.get("grade"),
            send: row.get("send"),
            flash: row.get("flash"),
            date: row.get("date"),
            user_id: row.get("user_id"),
        }
    }
}
