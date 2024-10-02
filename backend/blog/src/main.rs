use std::env;
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_web::{web, App, HttpServer};
use actix_web::cookie::Key;
use mongodb::Client;

mod routes;
mod blog;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let uri = env::var("BLOG_MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let client = Client::with_uri_str(uri).await.expect("failed to connect");

    let secret_key = Key::from(env::var("BLOG_SECRET_KEY")
        .map_err(|err| {
            eprintln!("Error fetching secret key: {}", err);
            std::process::exit(1);
        })
        .unwrap().as_bytes());

    let redis_uri = env::var("BLOG_REDIS_URI").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let redis_store = RedisSessionStore::new(redis_uri)
        .await
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::new(
                    redis_store.clone(),
                    secret_key.clone(),
                )
            )
            .app_data(web::Data::new(client.clone()))
            .configure(routes::init)
    })
        .bind(("127.0.0.1", 3002))?
        .run()
        .await
}
