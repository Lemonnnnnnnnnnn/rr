use std::io::{Read, Write};
use std::net::TcpStream;

/// 代理服务器结构体
pub struct Proxy {
    host: String,
    port: u16,
}

impl Proxy {
    /// 创建新的代理服务器实例
    pub fn new() -> Self {
        Proxy {
            host: "127.0.0.1".to_string(),
            port: 7890,
        }
    }

    /// 创建到代理服务器的连接
    pub fn connect(&self) -> Result<TcpStream, Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.host, self.port);
        let stream = TcpStream::connect(&addr)?;
        Ok(stream)
    }



    /// 通过代理发送HTTP请求
    pub fn send_http_via_proxy(
        &self,
        target_host: &str,
        target_port: u16,
        path: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut stream = self.connect()?;

        // 构造HTTP请求，通过代理发送
        let request = format!(
            "GET http://{}:{}{} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            target_host, target_port, path, target_host
        );

        stream.write_all(request.as_bytes())?;

        // 读取响应
        let mut response = Vec::new();
        let mut buffer = [0; 1024];

        loop {
            let n = stream.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            response.extend_from_slice(&buffer[..n]);
        }

        Ok(response)
    }
}
