use crate::common::{Error, Res};
use crate::core::manager::user::{User, UserRole};
use crate::core::manager::{Config, UserManager};
use crate::core::service::{check_permission, with_context, with_mut_context, with_option_context, UserContext};
use std::sync::Arc;

pub struct UserService {
    user: Arc<UserManager>,
}

impl UserService {
    pub fn new(config: &Config) -> Self {
        Self {
            user: config.user_manager.clone(),
        }
    }

    pub fn get_session_key(&self) -> String {
        with_option_context(|ctx| {
            match ctx {
                None => String::new(),
                Some(v) => v.token.clone(),
            }
        })
    }

    pub fn check(&self) -> Res<User> {
        with_option_context(|ctx| {
            match ctx {
                None => Err(Error::NeedLogin),
                Some(v) => Ok(v.as_user_vo())
            }
        })
    }

    pub fn refresh(&self, token: &str) -> Res<User> {
        with_mut_context(|ctx| {
            let user = self.user.check_token(token)?;
            *ctx = Some(UserContext::new(&user));
            Ok(user)
        })
    }

    pub fn clear(&self) {
        with_mut_context(|ctx| {
            *ctx = None;
        });
    }

    pub fn login(&self, nick: &str, password: &str) -> Res<User> {
        with_mut_context(|ctx| {
            let user = self.user.check_password(nick, password)?;
            match ctx {
                None => {
                    let context = UserContext::new(&user);
                    self.user.update_token(user.id, &context.token)?;
                    *ctx = Some(context);
                },
                Some(context) => {
                    self.user.clear_token(context.id)?;
                    self.user.update_token(user.id, &context.token)?;
                    context.update(&user);
                }
            }
            Ok(user)
        })
    }

    pub fn logout(&self) -> Res<()> {
        with_mut_context(|ctx| {
            match ctx {
                None => Ok(()),
                Some(context) => {
                    self.user.clear_token(context.id)?;
                    *ctx = None;
                    Ok(())
                }
            }
        })
    }

    pub fn create(&self, name: String, password: &str) -> Res<User> {
        check_permission(0, UserRole::Manager)?;
        self.user.create_user(name, password)
    }

    pub fn change_password(&self, old_password: &str, new_password: &str) -> Res<()> {
        with_context(|ctx| {
            self.user.check_password_by_id(ctx.id, old_password)?;
            self.user.update_password(ctx.id, new_password)?;
            Ok(())
        })
    }

    pub fn change_user_role(&self, user_id: i64, repo_id: i64, role: UserRole) -> Res<()> {
        if role == UserRole::Admin {
            return Err(Error::PermissionCheckFailed)
        }

        let pass1 = check_permission(repo_id, UserRole::Manager).is_err();
        let pass2 = check_permission(0, UserRole::Manager).is_err();
        if pass1 && pass2 {
            return Err(Error::PermissionCheckFailed)
        }

        with_context(|ctx| {
            if ctx.id == user_id {
                Err(Error::PermissionCheckFailed)
            } else { 
                self.user.update_user_role(user_id, repo_id, role)
            }
        })
    }
}
