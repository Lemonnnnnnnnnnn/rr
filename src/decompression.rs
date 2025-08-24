//! 响应体解压缩模块
//!
//! 支持gzip、deflate等压缩格式的自动解压缩

use flate2::read::{GzDecoder, DeflateDecoder};
use std::io::Read;
use crate::error::{Result, Error};

/// 压缩格式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Compression {
    Gzip,
    Deflate,
    Brotli,  // 预留
    None,
}

impl Compression {
    /// 从content-encoding头部值解析压缩格式
    pub fn from_content_encoding(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "gzip" => Compression::Gzip,
            "deflate" => Compression::Deflate,
            "br" => Compression::Brotli,
            _ => Compression::None,
        }
    }
}

/// 解压缩函数
pub fn decompress(data: &[u8], compression: Compression) -> Result<Vec<u8>> {
    match compression {
        Compression::Gzip => {
            let mut decoder = GzDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)
                .map_err(|e| Error::Decompression(format!("gzip解压缩失败: {}", e)))?;
            Ok(decompressed)
        }
        Compression::Deflate => {
            let mut decoder = DeflateDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)
                .map_err(|e| Error::Decompression(format!("deflate解压缩失败: {}", e)))?;
            Ok(decompressed)
        }
        Compression::Brotli => {
            // 预留实现
            Err(Error::Decompression("brotli解压缩暂未实现".to_string()))
        }
        Compression::None => Ok(data.to_vec()),
    }
}
