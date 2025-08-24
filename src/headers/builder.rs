//! HTTP头构建器
//!
//! 提供HeadersBuilder结构体用于构建HTTP头

use crate::error::Result;
use crate::headers::constants::{validate_header_name, validate_header_value, normalize_header_name, normalize_header_value};
use std::collections::HashMap;

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

#[cfg(test)]
mod tests {
    use super::*;

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
