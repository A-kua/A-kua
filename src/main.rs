mod bean;
mod route;
mod site;

use actix_web::{web, App, HttpServer, Responder, FromRequest, guard};
use std::sync::{Arc};
use std::sync::atomic::{AtomicUsize};
use crate::bean::{BlogState};
use crate::route::{article, css, image, js};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let guide_scope = web::scope("/")
            .guard(guard::Host("127.0.0.1"))
            .route("", web::get().to(site::guide::index));

        let akua_scope = web::scope("/")
            .guard(guard::Host("akua.fan"))
            .route("", web::get().to(site::akua::index));

        let polite_scope = web::scope("/")
            .guard(guard::Host("polite.cat"))
            .route("", web::get().to(site::polite::index));

        let blog_scope = web::scope("/")
            .guard(guard::Host("blog.akua.fan"))
            .route("", web::get().to(site::blog::index))
            .service(web::scope("/article")
                .service(article))
            .service(web::scope("/static")
                .service(image)
                .service(css)
                .service(js));

        App::new()
            .app_data(web::Data::new(BlogState {
                visit_count: Arc::new(AtomicUsize::new(0)),
            }))
            .service(guide_scope)
            .service(akua_scope)
            .service(polite_scope)
            .service(blog_scope)
    })
        .bind(("0.0.0.0", 8080))?
        // .workers(6)
        .keep_alive(None)
        .run()
        .await
}
