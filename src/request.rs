//! HTTP请求构造模块
//!
//! 提供HTTP请求的构建、序列化和相关功能

use crate::error::{Error, Result};
use crate::utils::{extract_domain, parse_host_port};
use crate::{HttpClient, Response};
use bytes::Bytes;
use std::collections::HashMap;

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

/// HTTP请求结构体
#[derive(Debug, Clone)]
pub struct Request {
    /// 请求方法
    pub method: Method,
    /// 请求URL
    pub url: String,
    /// HTTP版本
    pub version: Version,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 请求体
    pub body: Option<Bytes>,
}

impl Request {
    /// 创建新的GET请求
    pub fn get(url: &str) -> Self {
        Self::new(Method::GET, url)
    }

    /// 创建新的POST请求
    pub fn post(url: &str) -> Self {
        Self::new(Method::POST, url)
    }

    /// 创建新的PUT请求
    pub fn put(url: &str) -> Self {
        Self::new(Method::PUT, url)
    }

    /// 创建新的DELETE请求
    pub fn delete(url: &str) -> Self {
        Self::new(Method::DELETE, url)
    }

    /// 创建新的请求
    pub fn new(method: Method, url: &str) -> Self {
        let mut headers = HashMap::new();

        // 设置默认请求头
        headers.insert(
            "User-Agent".to_string(),
            "rust-my-request/0.1.0".to_string(),
        );
        headers.insert("Accept".to_string(), "*/*".to_string());
        headers.insert("Connection".to_string(), "close".to_string());

        Self {
            method,
            url: url.to_string(),
            version: Version::default(),
            headers,
            body: None,
        }
    }

    /// 设置请求头
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// 设置多个请求头
    pub fn headers<K, V, I>(mut self, headers: I) -> Self
    where
        K: Into<String>,
        V: Into<String>,
        I: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in headers {
            self.headers.insert(key.into(), value.into());
        }
        self
    }

    /// 设置请求体
    pub fn body<B: Into<Bytes>>(mut self, body: B) -> Self {
        let body = body.into();
        self.body = Some(body.clone());

        // 如果设置了请求体，自动设置Content-Length
        if !self.headers.contains_key("Content-Length") {
            self.headers
                .insert("Content-Length".to_string(), body.len().to_string());
        }

        self
    }

    /// 设置JSON请求体
    pub fn json<T: serde::Serialize>(self, _data: &T) -> Result<Self> {
        // TODO: 添加serde_json依赖后实现
        Err(Error::other(
            "JSON serialization not implemented yet. Add serde_json dependency.",
        ))
    }

    /// 设置表单数据请求体
    pub fn form<T: serde::Serialize>(self, _data: &T) -> Result<Self> {
        // TODO: 添加serde_urlencoded依赖后实现
        Err(Error::other(
            "Form serialization not implemented yet. Add serde_urlencoded dependency.",
        ))
    }

    /// 获取域名
    pub fn domain(&self) -> Result<String> {
        extract_domain(&self.url)
    }

    /// 序列化请求为字节流
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // 请求行 - 使用路径而不是完整URL
        // let path = extract_path(&self.url)?;
        let parsed_url = parse_host_port(&self.url)?;
        let path = parsed_url.path;
        let request_line = format!(
            "{} {} {}\r\n",
            self.method.as_str(),
            path,
            self.version.as_str()
        );
        buffer.extend_from_slice(request_line.as_bytes());

        // 请求头
        for (key, value) in &self.headers {
            let header_line = format!("{}: {}\r\n", key, value);
            buffer.extend_from_slice(header_line.as_bytes());
        }

        // 空行分隔
        buffer.extend_from_slice(b"\r\n");

        // 请求体
        if let Some(body) = &self.body {
            buffer.extend_from_slice(body);
        }

        Ok(buffer)
    }

    /// 获取请求体的长度
    pub fn content_length(&self) -> usize {
        self.body.as_ref().map(|b| b.len()).unwrap_or(0)
    }

}

impl Default for Request {
    fn default() -> Self {
        Self::new(Method::GET, "http://example.com")
    }
}

/// 异步请求构建器模式
pub struct AsyncRequestBuilder<'a> {
    request: Request,
    client: &'a HttpClient,
}

impl<'a> AsyncRequestBuilder<'a> {
    /// 创建新的异步请求构建器
    pub fn new(method: Method, url: &str, client: &'a HttpClient) -> Self {
        Self {
            request: Request::new(method, url),
            client: client,
        }
    }

    /// 设置请求头
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.request = self.request.header(key, value);
        self
    }

    /// 设置请求体
    pub fn body<B: Into<Bytes>>(mut self, body: B) -> Self {
        self.request = self.request.body(body);
        self
    }

    /// 构建请求
    pub fn build(self) -> Request {
        self.request
    }

    /// 异步发送请求
    pub async fn send(self) -> Result<Response> {
        self.client.execute(self.request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let req = Request::get("http://example.com");
        assert_eq!(req.method, Method::GET);
        assert_eq!(req.url, "http://example.com");
        assert!(req.headers.contains_key("User-Agent"));
    }

    #[test]
    fn test_request_with_body() {
        let req = Request::post("http://example.com").body("test body");
        assert_eq!(req.content_length(), 9);
        assert!(req.headers.contains_key("Content-Length"));
    }

    #[test]
    fn test_method_from_str() {
        assert_eq!(Method::from("GET"), Method::GET);
        assert_eq!(Method::from("POST"), Method::POST);
        assert_eq!(Method::from("post"), Method::POST);
    }

    #[test]
    fn test_request_serialization_with_path() {
        let req = Request::get("http://example.com/path");
        let serialized = req.serialize().unwrap();
        let serialized_str = String::from_utf8_lossy(&serialized);

        // 检查请求行是否使用路径而不是完整URL
        assert!(serialized_str.starts_with("GET /path HTTP/1.1\r\n"));
        assert!(!serialized_str.contains("GET http://example.com/path HTTP/1.1"));
    }

    #[test]
    fn test_request_serialization_root_path() {
        let req = Request::get("http://example.com");
        let serialized = req.serialize().unwrap();
        let serialized_str = String::from_utf8_lossy(&serialized);

        // 检查根路径请求
        assert!(serialized_str.starts_with("GET / HTTP/1.1\r\n"));
        assert!(!serialized_str.contains("GET http://example.com HTTP/1.1"));
    }

    #[test]
    fn test_request_serialization_with_query() {
        let req = Request::get("http://example.com/path?query=value");
        let serialized = req.serialize().unwrap();
        let serialized_str = String::from_utf8_lossy(&serialized);

        // 检查查询参数被正确剥离，只保留路径
        assert!(serialized_str.starts_with("GET /path HTTP/1.1\r\n"));
        assert!(!serialized_str.contains("query=value"));
    }

    #[test]
    fn test_http_request_format_validation() {
        let req = Request::get("http://example.com/test/path?param=value");
        let serialized = req.serialize().unwrap();
        let serialized_str = String::from_utf8_lossy(&serialized);

        // 验证HTTP/1.1请求格式的正确性
        let lines: Vec<&str> = serialized_str.split("\r\n").collect();

        // 第一行应该是请求行，格式为: METHOD path HTTP/version
        assert!(lines[0].starts_with("GET /test/path HTTP/1.1"));
        assert!(!lines[0].contains("http://example.com")); // 不应该包含完整URL

        // 空行分隔请求头和请求体
        assert!(lines.contains(&""));

        // 验证请求行格式正确
        assert_eq!(lines[0], "GET /test/path HTTP/1.1");
    }
}
