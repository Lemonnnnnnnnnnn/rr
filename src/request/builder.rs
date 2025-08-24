//! HTTP请求构建器
//!
//! 提供流畅的请求构建API

use bytes::Bytes;
use crate::error::Result;
use crate::response::Response;
use super::model::Request;
use super::types::Method;

/// 异步请求构建器模式
pub struct AsyncRequestBuilder<'a> {
    request: Request,
    client: &'a crate::client::HttpClient,
}

impl<'a> AsyncRequestBuilder<'a> {
    /// 创建新的异步请求构建器
    pub fn new(method: Method, url: &str, client: &'a crate::client::HttpClient) -> Self {
        let request = Request::new(method, url);

        Self {
            request,
            client,
        }
    }

    /// 设置请求头
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.request = self.request.header(key, value);
        self
    }

    /// 设置多个请求头
    pub fn headers<K, V, I>(mut self, headers: I) -> Self
    where
        K: Into<String>,
        V: Into<String>,
        I: IntoIterator<Item = (K, V)>,
    {
        self.request = self.request.headers(headers);
        self
    }

    /// 设置 HeaderMap 的请求头（兼容方法）
    pub fn headers_map(mut self, headers: &crate::HeaderMap) -> Self {
        self.request = self.request.headers(
            headers.inner().iter().map(|(k, v)| (k.clone(), v.clone()))
        );
        self
    }

    /// 设置请求体
    pub fn body<B: Into<Bytes>>(mut self, body: B) -> Self {
        self.request = self.request.body(body);
        self
    }

    /// 构建请求
    pub fn build(self) -> Request {
        self.request
    }

    /// 异步发送请求
    pub async fn send(self) -> Result<Response> {
        self.client.send_request(self.request).await
    }
}
