# ZeroWeb Product Version (`zero-product-version`)

> 产品构建日期版本号（`YY.M.D`），构建期从 UTC 日期推导

## 概述

`ZeroWeb Product Version` (`zero-product-version`) 提供 ZeroWeb 产品的版本号常量。版本号不从语义化版本号维护，而是从构建日期推导：格式为 `YY.M.D`（如 `25.8.9` 表示 2025 年 8 月 9 日构建），同时生成 Windows 兼容的 `VS_FIXEDFILEINFO` 数值（`0x0019_0008_0009_0000`）。版本解析与格式化逻辑由构建支持模块 `build-support/product_version.rs` 提供，本 crate 在编译期通过 `ZERO_BUILD_VERSION` 环境变量嵌入最终版本。

## 主要功能

- **构建期版本嵌入** — `VERSION` 常量经 `env!("ZERO_BUILD_VERSION")` 在编译期嵌入，运行期零开销
- **日期推导格式** — 版本号格式 `YY.M.D`，无手工版本号维护负担
- **Windows 版本数值** — 同时提供 `VS_FIXEDFILEINFO` 风格的二进制版本值（`wYear`/`wMonth`/`wDay` 布局），供 Windows 资源节使用
- **构建支持测试** — 共享 `build-support/product_version.rs` 的 `from_unix_seconds` / `resolve`，测试覆盖格式与嵌入一致性

## 使用示例

```rust
// 构建期嵌入的产品版本，格式 `YY.M.D`（如 "25.8.9"）
use zero_product_version::VERSION;

fn main() {
    println!("ZeroWeb {}", VERSION);
}
```
