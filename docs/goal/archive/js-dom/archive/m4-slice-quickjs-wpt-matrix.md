# M4 — QuickJS WPT dom 矩阵基线 + Proxy loose 语义修复（R76）

**日期**: 2026-08-16
**Commit**: `68f09748`
**前置**: R75（M6 全量收口，`4bfa87e3`）
**证据**: [evidence/2026-08-16-r76-quickjs-wpt-matrix-baseline.json](../evidence/2026-08-16-r76-quickjs-wpt-matrix-baseline.json)

## 背景

M6 收口后，DC-7「双 feature polyfill vs native A/B 行为等价」需要 WPT 级量化证据。此前 testharness-dom 双路径基线只在 v8 构建下跑过；quickjs feature 的 release runner 从未建立。

## 基础设施

- **quickjs release runner**：`cargo build --release --no-default-features --features quickjs --bin zero-wpt-runner --target-dir target/quickjs-release`（独立 target-dir 防覆盖 v8 二进制）。
- **native 路径**：`ZW_NATIVE_DOM=1` → `run_testharness_html_inner` 读 env → `WebViewConfig.native_dom` → quickjs 构建走 R57 `install_native_dom_bindings` 接线——**管线首次全量验证贯通**。

## 基线（178+81+10+47+17 用例）

| 路径 | collections | events | nodes | ranges | traversal | TOTAL |
|------|------------|--------|-------|--------|-----------|-------|
| quickjs polyfill | 38P/10F | 156P/99F | 2201P/1113F | 40P/72F | 7P/47F | **2442P/1341F = 64.55%** |
| quickjs native | 38P/10F | 156P/99F | 2200P/1114F | 40P/72F | 7P/47F | **2441P/1342F = 64.53%** |

**双路径对等差 0.02pp**（单 subtest 漂移）——用例侧 document 走 polyfill shim（R9 架构事实），native 叠加路径两引擎同构。DC-7 A/B 等价的直接量化证据。

## 发现的真缺口 + 修复

**quickjs collections 38P vs v8 48P**：10F 全簇 = HTMLCollection Proxy `set`/`deleteProperty` trap 返 false 在 QuickJS 下 **loose 调用也抛 TypeError**（V8 仅 strict 抛——引擎 Proxy invariant 实现差异），loose 断言崩整用例。

**修复**（part05.js）：`_zwV8ProxySetSemantics()` lazy 引擎探测（loose 必拒 set 不抛 = V8）+ trap 拒绝路径 QuickJS 分支返 true 不写（loose 静默，spec 观察语义）。

**结果**：quickjs collections loose 断言全过、用例完整执行（own-props 5P/indices 4P/names 4P/delete 2P）；10 strict 断言仍挂（引擎无 caller-mode API，无法在 trap 内区分 strict——取舍记录在案）；**v8 collections 48P/0F 零回归**；events/nodes/traversal 数字不变。

## 过程注记

- v8 全量 JSON 被 ranges/Range-mutations-insertBefore 慢用例（M1 L2 已知遗留）吃满 test-guard 1800s 墙钟（exit 124）——v8 对照以分目录跑法补齐。
- 清理 4 个历史 session 残留的 insertBefore runner 实例（累计 CPU 1138min）。
- WPT loose/strict 断言分布对称（10↔10）：修复把「崩用例」变为「完整执行但 strict 缺」——M1 L2 或引擎 strict 探测通道出现时可再收。

## 验证

engine quickjs 1419 / v8 2153、wpt-runner quickjs 106 / v8 171 全绿；clippy 零警告；fmt 无 diff；pre-commit-guard PASS。
