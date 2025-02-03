use crate::common::{Error, Res};
use crate::core::repository::holder::{exec, insert, query_one, update_check, RowData};
use rusqlite::params;

pub struct UserStorage {
    pub id: i64,
    pub name: String,
    pub password: String,
    pub token: Option<String>,
}

pub fn create_user(user: &UserStorage) -> Res<i64> {
    insert("INSERT INTO users (name, password, token) VALUES (?, ?, NULL)", params![user.name, user.password])
}

pub fn query_by_id(id: i64) -> Res<UserStorage> {
    query_one("SELECT * FROM users WHERE id = ?", params![id], map).map_err(|_| Error::NoSuchUser)
}

pub fn query_by_name(name: &str) -> Res<UserStorage> {
    query_one("SELECT * FROM users WHERE name = ?", params![name], map).map_err(|_| Error::NoSuchUser)
}

pub fn query_by_token(token: &str) -> Res<UserStorage> {
    query_one("SELECT * FROM users WHERE token = ?", params![token], map).map_err(|_| Error::NoSuchUser)
}

pub fn update_token(id: i64, session: &str) -> Res<()> {
    update_check(exec("UPDATE users SET token = ? WHERE id = ?", params![session, id]), Error::NoSuchUser)
}

pub fn update_password(id: i64, password: &str) -> Res<()> {
    update_check(exec("UPDATE users SET password = ? WHERE id = ?", params![password, id]), Error::NoSuchUser)
}

pub fn clear_token(id: i64) -> Res<()> {
    update_check(exec("UPDATE users SET token = NULL WHERE id = ?", params![id]), Error::NoSuchUser)
}

fn map(row: &RowData<'_>) -> Res<UserStorage> {
    Ok(UserStorage {
        id: row.get(0)?,
        name: row.get(1)?,
        password: row.get(2)?,
        token: row.get(3)?,
    })
}
