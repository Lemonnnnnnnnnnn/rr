# RT-1 HTTP 客户端 - 异步版本

一个轻量级的异步 HTTP 客户端，支持 HTTPS 和代理连接。

## 特性

- ✅ 异步 I/O 基于 tokio
- ✅ HTTPS 支持 (基于 tokio-rustls)
- ✅ HTTP/HTTPS 代理支持
- ✅ 流式请求构建器模式
- ✅ 完整的错误处理

## 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
rt-1 = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## 快速开始

### 基本 GET 请求

```rust
use rt_1::HttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = HttpClient::new();

    let response = client
        .get("https://httpbin.org/get")
        .send()
        .await?;

    println!("状态码: {}", response.status_code);
    println!("响应内容: {}", response.body);

    Ok(())
}
```

### 使用代理

```rust
use rt_1::{HttpClient, ProxyConfig};

let mut client = HttpClient::with_proxy(
    ProxyConfig::http("proxy.example.com", 8080)
);

let response = client
    .get("https://example.com")
    .send()
    .await?;
```

## API 参考

### HttpClient

- `HttpClient::new()` - 创建新客户端
- `HttpClient::with_proxy(config)` - 创建带代理的客户端
- `client.get(url)` - 创建 GET 请求构建器

### AsyncRequestBuilder

- `header(key, value)` - 添加请求头
- `body(data)` - 设置请求体
- `send().await` - 异步发送请求

### Response

- `status_code` - HTTP 状态码
- `status_message` - 状态消息
- `headers` - 响应头 HashMap
- `body` - 响应内容字符串
- `is_success()` - 是否为 2xx 状态码
- `is_redirect()` - 是否为 3xx 状态码
- `is_client_error()` - 是否为 4xx 状态码
- `is_server_error()` - 是否为 5xx 状态码

## 运行示例

```bash
# 运行异步示例
cargo run --example async_example

# 运行测试
cargo test

# 运行特定测试
cargo test --test integration_test
```

## 架构说明

本客户端采用纯异步架构：

- **连接层**: `AsyncConnection` trait 和 `AsyncHttpConnection` 实现
- **TLS 层**: `AsyncTlsManager` 基于 tokio-rustls
- **代理层**: `AsyncProxyConnection` 支持异步隧道建立
- **客户端层**: `HttpClient` 提供高层 API
- **构建器模式**: `AsyncRequestBuilder` 实现流式 API

所有 I/O 操作都是异步的，充分利用 tokio 的性能优势。

## 错误处理

所有方法返回 `Result<T>` 类型，错误信息通过 `Error` 枚举提供：

- 连接错误
- TLS 握手错误
- 代理连接错误
- 解析错误
- 其他 I/O 错误

## 许可证

MIT License
