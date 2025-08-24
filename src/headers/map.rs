//! HTTP头映射实现
//!
//! 提供HeaderMap结构体，兼容reqwest::header::HeaderMap的API

use crate::error::Result;
use crate::headers::constants::{validate_header_name, validate_header_value, normalize_header_name};
use std::collections::hash_map::Iter;
use std::collections::HashMap;

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
        self.inner.clear()
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

        // 无效的头值应该返回错误（以CR或LF开头）
        let result = headers.insert("Content-Type", "\r\nmalicious");
        assert!(result.is_err());
    }
}