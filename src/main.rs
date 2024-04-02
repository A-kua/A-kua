mod bean;
mod route;
mod sites;
mod structs;

use std::env::VarError;
use std::fs;
use actix_web::{web, App, HttpServer, Responder, FromRequest, guard, HttpResponse};
use std::sync::{Arc};
use std::sync::atomic::{AtomicUsize};
use actix_cors::Cors;
use toml::{to_string_pretty, from_str};
use serde::{Serialize, Deserialize};
use crate::bean::{BlogState};
use crate::route::{posts, css, image, js};
use crate::structs::template::{BlogIndexTemplate, Friend, Post, PostTemplate, Project};

async fn generate_index() -> String {
    let blog_index = BlogIndexTemplate {
        title: String::from("Akua's Blog"),
        motto: "Welcome to my blog!".to_string(),
        extra_js: "script.js".to_string(),
        abouts: vec![r#"学生@<a href="https://www.ahu.edu.cn/">安徽大学</a>、逆向CTFer@iSEAL; <a href="https://r3kapig.com/">r3kapig</a>、二进制安全爱好者"#.parse().unwrap(),
                     r#"gaoyucandev@gmail.com | <a href="https://github.com/GaoYuCan">github</a>"#.parse().unwrap()],
        posts: vec![
            Post { time: "2021-01-01".to_string(), name: "Post 1".to_string(), url: "/posts/aa".to_string() },
            Post { time: "2021-02-01".to_string(), name: "Post 2".to_string(), url: "/posts/bb".to_string() },
        ],
        projects: vec![
            Project { name: "Project 1".to_string(), url: "https://example.com/project1".to_string() },
            Project { name: "Project 2".to_string(), url: "https://example.com/project2".to_string() },
        ],
        friends: vec![
            Friend { name: "Friend 1".to_string(), url: "https://example.com/friend1".to_string() },
            Friend { name: "Friend 2".to_string(), url: "https://example.com/friend2".to_string() },
        ],
    };
    to_string_pretty(&blog_index).unwrap()
}

async fn generate_post(markdown_name: String) -> String {
    let file_path = std::path::PathBuf::from(std::env::var("BLOG_STATIC").unwrap())
        .as_path()
        .join("static-post")
        .join(markdown_name);
    let markdown_string = fs::read_to_string(file_path).unwrap();
    let markdown_html: String = markdown::to_html(markdown_string.as_str());
    let blog_post = PostTemplate {
        title: String::from("How LSPatch works"),
        time: "2024-04-01".to_string(),
        description: "While the LSPatch is amazing, there's no magic behind the technology. In this post, we will analyze it's source code, understand how it works and find ways to anagist it.".to_string(),
        markdown: markdown_html.to_string(),
    };
    to_string_pretty(&blog_post).unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Number of logical cores is {}", num_cpus::get());

    match std::env::var("BLOG_STATIC") {
        Ok(BLOG_STATIC) => {
            println!("BLOG_STATIC is {}", BLOG_STATIC);
        }
        Err(_) => {
            panic!("Pls set BLOG_STATIC")
        }
    }

    // fs::write("blog_index.toml", generate_index().await).unwrap();
    // fs::write("LSPatch.toml", generate_post("LSPatch.md".to_string()).await).unwrap();

    let toml_string_from_file = fs::read_to_string("blog_index.toml").unwrap();
    let deserialized_from_file: BlogIndexTemplate = from_str(&toml_string_from_file).unwrap();
    HttpServer::new(move || {
        let blog_scope = web::scope("")
            .route("/", web::get().to(sites::blog::index))
            .service(web::scope("posts")
                .service(posts))
            .service(web::scope("static")
                .service(image)
                .service(css)
                .service(js));

        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();

        App::new()
            .app_data(web::Data::new(BlogState::make(deserialized_from_file.clone())))
            .wrap(cors)
            .service(blog_scope)
    })
        .bind(("0.0.0.0", 8080))?
        .workers(4) // Why I select 4 in here? I think it is four core services that are used simultaneously, named blog, css, js, image.
        .keep_alive(None)
        .run()
        .await
}
