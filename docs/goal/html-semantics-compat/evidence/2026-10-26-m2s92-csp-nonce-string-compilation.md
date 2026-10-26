# M2-s92：CSP nonce 传播面——动态 import() 槽位 nonce + 内联投递保真（s88 挂账 4F 清盘）

日期：2026-10-26。归属：html-semantics-compat / s86 切片序③ the-script-element 基面（承接 s91，HEAD abdd1b178）。

## 1. 面与基线

s91 HEAD 基线（/tmp/s91-fixed-scan.txt 517 行）：**197P/295F/25T**。残面：

- **string-compilation-nonce-{classic,module} must-not-inherit 4F**（s88 挂账 CSP 面）：`setAttribute('onclick', …)` + `.onclick()`/`.click()` 两臂——spec 事件处理器不继承触发脚本 nonce，体内 `import()` 的 module fetch 无 nonce → 被 `script-src 'nonce-correct'` 政策拒 → TypeError reject。红因：引擎无 import fetch 的 CSP 检查，import 成功 → `promise_rejects_js` 断言失败。
- 绿臂自检：nonce 页 8 个 inherit 臂（setTimeout/eval/Function ×2 页）当前因「无 CSP 检查」平凡绿——本刀补检查后必须保持绿（nonce 传播语义）。

## 2. 既有基座与本刀刀位

security-hardening M2-s1~s7 已有：meta CSP 解析（extract）、`check_script(nonce, hash, url)` 脚本检查点（nonce 匹配不受源清单限制；无政策返 None）、`SecurityPolicyViolationEvent` 派发、eval 门（M2-s6，`unsafe-eval` 放行页不激活）、script-src-attr hash 门（M2-s7，同激活门）。**缺口**：classic 上下文动态 `import()` 的宿主解析路径无 CSP 检查、无 nonce 传播通道。

本刀三面（classic 槽位路径——string-compilation 各页全部动态 import 经 native 宿主回调）：

### 刀①：DynamicImportState nonce 槽位（script-sandbox）

- `DynamicImportState` 加 `nonce: Option<String>`；`DynamicImportResolver` 契约升三参 `resolver(spec, referrer, nonce)`（nonce 空串 ↔ `None` 双射——extract 侧裸 nonce 视缺失，合法 nonce 恒非空）；回调透传槽位现值。
- 新 `set_dynamic_import_nonce(Option<&str>)`（每脚本前刷新，同 referrer）与 `register_dynamic_import_nonce_swapper()`（`__zwSwapImportNonce(v)`：有参换入返旧值、无参纯 getter——与 s90 `__zwSwapImportReferrer` 同形）。`lib.rs` trait 同名默认 no-op（QuickJS 降级）。
- 空契约路径既有 `Exception::type_error` reject——CSP 阻止时宿主返空串即天然 TypeError，`promise_rejects_js(t, TypeError, …)` 判据零改动命中。
- 新单测 2：nonce 随 resolver 第三参透传（Some/None 双态）；swapper 换入/返旧/getter 序列。

### 刀②：webview 接线——逐脚本刷新 + resolver CSP 检查

- 脚本循环 s88 referrer 刷新点旁加 `set_dynamic_import_nonce(script_csp_info[i].nonce)`——spec 动态 import 继承触发脚本 nonce（eval/Function 同步窗直读槽位）。
  https://html.spec.whatwg.org/multipage/webappapis.html#hostimportmoduledynamically-retrieval
- resolver 闭包（script_source_fetcher 门内）捕获 `security_context` + violation 队列 Arc：resolve 前按 `parent`（脚本基 URL）解析 spec 绝对 URL，`check_script(nonce, None, Some(abs))`——violation 入 connect 共享队列（run 尾 document 站派发，M2 惯例）+ 返空串 → reject TypeError。
- 注册 `__zwSwapImportNonce` swapper（同 referrer swapper 门）。

### 刀③：shim 求值窗换入（engine js_dom_shim）

- part03 `_zwCallHandlerSwapBase` 加第 5 参 nonce：inline/反射/UA 处理器（`_zwCallHandlerDocBase`）传 `''`——spec 事件处理器不继承 nonce，体内 import() 落 None → CSP 拒（红臂）；字符串 timer 传注册期捕获值。swapper 未注册（QuickJS）typeof 降级零变更。
- part01 setTimeout/setInterval 字符串分支：注册期 `__zwSwapImportNonce()` 捕获（同 s91 base 捕获形）——spec 字符串编译继承触发脚本 nonce，fire 在求值窗外（绿臂）。
- part04 R387b 外链经典脚本求值窗换入 `''`（动态插入 script 元素无 nonce 属性）。

