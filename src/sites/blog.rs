use std::sync::atomic::Ordering;
use actix_web::{HttpResponse, Responder, web, Result};
use actix_web_lab::respond::Html;
use crate::bean::BlogState;
use askama::{Template};
use crate::structs::template::BlogIndexTemplate;

pub(crate) async fn index(data: web::Data<BlogState>) -> Result<impl Responder> {
    let html = data.blog_index
        .render()
        .expect("template should be valid");

    Ok(Html(html))
}