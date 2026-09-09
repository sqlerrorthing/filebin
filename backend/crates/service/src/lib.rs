#![feature(min_specialization)]

pub mod error;

pub use service_macros::service;
pub use service_macros::map;
pub use auto_impl;
pub use async_trait;
