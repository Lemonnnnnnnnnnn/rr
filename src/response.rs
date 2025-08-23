use std::collections::HashMap;
use std::fmt;
use crate::{error::Result, Error};

/// HTTP 响应结构体
#[derive(Debug, Clone)]
pub struct Response {
    /// HTTP 版本 (如 "HTTP/1.1")
    pub version: String,
    /// 状态码 (如 200, 404)
    pub status_code: u16,
    /// 状态消息 (如 "OK", "Not Found")
    pub status_message: String,
    /// 响应头部
    pub headers: HashMap<String, String>,
    /// 响应体
    pub body: String,
}

impl Response {
    /// 从原始 HTTP 响应字符串创建 Response 实例
    pub fn from_raw_response(raw_response: String) -> Result<Self> {
        let mut lines = raw_response.lines();
        let status_line = lines.next().ok_or(Error::Response("Empty response".to_string()))?;

        // 解析状态行: "HTTP/1.1 200 OK"
        let status_parts: Vec<&str> = status_line.split_whitespace().collect();
        if status_parts.len() < 3 {
            return Err(Error::Response("Invalid status line".to_string()));
        }

        let version = status_parts[0].to_string();
        let status_code: u16 = status_parts[1].parse().map_err(|_| Error::Response("Invalid status code".to_string()))?;
        let status_message = status_parts[2..].join(" ");

        // 解析头部
        let mut headers = HashMap::new();
        let mut body_start = false;
        let mut body_lines = Vec::new();

        for line in lines {
            if line.is_empty() {
                // 空行表示头部结束，接下来是响应体
                body_start = true;
                continue;
            }

            if body_start {
                body_lines.push(line);
            } else {
                // 解析头部行: "Content-Type: text/html"
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim().to_lowercase();
                    let value = value.trim().to_string();
                    headers.insert(key, value);
                }
            }
        }

        let body = body_lines.join("\n");

        Ok(Response {
            version,
            status_code,
            status_message,
            headers,
            body,
        })
    }

    /// 获取指定头部的值
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(&key.to_lowercase())
    }

    /// 检查是否为成功的响应 (状态码 200-299)
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    /// 检查是否为重定向响应 (状态码 300-399)
    pub fn is_redirect(&self) -> bool {
        self.status_code >= 300 && self.status_code < 400
    }

    /// 检查是否为客户端错误 (状态码 400-499)
    pub fn is_client_error(&self) -> bool {
        self.status_code >= 400 && self.status_code < 500
    }

    /// 检查是否为服务器错误 (状态码 500-599)
    pub fn is_server_error(&self) -> bool {
        self.status_code >= 500 && self.status_code < 600
    }

    /// 获取响应的完整状态行
    pub fn status_line(&self) -> String {
        format!("{} {} {}", self.version, self.status_code, self.status_message)
    }

    /// 获取内容长度
    pub fn content_length(&self) -> Option<usize> {
        self.get_header("content-length")
            .and_then(|s| s.parse().ok())
    }

    /// 获取内容类型
    pub fn content_type(&self) -> Option<&String> {
        self.get_header("content-type")
    }

    /// 获取响应的原始字符串表示
    pub fn to_raw_string(&self) -> String {
        let mut raw = format!("{} {} {}\r\n", self.version, self.status_code, self.status_message);

        for (key, value) in &self.headers {
            raw.push_str(&format!("{}: {}\r\n", capitalize_header(key), value));
        }

        raw.push_str("\r\n");
        raw.push_str(&self.body);

        raw
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HTTP响应: {} {} {}\n状态: {}\n内容长度: {} 字节\n内容类型: {}\n响应体长度: {} 字符",
            self.version,
            self.status_code,
            self.status_message,
            if self.is_success() { "成功" } else if self.is_client_error() { "客户端错误" } else if self.is_server_error() { "服务器错误" } else if self.is_redirect() { "重定向" } else { "未知" },
            self.content_length().unwrap_or(0),
            self.content_type().unwrap_or(&"未知".to_string()),
            self.body.len()
        )
    }
}

/// 将头部键转换为首字母大写的格式
fn capitalize_header(key: &str) -> String {
    key.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars.as_str().chars()).collect(),
            }
        })
        .collect::<Vec<String>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_response() {
        let raw = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 12\r\n\r\nHello World!".to_string();

        let response = Response::from_raw_response(raw).unwrap();

        assert_eq!(response.version, "HTTP/1.1");
        assert_eq!(response.status_code, 200);
        assert_eq!(response.status_message, "OK");
        assert_eq!(response.get_header("content-type").unwrap(), "text/plain");
        assert_eq!(response.get_header("content-length").unwrap(), "12");
        assert_eq!(response.body, "Hello World!");
        assert!(response.is_success());
    }

    #[test]
    fn test_response_methods() {
        let raw = "HTTP/1.1 404 Not Found\r\nContent-Type: text/html\r\n\r\nNot Found".to_string();
        let response = Response::from_raw_response(raw).unwrap();

        assert_eq!(response.status_code, 404);
        assert!(!response.is_success());
        assert!(response.is_client_error());
        assert_eq!(response.status_line(), "HTTP/1.1 404 Not Found");
    }
}
