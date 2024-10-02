use std::fmt::format;
use std::pin::Pin;
use std::task::{Context, Poll};
use actix_session::{SessionGetError, SessionInsertError};
use actix_web::body::{BodySize, MessageBody};
use actix_web::web::Bytes;
use mongodb::{Client, Collection};
use mongodb::bson::{doc, Bson};
use mongodb::results::InsertOneResult;
use serde::{Deserialize, Serialize};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use uuid::uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub name: Option<String>,
    pub uuid: String,
    pub github_id: u64,
    pub email: Option<String>,
    pub elevated: bool,
    pub sessions: Vec<Session>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Session {
    pub session_id: String,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct Criteria {
    pub github_id: u64,
}


pub async fn insert_account(client: &Client, account: Account) -> mongodb::error::Result<InsertOneResult> {
    let collection: Collection<Account> = client.database("account").collection("accounts");
    collection.insert_one(account).await
}

pub async fn account_exists(client: &Client, criteria: Criteria) -> Result<bool, mongodb::error::Error> {
    let account = find_account(client, criteria).await?;
    Ok(!account.is_none())
}

pub async fn find_account(client: &Client, criteria: Criteria) -> mongodb::error::Result<Option<Account>> {
    let collection: Collection<Account> = client.database("account").collection("accounts");
    let filter = doc! {"github_id": Bson::Int64(criteria.github_id as i64)};
    collection.find_one(filter).await
}

pub async fn create_session(client: &Client, github_id: u64) -> mongodb::error::Result<String> {
    let session_id = generate_session_id();
    let collection: Collection<Account> = client.database("account").collection("accounts");
    let filter = doc! {"github_id": Bson::Int64(github_id as i64)};
    let update = doc! { "$push": { "sessions": { "session_id": &session_id } } };
    let result = collection.update_one(filter, update).await?;
    if result.modified_count == 0 {
        return Err(mongodb::error::Error::custom("Error creating session".to_string()));
    }
    Ok(session_id)
}

pub async fn find_account_by_session_id(client: &Client, uuid: String, session_id: String, github_id: u64) -> mongodb::error::Result<Option<Account>> {
    println!("Trying to find account from accounts collection from id {}, {}", uuid, github_id as i64);

    let collection: Collection<Account> = client.database("account").collection::<Account>("accounts");
    let filter = doc! {
        "uuid": uuid,
        "github_id": github_id as i64,
        "sessions.session_id": session_id
    };
    collection.find_one(filter).await
}

pub async fn delete_session(client: &Client, session_id: &str) -> mongodb::error::Result<Option<Account>> {
    let collection: Collection<Account> = client.database("account").collection("accounts");
    let filter = doc! {"sessions.session_id": session_id};
    let update = doc! { "$pull": { "sessions": { "session_id": session_id } } };
    let result = collection.find_one_and_update(filter, update).await?;
    Ok(result)
}

#[derive(Debug)]
pub enum SessionError {
    MongoError(mongodb::error::Error),
    SessionError(SessionGetError),
    SessionInsertError(SessionInsertError),
    NoneFound(String),
    AccountNotFound(String),
}

pub async fn get_account_from_session(client: &Client, session: &actix_session::Session) -> Result<Account, SessionError> {
    session.entries().iter().for_each(|entry| {
        println!("{:?}", entry);
    });

    let session_id : String = match session.get("session") {
        Ok(session_id) => {
            if session_id.is_none() {
                return Err(SessionError::NoneFound("session id is not found".to_string()))
            }
            session_id.unwrap()
    },
        Err(err) => return Err(SessionError::SessionError(err)),
    };
    let account_uuid : String = match session.get("uuid") {
        Ok(account_uuid) => {
            if account_uuid.is_none() {
                return Err(SessionError::NoneFound("uuid is not found".to_string()))
            }
            account_uuid.unwrap()
        },
        Err(err) => return Err(SessionError::SessionError(err)),
    };
    let git_id : u64 = match session.get("git_id") {
        Ok(git_id) => {
            if git_id.is_none() {
                return Err(SessionError::NoneFound("git id not found".to_string()))
            }
            git_id.unwrap()
        },
        Err(err) => return Err(SessionError::SessionError(err)),
    };

    let account : Account = match find_account_by_session_id(client, account_uuid, session_id, git_id).await {
        Ok(acc) => {
            if acc.is_none() {
                return Err(SessionError::AccountNotFound("Account not found from database".to_string()))
            }
            acc.unwrap()
        },
        Err(err) => return Err(SessionError::AccountNotFound("Account not found from session id".to_string()))
    };
    Ok(account)
}

pub async fn set_account_session(session_id: &String, uuid: &String, git_id: &u64, session: &actix_session::Session) -> Result<(), SessionError> {
    match session.insert("session", &session_id) {
        Ok(_) => {},
        Err(err) => {
            return Err(SessionError::SessionInsertError(err))
        }
    }

    match session.insert("git_id", git_id) {
        Ok(_) => {},
        Err(err) => {
            return Err(SessionError::SessionInsertError(err))
        }
    }

    match session.insert("uuid", uuid) {
        Ok(_) => {},
        Err(err) => {
            return Err(SessionError::SessionInsertError(err))
        }
    }

    Ok(())
}

fn generate_session_id() -> String {
    String::from_utf8(
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .collect::<Vec<_>>(),
    ).unwrap()
}

pub fn main() {}