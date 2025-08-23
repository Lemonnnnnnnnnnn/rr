//! HTTP头处理模块
//!
//! 提供HTTP头的基本标准化、验证功能和兼容接口

use crate::error::{Error, Result};
use std::collections::HashMap;
use std::collections::hash_map::Iter;

/// 标准化HTTP头名称（转为小写，去除空格）
pub fn normalize_header_name(name: &str) -> String {
    name.to_lowercase().trim().to_string()
}

/// 标准化HTTP头值（去除前后空格）
pub fn normalize_header_value(value: &str) -> String {
    value.trim().to_string()
}

/// 验证HTTP头名称
pub fn validate_header_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::http_parse("Header name cannot be empty"));
    }

    // 检查是否包含非法字符
    for (i, ch) in name.chars().enumerate() {
        if ch.is_control() || ch == ' ' || ch == '\t' {
            return Err(Error::http_parse(format!(
                "Invalid character '{}' at position {} in header name",
                ch, i
            )));
        }
    }

    Ok(())
}

/// 验证HTTP头值
pub fn validate_header_value(value: &str) -> Result<()> {
    // HTTP头值可以包含控制字符，但不能以空格或制表符开始（除非是多行）
    if let Some(ch) = value.chars().next() {
        if ch == '\r' || ch == '\n' {
            return Err(Error::http_parse("Header value cannot start with CR or LF"));
        }
    }

    Ok(())
}

/// HTTP头构建器
#[derive(Debug, Clone)]
pub struct HeadersBuilder {
    headers: HashMap<String, String>,
}

impl HeadersBuilder {
    /// 创建新的头构建器
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    /// 添加头
    pub fn header<K, V>(mut self, key: K, value: V) -> Result<Self>
    where
        K: Into<String>,
        V: Into<String>,
    {
        let key = key.into();
        let value = value.into();

        validate_header_name(&key)?;
        validate_header_value(&value)?;

        self.headers.insert(normalize_header_name(&key), normalize_header_value(&value));
        Ok(self)
    }

    /// 添加多个头
    pub fn headers<K, V, I>(mut self, headers: I) -> Result<Self>
    where
        K: Into<String>,
        V: Into<String>,
        I: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in headers {
            self = self.header(key, value)?;
        }
        Ok(self)
    }

    /// 移除头
    pub fn remove<K: Into<String>>(mut self, key: K) -> Self {
        self.headers.remove(&normalize_header_name(&key.into()));
        self
    }

    /// 构建头HashMap
    pub fn build(self) -> HashMap<String, String> {
        self.headers
    }
}

impl Default for HeadersBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 常见的HTTP头常量
pub mod common_headers {
    pub const USER_AGENT: &str = "user-agent";
    pub const ACCEPT: &str = "accept";
    pub const ACCEPT_LANGUAGE: &str = "accept-language";
    pub const ACCEPT_ENCODING: &str = "accept-encoding";
    pub const CONTENT_TYPE: &str = "content-type";
    pub const CONTENT_LENGTH: &str = "content-length";
    pub const HOST: &str = "host";
    pub const CONNECTION: &str = "connection";
    pub const CACHE_CONTROL: &str = "cache-control";
    pub const AUTHORIZATION: &str = "authorization";
    pub const COOKIE: &str = "cookie";
    pub const REFERER: &str = "referer";
}

/// Content-Type 常量
pub mod content_types {
    pub const JSON: &str = "application/json";
    pub const JSON_UTF8: &str = "application/json; charset=utf-8";
    pub const FORM: &str = "application/x-www-form-urlencoded";
    pub const FORM_UTF8: &str = "application/x-www-form-urlencoded; charset=utf-8";
    pub const TEXT: &str = "text/plain";
    pub const TEXT_UTF8: &str = "text/plain; charset=utf-8";
    pub const HTML: &str = "text/html";
    pub const HTML_UTF8: &str = "text/html; charset=utf-8";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_normalization() {
        assert_eq!(normalize_header_name("Content-Type"), "content-type");
        assert_eq!(normalize_header_name(" CONTENT-LENGTH "), "content-length");
        assert_eq!(normalize_header_value(" text/html "), "text/html");
    }

    #[test]
    fn test_header_validation() {
        assert!(validate_header_name("Content-Type").is_ok());
        assert!(validate_header_name("").is_err());
        assert!(validate_header_name("Content\nType").is_err());

        assert!(validate_header_value("text/html").is_ok());
        assert!(validate_header_value("\r\nmalicious").is_err());
    }

    #[test]
    fn test_headers_builder() {
        let headers = HeadersBuilder::new()
            .header("Content-Type", "application/json")
            .unwrap()
            .header("Content-Length", "123")
            .unwrap()
            .build();

        assert_eq!(headers.get("content-type").unwrap(), "application/json");
        assert_eq!(headers.get("content-length").unwrap(), "123");
    }
}

