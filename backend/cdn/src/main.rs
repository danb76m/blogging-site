use std::env;
use actix_web::{web, App, HttpServer};
use minio_rsc::Minio;
use minio_rsc::provider::StaticProvider;

mod routes;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let minio = Minio::builder()
        .endpoint(env::var("BLOG_MINIO_ENDPOINT").unwrap_or_else(|_| "127.0.0.1:9000".to_owned()))
        .provider(StaticProvider::new(
            env::var("BLOG_MINIO_ACCESS_KEY").map_err(|err| {
                eprintln!("Error fetching secret key from env variables: {}", err);
                std::process::exit(1);
            }).unwrap(),
            env::var("BLOG_MINIO_SECRET_KEY").map_err(|err| {
                eprintln!("Error fetching secret key from env variables: {}", err);
                std::process::exit(1);
            }).unwrap(),
            None
        ))
        .secure(false) // TODO PRODUCTION MAKE THIS SECURE
        .build()
        .unwrap();

    // Move the minio instance into the closure
    let minio_data = web::Data::new(minio);

    HttpServer::new(move || {
        App::new()
            .app_data(minio_data.clone())
            .configure(routes::init)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
