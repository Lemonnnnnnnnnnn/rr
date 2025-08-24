use std::collections::HashMap;
use std::fmt;
use crate::{error::Result, Error};
use crate::decompression::{Compression, decompress};

/// HTTP 状态码结构体（兼容 reqwest::StatusCode）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode {
    pub code: u16,
}

impl StatusCode {
    /// 检查是否为成功状态码 (200-299)
    pub fn is_success(&self) -> bool {
        self.code >= 200 && self.code < 300
    }

    /// 检查是否为重定向状态码 (300-399)
    pub fn is_redirection(&self) -> bool {
        self.code >= 300 && self.code < 400
    }

    /// 检查是否为客户端错误状态码 (400-499)
    pub fn is_client_error(&self) -> bool {
        self.code >= 400 && self.code < 500
    }

    /// 检查是否为服务器错误状态码 (500-599)
    pub fn is_server_error(&self) -> bool {
        self.code >= 500 && self.code < 600
    }

    /// 获取状态码数值
    pub fn as_u16(&self) -> u16 {
        self.code
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

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
    /// 响应体 (原始字节数据)
    pub body: Vec<u8>,
}

impl Response {
    /// 从原始 HTTP 响应字节流创建 Response 实例
    pub fn from_raw_bytes(raw_response: Vec<u8>) -> Result<Self> {
        // 首先找到头部结束的位置（\r\n\r\n）
        let header_end = raw_response.windows(4).position(|w| w == b"\r\n\r\n")
            .ok_or(Error::Response("Invalid HTTP response format".to_string()))?;

        // 分离头部和响应体
        let header_bytes = &raw_response[..header_end];
        let body_bytes = &raw_response[header_end + 4..];

        // 解析头部
        let header_str = String::from_utf8_lossy(header_bytes);
        let mut lines = header_str.lines();

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
        for line in lines {
            if line.is_empty() {
                break;
            }
            // 解析头部行: "Content-Type: text/html"
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim().to_string();
                headers.insert(key, value);
            }
        }

        // 检查content-encoding头部并解压缩响应体
        let content_encoding = headers.get("content-encoding")
            .map(|v| v.as_str())
            .unwrap_or("");

        let compression = Compression::from_content_encoding(content_encoding);

        // 处理响应体
        let processed_body = if compression != Compression::None {
            decompress(body_bytes, compression)?
        } else {
            body_bytes.to_vec()
        };

        Ok(Response {
            version,
            status_code,
            status_message,
            headers,
            body: processed_body,
        })
    }

    /// 从原始 HTTP 响应字符串创建 Response 实例（向后兼容）
    pub fn from_raw_response(raw_response: String) -> Result<Self> {
        Self::from_raw_bytes(raw_response.into_bytes())
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

    /// 获取状态信息（兼容 reqwest::Response::status()）
    pub fn status(&self) -> StatusCode {
        StatusCode {
            code: self.status_code,
        }
    }

    /// 获取响应体文本（兼容 reqwest::Response::text()）
    pub async fn text(self) -> Result<String> {
        String::from_utf8(self.body).map_err(|e| Error::other(format!("Invalid UTF-8: {}", e)))
    }

    /// 获取响应体的字节流（兼容 reqwest::Response::bytes_stream()）
    pub fn bytes_stream(self) -> impl futures_util::Stream<Item = Result<Vec<u8>>> {
        use futures_util::stream;

        // 将已有的响应体数据转换为字节流
        let chunks = self.body.chunks(8192).map(|chunk| Ok(chunk.to_vec())).collect::<Vec<_>>();

        stream::iter(chunks)
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
        raw.push_str(&String::from_utf8_lossy(&self.body));

        raw
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HTTP响应: {} {} {}\n状态: {}\n内容长度: {} 字节\n内容类型: {}\n响应体长度: {} 字节",
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
        assert_eq!(String::from_utf8(response.body.clone()).unwrap(), "Hello World!");
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

    #[test]
    fn test_binary_response_body() {
        // 模拟二进制数据（包含非UTF-8字节）
        let binary_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0xFF, 0xFE, 0x00, 0x01];
        let raw = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\n\r\n",
            binary_data.len()
        );
        let mut raw_bytes = raw.into_bytes();
        raw_bytes.extend(binary_data.clone());

        let response = Response::from_raw_bytes(raw_bytes).unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, binary_data);
        assert!(response.is_success());
        assert_eq!(response.get_header("content-type").unwrap(), "image/png");

        // 验证String::from_utf8会失败，因为这不是有效的UTF-8
        let text_result = String::from_utf8(response.body.clone());
        assert!(text_result.is_err());
    }
}
