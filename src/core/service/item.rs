use crate::common::file::FileStream;
use crate::common::{build_thumbnail, build_thumbnail_from_file, file_check, get_size, ContentType, DirNode, Error, FileNode, FileType, Node, Res};
use crate::core::manager::tag::TagManager;
use crate::core::manager::ItemExtend::{Empty, Photo, Picture};
use crate::core::manager::{CommonConfig, Config, ImageExtend, Item, ItemExtend, ItemManager, PhotoExtend, PictureExtend, Repo, RepoConfig, RepoFileOrder, RepoManager, ResourceManager, UserRole};
use crate::core::service::check_permission;
use crate::core::service::item::condition::ItemCondition;
use crate::core::service::item::filter::{ConditionContext, ItemFilter};
use chrono::Datelike;
use std::cmp::min;
use std::sync::Arc;

pub struct ItemService {

    max_thumbnail_size: usize,

    repo: Arc<RepoManager>,
    resource: Arc<ResourceManager>,
    item: Arc<ItemManager>,
    tag: Arc<TagManager>,
}

impl ItemService {
    pub fn new(config: &Config) -> Self {
        Self {
            max_thumbnail_size: config.setting.max_thumbnail_size,
            repo: config.repo_manager.clone(),
            resource: config.resource_manager.clone(),
            item: config.item_manager.clone(),
            tag: config.tag_manager.clone(),
        }
    }

    pub fn create(&self, repo_id: i64, name: String, data: &Vec<u8>) -> Res<Item> {
        check_permission(repo_id, UserRole::Manager)?;
        let file_type = file_check(data)?;
        let repo = self.repo.select_repo_by_id(repo_id)?;
        let item = Item {
            id: 0,
            name,
            ext: file_type,
            size: data.len(),
            created_at: Default::default(),
            is_deleted: false,
            repo_id,
            path: "".to_string(),
            extend: Self::build_extend(&repo, data, file_type)?,
        };

        let mut item = self.item.create(item)?;
        let file = Self::build_repo_path(&repo, &item)?;
        let path = file.absolute_path();
        self.item.change_path(item.id, &path)?;
        let resource = self.resource.get_or_init(&repo.name)?;
        resource.write_file(data, file)?;
        item.path = path;
        Ok(item)
    }

    pub fn update_extend(&self, id: i64, extend: ItemExtend) -> Res<Item> {
        let mut item = self.item.select_by_id(id)?;
        check_permission(item.repo_id, UserRole::Manager)?;
        item.extend = extend;
        let item = self.item.update(item)?;
        Ok(item)
    }

    pub fn update_name(&self, id: i64, name: String) -> Res<Item> {
        let mut item = self.item.select_by_id(id)?;
        check_permission(item.repo_id, UserRole::Manager)?;
        item.name = name;
        let item = self.item.update(item)?;
        Ok(item)
    }

    pub fn select_by_id(&self, id: i64) -> Res<Item> {
        let item = self.item.select_by_id(id)?;
        check_permission(item.repo_id, UserRole::Viewer)?;
        Ok(item)
    }

    pub fn select_list(&self, repo_id: i64, limit: i64, from_big: bool,
                       condition: &Option<Vec<Box<dyn ItemCondition>>>,
                       filter: &Option<Vec<Box<dyn ItemFilter>>>) -> Res<Vec<Item>> {
        check_permission(repo_id, UserRole::Viewer)?;
        let limit = min(limit, 100);

        let mut tun = condition::ItemTun::new(from_big, repo_id, self.item.clone(), self.tag.clone());
        tun.init(condition)?;
        loop {
            let mut vec = tun.pull(limit as usize)?;
            if vec.is_empty() {
                return Ok(vec);
            }

            if let Some(cs) = &filter {
                let context = self.build_condition_context(&vec)?;
                cs.iter().for_each(|c| vec.retain(|item| c.check(item, &context)));
            }

            if !vec.is_empty() {
                return Ok(vec);
            }
        }
    }

