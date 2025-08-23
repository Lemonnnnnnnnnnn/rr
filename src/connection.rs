//! 连接管理模块
//!
//! 提供统一的连接接口，支持直接连接和代理连接
//! 支持HTTP和HTTPS协议

use crate::error::{Error, Result};
use crate::proxy::{ProxyConfig, ProxyConnection};
use crate::utils::ParsedUrl;
use rustls::{ClientConfig, ClientConnection, RootCertStore, Stream};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use webpki_roots::TLS_SERVER_ROOTS;

/// 连接类型枚举
#[derive(Debug, Clone)]
pub enum ConnectionType {
    /// 直接连接
    Direct,
    /// 代理连接
    Proxy(ProxyConfig),
}

/// 连接接口 trait
pub trait Connection: Read + Write {
    /// 发送请求并获取响应
    fn send_request(&mut self, request: &str, parsed_url: &ParsedUrl) -> Result<String>;
}

/// 直接TCP连接
pub struct DirectConnection {
    stream: TcpStream,
    tls_config: Option<Arc<ClientConfig>>,
}

impl DirectConnection {
    /// 创建新的直接连接
    pub fn new(parsed_url: &ParsedUrl) -> Result<Self> {
        let addr = format!("{}:{}", parsed_url.hostname, parsed_url.port);
        let stream = TcpStream::connect(&addr)
            .map_err(|e| Error::connection(format!("Failed to connect to {}: {}", addr, e)))?;

        // 设置超时
        stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(30)))?;
        stream.set_nodelay(true)?;

        // 为HTTPS连接准备TLS配置
        let tls_config = if parsed_url.is_https {
            let mut root_store = RootCertStore::empty();
            root_store.extend(TLS_SERVER_ROOTS.iter().cloned());

            Some(Arc::new(ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth()))
        } else {
            None
        };

        Ok(Self { stream, tls_config })
    }
}

impl Read for DirectConnection {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for DirectConnection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

impl Connection for DirectConnection {
    fn send_request(&mut self, request: &str, parsed_url: &ParsedUrl) -> Result<String> {
        if parsed_url.is_https {
            self.send_https_request(request, parsed_url)
        } else {
            self.send_http_request(request)
        }
    }
}

impl DirectConnection {
    /// 发送HTTPS请求
    fn send_https_request(&mut self, request: &str, parsed_url: &ParsedUrl) -> Result<String> {
        let tls_config = self.tls_config.as_ref()
            .ok_or_else(|| Error::other("TLS config not available for HTTPS"))?;

        let server_name = parsed_url
            .hostname
            .clone()
            .try_into()
            .map_err(|_| Error::other("Invalid DNS name"))?;

        let mut conn = ClientConnection::new(tls_config.clone(), server_name)
            .map_err(|_| Error::other("Failed to create client connection"))?;

        let mut tls_stream = Stream::new(&mut conn, &mut self.stream);

        // 发送请求
        tls_stream.write_all(request.as_bytes())?;
        tls_stream.flush()?;

        // 读取响应
        let mut response = Vec::new();
        let mut buffer = [0u8; 8192];

        loop {
            match tls_stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => response.extend_from_slice(&buffer[..n]),
                Err(e) => return Err(Error::other(e.to_string())),
            }
        }

        String::from_utf8(response).map_err(|e| Error::other(e.to_string()))
    }

    /// 发送HTTP请求
    fn send_http_request(&mut self, request: &str) -> Result<String> {
        // 发送请求
        self.stream.write_all(request.as_bytes())?;
        self.stream.flush()?;

        // 读取响应
        let mut response = Vec::new();
        let mut buffer = [0u8; 8192];

        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => response.extend_from_slice(&buffer[..n]),
                Err(e) => return Err(Error::other(e.to_string())),
            }
        }

        String::from_utf8(response).map_err(|e| Error::other(e.to_string()))
    }
}

/// 代理连接包装器
pub struct ProxyConnectionWrapper {
    proxy_conn: ProxyConnection,
    tls_config: Option<Arc<ClientConfig>>,
}

