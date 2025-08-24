//! HTTP请求构造模块
//!
//! 提供HTTP请求的构建、序列化和相关功能

pub mod types;
pub mod model;
pub mod builder;

// 导出主要类型
pub use types::{Method, Version};
pub use model::Request;
pub use builder::AsyncRequestBuilder;
