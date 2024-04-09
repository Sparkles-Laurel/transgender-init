use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Scope {
    pub name: String,
    pub start: Instant,
    pub duration: Option<Duration>,
    pub level: usize,
}
