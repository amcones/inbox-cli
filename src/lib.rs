pub mod cli;
pub mod deletion;
pub mod i18n;
pub mod markdown;
pub mod model;
pub mod query;
pub mod ranking;
pub mod storage;
pub mod views;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
