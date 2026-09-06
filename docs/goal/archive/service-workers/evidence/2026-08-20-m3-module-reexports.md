# M3-17 Service Worker Module Re-exports

**日期**：2026-08-20
**状态**：complete

## 实现

- static module dependency extraction 同时识别：
  - `export { name } from './dependency.js'`
  - `export * from './dependency.js'`
  - `export * as namespace from './dependency.js'`
- re-export dependency 与普通 import 使用同一 importer-relative canonical URL 解析，
  因此跨目录和递归 graph 不依赖 raw specifier 唯一性。
- named re-export 映射 imported/exported 名称；star re-export 排除 `default`；
  namespace re-export 保留完整 dependency namespace。
- Service Worker graph loader 会递归抓取 re-export-only dependency，纳入资源上限、
  persistence 和 update bytecheck。

## 验证

- V8 `zero-script-sandbox`：184/184。
- V8 re-export extraction/transform/runtime：3/3。
- QuickJS Service Worker re-export runtime：1/1。
- Service Worker core baseline：22 case / 91 subtest，91 Pass，两轮 deterministic。
- workspace Clippy `-D warnings` 通过。
- 定向性能门禁 `benchmark_20260820_212607.json`：
  - script-sandbox microbench 1/1；
  - startup 106.12 ms，peak RSS 155.56 MiB；
  - page total p95：19.74 / 442.52 / 103.04 ms；
  - retained form p95 0.0352 ms，absolute budgets 通过。

## 限制

- 转换式 module runtime 尚不提供 spec 精确的循环依赖 live bindings。
- dynamic `import()` 在 Service Worker module 中继续显式拒绝。

## 下一步

- 导入 `registration-script-module.https.html`，补 module parse/runtime/instantiation
  错误分类与 top-level await 拒绝路径。
