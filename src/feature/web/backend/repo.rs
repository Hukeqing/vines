use actix_web::Responder;
use actix_web::web::{Data, Json};
use crate::common::result::to_response;
use crate::core::service::{RepoConfig, RepoService};

pub(super) async fn list(repo: Data<RepoService>) -> impl Responder {
    to_response(repo.list())
}

pub(super) async fn create(repo: Data<RepoService>, request: Json<CreateRepoRequest>) -> impl Responder {
    let CreateRepoRequest { name, config } = request.0;
    let result = repo.create(name, config);
    to_response(result)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateRepoRequest {
    name: String,
    config: RepoConfig
}
