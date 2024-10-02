use crate::blog::{get_posts, Pagination};
use actix_web::{web, HttpResponse};
use mongodb::Client;

pub async fn list(client: web::Data<Client>, path: web::Path<(i64, i64)>) -> HttpResponse {
    let (mut page, mut limit) = path.into_inner();

    if limit > 50 {
        limit = 50
    } else if limit < 10 {
        limit = 10
    }

    if page < 1 {
        page = 1
    }

    let mongo: &Client = client.get_ref();
    let posts = get_posts(mongo, None, false, false,
                          Some(Pagination {
                              page,
                              limit,
                          })).await;

    let json_posts = serde_json::to_string(&posts.unwrap()).unwrap();
    HttpResponse::Ok().body(json_posts)
}
