use crate::blog::get_posts;
use actix_session::Session;
use actix_web::{web, HttpResponse};
use mongodb::Client;
use user::{get_account_from_session, Account};

pub async fn drafts(session: Session, client: web::Data<Client>) -> HttpResponse {
    let mongo: &Client = client.get_ref();

    let account : Account = match get_account_from_session(mongo, &session).await {
        Ok(account) => account,
        Err(e) => {
            return HttpResponse::InternalServerError().json(format!("{:?}", e));
        }
    };

    let posts = get_posts(mongo, Some(account.uuid), true, false, None).await;
    let json_posts = serde_json::to_string(&posts.unwrap()).unwrap();
    HttpResponse::Ok().body(json_posts)
}
