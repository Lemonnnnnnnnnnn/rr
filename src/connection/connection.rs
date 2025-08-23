//! 连接核心实现
//!
//! 包含 Connection trait 和 HttpConnection 实现

use crate::connection::{ProxyConfig, ProxyConnection, TlsManager};
use crate::error::{Error, Result};
use crate::utils::ParsedUrl;
use std::io::{Read, Write};
use std::net::TcpStream;

/// 连接接口 trait
pub trait Connection: Read + Write {
    /// 发送请求并获取响应
    fn send_request(&mut self, request: &str, parsed_url: &ParsedUrl) -> Result<String>;
}

/// HTTP 连接结构体
/// 负责 HTTP 数据传输，支持直接连接和代理连接
pub struct HttpConnection {
    stream: TcpStream,
    tls_manager: TlsManager,
}

impl HttpConnection {
    /// 创建直接连接
    pub fn direct(parsed_url: &ParsedUrl) -> Result<Self> {
        let addr = format!("{}:{}", parsed_url.hostname, parsed_url.port);
        let stream = TcpStream::connect(&addr)
            .map_err(|e| Error::connection(format!("Failed to connect to {}: {}", addr, e)))?;

        // 设置超时
        stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(30)))?;
        stream.set_nodelay(true)?;

        Ok(Self {
            stream,
            tls_manager: TlsManager::new(),
        })
    }

    /// 创建代理连接
    pub fn via_proxy(proxy_config: ProxyConfig, parsed_url: &ParsedUrl) -> Result<Self> {
        let mut proxy_conn = ProxyConnection::new(proxy_config)?;
        proxy_conn.establish_tunnel(&parsed_url.hostname, parsed_url.port)?;

        // 提取 stream，避免部分移动问题
        let stream = proxy_conn.stream;

        Ok(Self {
            stream,
            tls_manager: TlsManager::new(),
        })
    }
}

impl Read for HttpConnection {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for HttpConnection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

impl Connection for HttpConnection {
    fn send_request(&mut self, request: &str, parsed_url: &ParsedUrl) -> Result<String> {
        if parsed_url.is_https {
            self.send_https_request(request, parsed_url)
        } else {
            self.send_http_request(request)
        }
    }
}

impl HttpConnection {
    /// 通过HTTPS发送请求
    fn send_https_request(&mut self, request: &str, parsed_url: &ParsedUrl) -> Result<String> {
        let mut tls_stream = self
            .tls_manager
            .create_tls_stream(&mut self.stream, &parsed_url.hostname)?;

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

    /// 通过HTTP发送请求
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
