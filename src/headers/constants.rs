//! HTTP头常量和基础函数
//!
//! 包含HTTP头的标准化、验证函数和常用常量

use crate::error::{Error, Result};

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
}
