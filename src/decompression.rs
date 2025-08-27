//! 响应体解压缩模块
//!
//! 支持gzip、deflate、brotli等压缩格式的自动解压缩

use flate2::read::{GzDecoder, DeflateDecoder};
use std::io::{Read, Cursor};
use brotli::BrotliDecompress;
use crate::error::{Result, Error};

/// 压缩格式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Compression {
    Gzip,
    Deflate,
    Brotli,
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
            let mut decompressed = Vec::new();
            let mut input = Cursor::new(data);
            BrotliDecompress(&mut input, &mut decompressed)
                .map_err(|e| Error::Decompression(format!("brotli解压缩失败: {}", e)))?;
            Ok(decompressed)
        }
        Compression::None => Ok(data.to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_from_content_encoding() {
        assert_eq!(Compression::from_content_encoding("gzip"), Compression::Gzip);
        assert_eq!(Compression::from_content_encoding("deflate"), Compression::Deflate);
        assert_eq!(Compression::from_content_encoding("br"), Compression::Brotli);
        assert_eq!(Compression::from_content_encoding("unknown"), Compression::None);
        assert_eq!(Compression::from_content_encoding("GZIP"), Compression::Gzip); // 测试大小写不敏感
    }

    #[test]
    fn test_brotli_decompression_error_handling() {
        // 测试无效的 brotli 数据应该返回错误
        let invalid_brotli_data = b"This is not valid brotli compressed data!";
        let result = decompress(invalid_brotli_data, Compression::Brotli);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("brotli解压缩失败"));
    }

    #[test]
    fn test_no_compression() {
        // 测试无压缩情况
        let data = b"Hello, World!";
        let result = decompress(data, Compression::None).expect("无压缩解压失败");
        assert_eq!(result, data);
    }
}
