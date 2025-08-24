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

/// 浏览器请求头预设
pub mod browser_headers {
    use std::collections::HashMap;

    /// Chrome 浏览器请求头
    pub fn chrome() -> HashMap<String, String> {
        let mut headers = HashMap::new();

        // 基础浏览器请求头
        headers.insert(
            "User-Agent".to_string(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()
        );
        headers.insert(
            "Accept".to_string(),
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string()
        );
        headers.insert(
            "Accept-Language".to_string(),
            "zh-CN,zh;q=0.9,en;q=0.8".to_string()
        );
        headers.insert(
            "Accept-Encoding".to_string(),
            "gzip, deflate, br".to_string()
        );

        // 安全和隐私相关的头
        headers.insert("DNT".to_string(), "1".to_string());
        headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());

        // 客户端提示（Client Hints）
        headers.insert(
            "Sec-Ch-Ua".to_string(),
            "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"".to_string()
        );
        headers.insert("Sec-Ch-Ua-Mobile".to_string(), "?0".to_string());
        headers.insert("Sec-Ch-Ua-Platform".to_string(), "\"Windows\"".to_string());

        // Fetch Metadata
        headers.insert("Sec-Fetch-Dest".to_string(), "document".to_string());
        headers.insert("Sec-Fetch-Mode".to_string(), "navigate".to_string());
        headers.insert("Sec-Fetch-Site".to_string(), "none".to_string());
        headers.insert("Sec-Fetch-User".to_string(), "?1".to_string());

        headers
    }

    /// 获取浏览器的用户代理字符串
    pub mod user_agents {
        pub const CHROME_WINDOWS: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        pub const CHROME_MAC: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        pub const CHROME_LINUX: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        pub const CHROME_MOBILE: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/120.0.0.0 Mobile/15E148 Safari/604.1";
    }
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
