use actix_web::web;

pub mod request;
pub mod callback;
mod protected;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/request")
            .route(web::get().to(request::request))
    )
    .service(
        web::resource("/callback")
            .route(web::get().to(callback::callback))
    )
        .service(web::resource("/protected").route(web::get().to(protected::protected)));
}