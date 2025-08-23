//! 代理支持模块
//!
//! 提供HTTP/HTTPS代理和SOCKS代理支持

use crate::error::{Error, Result};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
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
}

/// 代理连接结构体
pub struct ProxyConnection {
    /// 底层TCP连接
    pub stream: TcpStream,
    /// 代理配置
    config: ProxyConfig,
}

impl ProxyConnection {
    /// 创建代理连接
    pub fn new(config: ProxyConfig) -> Result<Self> {
        let addr = (config.host.as_str(), config.port)
            .to_socket_addrs()
            .map_err(|e| Error::connection(format!("Failed to resolve proxy address: {}", e)))?
            .next()
            .ok_or_else(|| Error::connection("No proxy address found"))?;

        let stream = TcpStream::connect_timeout(&addr, config.timeout)
            .map_err(|e| Error::connection(format!("Failed to connect to proxy: {}", e)))?;

        stream.set_read_timeout(Some(config.timeout))?;
        stream.set_write_timeout(Some(config.timeout))?;
        stream.set_nodelay(true)?;

        Ok(Self { stream, config })
    }

    /// 建立到目标服务器的隧道（HTTP CONNECT方法）
    pub fn establish_tunnel(&mut self, target_host: &str, target_port: u16) -> Result<String> {
        match self.config.proxy_type {
            ProxyType::Http => self.establish_http_tunnel(target_host, target_port),
        }
    }

    /// 建立HTTP代理隧道
    fn establish_http_tunnel(
        &mut self,
        target_host: &str,
        target_port: u16,
    ) -> Result<String> {
        let mut request = format!("CONNECT {}:{} HTTP/1.1\r\n", target_host, target_port);
        request.push_str(&format!("Host: {}:{}\r\n", target_host, target_port));

        request.push_str("Connection: keep-alive\r\n");
        request.push_str("\r\n");

        // 发送CONNECT请求
        self.stream.write_all(request.as_bytes())?;
        self.stream.flush()?;

        // 读取响应
        let mut response = String::new();
        let mut buffer = [0u8; 8192];
        let mut total_read = 0;

        loop {
            let n = self.stream.read(&mut buffer)?;
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

        Ok(response)
    }

    /// 获取底层流的可变引用
    pub fn stream_mut(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    /// 获取底层流
    pub fn into_stream(self) -> TcpStream {
        self.stream
    }
}

