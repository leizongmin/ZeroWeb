---
date: 2026-10-05
modules: zero-product-version, build-support
---

# 陈旧 target 目录下新构建嵌入过期产品版本串

## 问题

slice30 集成验收（2026-10-05，merge 树 e721e05b3）在主工作区重构建 release 双 bin 后，headless devtools `/json/version` 报 `ZeroWeb/26.10.2`——按 `build-support/product_version.rs` 的日期推导（YY.M.D），当日构建应为 26.10.5。

## 根因

`ZERO_BUILD_VERSION` 由 product-version 的 build script 在**运行时**写入 `cargo:rustc-env=ZERO_BUILD_VERSION=…`。cargo 对 build script 的重跑条件是其 `rerun-if` 声明（该 crate 为 `rerun-if-env-changed=ZERO_BUILD_VERSION` + `rerun-if-changed=product_version.rs`）——**源码时间不在条件内**。target 目录缓存着 10-02 那次构建的 output（26.10.2），后续构建只要脚本不重跑，`rustc-env` 就复用旧值：二进制是新的，版本串是 10-02 的。

## 解决方案

- 需要版本串准确时：构建前显式 `ZERO_BUILD_VERSION=26.10.5 cargo build …`（或 `SOURCE_DATE_EPOCH`，二者均声明为 rerun-if），强制脚本重跑。
- 溯源判别：`ls target/release/build/zero-product-version-*/output` 的 mtime + 内容即当前嵌入值的事实源。
- provenance 纪律：活体证据的版本串字段不可作为二进制新鲜度证据；权威身份 = bin sha256 + commit。

## 如何避免

凡 build script 用 `rerun-if` 精确清单（而非默认全量重跑）发射时间类 `rustc-env`，都要意识到「重构建 ≠ 重发射」；时间敏感值要么走 `rerun-if-env-changed` 的显式注入，要么在验收记录里注明 embedded-version 的缓存语义。
