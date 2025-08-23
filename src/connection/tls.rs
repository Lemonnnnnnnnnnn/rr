//! TLS 加密支持
//!
//! 提供 TLS 配置和流包装功能

use crate::error::{Error, Result};
use rustls::{ClientConfig, ClientConnection, RootCertStore, Stream};
use std::io::{Read, Write};
use std::sync::Arc;
use webpki_roots::TLS_SERVER_ROOTS;

/// TLS 管理器
pub struct TlsManager {
    config: Arc<ClientConfig>,
}

impl TlsManager {
    /// 创建新的 TLS 管理器
    pub fn new() -> Self {
        let mut root_store = RootCertStore::empty();
        root_store.extend(TLS_SERVER_ROOTS.iter().cloned());

        let config = Arc::new(ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth());

        Self { config }
    }

    /// 创建 TLS 流
    pub fn create_tls_stream<T: Read + Write>(
        &self,
        stream: T,
        server_name: &str,
    ) -> Result<TlsStreamWrapper<T>> {
        use rustls::pki_types::ServerName;

        let server_name_owned = ServerName::try_from(server_name.to_string())
            .map_err(|_| Error::other("Invalid DNS name"))?;

        let conn = ClientConnection::new(self.config.clone(), server_name_owned)
            .map_err(|_| Error::other("Failed to create client connection"))?;

        Ok(TlsStreamWrapper::new(conn, stream))
    }
}

/// TLS 流包装器
pub struct TlsStreamWrapper<T> {
    conn: ClientConnection,
    stream: T,
}

impl<T> TlsStreamWrapper<T> {
    pub fn new(conn: ClientConnection, stream: T) -> Self {
        Self { conn, stream }
    }
}

impl<T: Read + Write> Read for TlsStreamWrapper<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut tls_stream = Stream::new(&mut self.conn, &mut self.stream);
        tls_stream.read(buf)
    }
}

impl<T: Read + Write> Write for TlsStreamWrapper<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut tls_stream = Stream::new(&mut self.conn, &mut self.stream);
        tls_stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut tls_stream = Stream::new(&mut self.conn, &mut self.stream);
        tls_stream.flush()
    }
}
