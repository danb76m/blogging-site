use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::{web, App, HttpServer};
use env_logger;
use mongodb::Client;
use std::env;

mod routes;


#[derive(Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub authorise: String
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let oauth_config_data = web::Data::new(OAuthConfig {
        client_id: env::var("BLOG_CLIENT_ID").unwrap().to_string(),
        client_secret: env::var("BLOG_CLIENT_SECRET").unwrap().to_string(),
        redirect_uri: env::var("BLOG_REDIRECT_URI").unwrap_or_else(|_| "http://127.0.0.1:3001/callback".into()).to_string(),
        authorise: "https://github.com/login/oauth/authorize?client_id={client_id}&state={state}&redirect_uri={redirect_uri}".to_string()
    });

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

    // mongo
    let uri = env::var("BLOG_MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let client = Client::with_uri_str(uri).await.expect("failed to connect");

	HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::new(
                    redis_store.clone(),
                    secret_key.clone(),
                )
            )
            .app_data(web::Data::new(client.clone()))
            .app_data(oauth_config_data.clone())
            .configure(routes::init)
    })
    .bind(("127.0.0.1", 3001))?
    .run()
    .await
}