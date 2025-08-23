use crate::error::Result;
use crate::request::{Method, Request, RequestBuilder};
use crate::utils::{parse_host_port, ParsedUrl};
use crate::connection::{ConnectionManager, ConnectionType};

use crate::proxy::ProxyConfig;
use crate::response::Response;

/// HTTP 客户端结构体
pub struct HttpClient {
    connection_manager: ConnectionManager,
}

impl HttpClient {
    /// 创建新的 HTTP 客户端（不使用代理）
    pub fn new() -> Result<Self> {
        let connection_manager = ConnectionManager::new(ConnectionType::Direct);
        Ok(HttpClient {
            connection_manager,
        })
    }

    /// 创建使用代理的HTTP客户端
    pub fn with_proxy(proxy: ProxyConfig) -> Result<Self> {
        let connection_manager = ConnectionManager::new(ConnectionType::Proxy(proxy));
        Ok(HttpClient {
            connection_manager,
        })
    }

    /// 发送 GET 请求
    pub fn get(&mut self, url: &str) -> RequestBuilder {
        RequestBuilder::new(Method::GET, url, self)
    }

    pub fn execute(&self, request: Request) -> Result<Response> {
        let parsed_url = parse_host_port(&request.url)?;

        // 创建连接
        let mut connection = self.connection_manager.create_connection(&parsed_url)?;

        // 构建HTTP请求
        let request_str = self.build_request_string(&request, &parsed_url)?;

        // 发送请求并获取响应
        let raw_response = connection.send_request(&request_str, &parsed_url)?;

        // 将原始响应字符串解析为 Response 结构
        Response::from_raw_response(raw_response)
    }

    /// 构建HTTP请求字符串
    fn build_request_string(&self, request: &Request, parsed_url: &ParsedUrl) -> Result<String> {
        let mut request_str = format!(
            "{} {} {}\r\n",
            request.method.as_str(),
            parsed_url.path,
            request.version.as_str()
        );

        // 添加Host头
        request_str.push_str(&format!("Host: {}\r\n", parsed_url.hostname));

        // 添加其他请求头
        for (key, value) in &request.headers {
            request_str.push_str(&format!("{}: {}\r\n", key, value));
        }

        // 添加Connection头
        request_str.push_str("Connection: close\r\n");

        // 添加请求体（如果有）
        if let Some(body) = &request.body {
            request_str.push_str(&format!("Content-Length: {}\r\n", body.len()));
            request_str.push_str("\r\n");
            request_str.push_str(&String::from_utf8_lossy(body));
        } else {
            request_str.push_str("\r\n");
        }

        Ok(request_str)
    }
}

