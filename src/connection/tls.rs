//! TLS 加密支持
//!
//! 提供异步 TLS 配置和流包装功能

use crate::error::{Error, Result};
use tokio_rustls::{TlsConnector, client::TlsStream};
use rustls::{ClientConfig, RootCertStore};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use webpki_roots::TLS_SERVER_ROOTS;

/// 异步 TLS 管理器
pub struct AsyncTlsManager {
    connector: TlsConnector,
}

impl AsyncTlsManager {
    /// 创建新的异步 TLS 管理器
    pub fn new() -> Self {
        let mut root_store = RootCertStore::empty();
        root_store.extend(TLS_SERVER_ROOTS.iter().cloned());

        let config = Arc::new(ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth());

        let connector = TlsConnector::from(config);

        Self { connector }
    }

    /// 创建异步 TLS 流
    pub async fn create_tls_stream<T: AsyncRead + AsyncWrite + Unpin>(
        &self,
        stream: T,
        server_name: &str,
    ) -> Result<TlsStream<T>> {
        use rustls::pki_types::ServerName;

        let server_name_owned = ServerName::try_from(server_name.to_string())
            .map_err(|_| Error::other("Invalid DNS name"))?;

        let tls_stream = self.connector.connect(server_name_owned, stream).await
            .map_err(|e| Error::other(format!("TLS handshake failed: {}", e)))?;

        Ok(tls_stream)
    }
}

// 注意：使用 tokio-rustls 的 TlsStream 类型，不需要自定义包装器
