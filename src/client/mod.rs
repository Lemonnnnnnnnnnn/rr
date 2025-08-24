//! HTTP客户端模块
//!
//! 提供HTTP客户端的功能，支持异步请求发送

pub mod types;
pub mod model;

// 导出主要类型
pub use types::ClientBuilder;
pub use model::HttpClient;
