# R148 — shadow-relatedTarget 2F→0F（焦点所有权统一 + relatedTarget shadow retargeting）

**日期**: 2026-08-21
**里程碑**: M4（WPT dom 上游基线扩展）
**驱动用例**: `dom/events/shadow-relatedTarget.html`（2 subtest）

## 用例链路与根因

用例：`host.attachShadow({mode:'closed'})` → `root.innerHTML = "<input id='shadowInput'>"`
→ `root.getElementById('shadowInput').focus()`（**解析节点** `_zwMEl` 形态）→
`lightInput.focus()`（**proxy** 形态）→ 断言 light DOM 上 focus 事件的
`e.relatedTarget === host`（shadow host——closed-shadow 内容不泄露）。

| # | 断点 | 根因 |
|---|------|------|
| ① | subtest 1 fail：`e.relatedTarget` undefined（期望 #host） | proxy focus() 的 focus/focusin 事件无 relatedTarget；旧焦点是 `_zwMElFocused`（解析节点）时无 FocusEvent 语义 |
| ② | subtest 2 Timeout：`lightInput.focus()` no-op（listener 永不触发） | `_zwMEl.focus()`（R114）只设 `_zwMElFocused` 不清 `_activeElKey`——第一 subtest 后 proxy 焦点态仍是 lightInput，第二 subtest 的 `.focus()` 命中「已聚焦 no-op」守卫 |

## 修复（两层 + 一附带）

1. **`_zwMEl.focus()`（part03）补焦点迁移**：解析节点获焦时取代 proxy 焦点态——
   `_activeElKey` 清空 + 旧 proxy 派 focusout/blur（spec 焦点迁移序 focusout(旧) →
   focus(新) → blur(旧)）+ 自身已聚焦 no-op 守卫（spec 不重派）。
2. **proxy `focus()`（part04）的 relatedTarget + retargeting**：旧焦点为解析节点时
   （`_zwMElFocused`）——清之（所有权互斥，proxy 接管）+ focus/focusin 携带
   `relatedTarget`：旧焦点在 shadow 树内（`__zwFragHostHandle` 命中
   `_shadowHandleMeta`——R136 宿主印章链）→ 以 **shadow host proxy** 为
   relatedTarget（spec UI Events Focus + Shadow DOM retargeting：焦点离开 shadow
   树时边界外观察者看到 host 而非内部节点）；非 shadow 解析节点原样（detached doc
   元素等）；旧 proxy 焦点路径保持原事件形态**零变化**（零回归面）。旧解析焦点的
   focusout/blur 同步补派。
3. **附带**：`document.activeElement` getter 的 `_zwMElFocused` 优先（双态并存时的
   防御性回落——focus() 已互斥清理，正常路径不并存）。

## A/B 结果（polyfill / native 双路径）

| 套件 | 结果 |
|---|---|
| shadow-relatedTarget | **2P/0F 双路径 100%** |
| dom/events 全量 | **440P/16F/9T**（vs R147 438P/17F/10T：+2P/-1F/-1T，fail 集仅该件消失）；native 440P/16F/9T **逐文件一致** |
| 焦点敏感回归 | no-focus-events 2P、focus-event-document-move 1P 不回归 |
| dom/nodes 全量 | 5472P/230F/10T——3 个 diff 项隔离复跑全 Pass 或既存（crash 类 + 静态 .ref 页的套件负载 flake；`remove-from-shadow-host-adopt-ref` 3/3 Pass） |
| `make test` | 66 套件全绿 |
| fmt / clippy | 零 diff / 零警告（v8 + quickjs 双矩阵） |

## 单元测试（part21.rs 追加）

`test_focus_ownership_and_related_target_r148`：shadowInput 经
`root.getElementById` 查取（innerHTML 解析子树可查）→ `shadowInput.focus()` 后
`activeElement === shadowInput` → `lightInput.focus()` 的 focus listener 读
`relatedTarget === host`（retargeting）→ `activeElement === lightInput`（接管）→
`_zwMElFocused` 已清（互斥）五段断言。
