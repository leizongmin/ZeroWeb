# R113 evidence — events 双簇：EventListener-handleEvent TypeError 上报 + webkit prefixed animation 四件套

**日期**: 2026-08-18
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**基线**: R112（events polyfill 397P/36F · native 386P/47F；`make test` 18,203P）

## 簇 1：EventListener-handleEvent.html 6P/0F（原 2P/3F + 1 timeout 修复后转 6P）

**根因（两处）**：

1. **handleEvent 非 callable 不抛 TypeError**（spec inner invoke 步骤 1-2 / WebIDL EventListener 非 nullable callback）：旧代码 `callable = fn && fn.handleEvent` 取不到就静默跳过。WPT「throws if `handleEvent` is falsy and not callable」（返 null）与「truthy and not callable」（返 42）都期望 TypeError。
   修复：`callable = fn ? fn.handleEvent : fn`；非 function 时构造 TypeError 经「report the exception」上报，listener 跳过（`part03.js` `_dispatchToListeners`）。

2. **report the exception 的标准形态**：R111 只调 `window.onerror` 属性——WPT 的 `EventWatcher(t, window, "error")` 等 **error 事件**（addEventListener 路径）超时。
   修复：`_zwReportListenerError`（`part03.js`）——fire ErrorEvent（message + error 字段）at window（`addEventListener('error')` listener 收到）+ onerror 属性 handler 走 legacy 5-arg 签名直调（暂移 listener 防双触发，返 true → preventDefault）+ console.error 兜底 + `_zwInReportError` 防递归。listener 异常路径（catch 块）同换本 helper，Event-dispatch-throwing 的 onerror 计数语义保持。

**cross-realm 5F 维持 Fail（非本簇修复目标）**：`eventListenerGlobalObject`（iframe named window）undefined——iframe contentWindow 深结构（master.md 未解决问题 #13 同族），不属轻量切片。

## 簇 2：webkit-animation/transition 四文件 16P/0F（原 4 timeout——resources .js fetch miss）

三处修复：

1. **`fetch-dom-subset.sh`**：补 `dom/events/resources/prefixed-animation-event-tests.js`（缺文件 → script fetch failed → 整文件 timeout）。
2. **`_ZW_PREFIXED_HANDLER_TYPES`**（`part01.js` 表 + `part04.js` get/set trap 接线）：spec HTML「event handler event type」表——`onwebkitanimationend` handler IDL 名全小写，但 event type 是 camelCase `webkitAnimationEnd`。on* setter 注册 listener 时经表换算真实 type 键（与 `addEventListener('webkitAnimationEnd')` 同键触发），getter 同映射读回；未命中保持原样（generic on* 面不受影响）。
3. **handle-based `<style>` 的 `.sheet`**（`part04.js` sheet getter + `part06.js` `_makeStyleSheet` handle 分支 + host `__zw_style_rules_handle` 回调）：用例形态 `createElement('style')` + `textContent = css` + `head.appendChild`（CSS-in-JS）——无 selector 可查快照，规则源 = mutation 历史（`query_inner_html_from_mutations` 的 SetTextOnHandle latest-wins + `query_history_text` 兜底）经 `style_rules_text`（`css_wire.rs` 从 `style_rules_wire` 提取的纯文本版）解析。写回经既有 `__zw_set_text_handle`。

## A/B 结果（WPT testharness 双路径，clean-HEAD stash 重建二进制对照）

| 路径 | R112 基线 | R113 | 净 |
|---|---|---|---|
| polyfill dom/events | 397P/36F | **415P/31F** | **+18** |
| native dom/events | 386P/47F | **404P/45F** | **+18** |
| dom/nodes | 6656P/1532F | 6656P/1534F | 逐值一致（name-validation 5F 差异为 flaky 采集，见下注） |
| dom/collections | 48P/1F | 48P/1F | 0 |
| dom/traversal | 1595P/9F | 1595P/9F | 0（逐值一致） |

> **nodes 采集注**：全量跑两次 count 波动（1532/1534/1536F）来自 `name-validation.html`——该文件 5 个 subtest 在 clean-HEAD 与 WIP 二进制下**单跑输出逐字节一致**（diff 验证），全量跑时受同批慢用例挤占出现挂起漂移；属既存 flaky 采集面（gitignored 数据 + CASE_TIMEOUT 边界），非本切片回归。

簇明细：EventListener-handleEvent 2P/3F+1timeout → **6P/0F**；webkit-animation 3 文件 timeout → **12P/0F**；webkit-transition timeout → **4P/0F**。

## 单测（part20.rs，engine，3 个新增）

- `test_handle_event_not_callable_reports_typeerror_r113`：handleEvent null/42 各上报一个 TypeError error 事件（ErrorEvent.error instanceof TypeError）。
- `test_prefixed_animation_handler_alias_r113`：`onwebkitanimationend` 与 `addEventListener('webkitAnimationEnd')` 同键触发、`onanimationend` 非别名独立、getter 读回、置 null 清除。
- `test_style_sheet_from_handle_style_element_r113`：handle-based `style.sheet.cssRules` 从 mutation 历史解析（selectorText/cssText 可读）。

## 验证

- `make test` **18,211 passed / 0 failed**（v8 全 workspace + quickjs 矩阵 + GPU 阶段，exit 0）
- `cargo fmt --all -- --check` 无 diff；v8 全 workspace clippy + quickjs clippy（engine）零警告
- engine js_dom_bridge 模块 594 单测全绿（含 3 新增）

## 教训

1. **report the exception ≠ 只调 onerror**：spec 的标准形态是 fire error event at window——`addEventListener('error')` 与 `window.onerror` 属性（legacy 5-arg 签名）是两条独立接收面，只覆盖一条会让 EventWatcher 类用例超时而非 fail（超时比 fail 更难归因，先查 script fetch 再查事件面）。
2. **handler IDL 名 ≠ event type**：prefixed 事件族的 on* 属性名全小写但 event type 是 camelCase——映射表要同时接 set（注册键）与 get（读回）两个 trap，只接一侧会表现为「设了读不回」。
3. **WPT timeout 三连查**：整文件 timeout 先查 ① resources .js 是否 fetch miss（fetch-dom-subset 补文件）② script 内同步死循环 ③ 事件回调永不被触发的等待。
