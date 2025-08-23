//! 连接管理模块
//!
//! 包含连接抽象、TLS 支持和代理连接功能

pub mod connection;
pub mod tls;
pub mod proxy;

pub use connection::{AsyncConnection, AsyncHttpConnection};
pub use tls::AsyncTlsManager;
pub use proxy::{ProxyConfig, ProxyType, AsyncProxyConnection};

