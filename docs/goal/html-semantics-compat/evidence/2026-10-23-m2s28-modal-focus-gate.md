# M2 切片 28 — corner-cases 双案真 Chrome 仲裁与模态焦点门（2026-10-23）

## 范围

s27 挂账 corner-cases 余 2F（`focusin calls showModal` 变体）收口。产出：
**28P/2F → 29P/1F**，同 key 回归 0；两案一真一假——案 A 非 bug（Chrome
同款上游失败），案 B 引擎分歧（模态 inertness 焦点切面缺位）。

## 真 Chrome 仲裁方法

本地无真浏览器基准是 s25–s27 三轮反复归因的主因，本轮先建基准：

- playwright-core 1.63 + chromium-1243（`~/.cache/ms-playwright`），
  `page.keyboard`/`page.mouse` 走 CDP 浏览器输入层——light dismiss、ESC
  close watcher、mousedown 焦点默认动作全部真实管线（合成 DOM 事件不
  触发 Chrome 内部 light dismiss，不可用）。
- 静态服务器映射 `wpt-data/`（语料 `/resources/testharness.js` 同根可
  达；`testdriver*.js` 语料树本无——runner 自带 stub，Chrome 侧由 init
  脚本定义 `test_driver` 桥：`send_keys`→keyboard、`Actions`→mouse，
  补 `ButtonType` 与 WDT 特殊键 U+E00C→Escape 映射）。
- 语料基线：**29/30，唯一失败 = `Pressing escape, when focusin calls
  showModal, modeless`**。

## 两案定性（探针页全事件流）

语料 handler：body focusin 上 `!dialog.contains(target)` 时 re-entrant
`showModal()`；`respondsTo` 语义 = modal 案期望 light dismiss/ESC 后仍开
（closing steps 内 focusin 再触发 showModal 成功），modeless 案期望关死。

- **案 A（esc-modeless）＝非 bug**。真 Chrome 流：`show()` 聚焦 dialog
  → ESC 关闭 → `fo:dialog->button / fc:button`（spec close-the-dialog
  步骤 12：焦点锚∈subject ∨ wasModal 即 restore；锚=dialog 成立）→
  handler `showModal` 成功重开 modal → 端态 open。引擎流逐条相同。
  上游期望（closed）在 Chrome 自身即失败。`dialog-focus-previous-outside`
  末案独立钉死：Chrome modeless `show()` 自聚焦、`close()` 还原——
  spec 字面序即 Chrome 行为，无引擎可改空间。
- **案 B（click-modal）＝引擎分歧**。真 Chrome 流：`md:dialog`——
  mousedown 命中 dialog（模态外 button inert，hit-test 跳过，焦点不出
  top-layer）→ light dismiss 关闭 → restore 把焦点交还 button →
  `fc:button` → handler `showModal` 重开 modal → 端态 open → 期望
  「不响应」✓。引擎流：runner pre-focus 步（testharness.rs click/
  pointer_down 前置 `globalThis._zwFocusSel`）先把焦点漏给 button（真
  浏览器不可能聚焦模态外元素）→ close 时 restore 对已聚焦元素 no-op →
  无 `fc:button` → 不重开 → 端态 closed。

## 修复（模态 inertness 焦点切面，共享路径）

- `crates/engine/src/js_dom_shim/part01.js` `_zwDialogModalFocusGateOk`：
  栈顶 open+模态 dialog 的 shadow-including 子树外元素非 focusable
  area（spec 模态 dialog inertness 的 focusable-area 切片；rendering
  inert 整体仍是挂账，此处只钉焦点门）。stale 栈项（open/modal 态已清）
  跳过；仅栈顶模态生效——stacked modal 时内层模态子树仍可聚焦（Chromium
  同款）。
- `crates/engine/src/js_dom_shim/part06.js` `_zwIsFocusableSel` 接入：
  宿主驱动焦点（runner pre-focus 步、mousedown 默认动作）共用本门。页面
  直调 `el.focus()` 路径不在本门，随 rendering inert 一并接入。

## 契约

`js_dom_bridge_tests::test_dialog_modal_focus_gate_s28`：G1 模态外拒焦
（焦点保持在模态子树）/ G2 模态子树内可聚焦 / G3 close 后模态外恢复 /
G4 stacked 仅栈顶生效。修复前红（G1 泄焦）、修复后绿。

测试面注意：元素须落在初始 `dom_html`——变更追加节点经 `_makeProxy`
selector 串解析回退 DIV（`document.querySelector` 可解析而 shim 内部
tag 读缺省），非本切片面，探针钉定后测试改为初始 HTML 构造。

## 账本（s27 → 修复树）

dialog 集群 82 files（数据见
[2026-10-23-m2s28-modal-focus-gate.json](2026-10-23-m2s28-modal-focus-gate.json)）：

- **303P/101F/3T** vs s27 同文件集 302P/102F/3T。
- 逐 subtest 对 s27 账本 diff：同 key 回归 **0**，同 key 增益 1
  （corner-cases click-modal F→P）。
- corner-cases：28P/2F → **29P/1F**，== 真 Chrome 29/30；余 1F
  esc-modeless 为 Chrome 同款上游失败（两端端态均 open、上游期望
  closed），记账为 Chrome parity 非 gap。
- 3T 全为 s26 基线既有（backdrop-receives-element-events /
  dialog-popover-closedby-simple / focus-after-close）。
- engine lib 全量 **3426P/0F**（含新契约）；clippy `-D warnings` 过；
  fmt clean；零新增依赖。
