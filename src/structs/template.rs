use askama::Template;
use toml::{to_string_pretty, from_str};
use serde::{Serialize, Deserialize};

#[derive(Template)]
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[template(path = "blog_index.html")]
pub struct BlogIndexTemplate {
    pub(crate) title: String,
    pub(crate) motto: String,

    pub(crate) extra_js: String,

    pub(crate) abouts: Vec<String>,
    pub(crate) posts: Vec<Post>,
    pub(crate) projects: Vec<Project>,
    pub(crate) friends: Vec<Friend>,
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct Post {
    pub(crate) time: String,
    pub(crate) name: String,
    pub(crate) url: String,
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct Project {
    pub(crate) name: String,
    pub(crate) url: String,
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct Friend {
    pub(crate) name: String,
    pub(crate) url: String,
}