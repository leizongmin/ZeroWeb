# M2-s90：string-compilation 动态 import() 字符串编译面（classic 三刀闭环）

日期：2026-10-26。归属：html-semantics-compat / s86 切片序③ the-script-element 基面（承接 s88 classic 动态 import() 宿主回调）。

## 1. 面与基线

s89 HEAD（44e23748f）基线全子树复扫（`make testharness-html-semantics FILTER=scripting-1/the-script-element`，test-guard 包裹）：**158P/294F/33T**（485 行）。string-compilation 族基线拆解：

- **string-compilation-classic**：eval 臂 1P（s88 收割）+ 整页 1T——setTimeout 臂（字符串 handler 被静默丢弃永不 fire）堵死 harness 串行队列，余 4 臂 pending 被 T 掩盖。
- **string-compilation-of-promise-result** 2F+2P：successful-import 两臂红（脚本执行中换 `<base>` 后 import 基 URL 走错），failed-import 两臂绿。
- **string-compilation-base-url-inline-classic** / **string-compilation-nonce-classic**：整页 2T（base URL 语义缺位叠加 timer 面）。
- **string-compilation-{module,-nonce-module,-base-url-inline-module,-base-url-external-module}** 4T + module eval 臂 1F、**base-url-external-classic** 1T：module 上下文页面基座 / 外链动态加载器，本轮不动。

## 2. 三刀

### 刀①：setTimeout/setInterval 字符串 handler（engine shim）

`crates/engine/src/js_dom_shim/part01.js`：字符串 handler 原被 `typeof fn !== 'function'` 早 return 静默丢弃（计时器永不 fire）。补 `if (typeof fn === 'string') fn = new Function(fn);`——spec TimerInitializationSteps 的 WebIDL callback 转换在注册期同步编译（语法错同步抛，同 Chrome）；其余非 function 维持静默 no-op。
https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#timer-initialisation-steps

效果：string-compilation-classic 5 臂全显（T→拆臂），nonce-classic/base-url-inline-classic 解堵。

### 刀②：parse 时刻脚本基 URL 捕获（engine extract + webview 接线）

- `crates/engine/src/pipeline/extract.rs` 新增 `script_effective_base_urls(html, page_url)`：zero_dom 树序（pre-order DFS）行走，遇 `<base href>`（非空、首个生效、hrefless 跳过）更新 current_base，遇 `<script>` 记当前基（无 base 落 page_url）。与 `extract_page_scripts_indexed` 同序号空间（template 内脚本计入）。纯函数于原文——后续 `<base>` 动态变更天然不回溯（spec 脚本基 URL = 创建时刻文档基 URL）。
- `crates/webview/src/webview.rs` `run_page_scripts_impl`：按 PageScript 形态算 referrer——外链经典脚本用自身 src 绝对 URL、其余取捕获表 `script_bases[script_index]`；s88 的 `set_dynamic_import_referrer` 每脚本槽位改供此值。
- 中途返工记录：首版在 webview 侧按原文偏移扫 `<base>`/`<script>`，DEBUG 探针实证 parent=document URL——**原文 `<script` 文本出现 ≠ DOM 序号**（runner 注入 harness 脚本 + JS 字符串字面量含 `<script` 过计数），改 zero_dom 树序行走（移 engine，webview 无 zero_dom 依赖）。

https://html.spec.whatwg.org/multipage/webappapis.html#concept-scripts-base-url

效果：promise-result 2F→P（eval/Function 臂 import 落对基）、base-url-inline-classic setTimeout/eval/Function 3 臂→P。

### 刀③：inline/反射事件处理器求值窗换基（shim + script-sandbox + webview）

反射/UA 触发的 inline handler 无 active script——体内 import() 基 URL = **求值时刻当前文档基 URL**（非任何脚本捕获基；`<base>` 动态变更即时生效）。webview 回调闭包无法在 JS 执行中借 `&mut sandbox`，故走回调面：

