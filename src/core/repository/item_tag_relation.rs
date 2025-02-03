use crate::common::{Error, Res};
use crate::core::repository::holder::{cast_list, cast_placeholder, exec, insert, map_count, query_all, query_one, update_check, RowData};
use rusqlite::params;

pub struct ItemTagRelation {
    pub id: i64,
    pub tag_id: i64,
    pub item_id: i64,
    pub creator: i64,
    pub is_delete: bool,
}

pub fn create(relation: &ItemTagRelation) -> Res<i64> {
    insert("INSERT INTO item_tag_relation (tag_id, item_id, creator, is_delete) VALUES (?, ?, ?, false)", params![relation.tag_id, relation.item_id, relation.creator])
}

pub fn delete(id: i64) -> Res<()> {
    update_check(exec("UPDATE item_tag_relation SET is_delete = true WHERE id = ?", params![id]), Error::TagRelationNotFound)
}

pub fn delete_tag(tag_id: i64) -> Res<usize> {
    exec("UPDATE item_tag_relation SET is_delete = true WHERE tag_id = ?", params![tag_id])
}

pub fn delete_item(item_id: i64) -> Res<usize> {
    exec("UPDATE item_tag_relation SET is_delete = true WHERE item_id = ?", params![item_id])
}

pub fn revert(id: i64) -> Res<()> {
    update_check(exec("UPDATE item_tag_relation SET is_delete = false WHERE id = ?", params![id]), Error::TagRelationNotFound)
}

pub fn revert_all(tag_id: i64) -> Res<usize> {
    exec("UPDATE item_tag_relation SET is_delete = false WHERE tag_id = ?", params![tag_id])
}

pub fn select_by_tag(tag_id: i64, skip: i64, limit: i64) -> Res<Vec<ItemTagRelation>> {
    query_all("SELECT * FROM item_tag_relation WHERE tag_id = ? AND is_delete = false ORDER BY id DESC SKIP ? LIMIT ?", params![tag_id, skip, limit], map)
}

pub fn select_by_tags(tags: &Vec<i64>) -> Res<Vec<ItemTagRelation>> {
    query_all(&format!("SELECT * FROM item_tag_relation WHERE tags IN ({}) = ? AND is_delete = false", cast_placeholder(tags)), cast_list(tags).as_slice(), map)
}

pub fn select_by_item(item_id: i64) -> Res<Vec<ItemTagRelation>> {
    query_all("SELECT * FROM item_tag_relation WHERE item_id = ? ORDER BY id DESC", params![item_id], map)
}

pub fn select_by_items(items: &Vec<i64>) -> Res<Vec<ItemTagRelation>> {
    query_all(&format!("SELECT * FROM item_tag_relation WHERE item_id IN {}", cast_placeholder(items)), cast_list(items).as_slice(), map)
}

pub fn select_by_both(item_id: i64, tag_id: i64) -> Res<ItemTagRelation> {
    query_one("SELECT * FROM item_tag_relation WHERE item_id = ? AND tag_id = ?", params![item_id, tag_id], map)
}

pub fn count_by_tag(tag_id: i64) -> Res<usize> {
    query_one("SELECT COUNT(*), tag_id FROM item_tag_relation WHERE tag_id = ? AND is_delete = false GROUP BY tag_id", params![tag_id], map_count)
}

fn map(row: &RowData<'_>) -> Res<ItemTagRelation> {
    Ok(ItemTagRelation {
        id: row.get(0)?,
        tag_id: row.get(1)?,
        item_id: row.get(2)?,
        creator: row.get(3)?,
        is_delete: row.get(4)?,
    })
}
