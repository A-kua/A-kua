use serde::Deserialize;
use std::{
    sync::atomic::{AtomicUsize},
    sync::Arc,
};
use std::sync::atomic::Ordering;
use crate::structs::template::{BlogIndexTemplate};

#[derive(Clone)]
pub(crate) struct BlogState {
    visit_count: Arc<AtomicUsize>,
    pub(crate) blog_index: BlogIndexTemplate,
}

impl BlogState {
    pub(crate) fn make(blog_index: BlogIndexTemplate) -> BlogState {
        return BlogState {
            visit_count: Arc::new(AtomicUsize::new(0)),
            blog_index,
        };
    }
    pub(crate) fn get_visit_count(&self) -> usize {
        self.visit_count.fetch_add(1, Ordering::Relaxed);
        return self.visit_count.load(Ordering::Relaxed);
    }
}

#[derive(Deserialize)]
pub(crate) struct Post {
    pub(crate) name: String,
    pub(crate) section: String,
}