//! 连接管理模块
//!
//! 包含连接抽象、TLS 支持和代理连接功能

pub mod connection;
pub mod tls;
pub mod proxy;

// 重新导出主要类型
pub use connection::{Connection, HttpConnection};
pub use tls::TlsManager;
pub use proxy::{ProxyConfig, ProxyType, ProxyConnection};
