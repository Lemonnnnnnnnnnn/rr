//! HTTP头处理模块
//!
//! 提供HTTP头的基本标准化、验证功能和兼容接口
//! 该模块被拆分为多个子模块以提高代码组织性

pub mod constants;
pub mod builder;
pub mod map;

// 重新导出主要类型和函数
pub use constants::{
    normalize_header_name,
    normalize_header_value,
    validate_header_name,
    validate_header_value,
    common_headers,
    content_types,
    browser_headers,
};

pub use builder::HeadersBuilder;
pub use map::{HeaderMap, HeaderMapIter};
