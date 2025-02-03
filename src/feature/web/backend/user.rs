use crate::common::result::to_response;
use crate::core::service::user::UserService;
use actix_web::cookie::Cookie;
use actix_web::web::{Data, Json};
use actix_web::{HttpResponse, Responder};
use actix_web::cookie::time::Duration;
use crate::core::service::UserRole;

pub(super) static SESSION_KEY: &str = "r_session";

pub(super) async fn check(user: Data<UserService>) -> impl Responder {
    to_response(user.check())
}

pub(super) async fn login(user: Data<UserService>, query: Json<LoginRequest>) -> impl Responder {
    let login_res = user.login(&query.username, &query.password);
    let session_id = user.get_session_key();
    match login_res {
        Ok(data) => HttpResponse::Ok().cookie(
            Cookie::build(SESSION_KEY, session_id).path("/").max_age(Duration::days(7)).finish()
        ).json(data),
        Err(e) => e.to_response()
    }
}

pub(super) async fn logout(user: Data<UserService>) -> impl Responder {
    match user.logout() {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => e.to_response()
    }
}

pub(super) async fn register(user: Data<UserService>, query: Json<CreateRequest>) -> impl Responder {
    let result = user.create(query.username.clone(), &query.password);
    let session_id = user.get_session_key();
    match result {
        Ok(data) => HttpResponse::Ok().cookie(Cookie::new(SESSION_KEY, session_id)).json(data),
        Err(e) => e.to_response()
    }
}

pub(super) async fn change_role(user: Data<UserService>, query: Json<ChangeUserRole>) -> impl Responder {
    let role = UserRole::from_int(query.role);
    let result = user.change_user_role(query.id, query.repo_id, role);
    to_response(result)
}

pub(super) async fn change_password(user: Data<UserService>, query: Json<ChangePasswordRequest>) -> impl Responder {
    let result = user.change_password(&query.old_password, &query.new_password);
    to_response(result)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct LoginRequest {
    username: String,
    password: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct CreateRequest {
    username: String,
    password: String,
    allow_tag: Vec<i64>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct ChangeUserRole {
    id: i64,
    repo_id: i64,
    role: i64,
}