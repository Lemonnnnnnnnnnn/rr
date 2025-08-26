//! HTTP chunked 传输编码解析模块
//!
//! 支持解析 Transfer-Encoding: chunked 的响应体

use crate::error::{Result, Error};

/// Chunked 传输编码解析器
pub struct ChunkedParser;

impl ChunkedParser {
    /// 解析 chunked 编码的数据
    ///
    /// # 参数
    /// * `data` - 包含 chunked 数据的字节数组
    ///
    /// # 返回
    /// 返回解析后的完整数据
    pub fn parse(data: &[u8]) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        let mut remaining = data;

        loop {
            // 找到第一个 \r\n 的位置
            let line_end = remaining.windows(2).position(|w| w == b"\r\n")
                .ok_or(Error::Response("Invalid chunked format: no CRLF found".to_string()))?;

            // 解析 chunk 大小（可能包含扩展信息，如 "6;chunkext=val"）
            let size_line = &remaining[..line_end];
            let chunk_size_str = String::from_utf8_lossy(size_line);

            // 提取 chunk size，忽略分号后的扩展信息
            let chunk_size_part = chunk_size_str.split(';').next().unwrap_or("").trim();
            let chunk_size = usize::from_str_radix(chunk_size_part, 16)
                .map_err(|_| Error::Response(format!("Invalid chunk size: {}", chunk_size_str)))?;

            // 移动到 chunk 数据开始位置
            remaining = &remaining[line_end + 2..];

            if chunk_size == 0 {
                // 最后一个 chunk，检查是否有 trailer headers
                Self::skip_trailer_headers(&mut remaining)?;
                break;
            }

            // 检查是否有足够的 chunk 数据
            if remaining.len() < chunk_size + 2 {
                return Err(Error::Response(format!(
                    "Incomplete chunk data: expected {} bytes, got {}",
                    chunk_size + 2,
                    remaining.len()
                )));
            }

            // 提取 chunk 数据
            let chunk_data = &remaining[..chunk_size];
            result.extend_from_slice(chunk_data);

            // 跳过 chunk 末尾的 \r\n
            remaining = &remaining[chunk_size + 2..];
        }

        Ok(result)
    }

    /// 跳过 trailer headers（如果存在）
    fn skip_trailer_headers(data: &mut &[u8]) -> Result<()> {
        loop {
            // 找到下一个 \r\n
            let line_end = data.windows(2).position(|w| w == b"\r\n")
                .ok_or(Error::Response("Invalid trailer format".to_string()))?;

            if line_end == 0 {
                // 空行，表示 trailer 结束
                *data = &data[2..];
                break;
            }

            let line = &data[..line_end];
            if line.is_empty() {
                *data = &data[2..];
                break;
            }

            // 跳过这个 header 行
            *data = &data[line_end + 2..];
        }
        Ok(())
    }

    /// 检查是否为 chunked 传输编码
    pub fn is_chunked(headers: &std::collections::HashMap<String, String>) -> bool {
        if let Some(transfer_encoding) = headers.get("transfer-encoding") {
            transfer_encoding.to_lowercase().contains("chunked")
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_simple_chunked() {
        // 简单的 chunked 数据: "Hello World!" 分成两个 chunk
        let chunked_data = b"6\r\nHello \r\n6\r\nWorld!\r\n0\r\n\r\n";
        let result = ChunkedParser::parse(chunked_data).unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "Hello World!");
    }

    #[test]
    fn test_parse_empty_chunked() {
        let chunked_data = b"0\r\n\r\n";
        let result = ChunkedParser::parse(chunked_data).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_single_chunk() {
        let chunked_data = b"b\r\nHello World\r\n0\r\n\r\n";
        let result = ChunkedParser::parse(chunked_data).unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "Hello World");
    }

    #[test]
    fn test_is_chunked() {
        let mut headers = HashMap::new();
        assert!(!ChunkedParser::is_chunked(&headers));

        headers.insert("transfer-encoding".to_string(), "chunked".to_string());
        assert!(ChunkedParser::is_chunked(&headers));

        headers.insert("transfer-encoding".to_string(), "gzip, chunked".to_string());
        assert!(ChunkedParser::is_chunked(&headers));

        headers.insert("transfer-encoding".to_string(), "deflate".to_string());
        assert!(!ChunkedParser::is_chunked(&headers));
    }

    #[test]
    fn test_invalid_chunk_size() {
        let chunked_data = b"invalid\r\nHello\r\n0\r\n\r\n";
        assert!(ChunkedParser::parse(chunked_data).is_err());
    }

    #[test]
    fn test_incomplete_chunk() {
        let chunked_data = b"6\r\nHello\r\n0\r\n\r\n"; // chunk size 6 but only 5 bytes
        assert!(ChunkedParser::parse(chunked_data).is_err());
    }

    #[test]
    fn test_chunk_with_extensions() {
        // chunked with extension: "6;chunkext=val\r\nHello \r\n6\r\nWorld!\r\n0\r\n\r\n"
        let chunked_data = b"6;chunkext=val\r\nHello \r\n6\r\nWorld!\r\n0\r\n\r\n";
        let result = ChunkedParser::parse(chunked_data).unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "Hello World!");
    }
}
