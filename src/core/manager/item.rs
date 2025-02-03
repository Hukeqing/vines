use chrono::{DateTime, NaiveDateTime, Utc};
pub use extend::{ImageExtend, ItemExtend, PhotoExtend, PictureExtend, RgbColor};

use crate::common::{from, json, ContentType, Error, Res};
use crate::core::manager::Setting;
use crate::core::repository::{item, ItemStorage};

pub struct ItemManager;

impl ItemManager {
    pub fn new(_: &Setting) -> Self {
        Self {}
    }

    pub fn create(&self, item: Item) -> Res<Item> {
        let mut temp = item.cast()?;
        temp.id = item::create(&temp)?;
        let temp = item::select_by_id(temp.id)?;
        Ok(Item::new(temp)?)
    }

    pub fn import(&self, item: Item) -> Res<Item> {
        let mut temp = item.cast()?;
        temp.id = item::import(&temp)?;
        Ok(Item::new(temp)?)
    }

    pub fn delete(&self, id: i64) -> Res<()> {
        item::mark_delete_item(id)
    }

    pub fn update(&self, item: Item) -> Res<Item> {
        let temp: ItemStorage = item.cast()?;
        item::update_item(&temp)?;
        Ok(Item::new(temp)?)
    }

    pub fn change_path(&self, id: i64, new_path: &str) -> Res<()> {
        item::change_path(id, new_path)
    }

    pub fn change_repo(&self, id: i64, new_repo_id: i64) -> Res<()> {
        item::change_repo(id, new_repo_id)
    }

    pub fn select_by_id(&self, id: i64) -> Res<Item> {
        let item = item::select_by_id(id)?;
        Ok(Item::new(item)?)
    }

    pub fn select_by_ids(&self, ids: &Vec<i64>) -> Res<Vec<Item>> {
        let item_list = item::select_item_by_ids(ids)?;
        item_list.into_iter().map(|item| Item::new(item)).collect()
    }

    pub fn select_start_time(&self, repo_id: i64, start_time: i64) -> Res<i64> {
        item::select_start_time(repo_id, start_time)
    }

    pub fn select_end_time(&self, repo_id: i64, end_time: i64) -> Res<i64> {
        item::select_end_time(repo_id, end_time)
    }
    
    pub fn select_start_end_id(&self, repo_id: i64) -> Res<(i64, i64)> {
        item::select_min_max_id(repo_id)
    }

    pub fn select_to(&self, repo_id: i64, start_id: i64, end_id: i64, limit: i64) -> Res<Vec<Item>> {
        item::select_to(repo_id, start_id, end_id, limit)?.into_iter().map(|item| Item::new(item)).collect()
    }

    pub fn select_from(&self, repo_id: i64, start_id: i64, end_id: i64, limit: i64) -> Res<Vec<Item>> {
        item::select_from(repo_id, start_id, end_id, limit)?.into_iter().map(|item| Item::new(item)).collect()
    }
}

// noinspection SpellCheckingInspection
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub ext: &'static ContentType,
    pub size: usize,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub repo_id: i64,
    pub path: String,
    pub extend: ItemExtend,
}

impl Item {
    fn new(item: ItemStorage) -> Res<Self> {
        let created_at = DateTime::from_timestamp(item.created_at, 0).ok_or(Error::TimestampError(item.created_at))?;
        Ok(Self {
            id: item.id,
            name: item.name,
            ext: from(item.ext),
            size: item.size,
            created_at,
            is_deleted: item.is_deleted,
            repo_id: item.repo_id,
            path: item.path,
            extend: json::parse(&item.extend)?,
        })
    }

    fn cast(self) -> Res<ItemStorage> {
        Ok(ItemStorage {
            id: self.id,
            name: self.name,
            ext: self.ext.id,
            size: self.size,
            created_at: self.created_at.timestamp(),
            is_deleted: self.is_deleted,
            repo_id: self.repo_id,
            path: self.path,
            extend: json::stringify(&self.extend)?,
        })
    }
}

pub mod extend {
    use std::ops::Deref;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum ItemExtend {
        // for plain=
        Empty,
        // Image in Illustration
        Picture(PictureExtend),
        // Image in Other
        Photo(PhotoExtend),
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct RgbColor {
        pub r: u8,
        pub g: u8,
        pub b: u8,
        pub a: u8,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct ImageExtend {
        pub w: u32,
        pub h: u32,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct PictureExtend {
        pub image_extend: ImageExtend,
        pub author: Option<i64>,
        pub url: Option<String>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct PhotoExtend {
        pub image_extend: ImageExtend,
        // prepare for EXIF info
    }
    
    impl Deref for PictureExtend {
        type Target = ImageExtend;

        fn deref(&self) -> &Self::Target {
            &self.image_extend
        }
    }
    
    impl Deref for PhotoExtend {
        type Target = ImageExtend;

        fn deref(&self) -> &Self::Target {
            &self.image_extend
        }
    }
}