impl ProxyConnectionWrapper {
    /// 创建代理连接
    pub fn new(proxy_config: ProxyConfig, parsed_url: &ParsedUrl) -> Result<Self> {
        let mut proxy_conn = ProxyConnection::new(proxy_config)?;

        // 建立隧道
        proxy_conn.establish_tunnel(&parsed_url.hostname, parsed_url.port)?;

        // 为HTTPS目标准备TLS配置
        let tls_config = if parsed_url.is_https {
            let mut root_store = RootCertStore::empty();
            root_store.extend(TLS_SERVER_ROOTS.iter().cloned());

            Some(Arc::new(ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth()))
        } else {
            None
        };

        Ok(Self { proxy_conn, tls_config })
    }
}

impl Read for ProxyConnectionWrapper {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let stream_ref = self.proxy_conn.stream_mut();
        stream_ref.read(buf)
    }
}

impl Write for ProxyConnectionWrapper {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let stream_ref = self.proxy_conn.stream_mut();
        stream_ref.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let stream_ref = self.proxy_conn.stream_mut();
        stream_ref.flush()
    }
}

impl Connection for ProxyConnectionWrapper {
    fn send_request(&mut self, request: &str, parsed_url: &ParsedUrl) -> Result<String> {
        if parsed_url.is_https {
            self.send_https_request_through_proxy(request, parsed_url)
        } else {
            self.send_http_request_through_proxy(request)
        }
    }
}

impl ProxyConnectionWrapper {
    /// 通过代理发送HTTPS请求
    fn send_https_request_through_proxy(&mut self, request: &str, parsed_url: &ParsedUrl) -> Result<String> {
        let tls_config = self.tls_config.as_ref()
            .ok_or_else(|| Error::other("TLS config not available for HTTPS"))?;

        let server_name = parsed_url
            .hostname
            .clone()
            .try_into()
            .map_err(|_| Error::other("Invalid DNS name"))?;

        let mut conn = ClientConnection::new(tls_config.clone(), server_name)
            .map_err(|_| Error::other("Failed to create client connection"))?;

        let stream_ref = self.proxy_conn.stream_mut();
        let mut tls_stream = Stream::new(&mut conn, stream_ref);

        // 发送请求
        tls_stream.write_all(request.as_bytes())?;
        tls_stream.flush()?;

        // 读取响应
        let mut response = Vec::new();
        let mut buffer = [0u8; 8192];

        loop {
            match tls_stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => response.extend_from_slice(&buffer[..n]),
                Err(e) => return Err(Error::other(e.to_string())),
            }
        }

        String::from_utf8(response).map_err(|e| Error::other(e.to_string()))
    }

    /// 通过代理发送HTTP请求
    fn send_http_request_through_proxy(&mut self, request: &str) -> Result<String> {
        let stream_ref = self.proxy_conn.stream_mut();

        // 发送请求
        stream_ref.write_all(request.as_bytes())?;
        stream_ref.flush()?;

        // 读取响应
        let mut response = Vec::new();
        let mut buffer = [0u8; 8192];

        loop {
            match stream_ref.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => response.extend_from_slice(&buffer[..n]),
                Err(e) => return Err(Error::other(e.to_string())),
            }
        }

        String::from_utf8(response).map_err(|e| Error::other(e.to_string()))
    }
}

/// 连接管理器
pub struct ConnectionManager {
    connection_type: ConnectionType,
}

impl ConnectionManager {
    /// 创建连接管理器
    pub fn new(connection_type: ConnectionType) -> Self {
        Self { connection_type }
    }

    /// 创建连接
    pub fn create_connection(&self, parsed_url: &ParsedUrl) -> Result<Box<dyn Connection>> {
        match &self.connection_type {
            ConnectionType::Direct => {
                Ok(Box::new(DirectConnection::new(parsed_url)?))
            }
            ConnectionType::Proxy(config) => {
                Ok(Box::new(ProxyConnectionWrapper::new(config.clone(), parsed_url)?))
            }
        }
    }
}

// 注意：stream_mut方法已在proxy.rs中定义，这里不再重复定义
