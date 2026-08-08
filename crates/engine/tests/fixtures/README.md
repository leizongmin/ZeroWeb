# 测试 fixtures

本目录存放单元测试引用的第三方静态资源（经 `include_str!` 编译期嵌入）。这些文件**仅用于测试**，
不进入任何构建产物或运行时依赖。

## dompurify.js

- **来源**：DOMPurify 3.2.7（`dist/purify.js`，可读非压缩版）
- **上游**：https://github.com/cure53/DOMPurify
- **许可证**：Apache-2.0 OR MPL-2.0（双授权，文件首行 `@license` 注释保留原始声明）
- **使用条款**：ZeroWeb 项目排除 MPL 技术线（见 `AGENTS.md`），此处按 **Apache-2.0** 条款引入；
  作为测试 fixture（非 Cargo 依赖、不链接进产物），用于验证自建 DOM 桥接对真实 sanitize 库的支撑能力
  （R3019：`js_dom_bridge_tests::test_real_dompurify_sanitize_r3019`）。
- **更新方式**：替换文件后须保留首行 `@license` 注释，并在本 README 核对版本号与许可证一致性。
