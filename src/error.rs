//! 错误处理模块
//!
//! 定义了库中使用的错误类型和Result类型别名

use std::io;
use thiserror::Error;

/// 库中使用的Result类型别名
pub type Result<T> = std::result::Result<T, Error>;

/// 主要错误类型
#[derive(Error, Debug)]
pub enum Error {
    /// I/O错误
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// URL解析错误
    #[error("URL parse error: {0}")]
    UrlParse(String),

    /// HTTP解析错误
    #[error("HTTP parse error: {0}")]
    HttpParse(String),

    /// TLS错误
    #[error("TLS error: {0}")]
    Tls(String),

    /// 连接错误
    #[error("Connection error: {0}")]
    Connection(String),

    /// 超时错误
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// 代理错误
    #[error("Proxy error: {0}")]
    Proxy(String),

    /// 状态码错误
    #[error("HTTP error: {status} - {message}")]
    Http {
        status: u16,
        message: String,
    },

    /// 其他错误
    #[error("Other error: {0}")]
    Other(String),

    /// 响应错误
    #[error("Response error: {0}")]
    Response(String),

    /// 解压缩错误
    #[error("Decompression error: {0}")]
    Decompression(String),
}

impl Error {
    /// 创建URL解析错误
    pub fn url_parse<S: Into<String>>(msg: S) -> Self {
        Error::UrlParse(msg.into())
    }

    /// 创建HTTP解析错误
    pub fn http_parse<S: Into<String>>(msg: S) -> Self {
        Error::HttpParse(msg.into())
    }

    /// 创建TLS错误
    pub fn tls<S: Into<String>>(msg: S) -> Self {
        Error::Tls(msg.into())
    }

    /// 创建连接错误
    pub fn connection<S: Into<String>>(msg: S) -> Self {
        Error::Connection(msg.into())
    }

    /// 创建超时错误
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Error::Timeout(msg.into())
    }

    /// 创建代理错误
    pub fn proxy<S: Into<String>>(msg: S) -> Self {
        Error::Proxy(msg.into())
    }

    /// 创建HTTP状态错误
    pub fn http_status(status: u16, message: String) -> Self {
        Error::Http { status, message }
    }

    /// 创建其他错误
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Error::Other(msg.into())
    }

    /// 创建响应错误
    pub fn response<S: Into<String>>(msg: S) -> Self {
        Error::Response(msg.into())
    }

    /// 创建解压缩错误
    pub fn decompression<S: Into<String>>(msg: S) -> Self {
        Error::Decompression(msg.into())
    }
}
