//! HTTP客户端模块
//!
//! 提供HTTP客户端的功能，支持异步请求发送

pub mod builder;
pub mod model;

// 导出主要类型
pub use builder::ClientBuilder;
pub use model::HttpClient;
