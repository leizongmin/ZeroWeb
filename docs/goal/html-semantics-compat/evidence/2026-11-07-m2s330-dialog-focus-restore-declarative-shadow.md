# M2 s330：dialog 焦点恢复 declarative-shadow retarget——2F→6P

日期：2026-11-07 ｜ 承接：s329（`2026-11-07-m2s329-canvas-zero-dim-to-dataurl-blob.md`）

## 目标与定谳

s329 六域普查再生队列 interactive 残红首选：`dialog-focus-previous-outside.html` 2F（subtest-2 shadowroot 外根 retarget 断言、subtest-4 slotted 焦点锚恢复断言）。勘根双根因：

- **RC1（retarget 臂）**：declarative 根 hydration 只登记不物化——part05 `_zwDeclarativeShadowEnsureContent` 把内容绑成 compound `zwsh` sel proxy（`zwsh<len>:<hostSel>|<localSel>`，engine `js_dom_bridge/shadow_address.rs` 铸造），engine 父链锚宿主（s176「shadow 内容不摊平」）。focus chain retarget walk（part06 `document.activeElement` node/proxy 双域 + part04 `ShadowRoot.activeElement` 双域）的 wrapper hop（parentNode 命中 `_shadowHandles`）在该面永不触发 → activeElement 直返 raw shadow 锚（subtest-2 expected host got shadow button）。
- **RC2（slot 摊平臂）**：slotted light 焦点锚经 slot 分配进 dialog 时纯 parentNode 链不可见归属（slotted 子 engine 父仍是宿主）。Chromium `HTMLDialogElement::close()` 同面用 `FlatTreeTraversal::Contains`（whatwg/html#8904 → PR #9178）；spec 文面「shadow-including inclusive descendant」在实现层即 flat-tree 包含。

## 施工（三文件六臂）

1. `crates/engine/src/js_dom_shim/part01.js`：
   - 新增 `_zwDsRootHandleForContentSel`：zwsh 复合地址解码 hostSel 段，反查 `_shadowRoots` 活注册 + `_shadowHandleMeta.declarative` 声明式面（不滤 mode——focus retarget 非封装边界）；前缀门外置，注册表扫描只在 declarative 内容焦点面付出。
   - RC2：`_zwDialogRestoreFocusR4600` anchorOk 补 `_zwLdFlatContains`（s176 slot 摊平机制复用，h2b→slot→sd）；modal gate `_zwDialogShadowInclContains` 不动（最小爆炸半径，A/B 判定）。
2. `crates/engine/src/js_dom_shim/part06.js`：`document.activeElement` node/proxy 双域 walk 插入 declarative 锚换 host 臂（本 root 不适用，换 host 继续——嵌套 declarative 逐轮外提至最外层）。
3. `crates/engine/src/js_dom_shim/part04.js`：`ShadowRoot.activeElement` node/proxy 双域同款（本 root → 锚即本 root 活跃元素；他 root → 换 host 继续）。

## 验证

| 面 | 命令/方式 | before | after | 结论 |
|---|---|---|---|---|
| WPT 目标文件 | `make testharness-html-semantics FILTER=dialog-focus-previous-outside` | 4P/2F | **6P/0F exit=0** | 修复面 solo 干净，零位移 |
| WPT dialog 域 | `FILTER=the-dialog-element`（stash 法 before 复跑） | 364P/45F | 366P/43F | (key,state) 归一 join 位移恰 2F→2P，409 keys 稳定，零意外 |
| WPT interactive 域 | `FILTER=interactive-elements`（before = s329 survey 日志同 HEAD） | 437P/46F | 439P/44F | 位移恰同 2F→2P，483 keys 稳定，零意外 |
| 契约 | `test_dialog_focus_restore_declarative_shadow_s330`（js_dom_bridge_tests/part66.rs） | — | 5 断言全 P | ① 外根 retarget host + SR.activeElement 内层锚 + close 不恢复；② 内根恢复 prevFocused；③ slot 摊平恢复 |
| engine lib | `cargo test -p zero-engine --lib`（guard） | — | 3795P/6F | +1 新契约测试全 P；6 红与 s329 stash 归因集原样（img settle×2/fetch matrix/iframe SW/websocket×2），零新增红 |

## 挂账与队列

- interactive 域残红：`dialog-popover-closedby-complex` 18F（shadow DOM slots closedby 面，需勘根）、audio_constructor 1F、audio loading=lazy 10F、object width/height rendered-dim IDL。
- canvas 域残红 3F+1T（承 s329：2d-getcontext-options/historical/toBlob.jpeg/security.pattern.fillStyle T）。
- CSP style-element 8T、microtasks 4F+2T（归域既有）。
- credentials 簇 12F+2T → runner-net-infra goal（移交已记）。

日志：`logs/s330-focus-outside-before.log`、`logs/s330-focus-outside-r1.log`、`logs/s330-dialog-domain-before.log`、`logs/s330-dialog-domain-r1.log`、`logs/s330-interactive-r1.log`（before 复用 `s329-survey-interactive-elements.log`）、`logs/s330-lib.log`、probe 存档 `logs/s330-probe*.log`（gitignored）。
