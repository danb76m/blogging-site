use std::string::ToString;
use crate::OAuthConfig;
use actix_session::Session;
use actix_web::{web::{self}, HttpResponse, Responder};
use mongodb::results::InsertOneResult;
use pwhash::bcrypt;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use user::{account_exists, create_session, find_account, insert_account, set_account_session, Account, Criteria};

use reqwest::Client;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
   state: String,
   code: String,
}

#[derive(Serialize)]
pub struct TokenRequest {
    client_id: String,
    client_secret: String,
    code: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    scope: String,
    token_type: String,
}

// Option<String> is used to represent that it can be either string or null
#[derive(Deserialize)]
struct UserResponse {
    login: Option<String>,
    id: u64,
    avatar_url: Option<String>,
    url: Option<String>,
    name: Option<String>,
    email: Option<String>,
    bio: Option<String>,
}

pub enum UserError {
    RequestError(reqwest::Error),
    DeserializationError(serde_json::Error),
}

pub async fn callback(session: Session, mongo: web::Data<mongodb::Client>, info: web::Query<AuthRequest>, data: web::Data<OAuthConfig>) -> impl Responder {
    let mongo_client: &mongodb::Client = mongo.get_ref();
    
    let token_request = TokenRequest {
        client_id: data.client_id.clone(),
        client_secret: data.client_secret.clone(),
        code: info.code.clone(),
    };

    if let Some(hash) = match session.get::<String>("auth") {
        Ok(hash) => Some(hash),
        Err(e) => {
            println!("Error exchanging access token: {}", e);
            return HttpResponse::InternalServerError().finish()
        }
    } {
        if hash.is_none() {
            return HttpResponse::InternalServerError().body("Hash failed");
        }
        if !bcrypt::verify(&info.state, &hash.unwrap()) {
            return HttpResponse::InternalServerError().body("Verification failed");
        }
    } else {
        return HttpResponse::InternalServerError().body("No session auth found.");
    }

    session.remove("auth");

    let client = Client::new();
    let res = client.post(
    format!("https://github.com/login/oauth/access_token?client_id={}&client_secret={}&code={}",
            token_request.client_id, token_request.client_secret, token_request.code
    )).header("Accept", "application/json");

    match res.send().await {
        Ok(response) => {
            
            let results = match response.text().await {
                Ok(results) => results,
                Err(err) => {
                    return HttpResponse::InternalServerError().body("Failed to serialize github response");
                }
            };

            let results: TokenResponse = match from_str(&results) {
                Ok(results) => results,
                Err(err) => {
                    return HttpResponse::InternalServerError().body("Failed to serialize github token to token response");
                }
            };

            let user : UserResponse = match get_user(client, results).await {
                Ok(user) => user,
                Err(err) => {
                    return HttpResponse::InternalServerError().body("Failed to get user json response");
                }
            };

            let account_exists: bool = match account_exists(mongo_client, Criteria {github_id: user.id}).await {
                Ok(accountExists) => accountExists,
                Err(err) => {
                    return HttpResponse::InternalServerError().body("Could not get account exists from database");
                }
            };

            /*
            If account exists we create a new session id and give it to the user so we can authenticate them
            If account does not exist we create a new account
             */
            if !account_exists {
                let result: InsertOneResult = match insert_account(mongo_client,
                                                                   Account {
                                                                       name: user.name,
                                                                       uuid:  Uuid::new_v4().to_string(),
                                                                       github_id: user.id,
                                                                       email: user.email,
                                                                       elevated: false,
                                                                       sessions: vec![],
                                                                   }).await {
                    Ok(account) => account,
                    Err(err) => {
                        return HttpResponse::InternalServerError().body("Could not insert account");
                    }
                };
            }

            let account: Account = match find_account(&mongo_client, Criteria {github_id: user.id}).await {
                    Ok(account) => {
                        if account.is_none() {
                            return HttpResponse::InternalServerError().body("Could not find account from database");
                        } else {
                            account.unwrap()
                        }
                    },
                    Err(err) => {
                        return HttpResponse::InternalServerError().body("Could not GET account");
                    }
                };

            let session_id: String = match create_session(mongo_client, account.github_id).await {
                Ok(session) => session,
                Err(err) => {
                    return HttpResponse::InternalServerError().body("Could not generate a session");
                }
            };

            match set_account_session(&session_id, &account.uuid, &account.github_id, &session).await {
                Ok(..) => {
                },
                Err(err) => {
                    return HttpResponse::InternalServerError().body("Could not set account session")
                }
            }

            HttpResponse::Ok().body("Successfully authenticated.")
        },
        Err(e) => {
            println!("Error exchanging access token: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}



pub async fn get_user(client: Client, results: TokenResponse) -> Result<UserResponse, UserError> {
    let url = "https://api.github.com/user";
    let res = match client.get(url)
                  .bearer_auth(results.access_token)
        .header("User-Agent", "Danielle's Blog API")
                  .send()
                  .await {
        Ok(res) => res,
        Err(err) => {
            return Err(UserError::RequestError(err))
        }
    };

    let text_result = match res.text().await {
        Ok(text_result) => text_result,
        Err(err) => {
            return Err(UserError::RequestError(err));
        }
    };

    let results: UserResponse = match from_str(&text_result) {
        Ok(results) => results,
        Err(err) => {
            eprintln!("Error: {}", err);
            return Err(UserError::DeserializationError(err))}
    };

    Ok(results) 
}