    pub fn change_repo(&self, id: i64, repo_id: i64) -> Res<()> {
        let mut item = self.item.select_by_id(id)?;
        check_permission(item.repo_id, UserRole::Manager)?;
        check_permission(repo_id, UserRole::Manager)?;
        item.repo_id = repo_id;
        self.item.change_repo(id, repo_id)?;
        Ok(())
    }

    pub fn read_item(&self, id: i64) -> Res<FileStream> {
        let item = self.item.select_by_id(id)?;
        check_permission(item.repo_id, UserRole::Viewer)?;
        let repo = self.repo.select_repo_by_id(item.repo_id)?;
        let resource = self.resource.get_or_init(&repo.name)?;
        resource.build_file(&item.path, item.ext).as_stream()
    }

    pub fn read_thumbnail(&self, id: i64) -> Res<FileStream> {
        let item = self.item.select_by_id(id)?;
        check_permission(item.repo_id, UserRole::Viewer)?;
        let repo = self.repo.select_repo_by_id(item.repo_id)?;
        let resource = self.resource.get_or_init(&repo.name)?;
        if item.size <= self.max_thumbnail_size {
            resource.build_file(&item.path, item.ext).as_stream()
        } else {
            let mut thumbnail_file_node = resource.build_thumbnail_file(&item.path);
            if thumbnail_file_node.is_exist() {
                thumbnail_file_node.as_stream()
            } else {
                let mut origin_file_node = resource.build_file(&item.path, item.ext);
                match item.ext.file {
                    FileType::Unknown => Err(Error::UnknownFileContentType),
                    FileType::Plain => Err(Error::UnknownFileContentType),
                    FileType::Image => {
                        build_thumbnail_from_file(&mut origin_file_node, &mut thumbnail_file_node)?;
                        thumbnail_file_node.as_stream()
                    },
                }
            }
        }
    }

    fn build_repo_path(repo: &Repo, item: &Item) -> Res<FileNode> {
        let callback = |x: &CommonConfig| {
            let naive_created_at = item.created_at.naive_utc();
            match x.order {
                RepoFileOrder::CreateYearTime => {
                    Ok(DirNode::new(naive_created_at.year().to_string())
                        .to(item.id.to_string(), item.ext))
                }
                RepoFileOrder::CreateMonthTime => {
                    Ok(DirNode::new(naive_created_at.year().to_string())
                        .next(naive_created_at.month().to_string())
                        .to(item.id.to_string(), item.ext))
                }
                RepoFileOrder::CreateDateTime => {
                    Ok(DirNode::new(naive_created_at.year().to_string())
                        .next(naive_created_at.month().to_string())
                        .next(naive_created_at.day().to_string())
                        .to(item.id.to_string(), item.ext))
                }
            }
        };
        match &repo.config {
            RepoConfig::Illustration(config) => callback(config),
            RepoConfig::UnSupportConfig => unreachable!(),
        }
    }

    fn build_extend(repo: &Repo, data: &Vec<u8>, file_type: &ContentType) -> Res<ItemExtend> {
        match file_type.file {
            FileType::Unknown => Err(Error::UnknownFileContentType),
            FileType::Plain => Ok(Empty),
            FileType::Image => {
                let (w, h) = get_size(data)?;
                match repo.config {
                    RepoConfig::Illustration(_) => {
                        Ok(Picture(PictureExtend { image_extend: ImageExtend { w, h }, author: None, url: None }))
                    }
                    _ => {
                        Ok(Photo(PhotoExtend { image_extend: ImageExtend { w, h } }))
                    }
                }
            }
        }
    }

    fn build_condition_context(&self, _: &Vec<Item>) -> Result<ConditionContext, Error> {
        Ok(ConditionContext {})
    }
}

