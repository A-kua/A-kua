use std::fs;
use actix_web::{Error, HttpRequest};
use futures::{future::ok, stream::once};
use actix_web::{get, HttpResponse, Responder, web, Result};
use crate::bean::{Post};
use actix_files::{Files, NamedFile};
use actix_web_lab::respond::Html;
use askama::Template;
use crate::structs::template::{BlogIndexTemplate, PostTemplate};
use markdown;
use toml::from_str;

#[get("/{name}")]
async fn posts(mut path: web::Path<Post>) -> HttpResponse {
    path.name.push_str(".toml");
    let post_file_name = &path.name;
    let file_path = std::path::PathBuf::from(std::env::var("BLOG_STATIC").unwrap())
        .as_path()
        .join("static-post")
        .join(post_file_name);

    match fs::read_to_string(file_path) {
        Ok(string_file) => {
            match from_str::<PostTemplate>(&string_file) {
                Ok(deserialized_from_file) => {
                    let html = deserialized_from_file.render()
                        .expect("template should be valid");
                    HttpResponse::Ok()
                        .body(html)
                }
                Err(_) => {
                    HttpResponse::InternalServerError()
                        .body("Post file deserialize fail")
                }
            }
        }
        Err(_) => {
            HttpResponse::InternalServerError()
                .body("Post file not found")
        }
    }
}

#[get("/image/{name}")]
async fn image(req: HttpRequest, name: web::Path<(String)>) -> HttpResponse {
    let file_path = std::path::PathBuf::from(std::env::var("BLOG_STATIC").unwrap())
        .as_path()
        .join("static-image")
        .join(&name.into_inner());

    match NamedFile::open_async(file_path).await {
        Ok(file) => {
            file.into_response(&req)
        }
        Err(_) => {
            HttpResponse::NotFound()
                .body("Image file not found")
        }
    }
}

#[get("/css/{name}")]
async fn css(req: HttpRequest, name: web::Path<(String)>) -> HttpResponse {
    let file_path = std::path::PathBuf::from(std::env::var("BLOG_STATIC").unwrap())
        .as_path()
        .join("static-css")
        .join(&name.into_inner());

    match NamedFile::open_async(file_path).await {
        Ok(file) => {
            file.into_response(&req)
        }
        Err(_) => {
            HttpResponse::NotFound()
                .body("Css file not found")
        }
    }
}

#[get("/js/{name}")]
async fn js(req: HttpRequest, name: web::Path<(String)>) -> HttpResponse {
    let file_path = std::path::PathBuf::from(std::env::var("BLOG_STATIC").unwrap())
        .as_path()
        .join("static-js")
        .join(&name.into_inner());

    match NamedFile::open_async(file_path).await {
        Ok(file) => {
            file.into_response(&req)
        }
        Err(_) => {
            HttpResponse::NotFound()
                .body("Javascript file not found")
        }
    }
}