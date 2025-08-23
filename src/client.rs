use crate::error::{Error, Result};
use rustls::{ClientConfig, ClientConnection, RootCertStore, Stream};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use webpki_roots::TLS_SERVER_ROOTS;

use crate::proxy::{ProxyConfig, ProxyConnection};
use crate::response::Response;

/// HTTP 客户端结构体
pub struct HttpClient {
    config: Arc<ClientConfig>,
    proxy: Option<ProxyConfig>,
}

impl HttpClient {
    /// 创建新的 HTTP 客户端（不使用代理）
    pub fn new() -> Result<Self> {
        let mut root_store = RootCertStore::empty();
        root_store.extend(TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(HttpClient {
            config: Arc::new(config),
            proxy: None,
        })
    }

    pub fn with_proxy(proxy: ProxyConfig) -> Result<Self> {
        let mut root_store = RootCertStore::empty();
        root_store.extend(TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        Ok(HttpClient {
            config: Arc::new(config),
            proxy: Some(proxy),
        })
    }

    /// 发送 GET 请求
    pub fn get(&self, url: &str) -> Result<Response> {
        let parsed_url = self.parse_url(url)?;
        let raw_response = if let Some(ref proxy) = self.proxy {
            let mut proxy_conn = ProxyConnection::new(proxy.clone())?;
            proxy_conn.establish_tunnel(&parsed_url.hostname, parsed_url.port)?
        } else {
            // 直接连接
            if parsed_url.is_https {
                self.send_https_request(&parsed_url)?
            } else {
                self.send_http_request(&parsed_url)?
            }
        };

        // 将原始响应字符串解析为 Response 结构
        Response::from_raw_response(raw_response)
    }

    /// 解析 URL
    fn parse_url(&self, url: &str) -> Result<ParsedUrl> {
        let (is_https, rest) = if url.starts_with("https://") {
            (true, url.trim_start_matches("https://"))
        } else if url.starts_with("http://") {
            (false, url.trim_start_matches("http://"))
        } else {
            // 默认使用 HTTPS，但检查端口
            if url.contains(":80") {
                (false, url)
            } else {
                (true, url)
            }
        };

        let (host, path) = if rest.contains('/') {
            let parts: Vec<&str> = rest.splitn(2, '/').collect();
            (parts[0], format!("/{}", parts[1]))
        } else {
            (rest, "/".to_string())
        };

        // 解析主机和端口
        let (hostname, port) = if host.contains(':') {
            let parts: Vec<&str> = host.split(':').collect();
            (
                parts[0].to_string(),
                parts[1].parse().unwrap_or(if is_https { 443 } else { 80 }),
            )
        } else {
            (host.to_string(), if is_https { 443 } else { 80 })
        };

        Ok(ParsedUrl {
            hostname,
            port,
            path,
            is_https,
        })
    }

    /// 发送 HTTPS 请求
    fn send_https_request(&self, parsed_url: &ParsedUrl) -> Result<String> {
        let server_name = parsed_url
            .hostname
            .clone()
            .try_into()
            .map_err(|_| Error::other("Invalid DNS name"))?;
        let mut conn = ClientConnection::new(self.config.clone(), server_name)
            .map_err(|_| Error::other("Failed to create client connection"))?;

        let addr = format!("{}:{}", parsed_url.hostname, parsed_url.port);
        let mut tcp_stream =
            TcpStream::connect(&addr).map_err(|_| Error::other("Failed to connect to server"))?;
        let mut tls_stream = Stream::new(&mut conn, &mut tcp_stream);

        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            parsed_url.path, parsed_url.hostname
        );

        tls_stream.write_all(request.as_bytes())?;

        let mut response = Vec::new();
        let mut buffer = [0; 1024];

        loop {
            match tls_stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => response.extend_from_slice(&buffer[..n]),
                Err(e) => return Err(Error::other(e.to_string())),
            }
        }

        String::from_utf8(response).map_err(|e| Error::other(e.to_string()))
    }

    /// 发送 HTTP 请求
    fn send_http_request(&self, parsed_url: &ParsedUrl) -> Result<String> {
        let addr = format!("{}:{}", parsed_url.hostname, parsed_url.port);
        let mut tcp_stream =
            TcpStream::connect(&addr).map_err(|_| Error::other("Failed to connect to server"))?;

        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            parsed_url.path, parsed_url.hostname
        );

        tcp_stream.write_all(request.as_bytes())?;

        let mut response = Vec::new();
        let mut buffer = [0; 1024];

        loop {
            match tcp_stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => response.extend_from_slice(&buffer[..n]),
                Err(e) => return Err(Error::other(e.to_string())),
            }
        }

        String::from_utf8(response).map_err(|e| Error::other(e.to_string()))
    }
}

/// 解析后的 URL 结构体
struct ParsedUrl {
    hostname: String,
    port: u16,
    path: String,
    is_https: bool,
}
