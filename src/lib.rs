//! # rr
//!
//! A simple HTTP client library in Rust.
//!
//! ## Features
//!
//! - Simple and easy-to-use HTTP client
//! - Support for custom headers at client level
//! - Proxy support
//! - Async/await support
//!
//! ## Example
//!
//! ```rust
//! use rt_1::HttpClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 创建带默认请求头的客户端
//!     let mut client = HttpClient::new()
//!         .default_header("User-Agent", "MyApp/1.0")
//!         .default_header("Authorization", "Bearer token123");
//!
//!     // 发送请求（会自动包含默认请求头）
//!     let response = client.get("https://httpbin.org/get").send().await?;
//!     println!("Status: {}", response.status_code);
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod response;
pub mod error;
pub mod request;
pub mod utils;
pub mod connection;
pub mod headers;

pub use client::HttpClient;
pub use response::Response;
pub use error::{Error, Result};
pub use connection::{AsyncConnection, AsyncHttpConnection, ProxyConfig, ProxyType, AsyncTlsManager, AsyncProxyConnection};
pub use request::AsyncRequestBuilder;
