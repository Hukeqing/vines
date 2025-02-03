use crate::common::result::to_response;
use crate::common::{json, Res};
use crate::core::service::item::condition::{EndIdCondition, EndTimeCondition, ItemCondition, StartIdCondition, StartTimeCondition, TagCondition};
use crate::core::service::item::filter::{ItemFilter, RectangleFilter, SizeFilter, UrlFilter};
use crate::core::service::{ItemService, MarkedTag, TagService};
use actix_web::web::{Data, Query};
use actix_web::{web::Bytes, HttpResponse, Responder};
use either::Either;

pub(super) async fn get(item: Data<ItemService>, request: Query<GetRequest>) -> impl Responder {
    to_response(item.select_by_id(request.id))
}

pub(super) async fn get_extend(tag: Data<TagService>, request: Query<GetRequest>) -> impl Responder {
    fn query(tag: Data<TagService>, item_id: i64) -> Res<ExtendItemResponse> {
        let tags = tag.list_item(item_id)?;
        Ok(ExtendItemResponse { tags })
    }
    to_response(query(tag, request.id))
}

pub(super) async fn read(item: Data<ItemService>, request: Query<GetRequest>) -> impl Responder {
    let result = item.read_item(request.id);
    match result {
        Ok(stream) => HttpResponse::Ok().streaming(stream),
        Err(e) => e.to_response()
    }
}

pub(super) async fn read_thumbnail(item: Data<ItemService>, request: Query<GetRequest>) -> impl Responder {
    let result = item.read_thumbnail(request.id);
    match result {
        Ok(stream) => HttpResponse::Ok().streaming(stream),
        Err(e) => e.to_response()
    }
}

pub(super) async fn list(item: Data<ItemService>, request: Query<ItemListRequest>) -> impl Responder {
    let (condition_list, filter_list) = match &request.condition {
        None => (None, None),
        Some(cond_list) => {
            let mut condition_list = Vec::new();
            let mut filter_list = Vec::new();
            for cond in cond_list {
                match cond.cast() {
                    Ok(v) => match v {
                        Either::Left(e) => condition_list.push(e),
                        Either::Right(e) => filter_list.push(e)
                    }
                    Err(e) => return e.to_response()
                }
            }
            (Some(condition_list), Some(filter_list))
        }
    };

    to_response(item.select_list(request.repo_id, request.limit, request.from_big, &condition_list, &filter_list))
}

pub(super) async fn create(item: Data<ItemService>, query: Query<CreateRequest>, body: Bytes) -> impl Responder {
    let result = item.create(query.repo_id, query.name.clone(), &body.to_vec());
    to_response(result)
}


#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct GetRequest {
    id: i64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct CreateRequest {
    repo_id: i64,
    name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum ItemListType {
    StartId,
    EndId,
    StartTime,
    EndTime,
    Tag,
    Size,
    Rectangle,
    Url,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct ItemListCondition {
    key: ItemListType,
    value: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct ItemListRequest {
    repo_id: i64,
    limit: i64,
    from_big: bool,
    condition: Option<Vec<ItemListCondition>>
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct ExtendItemResponse {
    tags: Vec<MarkedTag>,
}

impl ItemListCondition {
    fn cast(&self) -> Res<Either<Box<dyn ItemCondition>, Box<dyn ItemFilter>>> {
        match self.key {
            ItemListType::StartId => Ok(Either::Left(Box::new(json::parse::<StartIdCondition>(&self.value)?))),
            ItemListType::EndId => Ok(Either::Left(Box::new(json::parse::<EndIdCondition>(&self.value)?))),
            ItemListType::StartTime => Ok(Either::Left(Box::new(json::parse::<StartTimeCondition>(&self.value)?))),
            ItemListType::EndTime => Ok(Either::Left(Box::new(json::parse::<EndTimeCondition>(&self.value)?))),
            ItemListType::Tag => Ok(Either::Left(Box::new(json::parse::<TagCondition>(&self.value)?))),
            ItemListType::Size => Ok(Either::Right(Box::new(json::parse::<SizeFilter>(&self.value)?))),
            ItemListType::Rectangle => Ok(Either::Right(Box::new(json::parse::<RectangleFilter>(&self.value)?))),
            ItemListType::Url => Ok(Either::Right(Box::new(json::parse::<UrlFilter>(&self.value)?))),
        }
    }
}
