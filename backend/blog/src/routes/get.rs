use crate::blog::{get_post, Criteria};
use actix_session::Session;
use actix_web::{web, HttpResponse};
use mongodb::Client;
use user::Account;

pub async fn get(session: Session, client: web::Data<Client>, path: web::Path<(String)>) -> HttpResponse {
    let (id) = path.into_inner();

    let mongo: &Client = client.get_ref();
    let post = match get_post(mongo, Criteria {
        id: id.clone(),
    }).await {
        Ok(post) => post,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if post.is_none() {
        return HttpResponse::NotFound().finish();
    }

    let post = post.unwrap();

    if post.draft || post.hidden {
        let account : Account = match user::get_account_from_session(&client, &session).await {
            Ok(account) => account,
            Err(_) => {
                return HttpResponse::InternalServerError().body("You need to be the post creator to see this post")
            }
        };

        if !account.elevated || !account.uuid.eq(&post.creator) {
            return HttpResponse::Forbidden().finish();
        }
    }

    HttpResponse::Ok().body(serde_json::to_string(&post).unwrap())
}
