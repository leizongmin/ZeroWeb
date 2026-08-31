# R112 — checkbox/radio 激活后 input+change + detached doc·解析元素事件面（events +24P）

**日期**: 2026-08-18
**里程碑**: M4（WPT dom 上游基线扩展）
**Driving 用例**: `dom/events/Event-dispatch-detached-input-and-change.html`（8F→12P/0F 100%）+ `dom/events/Event-dispatch-bubbles-true.html`/`-false.html`（各 3F→5P/0F 100%）+ `dom/events/remove-all-listeners.html`（once 调用前移除 + 派发中移除跳过，R111 完成于本切片）
**基线（R110 后）**: polyfill 373P/54F · native 364P/63F
**结果**: **polyfill 397P/36F（+24 净）· native 386P/47F（+22 净）**

## 根因（三簇四层）

### ① Event-dispatch-detached-input-and-change（8F）

R108 的 pre-click activation 翻转了 checked，但 spec HTML input activation behavior 的**末段**「fire an event named input / change at el」从未实现——attached checkbox/radio click 后 input/change 不派发。

### ② Event-dispatch-bubbles（6F）

三重缺口：
1. `document.cloneNode` 主文档缺方法（TypeError 直接崩）。
2. `new Document()`/`createHTMLDocument()` 的 **doc/docEl/body 无 addEventListener/dispatchEvent**（detached doc 事件面真空）+ doc 级 `getElementsByTagName`/`getElementById`/`createEvent` 缺失。
3. detached 解析元素（`_zwParseEl`）无事件面，且**祖先链不可恢复**——detached 查询每次返新快照实例、各自 lazy 建独立 mut 树，树间无 parentNode 连通。

### ③ remove-all-listeners（R111 前置，本切片完成）

- once listener 须**调用前**移除（spec inner invoke 步骤 4 remove-then-call）——调用后移除在嵌套 dispatchEvent 时快照仍含本 listener → 无限递归。
- 快照迭代期间被 removeEventListener 移除的 listener 须跳过（「if listener's removed is true, continue」）。
- listener 异常须上报 `window.onerror`（string 形态首参，spec report the exception）而非仅 console.error。

## 修复

### part03.js

- `_zwFindClickActivation`（新）：pre-click 激活元素定位提取（与 `_zwPreClickActivation` 同一遍历）。
- `_zwFireInputChange`（新）：激活元素上派发 input（InputEvent 构造器优先）+ change（bubbles:true / cancelable:false）。
- `_zwClickActivationConnected`（新）：sel 经 `__zw_contains('html', sel)`，handle 经 `_zwNodeParent` 反链 + `_shadowHandleMeta` 跳 host（shadow 树随 host connected）。
- `_dispatchWithBubble` finally 尾部 post-activation：`_r112Act && _zwDispatching===0 && !canceled && connected` 时派发 input/change（canceled = cancelable + preventDefault——rollback 块已把 ledger 置 null，须按 canceled flag 判）。
- `_zwDispatchLocalDoc`（新）：detached doc 本地三阶段派发（doc→docEl→body 浅链，doc 存 `_zwLocalListeners` + docEl/body view 形态双存储）。
- detached doc：doc 级 `getElementsByTagName`/`getElementsByClassName`/`getElementById`/`createEvent` + doc 自身 addEventListener/dispatchEvent。
- `_zwWireLocalEvents`：docEl/body view 形态（`_zwEvLs`）+ `_zwEvTagRegistry['tag:HTML'/'tag:BODY']` 注册 + `_zwEvDocChain` 挂 globalThis。
- doc.appendChild：HTML 元素克隆入 doc 时其 innerHTML 提取 body 并入查询源（bodyHtml 串行化——handle 克隆 childNodes 走 host 侧数组不可靠，innerHTML 是可靠源）。
- body.appendChild：handle 元素子改为**串行合并**（outerHTML + 属性串拼入 bodyHtml）——`_zwMSerialize` 对 proxy child 序列化深度丢失（#table 可查而 #table-body 以下全 NULL 的根因）。

### part02.js

- `_zwParseEl`：`_zwPath`（host path 字段——祖先身份键数组）+ `_zwEvLs` listener 表 + `addEventListener`/`removeEventListener`/`dispatchEvent`。
- 身份键：id 优先（`id:<id>`）→ `sig:<TAG>|<class>|<outer前64>`；视图注册表 `_zwEvViewRegistry`（身份键→视图）+ `_zwEvTagRegistry`（tag 兜底——docEl/body 普通对象无 outerHTML 快照）。
- 派发：doc 站（capture 最先 / bubble 最后）→ capture 正序（path[0]=最外层→末端=最近父级，**首版逆序迭代致 currentTarget 序反转被 assert_array_equals 实证**）→ target 双 pass（capture 先）→ bubble 逆序。

