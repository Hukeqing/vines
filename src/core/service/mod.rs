pub mod repo;
pub mod user;
pub mod item;
pub mod tag;

use crate::common::{Error, Res};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub use item::ItemService;
pub use repo::RepoService;
pub use tag::TagService;
pub use user::UserService;
pub use crate::core::Config;

pub use crate::core::manager::{UserRole, User};
pub use crate::core::manager::{CommonConfig, Repo, RepoConfig, RepoFileOrder, IllustrationConfig};
pub use crate::core::manager::{Tag, MarkedTag};

// region Service for all service

pub struct Service {
    pub repo: Arc<RepoService>,
    pub user: Arc<UserService>,
    pub item: Arc<ItemService>,
    pub tag: Arc<TagService>,
}

impl Service {
    pub fn new(config: Config) -> Self {
        Self {
            repo: Arc::new(RepoService::new(&config)),
            user: Arc::new(UserService::new(&config)),
            item: Arc::new(ItemService::new(&config)),
            tag: Arc::new(TagService::new(&config)),
        }
    }
}

impl Clone for Service {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            user: self.user.clone(),
            item: self.item.clone(),
            tag: self.tag.clone(),
        }
    }
}

// 

pub(in crate::core::service) struct UserContext {
    pub(in crate::core::service) id: i64,
    pub(in crate::core::service) name: String,
    pub(in crate::core::service) repo: HashMap<i64, UserRole>,
    pub(in crate::core::service) token: String,
}

impl UserContext {
    pub(in crate::core) fn new(user: &User) -> Self {
        Self {
            id: user.id,
            name: user.name.clone(),
            repo: user.repo.clone(),
            token: Uuid::new_v4().to_string(),
        }
    }

    pub(in crate::core) fn as_user_vo(&self) -> User {
        User {
            id: self.id,
            name: self.name.clone(),
            repo: self.repo.clone(),
        }
    }

    pub(in crate::core) fn update(&mut self, user: &User) {
        self.id = user.id;
        self.name = user.name.clone();
        self.repo = user.repo.clone();
    }

    pub(in crate::core) fn permission_check(&self, repo_id: i64, need_role: UserRole) -> Res<()> {
        match self.repo.get(&repo_id) {
            Some(role) => if role.geq(&need_role) {
                Ok(())
            } else {
                Err(Error::PermissionCheckFailed)
            },
            None => {
                Err(Error::PermissionCheckFailed)
            }
        }
    }
}

thread_local! {
    static CONTEXT: RefCell<Option<UserContext>> = RefCell::new(None);
}

fn check_permission(repo_id: i64, need_role: UserRole) -> Res<()>
{
    CONTEXT.with(|ctx| {
        match ctx.borrow().as_ref() {
            None => Err(Error::NeedLogin),
            Some(context) => context.permission_check(repo_id, need_role),
        }
    })
}

fn with_context<F, R>(f: F) -> Res<R>
where
    F: FnOnce(&UserContext) -> Res<R>,
{
    CONTEXT.with(|ctx| {
        match ctx.borrow().as_ref() {
            None => Err(Error::NeedLogin),
            Some(context) => f(context)
        }
    })
}

pub(in crate::core) fn get_user_id() -> Res<i64> {
    CONTEXT.with(|ctx| {
        match ctx.borrow().as_ref() {
            None => Err(Error::NeedLogin),
            Some(context) => Ok(context.id)
        }
    })
}

fn with_option_context<F, R>(f: F) -> R
where
    F: FnOnce(Option<&UserContext>) -> R,
{
    CONTEXT.with(|ctx| {
        f(ctx.borrow().as_ref())
    })
}

fn with_mut_context<F, R>(f: F) -> R
where
    F: FnOnce(&mut Option<UserContext>) -> R,
{
    CONTEXT.with(|ctx| {
        let mut old_ctx = ctx.replace(None);
        let res = f(&mut old_ctx);
        ctx.replace(old_ctx);
        res
    })
}
