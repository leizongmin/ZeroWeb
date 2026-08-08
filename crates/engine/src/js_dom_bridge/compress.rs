//! CompressionStream / DecompressionStream host 实现（gzip/deflate，R2986）。
//! 经 flate2（既有 workspace crate——render-foundation 用于 WOFF zlib 解码）压缩/解压字节。
//! 字节经逗号分隔十进制串（"72,73,..."）wire，复用 crypto 模块的 [`bytes_from_csv`] / [`bytes_to_csv`]
//!（避免 UTF-8 歧义；CSV 串经 V8 字符串回调往返，4× 字节开销对 headless finite 流可接受，documented）。
//! `use super::*` glob 父模块项 + `use super::crypto::{bytes_from_csv, bytes_to_csv}` 取共享 byte wire。
//! pub 函数经 `pub use compress::*` 重导出，register_dom_callbacks 调用点零改动。

use super::crypto::{bytes_from_csv, bytes_to_csv};
use flate2::Compression;
use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};
use std::io::{Read, Write};

/// `new CompressionStream(format)`：压缩 bytes_csv（逗号分隔十进制）→ 逗号分隔十进制串。
/// format = `'gzip'` | `'deflate'` | `'deflate-raw'`（spec 三者；deflate = zlib 包装，deflate-raw = 裸 deflate）。
/// 不支持 format → 空串（shim 透传 reject `NotSupportedError`）。供 `__zw_compress` 回调 → shim
/// `CompressionStream.flush`（buffer-then-compress：transform 累积 chunk，flush 整体压缩）。
pub fn compress_bytes(format: &str, bytes_csv: &str) -> String {
    let data = bytes_from_csv(bytes_csv);
    let out: Vec<u8> = match format.to_ascii_lowercase().as_str() {
        "gzip" => {
            let mut enc = GzEncoder::new(Vec::new(), Compression::default());
            let _ = enc.write_all(&data);
            enc.finish().unwrap_or_default()
        }
        "deflate" => {
            let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
            let _ = enc.write_all(&data);
            enc.finish().unwrap_or_default()
        }
        "deflate-raw" => {
            let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
            let _ = enc.write_all(&data);
            enc.finish().unwrap_or_default()
        }
        _ => return String::new(),
    };
    bytes_to_csv(&out)
}

/// `new DecompressionStream(format)`：解压 bytes_csv → 逗号分隔十进制串。format 同 [`compress_bytes`]。
/// 损坏数据 / 不支持 format → 空串（shim 透传 error）。供 `__zw_decompress` 回调 → shim
/// `DecompressionStream.flush`。
pub fn decompress_bytes(format: &str, bytes_csv: &str) -> String {
    let data = bytes_from_csv(bytes_csv);
    let mut out = Vec::new();
    let ok = match format.to_ascii_lowercase().as_str() {
        "gzip" => GzDecoder::new(data.as_slice()).read_to_end(&mut out).is_ok(),
        "deflate" => ZlibDecoder::new(data.as_slice()).read_to_end(&mut out).is_ok(),
        "deflate-raw" => DeflateDecoder::new(data.as_slice()).read_to_end(&mut out).is_ok(),
        _ => false,
    };
    if !ok {
        return String::new();
    }
    bytes_to_csv(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gzip_round_trip() {
        // 大量重复文本 → gzip 压缩比远超 header/trailer 开销（~20B），压缩后严格更小。
        let original = "compression round trip payload ".repeat(40);
        let csv = bytes_to_csv(original.as_bytes());
        let compressed = compress_bytes("gzip", &csv);
        assert!(!compressed.is_empty(), "gzip 压缩产出非空");
        let compressed_bytes = bytes_from_csv(&compressed);
        assert!(compressed_bytes.len() < original.len(), "gzip 压缩后更小");
        assert_eq!(compressed_bytes[0], 0x1f, "gzip magic byte 1 (0x1f)");
        assert_eq!(compressed_bytes[1], 0x8b, "gzip magic byte 2 (0x8b)");
        let decompressed = decompress_bytes("gzip", &compressed);
        assert_eq!(
            bytes_from_csv(&decompressed),
            original.into_bytes(),
            "gzip 往返还原原文"
        );
    }

    #[test]
    fn test_deflate_round_trip() {
        let original = b"deflate-wrapped zlib stream payload repeat repeat repeat";
        let csv = bytes_to_csv(original);
        let compressed = compress_bytes("deflate", &csv);
        assert!(!compressed.is_empty(), "deflate 压缩产出非空");
        // zlib 头：0x78（CM=8 deflate, CINFO=7 → 0x78）。
        assert_eq!(bytes_from_csv(&compressed)[0], 0x78, "zlib header byte 0x78");
        let decompressed = decompress_bytes("deflate", &compressed);
        assert_eq!(bytes_from_csv(&decompressed), original.to_vec(), "deflate 往返还原原文");
    }

    #[test]
    fn test_deflate_raw_round_trip() {
        let original = b"raw deflate no zlib wrapper payload";
        let csv = bytes_to_csv(original);
        let compressed = compress_bytes("deflate-raw", &csv);
        assert!(!compressed.is_empty(), "deflate-raw 压缩产出非空");
        let decompressed = decompress_bytes("deflate-raw", &compressed);
        assert_eq!(
            bytes_from_csv(&decompressed),
            original.to_vec(),
            "deflate-raw 往返还原原文"
        );
    }

    #[test]
    fn test_unsupported_format_empty() {
        let csv = bytes_to_csv(b"data");
        assert_eq!(compress_bytes("brotli", &csv), "", "不支持 format 压缩 → 空串");
        assert_eq!(decompress_bytes("brotli", &csv), "", "不支持 format 解压 → 空串");
    }
}
