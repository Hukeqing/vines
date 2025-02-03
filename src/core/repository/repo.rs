use crate::common::{Error, Res};
use crate::core::repository::holder::{exec, insert, query_all, update_check, RowData};
use rusqlite::params;

pub struct RepoStorage {
    pub id: i64,
    pub name: String,
    pub config: String,
    pub is_delete: bool,
}

pub fn create_repo(repo: &RepoStorage) -> Res<i64> {
    insert("INSERT INTO repo (name, config, is_delete) VALUES (?, ?, false)", params![repo.name, repo.config])
}

pub fn update_repo(id: i64, new_name: &str, config: &str) -> Res<()> {
    update_check(exec("UPDATE repo SET name = ?, config = ? WHERE id = ?", params![new_name, config, id]), Error::NoSuchRepo)
}

pub fn delete_repo(id: i64) -> Res<()> {
    update_check(exec("UPDATE repo SET is_delete = true WHERE id = ?", params![id]), Error::NoSuchRepo)
}

pub fn select_all_repo() -> Res<Vec<RepoStorage>> {
    query_all("SELECT * FROM repo WHERE is_delete = false", params![], map)
}

fn map(row: &RowData<'_>) -> Res<RepoStorage> {
    Ok(RepoStorage {
        id: row.get(0)?,
        name: row.get(1)?,
        config: row.get(2)?,
        is_delete: row.get(3)?,
    })
}
