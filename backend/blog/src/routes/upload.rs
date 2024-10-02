use crate::blog;
use crate::blog::{Post, PostUpload};
use actix_session::Session;
use actix_web::{web, HttpResponse};
use mongodb::bson;
use user::Account;

pub async fn upload(session: Session, info: web::Json<PostUpload>, client: web::Data<mongodb::Client>) -> HttpResponse {
    let account : Account = match user::get_account_from_session(&client, &session).await {
        Ok(account) => account,
        Err(_) => {
            return HttpResponse::InternalServerError().body("No account found")
        }
    };
    
    if !account.elevated {
        return HttpResponse::Unauthorized().finish()
    }

    let post_upload = info.clone();

    let post : Post = Post {
        creator: account.uuid,
        id: blog::generate_id(&client).await,
        title: post_upload.title.unwrap().clone(),
        body: post_upload.body.unwrap().clone(),
        draft: true,
        hidden: false,
        created: Option::from(bson::DateTime::now()),
        published: None,
        last_edit: None,
    };

    blog::insert_post(&client, &post).await.expect("TODO: panic message");
    
    HttpResponse::Ok().content_type("text/json").body(
        serde_json::to_string(&post).unwrap()
    )
}
