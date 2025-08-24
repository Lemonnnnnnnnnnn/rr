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
    pub full_path: String,
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

    // 构建完整的路径，包括查询参数
    let path = parsed_url.path().to_string();
    let mut full_path = path.clone();
    if let Some(query) = parsed_url.query() {
        full_path.push('?');
        full_path.push_str(query);
    }

    Ok(ParsedUrl {
        hostname,
        port,
        path,
        full_path,
        is_https,
    })
}
