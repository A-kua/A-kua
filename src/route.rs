use actix_web::{Error};
use futures::{future::ok, stream::once};
use actix_web::{get, HttpResponse, Responder, web};
use crate::bean::{Article};

#[get("/{name}/{section}")]
async fn article(path: web::Path<Article>) -> impl Responder {
    format!("number {}, section {}!", path.name, path.section)
}

#[get("/image/{name}")]
async fn image(path: web::Path<(String)>) -> HttpResponse {
    let bytes = once(ok::<_, Error>(web::Bytes::from_static(b"test")));

    HttpResponse::Ok()
        .content_type("image/png")
        .streaming(bytes)
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