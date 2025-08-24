//! 工具函数模块
//!
//! 提供各种辅助函数和工具

use crate::error::{Error, Result};
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
