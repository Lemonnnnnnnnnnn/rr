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
    browser_headers_enabled: bool, // 是否启用浏览器请求头预设
}

impl ClientBuilder {
    /// 创建新的客户端构建器
    pub fn new() -> Self {
        Self {
            proxy_config: None,
            default_headers: HeaderMap::new(),
            browser_headers_enabled: true, // 默认启用浏览器请求头
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

    /// 启用或禁用浏览器请求头预设
    pub fn browser_headers(mut self, enabled: bool) -> Self {
        self.browser_headers_enabled = enabled;
        self
    }

    /// 禁用浏览器请求头预设
    pub fn no_browser_headers(mut self) -> Self {
        self.browser_headers_enabled = false;
        self
    }

    /// 构建 HTTP 客户端
    pub fn build(self) -> crate::error::Result<super::model::HttpClient> {
        // 确保 crypto provider 已初始化
        crate::tls::init_crypto_provider()?;

        let mut client = super::model::HttpClient {
            proxy_config: self.proxy_config,
            default_headers: self.default_headers,
        };

        // 如果启用了浏览器请求头，将其添加到默认请求头中
        if self.browser_headers_enabled {
            let browser_headers = crate::headers::browser_headers::chrome();
            for (key, value) in browser_headers {
                if !client.default_headers.contains_key(&key.to_lowercase()) {
                    // 忽略插入失败的错误，继续处理其他请求头
                    let _ = client.default_headers.insert(key, value);
                }
            }
        }

        Ok(client)
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}