- `crates/script-sandbox/src/v8_runtime.rs` 新增 `register_dynamic_import_referrer_swapper()`：注册全局 `__zwSwapImportReferrer(url)`——返回旧 referrer 并换入新值（捕获 `DynamicImportState` Arc，不经 `&mut self`；与 resolver 同理由回调面承载）。`lib.rs` Sandbox trait 同名默认 no-op（QuickJS 降级）。
- `crates/webview/src/webview.rs`：script_source_fetcher 就绪分支注册该回调（同 resolver 门）。
- `crates/engine/src/js_dom_shim/part03.js`：
  - `_zwCallHandlerDocBase(fn, thisArg, ev)`：求值窗 swap 进 `_zwDocBaseURL()`（每调用动态解析首个 `<base href>`，含 pending 增补）、finally 换回（零外溢——后续 timer/模块回调不被污染）。
  - `_zwWrapHandlerDocBase(fnC)`：编译产物包装（参数绑定——见下返工记录）。接入四处：`_ensureInlineHandler`（缓存编译，getter/dispatch/setAttribute 重编译共用）、`_zw_reflect_body_window_handlers`（body on* → window 反射）、R155 `node.click()` 直编译两处（onclick + oninput/onchange 显式编译）。
  - 宿主回调未注册（QuickJS 后端）→ typeof 检查降级直执行，零变更。

返工记录：body-reflect 循环首版 `var fnB` 直闭包捕获——包装在循环后调用，读到末轮（onresize）编译体，engine 单测 `test_body_onload_reflection_r2946` 红（onload 执行的是 resize 体）。改 `_zwWrapHandlerDocBase` 参数绑定修复（part03 无 let 先例，循 var 风格）。

https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-processing-model

效果：base-url-inline-classic reflected/UA 2 臂→P（该页 5/5 全绿）。

## 3. 验证

- **全子树 bitwise 复扫**（同 FILTER，基线/修复后双扫，修后 497 行）：**158P/294F/33T → 173P/294F/30T**（+15P −3T）。diff 恰 31 行全落 string-compilation 族内，族外零移动：
  - +20P：base-url-inline-classic 5、classic 4、nonce-classic 继承臂 4、promise-result 2（successful-import）、nonce-classic must-not-inherit 2（拆臂显形——基线整页 T 掩盖的既有红面，非回归）+nonce 显形计内；
  - −2F：promise-result successful-import 转 P；
  - −3T：classic / base-url-inline-classic / nonce-classic 整页 T 消除（拆臂无 pending 残留）。
  - 族终态：**18P/2F/5T**（基线 3P/3F/8T）。
- **单测四腿**（test-guard `--per-proc-mem 4 --total-mem 8 --time-limit 900`）：zero-engine **3526/3526**、zero-script-sandbox v8 **358/358**、quickjs **255/255**、zero-webview **800/800**。webview 首跑 1F 系 `target/debug/zero-image-decoder` 独立进程二进制被 target-disk-guard 清理（直跑单 crate 未带 make test 前置 build 腿），补建后全绿——非代码回归。
- **fmt**：`cargo fmt --all -- --check` clean（extract.rs/webview.rs 两处 rustfmt 归并）。
- **clippy**：`cargo clippy -p zero-script-sandbox -p zero-webview -p zero-engine --all-targets -- -D warnings` clean（test-guard 包裹）。
- **reftest**：`make reftest` rc=0，716/716 通过。
- 运行方式：make `testharness-html-semantics` + FILTER 透传（test-guard `--per-proc-mem 4 --total-mem 8 --time-limit 5400`）；探针页（s90probe-*）跑完即删，不入提交。

## 4. 挂账残面（本轮不动）

- **string-compilation-{module,-nonce-module,-base-url-inline-module,-base-url-external-module} 4T + module eval 臂 1F**：module script 页面基座（`<script type="module">` 页加载/评估链路）——module 面单独探，非 string-compilation 语义。
- **string-compilation-base-url-external-classic 1T**：dynamic `createElement('script')` 外链页面加载器（script_source_fetcher 页面 fetch 面）。
- **nonce-classic must-not-inherit 2F**：CSP nonce 语义（handler 不继承 nonce 须拒）引擎未建模——CSP 面（s88 已挂账）。
- **QuickJS 后端**：classic 动态 import() 维持不支持态，`__zwSwapImportReferrer` 注册为 trait 默认 no-op，shim typeof 降级直执行（契约单测钉死 v8-only 面）。
