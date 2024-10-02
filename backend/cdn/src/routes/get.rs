use actix_web::{get, web, HttpResponse};
use minio_rsc::Minio;
use reqwest::Response;
use blake2::{Blake2b512, Digest};

pub async fn get(db: web::Data<Minio>, param: web::Path<(String, String, String)>) -> HttpResponse {
    let bucket_str: String = param.0.clone();
    let name_str: String = param.1.clone();
    let hash_str: String = param.2.clone();

    match db.get_object(bucket_str, name_str).await {
        Ok(response) => {
            match response.bytes().await {
                Ok(response_bytes) => {
                    let mut hasher = Blake2b512::new();
                    hasher.update(response_bytes.clone());
                    let res = hasher.finalize();

                    let hex_res = format!("{:x}", res);
                    if hex_res != hash_str {
                    return HttpResponse::BadRequest()
                            .content_type("text/html; charset=utf-8")
                            .body("Invalid hash");
                    }

                    HttpResponse::Ok()
                        .body(response_bytes)
                }
                Err(error) => HttpResponse::InternalServerError()
                    .content_type("text/html; charset=utf-8")
                    .body(format!("Error fetching response body: {}", error)),
            }
        }
        Err(error) => HttpResponse::InternalServerError()
            .content_type("text/html; charset=utf-8")
            .body(format!("Error getting object: {}", error)),
    }
}
