//! 代理连接管理
//!
//! 只负责异步代理服务器连接建立和隧道创建

use crate::error::{Error, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::time::Duration;

/// 代理类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyType {
    /// HTTP代理
    Http,
}

/// 代理配置结构体
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// 代理类型
    pub proxy_type: ProxyType,
    /// 代理服务器主机
    pub host: String,
    /// 代理服务器端口
    pub port: u16,
    /// 连接超时
    pub timeout: Duration,
}

impl ProxyConfig {
    /// 创建HTTP代理配置
    pub fn http(host: &str, port: u16) -> Self {
        Self {
            proxy_type: ProxyType::Http,
            host: host.to_string(),
            port,
            timeout: Duration::from_secs(30),
        }
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 从 URL 字符串创建代理配置
    /// 支持格式：http://proxy.example.com:8080
    pub fn from_url(url: &str) -> Result<Self> {
        if url.is_empty() {
            return Err(crate::error::Error::connection("Proxy URL cannot be empty"));
        }

        // 解析 URL
        let url = url::Url::parse(url)
            .map_err(|e| crate::error::Error::connection(format!("Invalid proxy URL: {}", e)))?;

        // 检查协议
        let scheme = url.scheme();
        if scheme != "http" {
            return Err(crate::error::Error::connection(format!("Unsupported proxy protocol: {}", scheme)));
        }

        // 获取主机和端口
        let host = url.host_str()
            .ok_or_else(|| crate::error::Error::connection("Proxy URL missing host"))?;

        let port = url.port().unwrap_or(80); // HTTP 默认端口

        Ok(Self::http(host, port))
    }
}

/// 异步代理连接结构体
/// 只负责异步连接到代理服务器并建立隧道
pub struct AsyncProxyConnection {
    /// 底层TCP连接
    pub stream: TcpStream,
}

impl AsyncProxyConnection {
    /// 创建到代理服务器的连接
    pub async fn new(config: ProxyConfig) -> Result<Self> {
        let addr = format!("{}:{}", config.host, config.port);
        let stream = tokio::net::TcpStream::connect(&addr)
            .await
            .map_err(|e| {
                Error::connection(format!("Failed to connect to proxy {}: {}", addr, e))
            })?;

        stream.set_nodelay(true)
            .map_err(|e| Error::connection(format!("Failed to set TCP_NODELAY: {}", e)))?;

        Ok(Self { stream })
    }

    /// 建立到目标服务器的隧道
    pub async fn establish_tunnel(&mut self, target_host: &str, target_port: u16) -> Result<()> {
        let request = format!(
            "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\nConnection: keep-alive\r\n\r\n",
            target_host, target_port, target_host, target_port
        );

        self.stream.write_all(request.as_bytes()).await
            .map_err(|e| Error::proxy(format!("Failed to write CONNECT request: {}", e)))?;
        self.stream.flush().await
            .map_err(|e| Error::proxy(format!("Failed to flush CONNECT request: {}", e)))?;

        // 读取并验证代理响应
        let mut response = String::new();
        let mut buffer = [0u8; 8192];
        let mut total_read = 0;

        loop {
            let n = self.stream.read(&mut buffer).await
                .map_err(|e| Error::proxy(format!("Failed to read proxy response: {}", e)))?;
            if n == 0 {
                break;
            }

            response.push_str(&String::from_utf8_lossy(&buffer[..n]));
            total_read += n;

            // 检查是否收到完整的响应头
            if response.contains("\r\n\r\n") {
                break;
            }

            if total_read > 8192 {
                return Err(Error::proxy("Proxy response too large"));
            }
        }

        // 解析响应状态
        let status_line = response.lines().next().unwrap_or("");
        let parts: Vec<&str> = status_line.split_whitespace().collect();

        if parts.len() < 2 {
            return Err(Error::proxy("Invalid proxy response"));
        }

        let status_code: u16 = parts[1]
            .parse()
            .map_err(|_| Error::proxy("Invalid status code in proxy response"))?;

        if status_code != 200 {
            return Err(Error::proxy(format!(
                "Proxy connection failed: {}",
                status_code
            )));
        }

        Ok(())
    }
}
