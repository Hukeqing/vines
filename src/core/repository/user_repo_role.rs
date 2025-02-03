use crate::common::{Error, Res};
use rusqlite::params;
use crate::core::repository::holder::{exec, insert, query_all, update_check, RowData};

pub struct UserRepoRoleStorage {
    pub id: i64,
    pub user_id: i64,
    pub repo_id: i64,
    pub role: i64
}

pub fn create_user_repo_role(params: &UserRepoRoleStorage) -> Res<i64> {
    insert("INSERT INTO user_repo_role (user_id, repo_id, role) VALUES (?, ?, ?);", 
           params![params.user_id, params.repo_id, params.role])
}

pub fn select_user_role(user_id: i64) -> Res<Vec<UserRepoRoleStorage>> {
    query_all("SELECT * FROM user_repo_role WHERE user_id = ?", params![user_id], map)
}

pub fn update_user_role(id: i64, role: i64) -> Res<()> {
    update_check(exec("UPDATE user_repo_role SET role = ? WHERE id = ?", params![role, id]), Error::NoSuchUser)
}

fn map(row: &RowData) -> Res<UserRepoRoleStorage> {
    Ok(UserRepoRoleStorage {
        id: row.get(0)?,
        user_id: row.get(1)?,
        repo_id: row.get(2)?,
        role: row.get(3)?
    })
}
