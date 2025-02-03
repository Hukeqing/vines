use crate::common::file::content_type;
use crate::common::Error::{NickOrPasswordError, NoSuchFile};
use crate::common::{ContentType, DirNode, FileNode, Node, Res};
use crate::core::manager::Setting;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

pub struct ResourceManager {
    root: DirNode,
    repo_cache: RwLock<HashMap<String, Arc<RepoResourceManager>>>,
}

pub struct RepoResourceManager {
    home: DirNode,
    cache: DirNode,
    temp: DirNode,
}

impl ResourceManager {
    pub fn new(setting: &Setting) -> Self {
        Self {
            root: setting.root.clone(),
            repo_cache: RwLock::new(HashMap::new()),
        }
    }

    // region repo

    pub fn get_or_init(&self, name: &str) -> Res<Arc<RepoResourceManager>> {
        if self.repo_cache.read()?.contains_key(name) {
            return Ok(self.repo_cache.read()?[name].clone());
        }

        let node = self.root.next(String::from(name));
        let manager = Arc::new(RepoResourceManager::new(node)?);
        self.repo_cache
            .write()?
            .insert(String::from(name), manager.clone());
        Ok(manager)
    }

    pub fn rename_repo(&self, old_name: &str, new_name: String) -> Res<()> {
        let mut node = self.root.next(String::from(old_name));
        node.rename(new_name)?;
        self.repo_cache.write()?.remove(old_name);
        Ok(())
    }

    // endregion
}

impl RepoResourceManager {
    pub fn new(home: DirNode) -> Res<Self> {
        let cache = home.next(String::from(".cache"));
        let temp = home.next(String::from(".temp"));
        home.mkdir()?;
        cache.mkdir()?;
        temp.mkdir()?;
        Ok(Self { home, cache, temp })
    }

    pub fn create_temp_file(&self, data: &Vec<u8>) -> Res<FileNode> {
        let file_name = Uuid::new_v4().to_string();
        let mut file_node = self.temp.to(file_name, &content_type::UNKNOWN);
        file_node.touch()?;
        file_node.write(data)?;
        Ok(file_node)
    }

    pub fn write_file(&self, data: &Vec<u8>, mut file: FileNode) -> Res<FileNode> {
        file.set_root_parent(&self.home);
        file.touch()?;
        file.write(data)?;
        Ok(file)
    }

    pub fn build_file(&self, path: &str, ext: &'static ContentType) -> FileNode {
        let mut file_node = <FileNode as Node>::from(path);
        file_node.set_root_parent(&self.home);
        file_node
    }

    pub fn build_thumbnail_file(&self, path: &str) -> FileNode {
        let mut file_node = <FileNode as Node>::from(path);
        file_node.set_content_type(&content_type::JPEG);
        file_node.set_root_parent(&self.cache);
        file_node
    }
}
