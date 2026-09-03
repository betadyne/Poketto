pub mod db;
pub mod discord;
pub mod models;
pub mod process;
pub mod vndb;
pub mod wine;
mod error;

pub use error::{AppError, AppResult};
