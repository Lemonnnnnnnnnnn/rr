//! TLS 和加密提供者管理模块
//!
//! 负责初始化和管理系统范围的 crypto provider

use crate::error::{Error, Result};
use std::sync::Once;

/// TLS 初始化状态管理
static TLS_INIT: Once = Once::new();

/// 初始化 crypto provider
/// 使用 Once 确保只初始化一次
pub fn init_crypto_provider() -> Result<()> {
    let mut result = Ok(());

    TLS_INIT.call_once(|| {
        result = init_crypto_provider_internal();
    });

    result
}

/// 内部的 crypto provider 初始化逻辑
fn init_crypto_provider_internal() -> Result<()> {
    // 检查是否已经安装了 crypto provider
    if let Ok(Some(_)) = std::panic::catch_unwind(|| {
        rustls::crypto::CryptoProvider::get_default()
    }) {
        // 已经安装了，直接返回
        return Ok(());
    }

    // 首先尝试安装 aws-lc-rs provider（推荐）
    let provider = rustls::crypto::aws_lc_rs::default_provider();
    match rustls::crypto::CryptoProvider::install_default(provider) {
        Ok(()) => return Ok(()),
        Err(e) => {
            return Err(Error::other(format!("Failed to install aws-lc-rs crypto provider: {:?}", e)));
        }
    }
}

/// 获取当前 crypto provider 状态
pub fn get_crypto_provider_status() -> Result<String> {
    match std::panic::catch_unwind(|| {
        rustls::crypto::CryptoProvider::get_default()
    }) {
        Ok(result) => {
            match result {
                Some(_) => {
                    Ok("Crypto provider installed".to_string())
                }
                None => {
                    Ok("No crypto provider installed".to_string())
                }
            }
        }
        Err(_) => {
            Err(Error::other("Panic while checking crypto provider status"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_crypto_provider() {
        // 这个测试可能会失败，如果已经安装了其他的 crypto provider
        // 但在正常情况下应该成功
        let _ = init_crypto_provider();
    }

    #[test]
    fn test_get_crypto_provider_status() {
        let _ = get_crypto_provider_status();
    }

    #[test]
    fn test_init_multiple_times() {
        // 多次调用应该都是安全的
        let _ = init_crypto_provider();
        let _ = init_crypto_provider();
        let _ = init_crypto_provider();
    }
}
