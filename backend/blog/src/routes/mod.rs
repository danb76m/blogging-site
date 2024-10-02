use actix_web::web;

pub mod list;
mod upload;
mod drafts;
mod draft;
mod edit;
mod hide;
mod get;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/posts")
            .service(web::resource("/list/{page}/{limit}").to(list::list))
            .service(web::resource("/get/{id}").route(web::get().to(get::get)))
            .service(web::resource("/drafts").to(drafts::drafts))
            .service(web::resource("/draft/{id}/{draft}").route(web::patch().to(draft::draft)))
            .service(web::resource("/hide/{id}/{hide}").route(web::patch().to(hide::hide)))
            .service(web::resource("/edit/{id}").route(web::patch().to(edit::edit)))
    ).service(
        web::resource("/upload")
            .route(web::post().to(upload::upload))
    );
}