### js_dom_bridge.rs

- `parse_html_element_json` 增 `path` 字段：祖先身份键数组（`id:` / `sig:` 键，`\x1f` 分隔，json_str 转义为 ``）——detached 解析元素祖先链的唯一来源（shim 侧快照无树上下文）。

### part06.js

- 主文档 `document.cloneNode(deep)`：返回可查询 detached doc（body 快照经 `__zw_get_inner_html('body')`）。

## A/B 结果（WPT testharness 双路径，clean-HEAD stash 重建二进制）

| 路径 | R110 基线 | R112 | 净 |
|---|---|---|---|
| polyfill dom/events | 373P/54F | **397P/36F** | **+24** |
| native dom/events | 364P/63F | **386P/47F** | **+22** |
| dom/nodes | 6671P/1536F | 6671P/1536F | 0（逐值一致） |
| dom/collections | 48P/0F | 48P/0F | 0 |
| dom/traversal | 1595P/9F | 1595P/9F | 0（逐值一致） |
| dom/ranges（3 文件抽样） | 0/0/1P | 0/0/1P | 0（逐值一致；全量超时不在门禁） |

簇明细：detached-input-and-change 4P/8F→**12P/0F**；dispatch-bubbles-true 2P/3F→**5P/0F**；dispatch-bubbles-false 2P/3F→**5P/0F**；remove-all-listeners 2F→**2P**（once 嵌套 + 派发中移除）。

## 单测（part20.rs 新文件，engine）

- `test_event_once_removed_before_invoke_r111`：once 调用前移除（嵌套 dispatchEvent 恰 1 次）。
- `test_event_listener_removed_during_dispatch_skipped_r111`：l1 内移除 l2 → l2 不触发。
- `test_click_activation_fires_input_and_change_when_attached_r112`：attached input,change / detached 不派发。
- `test_click_activation_input_change_bubbles_and_checked_rolled_back_on_prevent_r112`：radio dispatchEvent 激活 + 冒泡到 form + preventDefault 回滚后 input 不派发。
- `test_detached_document_event_surface_r112`：createHTMLDocument doc 级查询 + doc/html/body/x 四站 capture→target→bubble 顺序与 eventPhase。
- `test_parse_html_query_path_field_r112`：path 字段键序列（sig:HTML → sig:BODY → id:t → id:tb → id:r）。

## 验证

- `make test` **18,203 passed / 0 failed**（v8 + quickjs 双矩阵 + GPU 阶段，exit 0）
- `cargo fmt --all -- --check` 无 diff；v8 全 workspace clippy + quickjs clippy（engine/webview/script-sandbox）零警告
- `make product-smoke` 23.61% = clean-HEAD 同值（ZRG-2026-08-17-01 渲染流 hmtx 既存，非本切片回归）；struct-check PASS
- `make bench-gate`：**FAIL 19 项全部落在 CI-GUARD-20260818 已记录预存漂移签名内**（stroke_rect ~16× / worker_create_terminate ~12× / script-sandbox execute·sandbox 族 + canvas·engine paint + css-parse 族 1.4-1.7× runner 漂移，NEW=1 为 render-foundation raster_1500_fills 与 R100 同指标）——**本切片改动面指标（zero-webview 全部 8 项 + page 系全部 13 项）全 PASS**；run-rules §12 禁自主 relax，re-capture 属 infra/用户决策

## 教训

1. **capture 顺序的方向**：path 数组是「根→父」序，capture 沿 path **正序**（最外层先）——镜像直觉的「逆序迭代」会把 currentTarget 序整个反转；WPT assert_array_equals 的 per-index 报错是定位这类序反转的最快手段（expected property 1 不匹配 → 直接索引定位）。
2. **detached 视图的身份问题**：每次查询返新实例 + 各自独立 mut 树 → 祖先链必须经**身份键注册表**（id 优先 / sig 兜底 / tag 第三层）间接连通，不能走树 parentNode。
3. **proxy child 的序列化深度不可靠**：handle 元素 append 进 detached body 后，序列化读 proxy 的 childNodes 走 host 侧数组（深度丢失）——串行合并（outerHTML 字符串并入查询源）是可靠形态。
4. **post-activation 的 canceled 判定**：rollback 块先跑且把 ledger 置 null，后续条件按 canceled flag（cancelable + _defaultPrevented）判，不能读已被消费的 ledger。
