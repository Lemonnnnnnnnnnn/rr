//! HTTP请求相关类型定义
//!
//! 包含HTTP方法、版本等基础类型



/// HTTP方法枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
    TRACE,
}

impl Method {
    /// 将方法转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::HEAD => "HEAD",
            Method::OPTIONS => "OPTIONS",
            Method::PATCH => "PATCH",
            Method::TRACE => "TRACE",
        }
    }
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            "PATCH" => Method::PATCH,
            "TRACE" => Method::TRACE,
            _ => Method::GET,
        }
    }
}

/// HTTP版本枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    Http1_0,
    Http1_1,
}

impl Version {
    pub fn as_str(&self) -> &'static str {
        match self {
            Version::Http1_0 => "HTTP/1.0",
            Version::Http1_1 => "HTTP/1.1",
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Version::Http1_1
    }
}
