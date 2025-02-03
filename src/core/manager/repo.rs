pub use config::{CommonConfig, RepoConfig, RepoFileOrder, IllustrationConfig};

use crate::common::{json, Error, Res};
use crate::core::manager::Setting;
use crate::core::repository::repo;
use crate::core::repository::repo::RepoStorage;

pub struct RepoManager;

impl RepoManager {
    pub fn new(_: &Setting) -> Self {
        Self {}
    }

    pub fn list_repo(&self) -> Res<Vec<Repo>> {
        Ok(repo::select_all_repo()?.into_iter()
            .map(|repo| Repo::new(repo))
            .filter_map(|repo| repo.ok())
            .collect())
    }

    pub fn create_repo(&self, name: String, config: RepoConfig) -> Res<Repo> {
        let repo_vo = Repo { id: 0, name, config };
        let mut repo = repo_vo.cast()?;
        repo.id = repo::create_repo(&repo)?;
        Ok(Repo::new(repo)?)
    }

    pub fn delete_repo(&self, id: i64) -> Res<()> {
        repo::delete_repo(id)
    }

    pub fn get_repo_list(&self, id: &Vec<i64>) -> Res<Vec<Repo>> {
        let repo_vec = repo::select_all_repo()?;
        Ok(repo_vec.into_iter()
            .filter(|repo| id.contains(&repo.id))
            .map(|repo| Repo::new(repo))
            .filter_map(|repo| repo.ok())
            .collect())
    }

    pub fn select_repo_by_name(&self, name: &str) -> Res<Repo> {
        let repo_vec = repo::select_all_repo()?;
        match repo_vec.into_iter().find(|repo| repo.name == name) {
            None => Err(Error::NoSuchRepo),
            Some(repo) => Ok(Repo::new(repo)?),
        }
    }

    pub fn select_repo_by_id(&self, id: i64) -> Res<Repo> {
        let repo_vec = repo::select_all_repo()?;
        match repo_vec.into_iter().find(|repo| repo.id == id) {
            None => Err(Error::NoSuchRepo),
            Some(repo) => Ok(Repo::new(repo)?),
        }
    }

    pub fn update_repo(&self, repo: &Repo) -> Res<()> {
        repo::update_repo(repo.id, &repo.name, &json::stringify(&repo.config)?)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Repo {
    pub id: i64,
    pub name: String,
    pub config: RepoConfig,
}

impl Repo {
    fn new(repo: RepoStorage) -> Res<Self> {
        Ok(Self {
            id: repo.id,
            name: repo.name,
            config: json::parse(&repo.config)?,
        })
    }

    fn cast(self) -> Res<RepoStorage> {
        Ok(RepoStorage {
            id: self.id,
            name: self.name,
            config: json::stringify(&self.config)?,
            is_delete: false,
        })
    }
}

pub mod config {
    use crate::common::{Error, Res};
    use std::ops::Deref;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum RepoConfig {
        Illustration(IllustrationConfig),
        UnSupportConfig,
    }

    #[derive(serde::Serialize, serde::Deserialize, PartialEq)]
    pub enum RepoFileOrder {
        CreateYearTime,
        CreateMonthTime,
        CreateDateTime,
        // Gallery,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct CommonConfig {
        pub order: RepoFileOrder,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct IllustrationConfig {
        pub common_config: CommonConfig,
    }

    impl RepoConfig {
        pub fn update_check(old: &RepoConfig, cur: &RepoConfig) -> Res<()> {
            match (old, cur) {
                (RepoConfig::Illustration(old_conf), RepoConfig::Illustration(new_conf)) => { IllustrationConfig::update_check(old_conf, new_conf) }
                (_, _) => Err(Error::SomeConfigCanNotChange),
            }
        }
    }

    impl CommonConfig {
        fn update_check(old: &CommonConfig, cur: &CommonConfig) -> Res<()> {
            if old.order != cur.order {
                return Err(Error::SomeConfigCanNotChange);
            }

            Ok(())
        }
    }

    impl IllustrationConfig {
        fn update_check(old_conf: &IllustrationConfig, new_conf: &IllustrationConfig) -> Res<()> {
            CommonConfig::update_check(old_conf, new_conf)
        }
    }

    impl Deref for IllustrationConfig {
        type Target = CommonConfig;

        fn deref(&self) -> &Self::Target {
            &self.common_config
        }
    }
}
