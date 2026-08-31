# M4 Slice R21 — 导入 dom/events 子目录建立 polyfill 通过率基线

**日期**: 2026-08-14
**里程碑**: M4 — WPT dom 上游基线 + 按聚类驱动修复
**切片**: R21
**前置**: R20（testharness status 精确化，dom/nodes polyfill 55.63% / native 54.98%）

## 范围

扩展 `DOM_TEST_SUBDIRS` 加入 `dom/events`（81 .html 主线程用例 + 16 .js 依赖），扩大 WPT dom 通过率面，暴露事件桥（addEventListener/dispatchEvent/Event/EventTarget）真实缺口。纯资产（用例导入 + 常量扩展），零生产逻辑改动。

## 实现

- `fetch-dom-subset.sh`：SUBDIRS 追加 `dom/events`（注释说明 Event-dispatch 系列是事件桥核心面）。
- `testharness.rs`：`DOM_TEST_SUBDIRS` 追加 `dom/events`（注释）。
- 用例拉取：GitHub API 列目录 + jsdelivr CDN 拉单文件（raw.githubusercontent 间歇超时，jsdelivr 更稳），81 .html + 16 .js 全部就位，0 失败。

## 验证

- **fmt + clippy 双矩阵**：wpt-runner v8 + quickjs 零警告（改动是脚本 + 常量注释，零逻辑）。
- **dom/nodes 回归**：native classlist 仍 1420P/0F（=R20，零回归——SUBDIRS 扩展不破坏既有 nodes 路径）。
- **polyfill dom/events 基线**（完整 JSON 入 evidence）：**98P/212F/9 timeout，319 subtest，81 cases，pass/(pass+fail)=31.61%**。

  失败聚类（暴露事件桥真实缺口，后续高 ROI）：
  1. **[30] Event 对象缺属性**（`should have`）—— Event 缺 timeStamp/NONE 常量/composedPath 等。
  2. **[25] 事件分发断言**（expected true got false）+ **[12] got undefined**—— capture/bubble/stopPropagation 语义。
  3. **[12] Convert to function** + **[12] page script threw**—— EventListener/handleEvent 调用。
  4. **[4] returnValue**—— Event.returnValue 缺失。
  5. **[5] eventListenerGlobalObject is not defined**—— incumbent-global 系列依赖测试设施（跨 realm），非事件桥缺陷。

  **54 个用例 0-pass**（Event-dispatch 系列 + EventTarget 系列），是 polyfill 事件分发的系统性缺口（后续 R23+ 聚类驱动修复）。

## 关键发现：native dom/events 路径 hang（R22 首要目标）

`testharness-dom-native`（ZW_NATIVE_DOM=1）跑 dom/events **整体超时卡死**（>500s，test-guard 杀进程）。诊断：
- **简单用例不 hang**：Event-type（3P）/ CustomEvent（有输出 exit 1）native 正常。
- **dispatchEvent 用例 hang**：Event-dispatch-click / event-global（polyfill 下 timeout 的用例）native 下 20s 无输出。
- **根因方向**：native_dom=1 时 element prototype 上 `dispatchEvent`（html_element.rs:288 `native_dispatch_event_invoke`）+ `addEventListener`（:281）覆盖 polyfill。native dispatchEvent 经 `dispatch_event_impl`（event_target.rs:121）三阶段派发，listener 存 gc.rs LISTENERS（thread-local）。疑似：① listener 回调内再 dispatchEvent / 改 DOM 触发 reentrant `sync_render_after_native_dom` 重渲染循环；② native listener 存储（gc.rs）与 polyfill `_listenerStore` 割裂导致空转；③ `get_or_create_native_element`（:221）对每节点 create 可能触发重渲染。
- **不影响常规门禁**：`make test` 只跑 `cargo test --workspace`（不含 testharness-dom-native），CI 不受影响。`testharness-dom-native` 是手动诊断入口。
- **dom/nodes native 不受影响**：native classlist 1420P/0F（=R20），hang 仅在 events dispatchEvent 路径。

## 决策记录

- **为何 R21 land polyfill 基线而不阻塞于 native hang**：R21 是纯资产切片（导入用例 + 建基线），核心价值是 polyfill events 基线（31.61%，暴露 54 个 0-pass 事件分发缺口）。native events hang 是 native 路径 dispatchEvent 的独立缺陷，根因需深入 native listener 存储 + 重渲染链路（R22 专项），非 R21 资产切片范围。按护栏「轻量修复优先、永不停」，polyfill 基线 land，native hang 详记根因线索转 R22。
- **为何用 jsdelivr CDN**：raw.githubusercontent 在本环境间歇 30s 超时（fetch-dom-subset.sh 的 `--retry 3` 无法解决连接级超时），jsdelivr CDN 内容一致且稳定，手动 fallback 拉取 97 文件 0 失败。fetch-dom-subset.sh 保持 raw 源（CI 环境可能不同），本地手动用 jsdelivr。
- **native hang 不阻塞 CI 的保证**：`make test` 经核实不含 testharness-dom-native（Makefile test target 仅 cargo test --workspace + clippy）。native events hang 是已知诊断限制，记录在案，R22 修复后 native events 基线随之建立。

## 残留（转 R22 / R23+）

- **R22（首要）**：native dom/events dispatchEvent hang 根因定位 + 修复（native listener 存储 / reentrant 重渲染 / get_or_create），建立 native events 基线。
- **R23+**：polyfill 事件分发系统性缺口（54 个 0-pass 用例）—— Event 对象属性补全（timeStamp/NONE/composedPath/returnValue）、事件三阶段分发语义（capture/bubble/stopPropagation）、EventListener handleEvent。每聚类 net≥0 land。
