//! HTTP请求模型
//!
//! 包含Request结构体的定义和实现

use std::collections::HashMap;
use bytes::Bytes;
use crate::error::{Error, Result};
use crate::utils::{extract_domain, parse_host_port};

use super::types::{Method, Version};

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
        let parsed_url = parse_host_port(&self.url)?;
        let request_str = self.serialize_to_string(&parsed_url)?;
        Ok(request_str.into_bytes())
    }

    /// 序列化请求为字符串
    pub fn serialize_to_string(&self, parsed_url: &crate::utils::ParsedUrl) -> Result<String> {
        let mut request_str = format!(
            "{} {} {}\r\n",
            self.method.as_str(),
            parsed_url.path,
            self.version.as_str()
        );

        // 添加Host头
        request_str.push_str(&format!("Host: {}\r\n", parsed_url.hostname));

        // 添加其他请求头
        for (key, value) in &self.headers {
            request_str.push_str(&format!("{}: {}\r\n", key, value));
        }

        // 添加Connection头
        request_str.push_str("Connection: close\r\n");

        // 添加请求体（如果有）
        if let Some(body) = &self.body {
            request_str.push_str(&format!("Content-Length: {}\r\n", body.len()));
            request_str.push_str("\r\n");
            request_str.push_str(&String::from_utf8_lossy(body));
        } else {
            request_str.push_str("\r\n");
        }

        Ok(request_str)
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
