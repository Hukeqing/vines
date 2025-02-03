use crate::common::{Error, Res};
use crate::core::manager::tag::TagManager;
use crate::core::manager::{Config, ItemManager, MarkedTag, Tag};
use crate::core::service::{check_permission, get_user_id, UserRole};
use std::sync::Arc;

pub struct TagService {
    item: Arc<ItemManager>,
    tag: Arc<TagManager>,
}

impl TagService {
    pub fn new(config: &Config) -> Self {
        Self {
            item: config.item_manager.clone(),
            tag: config.tag_manager.clone(),
        }
    }

    pub fn list(&self, repo_id: i64) -> Res<Vec<Tag>> {
        self.tag.select_all(repo_id)
    }

    pub fn list_item(&self, item_id: i64) -> Res<Vec<MarkedTag>> {
        self.tag.select_item_tag(item_id)
    }

    pub fn create(&self, name: String, repo_id: i64, parent: i64) -> Res<Tag> {
        check_permission(repo_id, UserRole::User)?;
        self.tag.create(name, get_user_id()?, repo_id, parent)
    }

    pub fn delete(&self, id: i64) -> Res<()> {
        let tag = self.tag.select_by_id(id)?;
        check_permission(tag.repo_id, UserRole::Manager)?;
        self.tag.delete(tag.id)?;
        Ok(())
    }

    pub fn change_parent(&self, id: i64, parent: i64) -> Res<()> {
        let mut tag = self.tag.select_by_id(id)?;
        let parent_tag = self.tag.select_by_id(parent)?;
        if tag.repo_id != parent_tag.repo_id {
            return Err(Error::TagNotFound)
        }

        let user_id = get_user_id()?;
        if user_id == tag.creator {
            check_permission(tag.repo_id, UserRole::User)?;
        } else {
            check_permission(tag.repo_id, UserRole::Manager)?;
        }
        tag.parent = parent;
        self.tag.update(tag)?;
        Ok(())
    }

    pub fn change_repo(&self, id: i64, repo_id: i64) -> Res<()> {
        let mut tag = self.tag.select_by_id(id)?;
        let user_id = get_user_id()?;
        if user_id == tag.creator {
            check_permission(tag.repo_id, UserRole::User)?;
            check_permission(repo_id, UserRole::User)?;
        } else {
            check_permission(tag.repo_id, UserRole::Manager)?;
            check_permission(repo_id, UserRole::User)?;
        }
        tag.repo_id = repo_id;
        self.tag.remove_all_tag(id)?;
        self.tag.update(tag)?;
        Ok(())
    }

    pub fn apply_tag(&self, item_id: i64, tag_id: i64) -> Res<()> {
        let tag = self.tag.select_by_id(tag_id)?;
        let item = self.item.select_by_id(item_id)?;
        let repo_id = item.repo_id;
        if repo_id != tag.repo_id {
            Err(Error::TagNotFound)
        } else {
            check_permission(repo_id, UserRole::User)?;
            let user_id = get_user_id()?;
            self.tag.apply_tag(tag_id, item_id, user_id)
        }
    }

    pub fn remove_tag(&self, item_id: i64, tag_id: i64) -> Res<()> {
        let item = self.item.select_by_id(item_id)?;
        let marked_tag = self.tag.select_marked_tag(tag_id, item_id)?;
        let user_id = get_user_id()?;
        if marked_tag.creator == user_id {
            check_permission(item.repo_id, UserRole::User)?;
            self.tag.remove_tag(tag_id, item_id)
        } else {
            check_permission(item.repo_id, UserRole::Manager)?;
            self.tag.remove_tag(tag_id, item_id)
        }
    }
}
