//! 工具函数模块
//!
//! 提供各种辅助函数和工具

use crate::error::{Error, Result};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub hostname: String,
    pub port: u16,
    pub path: String,
    pub is_https: bool,
}

/// 解析URL为主机和端口
pub fn parse_host_port(url: &str) -> Result<ParsedUrl> {
    let parsed_url = url
        .parse::<Url>()
        .map_err(|e| Error::url_parse(format!("parse_host_port error:{}", e)))?;

    let hostname = parsed_url.host_str().unwrap().to_string();
    let is_https = parsed_url.scheme() == "https";

    // 为HTTPS使用默认端口443，为HTTP使用默认端口80
    let port = parsed_url.port().unwrap_or(if is_https { 443 } else { 80 });

    Ok(ParsedUrl {
        hostname,
        port,
        path: parsed_url.path().to_string(),
        is_https,
    })
}

pub fn is_url_https(url: &str) -> bool {
    url.starts_with("https://")
}

/// 解析查询参数
pub fn parse_query_params(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    if query.is_empty() {
        return params;
    }

    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if let (Ok(key), Ok(value)) = (urlencoding::decode(key), urlencoding::decode(value)) {
                params.insert(key.to_string(), value.to_string());
            }
        }
    }

    params
}

/// 构建查询字符串
pub fn build_query_string(params: &HashMap<String, String>) -> String {
    if params.is_empty() {
        return String::new();
    }

    let mut pairs = Vec::new();
    for (key, value) in params {
        let encoded_key = urlencoding::encode(key);
        let encoded_value = urlencoding::encode(value);
        pairs.push(format!("{}={}", encoded_key, encoded_value));
    }

    format!("?{}", pairs.join("&"))
}

/// 检查是否为有效的HTTP状态码
pub fn is_valid_status_code(code: u16) -> bool {
    (100..=599).contains(&code)
}

/// 获取HTTP状态码描述
pub fn get_status_description(code: u16) -> &'static str {
    match code {
        100 => "Continue",
        101 => "Switching Protocols",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Unknown Status",
    }
}

/// 解析Content-Length头
pub fn parse_content_length(header_value: &str) -> Result<usize> {
    header_value
        .trim()
        .parse::<usize>()
        .map_err(|_| Error::http_parse(format!("Invalid Content-Length: {}", header_value)))
}

/// 检查URL是否为HTTPS
pub fn is_https(url: &str) -> bool {
    url.starts_with("https://")
}

/// 提取域名从URL
pub fn extract_domain(url: &str) -> Result<String> {
    let parsed_url = parse_host_port(url)?;
    Ok(parsed_url.hostname)
}

