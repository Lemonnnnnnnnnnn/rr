//! 连接管理模块
//!
//! 包含连接抽象、TLS 支持和代理连接功能

pub mod connection;
pub mod tls;
pub mod proxy;

// 重新导出主要类型（保持向后兼容）
pub use connection::{AsyncConnection, AsyncHttpConnection};
pub use tls::AsyncTlsManager;
pub use proxy::{ProxyConfig, ProxyType, AsyncProxyConnection};

// 注意：原有的同步类型已不再导出，建议使用异步版本
