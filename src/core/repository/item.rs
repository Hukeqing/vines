use crate::common::{Error, Res};
use crate::core::repository::holder::{cast_list, cast_placeholder, exec, insert, map_id, query_all, query_one, update_check, RowData};
use rusqlite::params;

pub struct ItemStorage {
    pub id: i64,
    pub name: String,
    pub ext: i64,
    pub size: usize,
    pub created_at: i64,
    pub is_deleted: bool,
    pub repo_id: i64,
    pub path: String,
    pub extend: String,
}

pub fn create(item: &ItemStorage) -> Res<i64> {
    insert("INSERT INTO items (name, ext, size, created_at, is_deleted, repo_id, path, extend) VALUES (?, ?, ?, DATETIME('NOW'), false, ?, ?, ?)",
           params![item.name, item.ext, item.size, item.repo_id, item.path, item.extend])
}

pub fn import(item: &ItemStorage) -> Res<i64> {
    insert("INSERT INTO items (name, ext, size, created_at, is_deleted, repo_id, path, extend) VALUES (?, ?, ?, ?, false, ?, ?, ?)",
           params![item.name, item.ext, item.size, item.created_at, item.repo_id, item.path, item.extend])
}

pub fn select_by_id(id: i64) -> Res<ItemStorage> {
    query_one("SELECT * FROM items WHERE id = ?", params![id], map)
}

pub fn select_start_time(repo_id: i64, start_time: i64) -> Res<i64> {
    query_one("SELECT MIN(id) FROM items WHERE repo_id = ? AND created_at >= ? AND is_deleted = false", params![repo_id, start_time], map_id)
}

pub fn select_end_time(repo_id: i64, end_time: i64) -> Res<i64> {
    query_one("SELECT MAX(id) FROM items WHERE repo_id = ? AND created_at < ? AND is_deleted = false", params![repo_id, end_time], map_id)
}

pub fn select_min_max_id(repo_id: i64) -> Res<(i64, i64)> {
    query_one("SELECT MIN(id), MAX(id) FROM items WHERE repo_id = ? AND is_deleted = false", params![repo_id], |row| {
        Ok((row.get::<i64>(0)?, row.get::<i64>(1)?))
    })
}

pub fn select_item_by_ids(ids: &Vec<i64>) -> Res<Vec<ItemStorage>> {
    query_all(&format!("SELECT * FROM items WHERE id IN ({})", cast_placeholder(ids)), cast_list(ids).as_slice(), map)
}

pub fn select_from(repo_id: i64, start_id: i64, end_id: i64, limit: i64) -> Res<Vec<ItemStorage>> {
    query_all("SELECT * FROM items WHERE repo_id = ? AND id >= ? AND id < ? AND is_deleted = false ORDER BY id ASC LIMIT ?", params![repo_id, start_id, end_id, limit], map)
}

pub fn select_to(repo_id: i64, start_id: i64, end_id: i64, limit: i64) -> Res<Vec<ItemStorage>> {
    query_all("SELECT * FROM items WHERE repo_id = ? AND id >= ? AND id < ? AND is_deleted = false ORDER BY id DESC LIMIT ?", params![repo_id, start_id, end_id, limit], map)
}

pub fn mark_delete_item(id: i64) -> Res<()> {
    update_check(exec("UPDATE items SET is_deleted = true WHERE id = ?", params![id]), Error::ItemNotFound)
}

pub fn update_item(item: &ItemStorage) -> Res<()> {
    update_check(exec("UPDATE items SET name = ?, ext = ?, size = ?, extend = ? WHERE id = ?", params![item.name, item.ext,  item.size, item.extend, item.id]), Error::ItemNotFound)
}

pub fn change_path(id: i64, path: &str) -> Res<()> {
    update_check(exec("UPDATE items SET path = ? WHERE id = ?", params![path, id]), Error::ItemNotFound)
}

pub fn change_repo(id: i64, repo_id: i64) -> Res<()> {
    update_check(exec("UPDATE items SET repo_id = ? WHERE id = ?", params![repo_id, id]), Error::ItemNotFound)
}

pub fn reset_item(id: i64) -> Res<()> {
    update_check(exec("UPDATE items SET is_deleted = false WHERE id = ?", params![id]), Error::ItemNotFound)
}

fn map(row: &RowData<'_>) -> Res<ItemStorage> {
    Ok(ItemStorage {
        id: row.get(0)?,
        name: row.get(1)?,
        ext: row.get(2)?,
        size: row.get(3)?,
        created_at: row.get_timestamp(4)?,
        is_deleted: row.get(5)?,
        repo_id: row.get(6)?,
        path: row.get(7)?,
        extend: row.get(8)?,
    })
}