## 3. 附带损伤与返工（本刀主返工）

首轮全子树复扫 **4 目标面翻绿，但出现 2 行附带**：`propagate-nonce-external-{classic,module}` Pass→Fail（`TypeError: Failed to fetch module`）。ZWDBG 双探针（resolver 收参 + 逐脚本刷新序列，跑完即撤）定位：

- resolver 收到 `nonce=""`——槽位被后续脚本刷空。刷新序列仅 2 条且**都标 zw_harness**：页面外链测试脚本（`propagate-nonce-external.js`，`nonce="correct"`）经 runner `inline_local_scripts` 内联投递时**丢了 nonce 属性**（`<script data-zw-harness data-inline=…>` 无 nonce）→ webview 刷新读 None。
- 对照：inline 页原标签在页内（nonce 属性在）→ 槽位正确 → Pass；external 页投递丢 nonce → Fail。s90 刀②「原文偏移 ≠ DOM 序号」同族的**投递形态保真缺口**。
- 修复（runner 侧，`tests/wpt-runner/src/testharness.rs`）：内联投递保留原标签 nonce 属性——复用 engine extract 的 `attr_from_open_tag`（升 pub，`pub use extract::*` 根导出），HTML 属性值转义（`&`/`"`）后拼回。上方 harness 注入块 `csp_nonce_attr` 同款保真先例（8917 行）。
- 修后两页复跑翻绿，三扫确认零附带。

## 4. 验证

- **定向**：nonce 两页 12/12 全绿（4 must-not-inherit F→P + 8 inherit 臂保持绿——nonce 传播语义在 CSP 检查开启下成立）；propagate-nonce 4 页 3 绿 1 红（inline-module 红 = s91 前既有 module 路径面，非本刀面，见挂账）。
- **全子树 bitwise 三扫**（同 FILTER，test-guard 包裹）：s91 基线 197P/295F/25T → 首扫 199P/293F/25T（+4 目标翻绿 −2 投递附带）→ 修投递后终扫 **201P/291F/25T**（+4P −4F）。终扫 diff vs 基线恰 4 行 F→P、全落 string-compilation-nonce 两页 must-not-inherit 臂；propagate-nonce-external 两行首扫翻红、修后回基线态（不入 diff）——族外零移动。**string-compilation 族终态 46P/0F/0T 完全清盘**（s88 入场时 3P/3F/8T）。域账自此按 **202P/291F/25T** 记（raw 201P + 惯例 +1）。
- **单测六腿**（test-guard `--per-proc-mem 4 --total-mem 8 --time-limit 1500`）：zero-wpt-runner **226**（+76 ignored）、zero-engine **3526/3526**、zero-script-sandbox v8 **364/364**（含新增 2 测）、quickjs **259/259**、zero-webview **800/800**（含 csp_gate 面）。
- **fmt / clippy**：四 crate `--all-targets -- -D warnings` clean（用户 systemd scope 本环境不可用，`ulimit -v 24G + timeout 900` shell 兜底，同 s91）。
- reftest 未跑：本刀无 render/layout 面（script 管线 + CSP 检查点；无政策文档 `check_script` 快路径返 None 零行为变化），沿 s91 先例按影响面口径跳过。
- 运行方式：make `testharness-html-semantics` + FILTER 透传；ZWDBG 探针临时补丁跑完即撤不入提交。

## 5. 挂账残面（本轮不动）

- **propagate-nonce-inline-module 1F**：module 上下文 code-context import( 经 es_module 重写走 `__zw_dynamic_import.call({referrer})` shim 路径 → `__zw_compile_module` 回调，无 CSP 检查点——module 路径 nonce 传播面（该页红因在 s91 前即存在，非本刀附带；与本刀 classic 槽位路径正交）。若后续 CSP 域测试覆盖 module 动态 import，需把检查点对称接进 `__zw_compile_module`/module prelude。
- **module 路径动态 import 的 CSP 检查缺口**：同上——shim 路径完全绕开 script-src 检查（非 nonce 传播问题，是检查点缺位）。
- **`inline_local_scripts` 投递不保留 `type="module"`**：module 形外链脚本内联后按 classic 执行——本刀语料（文件无静态 import/export 语法）不受影响，语义保真缺口记录在案。
- **跨域旁证**：security-csp 域（security-hardening 控制面）语料含 script-src/unsafe-eval 目录——本刀动了 CSP script 检查语义，该域下次跑批建议重跑基线对照（两流域归因纪律：本流不动其控制面）。
- **QuickJS 后端**：维持不支持态，nonce setter/swapper 走 trait 默认 no-op，shim typeof 降级直执行。
