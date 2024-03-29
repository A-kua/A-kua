use std::sync::atomic::Ordering;
use actix_web::{HttpResponse, Responder, web};
use crate::bean::BlogState;

pub(crate) async fn index(data: web::Data<BlogState>) -> impl Responder {
    data.visit_count.fetch_add(1, Ordering::Relaxed);
    let mut counter = data.visit_count.load(Ordering::Relaxed);

    HttpResponse::Ok().body(format!("Polite number: {counter}"))
}