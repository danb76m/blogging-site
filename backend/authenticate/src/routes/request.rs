use std::iter;
use actix_session::Session;
use actix_web::{web::{self, Redirect}, App, HttpServer, Responder};
use mongodb::Client;
// https://www.linkedin.com/pulse/using-redis-rust-amit-nadiger/

use crate::OAuthConfig;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use pwhash::bcrypt;

pub async fn request(session: Session, data: web::Data<OAuthConfig>) -> impl Responder {
    let s = String::from_utf8(
     thread_rng()
         .sample_iter(&Alphanumeric)
         .take(30)
        .collect::<Vec<_>>(),
        ).unwrap();

    // Hash it so even the session doesn't know what it is
    session.insert("auth", bcrypt::hash(&s).unwrap()).expect("TODO: panic message");

	Redirect::to(data.authorise.clone()
    .replace("{client_id}", &data.client_id)
    .replace("{state}", &s)
    .replace("{redirect_uri}", &data.redirect_uri))
}