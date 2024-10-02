use actix_session::Session;
use actix_web::{web::{self}, HttpResponse};
use user::{get_account_from_session, Account};

pub async fn protected(session: Session, mongo: web::Data<mongodb::Client>) -> HttpResponse {
    let mongo_client: &mongodb::Client = mongo.get_ref();
    let account: Account = match get_account_from_session(mongo_client, &session).await {
        Ok(account) => account,
        Err(err) => {
            return HttpResponse::Unauthorized().body(format!("{:?}", err)) },
    };

    HttpResponse::Ok().body("")
}