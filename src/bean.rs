use serde::Deserialize;
use std::{
    sync::atomic::{AtomicUsize},
    sync::Arc,
};

#[derive(Clone)]
pub(crate) struct BlogState {
    pub(crate) visit_count: Arc<AtomicUsize>,
}

#[derive(Deserialize)]
pub(crate) struct Article {
    pub(crate) name: String,
    pub(crate) section: String,
}