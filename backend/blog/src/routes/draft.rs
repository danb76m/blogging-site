use crate::blog;
use crate::blog::{get_post, Criteria, Post};
use actix_session::Session;
use actix_web::{web, HttpResponse};
use mongodb::Client;
use user::{get_account_from_session, Account};

// post_id, boolean draft
pub async fn draft(session: Session,path: web::Path<(String, bool)>, client: web::Data<Client>) -> HttpResponse {
    let mongo: &Client = client.get_ref();

    let (post_id, draft) = path.into_inner();

    let account : Account = match get_account_from_session(mongo, &session).await {
        Ok(account) => account,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body("Could not get account");
        }
    };

    println!("Trying to find post matching {}", post_id);

    let post_op : Option<Post> = match get_post(mongo, Criteria {
        id: post_id
    }).await {
        Ok(post_op) =>
            post_op,
        Err(e) => {
            return HttpResponse::InternalServerError().body("Could not get account");
        }
    };

    if post_op.is_none() {
        return HttpResponse::NotFound().body("Could not find post matching");
    }

    let mut post = post_op.unwrap();

    if !account.uuid.eq(&post.creator) {
        return HttpResponse::Unauthorized().body("Not authorised")
    }

    post.draft = draft;
    match blog::update_post(mongo, post).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            HttpResponse::InternalServerError().body("Failed to update")
        }
    }
}
