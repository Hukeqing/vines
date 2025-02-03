use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use crate::common::{Error, Res};
use crate::core::manager::Setting;
use crate::core::repository::{item_tag_relation, tag, ItemTagRelation, TagStorage};

pub struct TagManager;

impl TagManager {
    pub fn new(_: &Setting) -> Self { Self {} }

    pub fn create(&self, name: String, creator: i64, repo_id: i64, parent: i64) -> Res<Tag> {
        let mut tmp = TagStorage { id: 0, name, repo_id, parent, creator, is_delete: false };
        tmp.id = tag::create_tag(&tmp)?;
        Tag::new(tmp)
    }

    pub fn select_by_id(&self, id: i64) -> Res<Tag> {
        let tag = tag::select_by_id(id)?;
        Tag::new(tag)
    }

    pub fn select_all(&self, repo_id: i64) -> Res<Vec<Tag>> {
        tag::select_all(repo_id)?.into_iter().map(|t| Tag::new(t)).collect()
    }

    pub fn update(&self, tag: Tag) -> Res<Tag> {
        let tmp = tag.cast()?;
        tag::update_tag(&tmp)?;
        Tag::new(tmp)
    }

    pub fn delete(&self, tag_id: i64) -> Res<usize> {
        tag::delete_tag(tag_id)?;
        item_tag_relation::delete_tag(tag_id)
    }

    pub fn delete_all(&self, item_id: i64) -> Res<usize> {
        item_tag_relation::delete_item(item_id)
    }

    pub fn reset(&self, id: i64) -> Res<usize> {
        tag::rest_tag(id)?;
        item_tag_relation::revert_all(id)
    }

    pub fn apply_tag(&self, tag_id: i64, item_id: i64, creator: i64) -> Res<()> {
        let relation = ItemTagRelation { id: 0, tag_id, item_id, creator, is_delete: false };
        item_tag_relation::create(&relation)?;
        Ok(())
    }

    pub fn remove_tag(&self, tag_id: i64, item_id: i64) -> Res<()> {
        let relation = item_tag_relation::select_by_both(tag_id, item_id)?;
        item_tag_relation::delete(relation.id)
    }

    pub fn select_item_tag(&self, item_id: i64) -> Res<Vec<MarkedTag>> {
        let relation_list = item_tag_relation::select_by_item(item_id)?;
        let tag_id_list = relation_list.iter().map(|relation| relation.tag_id).collect();
        let tag_list: Vec<Tag> = tag::select_by_ids(&tag_id_list)?.into_iter().map(|tag| Tag::new(tag)).collect::<Res<Vec<_>>>()?;
        let mut tag_map: HashMap<i64, Tag> = tag_list.into_iter().map(|tag| (tag.id, tag)).collect();
        relation_list.into_iter().map(|relation| {
            let tag = tag_map.remove(&relation.tag_id).ok_or(Error::TagNotFound)?;
            Ok(MarkedTag::new(relation, tag))
        }).collect()
    }

    pub fn select_items_tag(&self, items: &Vec<i64>) -> Res<HashMap<i64, HashSet<i64>>> {
        let relation_list = item_tag_relation::select_by_items(items)?;
        let mut result = HashMap::new();
        for relation in relation_list {
            let option = result.get_mut(&relation.item_id);
            match option {
                None => {
                    let mut set = HashSet::new();
                    set.insert(relation.tag_id);
                    result.insert(relation.item_id, set); 
                },
                Some(v) => { v.insert(relation.tag_id); }
            }
        }
        Ok(result)
    }

    pub fn select_marked_tag(&self, tag_id: i64, item_id: i64) -> Res<MarkedTag> {
        let relation = item_tag_relation::select_by_both(tag_id, item_id)?;
        let tag = tag::select_by_id(relation.tag_id)?;
        Ok(MarkedTag::new(relation, Tag::new(tag)?))
    }

    pub fn remove_all_tag(&self, tag_id: i64) -> Res<usize> {
        item_tag_relation::delete_tag(tag_id)
    }
    
    pub fn select_item_by_tags(&self, tags: &Vec<i64>) -> Res<Vec<i64>> {
        let relations = item_tag_relation::select_by_tags(tags)?;
        Ok(relations.into_iter().map(|relation| relation.item_id).collect())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub repo_id: i64,
    pub parent: i64,
    pub creator: i64,
    pub is_delete: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MarkedTag {
    tag: Tag,
    pub marker: i64,
    pub is_remove: bool,
}

impl Tag {
    fn new(tag: TagStorage) -> Res<Self> {
        Ok(Self {
            id: tag.id,
            name: tag.name,
            repo_id: tag.repo_id,
            parent: tag.parent,
            creator: tag.creator,
            is_delete: tag.is_delete,
        })
    }

    fn cast(self) -> Res<TagStorage> {
        Ok(TagStorage {
            id: self.id,
            name: self.name,
            repo_id: self.repo_id,
            parent: self.parent,
            creator: self.creator,
            is_delete: self.is_delete,
        })
    }
}

impl MarkedTag {
    fn new(relation: ItemTagRelation, tag: Tag) -> Self {
        MarkedTag {
            tag,
            marker: relation.creator,
            is_remove: relation.is_delete,
        }
    }
}

impl Deref for MarkedTag {
    type Target = Tag;

    fn deref(&self) -> &Self::Target {
        &self.tag
    }
}