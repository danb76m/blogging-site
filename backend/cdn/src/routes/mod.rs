use actix_web::{web, HttpResponse};

pub mod get;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/get/{bucket}/{name}/{hash}")
            .route(web::get().to(get::get))
    );
}