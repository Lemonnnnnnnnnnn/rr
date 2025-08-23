//! 连接核心实现
//!
//! 包含异步 Connection trait 和 AsyncHttpConnection 实现

use crate::connection::{ProxyConfig, AsyncProxyConnection, AsyncTlsManager};
use crate::error::{Error, Result};
use crate::utils::ParsedUrl;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use async_trait::async_trait;

/// 异步连接接口 trait
#[async_trait]
pub trait AsyncConnection: Send + Sync {
    /// 发送请求并获取响应
    async fn send_request(&mut self, request: &str, parsed_url: &ParsedUrl) -> Result<String>;
}

/// 异步 HTTP 连接结构体
/// 负责异步 HTTP 数据传输，支持直接连接和代理连接
pub struct AsyncHttpConnection {
    stream: TcpStream,
    tls_manager: AsyncTlsManager,
}

impl AsyncHttpConnection {
    /// 创建直接连接
    pub async fn direct(parsed_url: &ParsedUrl) -> Result<Self> {
        let addr = format!("{}:{}", parsed_url.hostname, parsed_url.port);
        let stream = tokio::net::TcpStream::connect(&addr)
            .await
            .map_err(|e| Error::connection(format!("Failed to connect to {}: {}", addr, e)))?;

        // 设置 TCP 参数
        stream.set_nodelay(true)
            .map_err(|e| Error::connection(format!("Failed to set TCP_NODELAY: {}", e)))?;

        Ok(Self {
            stream,
            tls_manager: AsyncTlsManager::new(),
        })
    }

    /// 创建代理连接
    pub async fn via_proxy(proxy_config: ProxyConfig, parsed_url: &ParsedUrl) -> Result<Self> {
        let mut proxy_conn = AsyncProxyConnection::new(proxy_config).await?;
        proxy_conn.establish_tunnel(&parsed_url.hostname, parsed_url.port).await?;

        // 提取 stream，避免部分移动问题
        let stream = proxy_conn.stream;

        Ok(Self {
            stream,
            tls_manager: AsyncTlsManager::new(),
        })
    }
}

#[async_trait]
impl AsyncConnection for AsyncHttpConnection {
    async fn send_request(&mut self, request: &str, parsed_url: &ParsedUrl) -> Result<String> {
        if parsed_url.is_https {
            self.send_https_request(request, parsed_url).await
        } else {
            self.send_http_request(request).await
        }
    }
}

impl AsyncHttpConnection {
    /// 通过HTTPS发送请求
    async fn send_https_request(&mut self, request: &str, parsed_url: &ParsedUrl) -> Result<String> {
        let mut tls_stream = self
            .tls_manager
            .create_tls_stream(&mut self.stream, &parsed_url.hostname).await?;

        // 发送请求
        tls_stream.write_all(request.as_bytes()).await
            .map_err(|e| Error::other(format!("Failed to write request: {}", e)))?;
        tls_stream.flush().await
            .map_err(|e| Error::other(format!("Failed to flush request: {}", e)))?;

        // 读取响应
        let mut response = Vec::new();
        let mut buffer = [0u8; 8192];

        loop {
            match tls_stream.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => response.extend_from_slice(&buffer[..n]),
                Err(e) => return Err(Error::other(format!("Failed to read response: {}", e))),
            }
        }

        String::from_utf8(response).map_err(|e| Error::other(format!("Invalid UTF-8: {}", e)))
    }

    /// 通过HTTP发送请求
    async fn send_http_request(&mut self, request: &str) -> Result<String> {
        // 发送请求
        self.stream.write_all(request.as_bytes()).await
            .map_err(|e| Error::other(format!("Failed to write request: {}", e)))?;
        self.stream.flush().await
            .map_err(|e| Error::other(format!("Failed to flush request: {}", e)))?;

        // 读取响应
        let mut response = Vec::new();
        let mut buffer = [0u8; 8192];

        loop {
            match self.stream.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => response.extend_from_slice(&buffer[..n]),
                Err(e) => return Err(Error::other(format!("Failed to read response: {}", e))),
            }
        }

        String::from_utf8(response).map_err(|e| Error::other(format!("Invalid UTF-8: {}", e)))
    }
}