// ===== HeaderMap 实现（兼容 reqwest::header::HeaderMap）=====

/// HTTP 头映射结构体
/// 提供与 reqwest::header::HeaderMap 类似的 API
#[derive(Debug, Clone, Default)]
pub struct HeaderMap {
    inner: HashMap<String, String>,
}

impl HeaderMap {
    /// 创建新的空的 HeaderMap
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// 插入头信息
    /// 返回之前的值（如果存在）
    pub fn insert<K, V>(&mut self, key: K, value: V) -> Result<Option<String>>
    where
        K: Into<String>,
        V: Into<String>,
    {
        let key = key.into();
        let value = value.into();

        // 验证头名称和值
        validate_header_name(&key)?;
        validate_header_value(&value)?;

        // 标准化键名（转为小写）
        let normalized_key = normalize_header_name(&key);

        Ok(self.inner.insert(normalized_key, value))
    }

    /// 获取头信息的值
    pub fn get(&self, key: &str) -> Option<&String> {
        let normalized_key = normalize_header_name(key);
        self.inner.get(&normalized_key)
    }

    /// 检查是否包含指定的头
    pub fn contains_key(&self, key: &str) -> bool {
        let normalized_key = normalize_header_name(key);
        self.inner.contains_key(&normalized_key)
    }

    /// 移除指定的头
    pub fn remove(&mut self, key: &str) -> Option<String> {
        let normalized_key = normalize_header_name(key);
        self.inner.remove(&normalized_key)
    }

    /// 获取迭代器
    pub fn iter(&self) -> HeaderMapIter<'_> {
        HeaderMapIter {
            inner: self.inner.iter(),
        }
    }

    /// 合并另一个 HeaderMap
    /// 如果存在相同的键，other 的值会覆盖当前值
    pub fn merge(&mut self, other: &HeaderMap) {
        for (key, value) in &other.inner {
            self.inner.insert(key.clone(), value.clone());
        }
    }

    /// 清空所有头信息
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// 获取头信息的数量
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 从现有的 HashMap 创建 HeaderMap
    pub fn from_hashmap(hashmap: HashMap<String, String>) -> Result<Self> {
        let mut header_map = Self::new();

        for (key, value) in hashmap {
            header_map.insert(key, value)?;
        }

        Ok(header_map)
    }

    /// 转换为 HashMap
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        self.inner.clone()
    }

    /// 获取内部 HashMap 的引用（用于迭代）
    pub fn inner(&self) -> &HashMap<String, String> {
        &self.inner
    }
}

/// HeaderMap 的迭代器
pub struct HeaderMapIter<'a> {
    inner: Iter<'a, String, String>,
}

impl<'a> Iterator for HeaderMapIter<'a> {
    type Item = (&'a String, &'a String);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod header_map_tests {
    use super::*;

    #[test]
    fn test_header_map_new() {
        let headers = HeaderMap::new();
        assert!(headers.is_empty());
        assert_eq!(headers.len(), 0);
    }

    #[test]
    fn test_header_map_insert_get() {
        let mut headers = HeaderMap::new();

        // 插入头信息
        headers.insert("Content-Type", "application/json").unwrap();
        headers.insert("User-Agent", "test-agent").unwrap();

        // 获取头信息（应该自动标准化为小写）
        assert_eq!(headers.get("content-type").unwrap(), "application/json");
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
        assert_eq!(headers.get("user-agent").unwrap(), "test-agent");
    }

    #[test]
    fn test_header_map_iter() {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json").unwrap();
        headers.insert("User-Agent", "test-agent").unwrap();

        let mut count = 0;
        for (key, value) in headers.iter() {
            match key.as_str() {
                "content-type" => assert_eq!(value, "application/json"),
                "user-agent" => assert_eq!(value, "test-agent"),
                _ => panic!("Unexpected header: {}", key),
            }
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_header_map_merge() {
        let mut headers1 = HeaderMap::new();
        headers1.insert("Content-Type", "application/json").unwrap();

        let mut headers2 = HeaderMap::new();
        headers2.insert("User-Agent", "test-agent").unwrap();
        headers2.insert("Content-Type", "text/html").unwrap(); // 应该覆盖

        headers1.merge(&headers2);

        assert_eq!(headers1.get("content-type").unwrap(), "text/html");
        assert_eq!(headers1.get("user-agent").unwrap(), "test-agent");
    }

    #[test]
    fn test_header_map_invalid_name() {
        let mut headers = HeaderMap::new();

        // 无效的头名称应该返回错误
        let result = headers.insert("Content\nType", "application/json");
        assert!(result.is_err());
    }

    #[test]
    fn test_header_map_invalid_value() {
        let mut headers = HeaderMap::new();

        // 无效的头值应该返回错误
        let result = headers.insert("Content-Type", "value\r\n");
        assert!(result.is_err());
    }
}
