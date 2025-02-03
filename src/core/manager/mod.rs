pub(in crate::core) mod user;
pub(in crate::core) mod repo;
pub(in crate::core) mod resource;
pub(in crate::core) mod item;
pub(in crate::core) mod tag;

pub(in crate::core) use item::ItemManager;
pub(in crate::core) use repo::RepoManager;
pub(in crate::core) use resource::ResourceManager;
pub(in crate::core) use user::UserManager;

pub use item::{ImageExtend, Item, ItemExtend, PhotoExtend, PictureExtend};
pub use repo::{CommonConfig, Repo, RepoConfig, RepoFileOrder, IllustrationConfig};
pub use tag::{Tag, MarkedTag};
pub use user::{User, UserRole};

use crate::common::DirNode;
use crate::core::manager::tag::TagManager;
use std::sync::Arc;

pub struct Setting {
    pub root: DirNode,
    pub max_thumbnail_size: usize,
}

pub struct Config {
    pub(in crate::core) setting: Setting,
    pub(in crate::core) repo_manager: Arc<RepoManager>,
    pub(in crate::core) user_manager: Arc<UserManager>,
    pub(in crate::core) resource_manager: Arc<ResourceManager>,
    pub(in crate::core) item_manager: Arc<ItemManager>,
    pub(in crate::core) tag_manager: Arc<TagManager>,
}

impl Config {
    pub fn new(root: DirNode, max_thumbnail_size: usize) -> Self {
        let setting = Setting { root, max_thumbnail_size };
        let repo = Arc::new(RepoManager::new(&setting));
        let user = Arc::new(UserManager::new(&setting));
        let resource = Arc::new(ResourceManager::new(&setting));
        let item = Arc::new(ItemManager::new(&setting));
        let tag = Arc::new(TagManager::new(&setting));

        Config {
            setting,
            repo_manager: repo,
            user_manager: user,
            resource_manager: resource,
            item_manager: item,
            tag_manager: tag,
        }
    }
}
