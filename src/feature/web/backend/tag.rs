use actix_web::Responder;
use actix_web::web::{Data, Query};
use crate::common::result::to_response;
use crate::core::service::TagService;

pub(super) async fn list(tag: Data<TagService>, request: Query<ListRequest>) -> impl Responder {
    to_response(tag.list(request.repo_id))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct ListRequest {
    repo_id: i64
}