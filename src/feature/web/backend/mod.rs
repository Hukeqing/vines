use crate::core::service::Service;
use crate::feature::web::backend::wrap::{SessionWrap, TimeLoggerWrap, Wrap, WrapContext};
use actix_web::web;
use actix_web::web::Data;
use std::sync::Arc;

mod user;
mod item;
mod repo;
mod tag;

#[actix_web::main]
pub async fn init(service: Service) -> std::io::Result<()> {
    use actix_web::{App, HttpServer};
    use actix_web::dev::Service;

    HttpServer::new(move || {
        let wraps: Arc<Vec<Box<dyn Wrap>>> = Arc::new(vec![
            Box::new(SessionWrap { user: service.user.clone() }),
            Box::new(TimeLoggerWrap),
        ]);
        App::new()
            .wrap_fn(move |req, srv| {
                let mut ctx = WrapContext::new();
                let my_wraps = wraps.clone();
                my_wraps.iter().for_each(|wrap| wrap.pre(&req, &mut ctx));
                let fut = srv.call(req);
                async move {
                    let mut res = fut.await;
                    for wrap in my_wraps.iter().rev() {
                        res = wrap.post(res, &mut ctx)
                    }
                    res
                }
            })
            .app_data(web::PayloadConfig::new(100 * 1024 * 1024))
            .app_data(Data::from(service.repo.clone()))
            .app_data(Data::from(service.user.clone()))
            .app_data(Data::from(service.item.clone()))
            .app_data(Data::from(service.tag.clone()))
            .service(
                web::scope("/api/repo")
                    .route("/list", web::get().to(repo::list))
                    .route("/create", web::post().to(repo::create))
            )
            .service(
                web::scope("/api/user")
                    .route("/login", web::post().to(user::login))
                    .route("/logout", web::post().to(user::logout))
                    .route("/check", web::get().to(user::check))
                    .route("/register", web::post().to(user::register))
                    .route("/change_role", web::post().to(user::change_role))
                    .route("/change_password", web::post().to(user::change_password))
            )
            .service(
                web::scope("/api/item")
                    .route("/list", web::get().to(item::list))
                    .route("/get", web::get().to(item::get))
                    .route("/get_extend", web::get().to(item::get_extend))
                    .route("/read", web::get().to(item::read))
                    .route("/read_thumbnail", web::get().to(item::read_thumbnail))
                    .route("/create", web::post().to(item::create))
            )
    })
        .bind("127.0.0.1:8080")?
        .workers(4)
        .run()
        .await
}

mod wrap {
    use crate::core::service::UserService;
    use crate::feature::web::backend::user;
    use actix_web::dev::{ServiceRequest, ServiceResponse};
    use actix_web::Error;
    use log::info;
    use std::sync::Arc;
    use std::time::Instant;

    pub(super) struct WrapContext {
        start_time: Option<Instant>,
        method: Option<String>,
        path: Option<String>,
        user_id: Option<i64>,
    }

    pub(super) trait Wrap {
        fn pre(&self, req: &ServiceRequest, ctx: &mut WrapContext);

        fn post(&self, res: Result<ServiceResponse, Error>, ctx: &mut WrapContext) -> Result<ServiceResponse, Error>;
    }

    pub(super) struct TimeLoggerWrap;

    pub(super) struct SessionWrap {
        pub user: Arc<UserService>,
    }

    impl WrapContext {
        pub fn new() -> Self {
            Self {
                start_time: None,
                method: None,
                path: None,
                user_id: None,
            }
        }
    }

    impl Wrap for TimeLoggerWrap {
        fn pre(&self, req: &ServiceRequest, ctx: &mut WrapContext) {
            ctx.start_time = Some(Instant::now());
            ctx.method = Some(req.method().to_string());
            ctx.path = Some(String::from(req.uri().path()));
        }

        fn post(&self, res: Result<ServiceResponse, Error>, ctx: &mut WrapContext) -> Result<ServiceResponse, Error> {
            let elapsed_time = ctx.start_time.unwrap().elapsed();
            info!("[{}] to {} from user {} took {:?}", 
                &ctx.method.as_ref().unwrap(),
                &ctx.path.as_ref().unwrap(),
                &ctx.user_id.as_ref().unwrap_or(&0),
                elapsed_time);
            res
        }
    }

    impl Wrap for SessionWrap {
        fn pre(&self, req: &ServiceRequest, ctx: &mut WrapContext) {
            if let Some(cookie) = req.cookie(user::SESSION_KEY) {
                let session = String::from(cookie.value());
                match self.user.refresh(&session) {
                    Ok(user) => ctx.user_id = Some(user.id),
                    Err(_) => {}
                }
            }
        }

        fn post(&self, res: Result<ServiceResponse, Error>, _: &mut WrapContext) -> Result<ServiceResponse, Error> {
            self.user.clear();
            res
        }
    }
}
