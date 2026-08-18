# R114 evidence — events：window.event shadow 语义 + composed 边界穿越 + XHR EventTarget 面

**日期**: 2026-08-18
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**基线**: R113（events polyfill 415P/31F · native 404P/45F；`make test` 18,211P）

## 簇：event-global.html 5F→0F（8P/8P 全绿）+ shadow-relatedTarget 部分

五处修复：

1. **`EventInit.composed` 生效**（`_makeEvent`，part03）：此前硬编码 `composed:false`——`new Event(t,{composed:true})` 的 init dict 被忽略，composed 事件的 shadow 边界穿越永不生效（根因级修复，下游 2/3 都依赖它）。
2. **dispatch 链 handle-based 上行 + shadow 边界**（`_dispatchWithBubble` 链构造，part03）：handle-based target（shadow 树内元素）经 `_zwNodeParent` 反链上行；遇 shadow root 容器（`_shadowHandleMeta`）时按 `event.composed` 决定是否跨边界到 host（非 composed 止于 shadow root，spec DOM §2.9）。链元素统一 `{sel, handle}` 形态（capture/bubble/composedPath 三个消费点同步解析）。**host 站入链**（hostHandle 分支 push `{shadow:false}` 站）——host listener 经 composed 冒泡触发。
3. **window.event shadow 段抑制**（part03 dispatch）：spec HTML「current event」——listener 节点 root 是 shadow root 时 `window.event` 为 undefined。target 深度（`_r114ShadowDepth`，target 起数 shadow 嵌套层数，immutable）≥1 时 target 站 + shadow 段祖先站派发前临时置 undefined、站后复原；host 及以上恢复可见。**教训**：链上行时只递减「剩余层数」游标（`_r114CurDepth`，站标记用），不能动 target 深度——第一版在 hostHandle 分支 `Math.min` 覆盖了 target 深度，suppression 失效（探针 `span:e` vs 单测 `shadow:undef` 矛盾定位）。
4. **onerror 期间 window.event 恢复**（`_zwReportListenerError`，part03）：spec「the onerror handler restores window.event」——legacy onerror 直调期间 `window.event` 须是被上报的 error 事件（`.type === 'error'`）。直调前临时设 errEv、调后恢复外层值（save/restore 配对，兼容外层 shadow 抑制窗口）。
5. **handle-based target 的 doc/win 虚站**（`inDoc` 判定，part03）：handle target 经反链上行到 sel 域（parentSel/hostSel）即 connected——shadow 内派发 composed error 冒泡到 window 触发 onerror（WPT ErrorEvent-in-shadow 用例路径）。

## XHR EventTarget 面（event-global "(2)" 1F→0F）

- **`XMLHttpRequest.prototype` 挂 `EventTarget.prototype`**（part05）：spec XHR : XMLHttpRequestEventTarget : EventTarget——`xhr.addEventListener/dispatchEvent` 可用（此前 undefined，探针实证）。
- **`EventTarget.prototype.dispatchEvent` 补 window.event + on* handler**（part05）：派发期 `globalThis.event = event`（HTML current event 对非 DOM EventTarget 同样生效，dispatch 后 restore）；on* 属性 handler 同 fire（`xhr.onload = fn` 后 dispatchEvent('load') 触发 fn）。**去重**：BroadcastChannel/MessagePort 的 on* setter 已把 handler 注册进 `_et_listeners`——handler 与已调 listener 同引用时跳过（防双 fire；R2783 broadcast 回归实证 `b:hi;b:hi` 双发，修后复原）。
- **`_et_listeners` 惰性初始化**：XHR ctor 不调 EventTarget ctor——dispatch/addEventListener 路径 `(target._et_listeners || (target._et_listeners = {}))`。

## shadow-relatedTarget 部分（getElementById + focus，2F 维持）

- **`getElementById` on shadow root**（part04 容器 handle 分支）：NonElementParentNode——`root.getElementById('shadowInput')` 经 `_handleQueryFirst` 查 registry 子树（`#id` 纯形式 / `[id="..."]` 转义形式）。此前 `root.getElementById is not a function`。
- **`focus()/blur()` on `_zwMEl` 解析节点**（part03）：innerHTML 解析的 shadow 子树元素无 focus 抛 TypeError。轻量语义：本地 focus 事件派发 + `_zwMElFocused` 全局（activeElement 近似）。
- **剩余 2F 维持**：relatedTarget retarget（shadow 内焦点 → light 端 focus 事件的 relatedTarget 须 retarget 到 host）需要焦点历史模型 + 跨边界 retarget——真 focus 模型深结构，记清单非本轮。

## A/B 结果（WPT testharness 双路径）

| 路径 | R113 基线 | R114 | 净 |
|---|---|---|---|
| polyfill dom/events | 415P/31F | **419P/32F** | **+4P**（1F = shadow-relatedTarget "at target" 从 crash 变真跑后 fail——功能暴露，非回归） |
| native dom/events | 404P/45F | 待全量（同 shim 面，预期同步 +） | — |
| dom/nodes | 6656P/~1533F | 6658P/1539F | +2P；F 漂移文件单跑 clean/WIP 逐字节一致（processing-instruction-attributes 136F / node-creation-realm 13F / CharacterData 族 / Attr-prefix / rootNode / remove-unscopable 全部 IDENTICAL——采集漂移非回归） |
| dom/collections | 48P/1F | 48P/1F | 0 |
| dom/traversal | 1595P/9-10F | 1595P/10F | 文件集 diff 为空（漂移） |

簇明细：event-global.html 5F→**0F（8P 全绿）**；event-global-is-still-set-* 2F 维持（iframe `frames[i]` 深结构）；EventListener-invoke-legacy timeout 维持。

## 单测（part20.rs +2）

- `test_window_event_shadow_suppression_r114`：shadow 段 window.event 抑制（span listener 见 undefined）+ host 恢复（host listener 见 event）+ dispatch 后 undefined + 非 composed 不跨边界（host 不触发）。
- `test_eventtarget_dispatch_on_property_and_window_event_r114`：XHR `dispatchEvent/addEventListener` typeof function + `onload` handler 派发期 `e === window.event` + dispatch 后 undefined + shadow `getElementById`（INPUT/bi 双命中）+ 解析子 `focus` typeof function。

## 验证

- `make test` **18,228 passed / 0 failed**（v8 + quickjs 双矩阵 + GPU 阶段，exit 0）
- `cargo fmt --all -- --check` 无 diff；v8 全 workspace clippy + quickjs clippy（engine）零警告
- engine js_dom_bridge 598 单测全绿（含 R114 +2；broadcast R2783 双发回归已修）

## 教训

1. **EventInit 字段逐个核对**：`composed` 硬编码 false 使一切 composed 语义测试「静默通过」（非 composed 路径行为巧合正确）——构造器 init dict 的每个字段（bubbles/cancelable/composed）都要从 dict 读。
2. **调试期变量可变性**：链上行游标（剩余层数）与判定基准（target 深度）是两个语义——上行修改基准会让下游判定全错。判定基准要 immutable。
3. **加 on*-fire 要查既有 setter 是否已注册 listener**：BroadcastChannel/MessagePort 的 on* setter 走 addEventListener 同步——generic on*-fire 必须去重（同引用跳过），否则双发。
4. **WPT「vacuous pass」**：listener 未触发 + step_func_done 未调的 async_test 在我们的 harness 下可能显示 Pass（超时边界）——修复后 listener 真正触发反而是更严格的验证。