pub mod filter {
    use crate::core::manager::{Item, ItemExtend};
    use std::cmp::{max, min};

    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum Rectangle {
        Horizontal,
        Vertical,
        NearlySquare,
        Square,
        P1080,
        P1440,
        P2160,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum CompareType {
        Exactly,
        Prefix,
        Suffix,
        Includes,
        Excludes,
    }

    pub struct ConditionContext {}

    pub trait ItemFilter {
        fn check(&self, item: &Item, context: &ConditionContext) -> bool;
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct SizeFilter {
        pub min_w: Option<u32>,
        pub max_w: Option<u32>,
        pub min_h: Option<u32>,
        pub max_h: Option<u32>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct RectangleFilter {
        pub rectangle: Rectangle,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct UrlFilter {
        pub url: String,
        pub compare: CompareType,
    }

    impl ItemFilter for SizeFilter {
        fn check(&self, item: &Item, _: &ConditionContext) -> bool {
            let checker = |w: u32, h: u32| {
                let mut result = true;
                result &= self.min_w.map(|v| w >= v).unwrap_or(true);
                result &= self.max_w.map(|v| w <= v).unwrap_or(true);
                result &= self.min_h.map(|v| h >= v).unwrap_or(true);
                result &= self.max_h.map(|v| h <= v).unwrap_or(true);
                result
            };

            match &item.extend {
                ItemExtend::Empty => false,
                ItemExtend::Picture(extend) => checker(extend.w, extend.h),
                ItemExtend::Photo(extend) => checker(extend.w, extend.h),
            }
        }
    }

    impl ItemFilter for RectangleFilter {
        fn check(&self, item: &Item, _: &ConditionContext) -> bool {
            let size = match &item.extend {
                ItemExtend::Picture(extend) => Some((extend.w, extend.h)),
                ItemExtend::Photo(extend) => Some((extend.w, extend.h)),
                _ => None
            };
            match size {
                None => false,
                Some((w, h)) =>
                    match self.rectangle {
                        Rectangle::Horizontal => w > h,
                        Rectangle::Vertical => w < h,
                        Rectangle::NearlySquare => (min(w, h) as f64 / max(w, h) as f64) > 0.9,
                        Rectangle::Square => w == h,
                        Rectangle::P1080 => (w >= 1920 && h >= 1080) || (w >= 1080 && h >= 1920),
                        Rectangle::P1440 => (w >= 2560 && h >= 1440) || (w >= 1440 && h >= 2560),
                        Rectangle::P2160 => (w >= 4096 && h >= 2160) || (w >= 2160 && h >= 4096)
                    }
            }
        }
    }

    impl ItemFilter for UrlFilter {
        fn check(&self, item: &Item, _: &ConditionContext) -> bool {
            let url_option = match &item.extend {
                ItemExtend::Empty => &None,
                ItemExtend::Picture(extend) => &extend.url,
                ItemExtend::Photo(_) => &None
            };
            match url_option {
                None => false,
                Some(url) =>
                    match self.compare {
                        CompareType::Exactly => &self.url == url,
                        CompareType::Prefix => url.starts_with(&self.url),
                        CompareType::Suffix => url.ends_with(&self.url),
                        CompareType::Includes => url.contains(&self.url),
                        CompareType::Excludes => !url.contains(&self.url)
                    }
            }
        }
    }
}

pub mod condition {
    use crate::common::Res;
    use crate::core::manager::tag::TagManager;
    use crate::core::manager::{Item, ItemManager};
    use chrono::{DateTime, Utc};
    use std::cmp::{max, min};
    use std::collections::HashSet;
    use std::sync::Arc;

    pub struct ItemTun {
        start_id: i64,
        end_id: i64,
        from_big: bool,
        repo_id: i64,
        id_list: Option<Vec<i64>>,

        item: Arc<ItemManager>,
        tag: Arc<TagManager>,
    }

    pub trait ItemCondition {
        fn apply(&self, tun: &mut ItemTun) -> Res<()>;
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct StartIdCondition {
        pub id: i64,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct EndIdCondition {
        pub id: i64,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct StartTimeCondition {
        pub time: DateTime<Utc>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct EndTimeCondition {
        pub time: DateTime<Utc>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct TagCondition {
        pub tags: Vec<i64>,
    }

    impl ItemTun {
        pub fn new(from_big: bool, repo_id: i64, item: Arc<ItemManager>, tag: Arc<TagManager>) -> Self {
            Self { start_id: 0, end_id: 0x7FFFFFFFFFFFFFFF, from_big, repo_id, id_list: None, item, tag }
        }

        pub fn init(&mut self, condition_option: &Option<Vec<Box<dyn ItemCondition>>>) -> Res<()> {
            (self.start_id, self.end_id) = self.item.select_start_end_id(self.repo_id)?;
            self.end_id += 1;
            if let Some(conditions) = condition_option {
                for condition in conditions {
                    condition.apply(self)?;
                }
            }

            if let Some(id_list) = &mut self.id_list {
                if self.from_big {
                    id_list.sort_unstable_by(|lhs, rhs| rhs.cmp(lhs))
                } else {
                    id_list.sort_unstable_by(|lhs, rhs| lhs.cmp(rhs))
                };
            }
            Ok(())
        }

        pub fn pull(&mut self, limit: usize) -> Res<Vec<Item>> {
            let mut items = if let Some(id_list) = &mut self.id_list {
                let vec: Vec<_> = id_list.drain(..limit).collect();
                self.item.select_by_ids(&vec)?
            } else if self.from_big {
                self.item.select_to(self.repo_id, self.start_id, self.end_id, limit as i64)?
            } else {
                self.item.select_from(self.repo_id, self.start_id, self.end_id, limit as i64)?
            };
            
            if items.is_empty() {
                return Ok(items);
            }

            if self.from_big {
                items.sort_unstable_by(|lhs, rhs| rhs.id.cmp(&lhs.id));
                self.end_id = items.last().unwrap().id;
            } else {
                items.sort_unstable_by(|lhs, rhs| lhs.id.cmp(&rhs.id));
                self.start_id = items.last().unwrap().id + 1;
            }

            Ok(items)
        }
    }

    impl ItemCondition for StartIdCondition {
        fn apply(&self, tun: &mut ItemTun) -> Res<()> {
            tun.start_id = max(tun.start_id, self.id);
            Ok(())
        }
    }

    impl ItemCondition for EndIdCondition {
        fn apply(&self, tun: &mut ItemTun) -> Res<()> {
            tun.end_id = min(tun.end_id, self.id);
            Ok(())
        }
    }

    impl ItemCondition for StartTimeCondition {
        fn apply(&self, tun: &mut ItemTun) -> Res<()> {
            let start_id = tun.item.select_start_time(tun.repo_id, self.time.timestamp())?;
            tun.start_id = max(tun.start_id, start_id);
            Ok(())
        }
    }

    impl ItemCondition for EndTimeCondition {
        fn apply(&self, tun: &mut ItemTun) -> Res<()> {
            let end_id = tun.item.select_end_time(tun.repo_id, self.time.timestamp())?;
            tun.end_id = min(tun.end_id, end_id);
            Ok(())
        }
    }

    impl ItemCondition for TagCondition {
        fn apply(&self, tun: &mut ItemTun) -> Res<()> {
            let item_id_list = tun.tag.select_item_by_tags(&self.tags)?;
            match &mut tun.id_list {
                None => tun.id_list = Some(item_id_list),
                Some(id_list) => {
                    let item_id_set = item_id_list.iter().collect::<HashSet<_>>();
                    let mut new_item_id_list = Vec::new();
                    for id in id_list {
                        if item_id_set.contains(id) {
                            new_item_id_list.push(*id);
                        }
                    }
                    tun.id_list = Some(new_item_id_list);
                }
            }
            Ok(())
        }
    }
}
