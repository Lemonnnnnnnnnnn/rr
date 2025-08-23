use crate::error::Result;
use crate::request::{Method, Request, AsyncRequestBuilder};
use crate::utils::{parse_host_port, ParsedUrl};
use crate::connection::{AsyncConnection, AsyncHttpConnection, ProxyConfig};
use crate::response::Response;

/// HTTP 客户端结构体
pub struct HttpClient {
    proxy_config: Option<ProxyConfig>,
}

impl HttpClient {
    /// 创建新的 HTTP 客户端（不使用代理）
    pub fn new() -> Self {
        Self {
            proxy_config: None,
        }
    }

    /// 创建使用代理的HTTP客户端
    pub fn with_proxy(proxy_config: ProxyConfig) -> Self {
        Self {
            proxy_config: Some(proxy_config),
        }
    }


    /// 发送 GET 请求
    pub fn get(&mut self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::GET, url, self)
    }

    /// 发送 POST 请求
    pub fn post(&mut self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::POST, url, self)
    }

    /// 发送 PUT 请求
    pub fn put(&mut self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::PUT, url, self)
    }

    /// 发送 DELETE 请求
    pub fn delete(&mut self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::DELETE, url, self)
    }

    /// 异步执行请求
    pub async fn execute(&self, request: Request) -> Result<Response> {
        let parsed_url = parse_host_port(&request.url)?;

        // 创建连接
        let mut connection = self.create_connection(&parsed_url).await?;

        // 构建HTTP请求
        let request_str = self.build_request_string(&request, &parsed_url)?;

        // 发送请求并获取响应
        let raw_response = connection.send_request(&request_str, &parsed_url).await?;

        // 将原始响应字符串解析为 Response 结构
        Response::from_raw_response(raw_response)
    }

    /// 创建连接
    async fn create_connection(&self, parsed_url: &ParsedUrl) -> Result<Box<dyn AsyncConnection>> {
        match &self.proxy_config {
            Some(config) => Ok(Box::new(AsyncHttpConnection::via_proxy(config.clone(), parsed_url).await?)),
            None => Ok(Box::new(AsyncHttpConnection::direct(parsed_url).await?)),
        }
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

