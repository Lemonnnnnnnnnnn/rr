//! HTTP客户端相关类型定义
//!
//! 包含客户端构建器和相关类型

use crate::connection::ProxyConfig;
use crate::headers::HeaderMap;

/// HTTP 客户端构建器
/// 支持链式构建，类似 reqwest::Client::builder()
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    proxy_config: Option<ProxyConfig>,
    default_headers: HeaderMap,
}

impl ClientBuilder {
    /// 创建新的客户端构建器
    pub fn new() -> Self {
        Self {
            proxy_config: None,
            default_headers: HeaderMap::new(),
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

    /// 构建 HTTP 客户端
    pub fn build(self) -> crate::error::Result<super::model::HttpClient> {
        // 确保 crypto provider 已初始化
        crate::tls::init_crypto_provider()?;

        Ok(super::model::HttpClient {
            proxy_config: self.proxy_config,
            default_headers: self.default_headers,
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}
