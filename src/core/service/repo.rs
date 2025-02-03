use crate::common::{Error, Res};
use crate::core::manager::{Config, Repo, RepoConfig, RepoManager, ResourceManager, UserManager, UserRole};
use crate::core::service::{check_permission, with_context};
use std::sync::Arc;

pub struct RepoService {
    repo: Arc<RepoManager>,
    user: Arc<UserManager>,
    resource: Arc<ResourceManager>
}

impl RepoService {
    pub fn new(config: &Config) -> RepoService {
        Self {
            repo: config.repo_manager.clone(),
            user: config.user_manager.clone(),
            resource: config.resource_manager.clone()
        }
    }

    pub fn list(&self) -> Res<Vec<Repo>> {
        with_context(|ctx| {
            let vec = self.user.list_user_repo(ctx.id)?;
            self.repo.get_repo_list(&vec)
        })
    }

    pub fn create(&self, name: String, config: RepoConfig) -> Res<Repo> {
        check_permission(0, UserRole::Manager)?;
        with_context(|ctx| {
            match self.repo.select_repo_by_name(&name) {
                Ok(_) => Err(Error::UsedRepoName),
                Err(_) => {
                    let repo = self.repo.create_repo(name, config)?;
                    self.user.update_user_role(ctx.id, repo.id, UserRole::Admin)?;
                    Ok(repo)
                }
            }
        })
    }

    pub fn update(&self, id: i64, name: String, config: RepoConfig) -> Res<Repo> {
        check_permission(0, UserRole::Manager)?;
        let mut repo = self.repo.select_repo_by_id(id)?;
        RepoConfig::update_check(&repo.config, &config)?;
        match self.repo.select_repo_by_name(&name) {
            Ok(_) => Err(Error::UsedRepoName),
            Err(_) => {
                self.resource.rename_repo(&repo.name, name.clone())?;
                repo.config = config;
                repo.name = name;
                self.repo.update_repo(&repo)?;
                Ok(repo)
            }
        }
    }
}
