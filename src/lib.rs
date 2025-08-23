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
//! - Automatic TLS/crypto provider initialization
//!
//! ## HTTPS Support
//!
//! rr automatically initializes the crypto provider for HTTPS requests.
//! No manual initialization is required.
//!
//! ## Example
//!
//! ```rust
//! use rr::HttpClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 创建带默认请求头的客户端
//!     let client = HttpClient::builder()
//!         .default_headers(headers)
//!         .build()?;
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
pub mod tls;

pub use client::{HttpClient, ClientBuilder};
pub use response::{Response, StatusCode};
pub use error::{Error, Result};
pub use connection::{AsyncConnection, AsyncHttpConnection, ProxyConfig, ProxyType, AsyncTlsManager, AsyncProxyConnection};
pub use request::AsyncRequestBuilder;
pub use headers::HeaderMap;
