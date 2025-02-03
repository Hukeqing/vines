use crate::common::{Error, Res};
use crate::core::repository::holder::{insert, query_all, exec, RowData, update_check, cast_placeholder, cast_list, query_one};
use rusqlite::params;

pub struct TagStorage {
    pub id:          i64,
    pub name:        String,
    pub repo_id:     i64,
    pub parent:      i64,
    pub creator:     i64,
    pub is_delete:   bool,
}

pub fn create_tag(tag: &TagStorage) -> Res<i64> {
    insert("INSERT INTO tags (name, repo_id, parent, creator, is_delete) VALUES (?, ?, ?, false);", params![&tag.name, tag.repo_id, tag.parent, tag.creator])
}

pub fn select_by_id(id: i64) -> Res<TagStorage> {
    query_one("SELECT * FROM tags WHERE is_delete = false AND id = ?", params![id], map)
}

pub fn select_all(repo_id: i64) -> Res<Vec<TagStorage>> {
    query_all("SELECT * FROM tags WHERE is_delete = false AND repo_id = ?", params![repo_id], map)
}

pub fn select_by_ids(ids: &Vec<i64>) -> Res<Vec<TagStorage>> {
    query_all(&format!("SELECT * FROM tags WHERE id IN ({})", cast_placeholder(ids)), cast_list(ids).as_slice(), map)
}

pub fn update_tag(tag: &TagStorage) -> Res<()> {
    update_check(exec("UPDATE tags SET name = ?, repo_id = ?, parent = ? WHERE id = ?", params![&tag.name, tag.repo_id, tag.parent, tag.id]), Error::TagNotFound)
}

pub fn delete_tag(id: i64) -> Res<()> {
    update_check(exec("UPDATE tags SET is_delete = true WHERE id = ?", params![id]), Error::TagNotFound)
}

pub fn rest_tag(id: i64) -> Res<()> {
    update_check(exec("UPDATE tags SET is_delete = false WHERE id = ?", params![id]), Error::TagNotFound)
}

fn map(row: &RowData<'_>) -> Res<TagStorage> {
    Ok(TagStorage {
        id: row.get(0)?,
        name: row.get(1)?,
        repo_id: row.get(2)?,
        parent: row.get(3)?,
        creator: row.get(4)?,
        is_delete: row.get(5)?,
    })
}
