use futures::stream::{StreamExt, TryStreamExt};
use mongodb::bson::{doc, DateTime};
use mongodb::error::Error;
use mongodb::results::{DeleteResult, InsertOneResult, UpdateResult};
use mongodb::{bson, Client, Collection, Cursor};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Post {
    pub creator: String,
    pub id: String,
    pub title: String,
    pub body: String,
    pub draft: bool,
    pub hidden: bool,
    pub created: Option<DateTime>,
    pub published: Option<DateTime>,
    pub last_edit: Option<DateTime>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PostUpload {
    pub title: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Criteria {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pagination {
    pub page: i64,
    pub limit: i64,
}


pub async fn insert_post(client: &Client, post: &Post) -> mongodb::error::Result<InsertOneResult> {
    let collection: Collection<Post> = client.database("blog").collection("posts");
    collection.insert_one(post).await
}

pub async fn update_post(client: &Client, post: Post) -> mongodb::error::Result<UpdateResult> {
    let collection: Collection<Post> = client.database("blog").collection("posts");
    let update_doc = doc! { "$set": bson::to_document(&post)? }; // Convert Post to BSON document
    collection.update_one(
        doc! { "id": post.id },
        update_doc,
    ).await
}

pub async fn delete_post(client: &Client, criteria: Criteria) -> mongodb::error::Result<DeleteResult> {
    let collection: Collection<Post> = client.database("blog").collection("posts");
    collection.delete_one(doc! { "id": criteria.id  }).await
}


pub async fn post_exists(client: &Client, criteria: Criteria) -> mongodb::error::Result<bool> {
    Ok(get_post(client, criteria).await?.is_some())
}

pub async fn get_post(client: &Client, criteria: Criteria) -> mongodb::error::Result<Option<Post>> {
    let collection: Collection<Post> = client.database("blog").collection("posts");
    collection.find_one(doc! { "id": criteria.id }).await
}

pub async fn get_posts(client: &Client, creator: Option<String>, drafts: bool, hidden: bool, pagination: Option<Pagination>) -> Result<Vec<Post>, Error> {
    let collection: Collection<Post> = client.database("blog").collection("posts");

    let mut filter = doc! {};
    filter.insert("hidden", hidden);
    if drafts && creator.is_some() {
        filter.insert("draft", true);
        filter.insert("creator", creator.unwrap());
    } else {
        filter.insert("draft", false);
    }

    let skip = match pagination.as_ref() {
        Some(pagination) => (pagination.page - 1) * pagination.limit,
        None => 0, // Default to starting from the beginning
    };

    let limit = match pagination.as_ref() {
        Some(pagination) => pagination.limit,
        None => i64::MAX, // Default to no limit
    };

    let cursor: Cursor<Post> = collection.find(filter)
        .skip(skip as u64)
        .limit(limit)
        .await.expect("Failed to find posts");

    cursor.try_collect().await
}

// THANKS https://www.reddit.com/r/learnrust/comments/lnewid/create_a_random_fixed_digitlength_i32_in_which/
pub async fn generate_id(client: &Client) -> String {
    let id: String = String::from_utf8(
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .collect::<Vec<_>>(),
    ).unwrap();

    let exists: bool = post_exists(client, Criteria { id: id.to_string() }).await.unwrap_or_else(|_| false);
    if exists {
        return Box::pin(generate_id(client)).await;
    }
    id // return id
}