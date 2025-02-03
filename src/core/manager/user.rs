use crate::common::{Error, Res};
use crate::core::manager::Setting;
use crate::core::repository::{user, user_repo_role, UserStorage, UserRepoRoleStorage};
use rand::Rng;
use std::collections::HashMap;
use std::ops::Add;

pub struct UserManager;

impl UserManager {

    pub fn new(_: &Setting) -> Self {
        Self {}
    }
    
    pub fn create_user(&self, name: String, password: &str) -> Res<User> {
        if let Ok(_) = user::query_by_name(&name) {
            return Err(Error::UsedNick);
        }
        let mut user = UserStorage { id: 0, name, password: Self::encode_password(password), token: None };
        user.id = user::create_user(&user)?;
        Ok(User::new(user, &Vec::new()))
    }

    pub fn check_password(&self, name: &str, password: &str) -> Res<User> {
        let user = user::query_by_name(name)?;
        Self::password_check(password, &user.password)?;
        let role_vec = user_repo_role::select_user_role(user.id)?;
        Ok(User::new(user, &role_vec))
    }
    
    pub fn check_password_by_id(&self, id: i64, password: &str) -> Res<()> {
        let user = user::query_by_id(id)?;
        Self::password_check(password, &user.password)?;
        Ok(())
    }

    pub fn check_token(&self, token: &str) -> Res<User> {
        let user = user::query_by_token(token)?;
        let role_vec = user_repo_role::select_user_role(user.id)?;
        Ok(User::new(user, &role_vec))
    }

    pub fn clear_token(&self, id: i64) -> Res<()> {
        user::clear_token(id)
    }

    pub fn update_token(&self, id: i64, token: &str) -> Res<()> {
        user::update_token(id, token)
    }

    pub fn update_password(&self, id: i64, password: &str) -> Res<()> {
        user::update_password(id, &Self::encode_password(password))
    }

    pub fn update_user_role(&self, user_id: i64, repo_id: i64, role: UserRole) -> Res<()> {
        let role_vec = user_repo_role::select_user_role(user_id)?;
        let match_role: Vec<UserRepoRoleStorage> = role_vec.into_iter().filter(|role| role.repo_id == repo_id).collect();
        if match_role.is_empty() {
            let role = UserRepoRoleStorage { id: 0, user_id, repo_id, role: role.to_int() };
            user_repo_role::create_user_repo_role(&role).map(|_| ())
        } else {
            user_repo_role::update_user_role(match_role.get(0).unwrap().id, role.to_int())
        }
    }

    pub fn list_user_repo(&self, user_id: i64) -> Res<Vec<i64>> {
        user_repo_role::select_user_role(user_id).map(|vec| vec.iter().map(|r| r.id).collect())
    }

    fn password_check(input: &str, db_value: &str) -> Res<()> {
        match db_value.split_once('|') {
            None => Err(Error::NickOrPasswordError),
            Some((_, salt)) => if db_value == Self::encode_password0(input, salt) {
                Ok(())
            } else {
                Err(Error::NickOrPasswordError)
            }
        }
    }

    fn encode_password(input: &str) -> String {
        let salt = Self::random_salt(5);
        let digest = md5::compute(String::from(input).add(&salt));
        format!("{:x}|{}", digest, salt)
    }

    fn encode_password0(input: &str, salt: &str) -> String {
        let digest = md5::compute(String::from(input).add(salt));
        format!("{:x}|{}", digest, salt)
    }

    fn random_salt(len: i8) -> String {
        let mut result = String::with_capacity(len as usize);
        let mut rng = rand::thread_rng();
        for _ in 0..len {
            let v = rng.gen_range(0..26);
            result.push(char::from(b'A' + v))
        }

        result
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub repo: HashMap<i64, UserRole>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum UserRole {
    Admin = 0x0000ffff,         // grant other role to other one
    Manager = 0x00000fff,       // create/modify/delete any item/tag, remove any tag from item
    User = 0x000000ff,          // apple tag to item, also create/delete tag
    Viewer = 0x0000000f,        // only view any
    None = 0x00000000,          // nothing
}

impl UserRole {
    pub fn to_int(&self) -> i64 {
        self.clone() as i64
    }

    pub fn from_int(value: i64) -> UserRole {
        match value {
            0xffff => UserRole::Admin,
            0x0fff => UserRole::Manager,
            0x00ff => UserRole::User,
            0x000f => UserRole::Viewer,
            0x0000 | _ => UserRole::None,
        }
    }

    pub fn geq(&self, other: &UserRole) -> bool {
        self.to_int() & other.to_int() == other.to_int()
    }
}

impl User {
    pub fn new(user: UserStorage, user_repo_list: &Vec<UserRepoRoleStorage>) -> Self {
        let mut repo_role_map = HashMap::new();
        for user_repo_role in user_repo_list {
            repo_role_map.insert(user_repo_role.repo_id, UserRole::from_int(user_repo_role.role));
        }

        User {
            id: user.id,
            name: user.name,
            repo: repo_role_map,
        }
    }
}
