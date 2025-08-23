use crate::error::Result;
use crate::request::{Method, Request, AsyncRequestBuilder};
use crate::utils::{parse_host_port, ParsedUrl};
use crate::connection::{AsyncConnection, AsyncHttpConnection, ProxyConfig};
use crate::response::Response;
use crate::headers::HeaderMap;
use std::time::Duration;

/// HTTP 客户端构建器
/// 支持链式构建，类似 reqwest::Client::builder()
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    proxy_config: Option<ProxyConfig>,
    default_headers: HeaderMap,
    timeout: Duration,
}

impl ClientBuilder {
    /// 创建新的客户端构建器
    pub fn new() -> Self {
        Self {
            proxy_config: None,
            default_headers: HeaderMap::new(),
            timeout: Duration::from_secs(30),
        }
    }

    /// 设置默认请求头
    pub fn default_headers(mut self, headers: HeaderMap) -> Self {
        self.default_headers = headers;
        self
    }

    /// 设置代理配置
    pub fn proxy(mut self, config: ProxyConfig) -> Self {
        self.proxy_config = Some(config);
        self
    }

    /// 设置超时时间
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 构建 HTTP 客户端
    pub fn build(self) -> Result<HttpClient> {
        // 确保 crypto provider 已初始化
        crate::tls::init_crypto_provider()?;

        Ok(HttpClient {
            proxy_config: self.proxy_config,
            default_headers: self.default_headers,
            timeout: self.timeout,
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP 客户端结构体
pub struct HttpClient {
    proxy_config: Option<ProxyConfig>,
    default_headers: HeaderMap,
    timeout: Duration,
}

impl Clone for HttpClient {
    fn clone(&self) -> Self {
        Self {
            proxy_config: self.proxy_config.clone(),
            default_headers: self.default_headers.clone(),
            timeout: self.timeout,
        }
    }
}

impl HttpClient {
    /// 创建新的 HTTP 客户端（不使用代理）
    pub fn new() -> Self {
        // 确保 crypto provider 已初始化
        let _ = crate::tls::init_crypto_provider();

        Self {
            proxy_config: None,
            default_headers: HeaderMap::new(),
            timeout: Duration::from_secs(30),
        }
    }

    /// 获取客户端构建器
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// 创建使用代理的HTTP客户端
    pub fn with_proxy(proxy_config: ProxyConfig) -> Self {
        // 确保 crypto provider 已初始化
        let _ = crate::tls::init_crypto_provider();

        Self {
            proxy_config: Some(proxy_config),
            default_headers: HeaderMap::new(),
            timeout: Duration::from_secs(30),
        }
    }


    /// 发送 GET 请求
    pub fn get(&self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::GET, url, self.clone())
    }

    /// 发送 POST 请求
    pub fn post(&self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::POST, url, self.clone())
    }

    /// 发送 PUT 请求
    pub fn put(&self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::PUT, url, self.clone())
    }

    /// 发送 DELETE 请求
    pub fn delete(&self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::DELETE, url, self.clone())
    }

    /// 发送 HEAD 请求
    pub fn head(&self, url: &str) -> AsyncRequestBuilder {
        AsyncRequestBuilder::new(Method::HEAD, url, self.clone())
    }

    /// 异步执行请求
    pub async fn execute(&self, mut request: Request) -> Result<Response> {
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
        let request_str = self.build_request_string(&request, &parsed_url)?;

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

