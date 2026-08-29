# R373 — dom/abort 域导入 + AbortSignal realm 面（M4/DC-3 基线扩展）

**日期**: 2026-08-30
**切片**: M4 基线扩展——`dom/abort` 子目录导入（第 6 个 dom/ 子分类）+
AbortSignal 的 iframe realm 面（forwarding + frame-scoped timeout）
**改动面**: `testharness.rs` + `fetch-dom-subset.sh`（SUBDIRS 扩展）+
`js_dom_shim/part02.js`（frame-scoped timeout）+ `part05.js`（per-realm
AbortSignal 包装）+ `part01.js`（iframe 移除取消登记定时器）

## 1. 本轮方向

已知 Fail 集合复核：MutationObserver-document 3F 评估为**架构域**——测试依赖
「解析流式交错执行」（inline script 注册 observer 后，解析器**后续**插入的
元素要产生 MO record），本仓架构是「整树解析完 → 按文档序执行脚本」，解析与
执行不交错——host 解析管线域深结构，转档备档。pivot 到 M4 基线扩展（DC-3
「按子分类持续扩展」）。

## 2. 改动三件

1. **dom/abort 导入**（SUBDIRS 扩展 + 3 个 .html 用例：reason-constructor /
   abort-signal-timeout / abort-signal-any-crash）：AbortSignal 域与既有
   fetch shim 的 AbortSignal 基建同域。
2. **iframe realm 转发 + per-realm 包装**（part05）：`iframe.contentWindow.
   AbortSignal` 旧 undefined——AbortSignal 用 R295 Text 同款 per-realm 包装
   （prototype 链接保持 instanceof；`.abort` 转发主构造器；`.timeout(ms)` 走
   `_zwTimeoutFor(win, ms)` frame-scoped 路径）；AbortController 纯转发。
3. **frame-scoped timeout**（part02 + part01）：spec `abortsignal-timeout`
   定时器归属「当前全局」——iframe realm 调用的 timeout 定时器登记到
   `win.__zwAbortTimerIds`，iframe 移除路径（part01
   `_zwRemoveIframeWindowClientForNode` 的 IFRAME 分支）按登记逐个 clearTimeout
   → `signal.aborted` 保持 false（WPT "not aborted after frame detach"）。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 目标域 | dom/abort 3 用例全 Pass（reason-constructor realm 断言 / frame-detach timeout / any-crash vacuous） |
| 全量 dom sweep（polyfill，TIME_LIMIT=2400） | **55498P（+3）/6 已知 Fail 文件恒等零新增**——Timeout 16 为并发噪声轮转族 |
| 哨兵 | MO 族 135P/3F 恒等；engine v8 2499 / quickjs 1474 全绿；webview 658P；integration 784P |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

**过程记录**：FORCE=1 全量重列触发 API 限流超时——新子目录用例改为直接
raw.githubusercontent 逐文件拉取（fetch 脚本的幂等快路径对已存在文件跳过）。
已知 Fail 集合维持 6：MutationObserver-document 3F（parse-time MO——解析流式
交错执行架构域，转档备档）、remove-and-adopt-thcrash（window.open）、
remove-next-sibling（插入期脚本执行[R328] + fused innerHTML）、
click-on-absolute-pseudo（Chromium 专有不追）、ranges dataChange/replaceData
2F（R353 游离树堆积域）。

## 4. 后续

- M4 基线扩展候选续：dom/lists（旧 NodeList 域）评估；dom/observable
  （Observable API——未实现域，导入即基线）。
- 已知 Fail 6 项全部深结构/架构域定性。
- 主线剩余：M5/M7 default-on（待用户点名，改 Mission 级单向门）；M3 已达成；
  M4 基线持续维护；M2 已收口；M8/DC-8 已收敛。
