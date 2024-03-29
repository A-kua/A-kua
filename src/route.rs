use actix_web::{Error, HttpRequest};
use futures::{future::ok, stream::once};
use actix_web::{get, HttpResponse, Responder, web};
use crate::bean::{Article};
use actix_files::{Files, NamedFile};

#[get("/{name}/{section}")]
async fn article(path: web::Path<Article>) -> impl Responder {
    format!("number {}, section {}!", path.name, path.section)
}

#[get("/image/{name}")]
async fn image(req: HttpRequest,name: web::Path<(String)>) -> HttpResponse {
    let file_path = std::path::PathBuf::from(std::env::var("BLOG_STATIC").unwrap())
        .as_path()
        .join("static-image")
        .join(&name.into_inner());

    match NamedFile::open_async(file_path).await {
        Ok(file) => {
            file.into_response(&req)
        },
        Err(_) => {
            HttpResponse::NotFound()
                .body("File not found")
        },
    }
}

#[get("/css/{name}")]
async fn css(path: web::Path<(String)>) -> HttpResponse {
    let body = once(ok::<_, Error>(web::Bytes::from_static(b"test")));

    HttpResponse::Ok()
        .content_type("text/css")
        .streaming(body)
}

#[get("/js/{name}")]
async fn js(path: web::Path<(String)>) -> HttpResponse {
    let body = once(ok::<_, Error>(web::Bytes::from_static(b"test")));

    HttpResponse::Ok()
        .content_type("application/javascript")
        .streaming(body)
}