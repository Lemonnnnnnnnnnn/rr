//! HTTP客户端模型
//!
//! 包含HttpClient结构体的定义和实现

use crate::error::Result;
use crate::request::{Method, Request, AsyncRequestBuilder};
use crate::utils::{parse_host_port, ParsedUrl};
use crate::connection::{AsyncConnection, AsyncHttpConnection, ProxyConfig};
use crate::response::Response;
use crate::headers::HeaderMap;

/// HTTP 客户端结构体
#[derive(Clone)]
pub struct HttpClient {
    pub(crate) proxy_config: Option<ProxyConfig>,
    pub(crate) default_headers: HeaderMap,
}

impl HttpClient {
    /// 创建新的 HTTP 客户端（不使用代理）
    pub fn new() -> Self {
        // 确保 crypto provider 已初始化
        let _ = crate::tls::init_crypto_provider();

        Self {
            proxy_config: None,
            default_headers: HeaderMap::new(),
        }
    }

    /// 获取客户端构建器
    pub fn builder() -> super::types::ClientBuilder {
        super::types::ClientBuilder::new()
    }

    /// 创建一个禁用浏览器请求头的客户端
    pub fn without_browser_headers() -> Self {
        // 确保 crypto provider 已初始化
        let _ = crate::tls::init_crypto_provider();

        Self {
            proxy_config: None,
            default_headers: HeaderMap::new(),
        }
    }

    /// 创建使用代理的HTTP客户端
    pub fn with_proxy(proxy_config: ProxyConfig) -> Self {
        // 确保 crypto provider 已初始化
        let _ = crate::tls::init_crypto_provider();

        Self {
            proxy_config: Some(proxy_config),
            default_headers: HeaderMap::new(),
        }
    }

    /// 发送 GET 请求
    pub fn get(&self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::GET, url, self)
    }

    /// 发送 POST 请求
    pub fn post(&self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::POST, url, self)
    }

    /// 发送 PUT 请求
    pub fn put(&self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::PUT, url, self)
    }

    /// 发送 DELETE 请求
    pub fn delete(&self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::DELETE, url, self)
    }

    /// 发送 HEAD 请求
    pub fn head(&self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::HEAD, url, self)
    }

    /// 发送请求（直接发送Request对象）
    pub async fn send_request(&self, mut request: Request) -> Result<Response> {
        // 合并默认请求头
        for (key, value) in self.default_headers.inner() {
            if !request.headers.contains_key(key) {
                request.headers.insert(key.clone(), value.clone());
            }
        }

        let parsed_url = parse_host_port(&request.url)?;

        // 创建连接
        let mut connection = self.create_connection(&parsed_url).await?;

        // 构建HTTP请求
        let request_str = request.serialize_to_string(&parsed_url)?;

        // 发送请求并获取响应
        let raw_response = connection.send_request(&request_str, &parsed_url).await?;

        // 将原始响应字节流解析为 Response 结构
        Response::from_raw_bytes(raw_response)
    }

    /// 创建连接
    async fn create_connection(&self, parsed_url: &ParsedUrl) -> Result<Box<dyn AsyncConnection>> {
        match &self.proxy_config {
            Some(config) => Ok(Box::new(AsyncHttpConnection::via_proxy(config.clone(), parsed_url).await?)),
            None => Ok(Box::new(AsyncHttpConnection::direct(parsed_url).await?)),
        }
    }
}
