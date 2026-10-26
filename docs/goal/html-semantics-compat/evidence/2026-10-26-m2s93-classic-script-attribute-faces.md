# M2-s93：classic 脚本属性面批刀——HTMLScriptElement.supports + type ASCII whitespace 归一化（9F 清盘）

日期：2026-10-26。归属：html-semantics-compat / s86 切片序③ the-script-element 基面（承接 s92，HEAD 2cf2a5c5c）。

## 1. 面与基线

s92 HEAD 基线（/tmp/s92-final-scan.txt，the-script-element 子树 517 行）：**201P/291F/25T**。本刀两红面（s93 入场探针：module 尾片 43F 簇被 wptserve .py 执行语义缺口封顶——serve-custom-response.py 等 CGI 以静态文本投递，runner 无 .py 执行器，**转向 runner 可达的 classic 属性面**；控制面 s92「下一步 s93 module 尾片盘点」据此修正）：

- **script-supports 5F**：`HTMLScriptElement.supports` 整体缺失（`typeof === 'function'` 即断言失败）——classic/module 两值 + JavaScript MIME 类型/空白沾染/大小写变体 3 组 false 断言。
- **script-type-whitespace 4F（5 例中 1 例基线已绿）**：`type="text/javascript&#x000B;"` 等尾随非 ASCII 空白（VT/NEL/NBSP/U+1680/U+3000）——spec 只剥 ASCII whitespace，脚本应为 data block 不执行；红因 = 引擎 `str::trim()` 剥 Unicode White_Space → 误判 JS → `window.ran = true` → `assert_false` 失败（\x0B/\xA0/ /　 四例；\u0085 例基线已 Pass，投递层对 NEL 有既有处理）。

## 2. spec 依据与刀位

- **supports**（webReader 取回 spec 原文定谳）：HTML #dom-script-supports——`supports(type)` 步骤：type **精确匹配** `classic`/`module`/`importmap`/`speculationrules` 返 true，余 false；spec 明示「不做 ASCII case-insensitive match、不剥空白」。语料（script-supports.html）无 importmap/speculationrules 断言（旧版测试），按 spec 现行四值实现，两相兼容。
- **type 归一化**：HTML prepare-the-script-element——type 值剥首尾 **ASCII whitespace**（TAB/LF/FF/CR/SPACE，#ascii-whitespace）后为空或 JavaScript MIME type essence match。

### 刀①：shim 接口对象 + 静态 supports（engine part03.js）

R3019 接口构造器占位区（HTMLFormElement 行后）加 `globalThis.HTMLScriptElement = globalThis.HTMLScriptElement || function HTMLScriptElement() {}` + 静态 `supports`（四值 `String(type)` 精确相等；`||` 幂等守卫同区习语）。shim/Rust 侧此前零 `HTMLScriptElement` 全局引用（grep 定谳，仅 proto 表字符串名 + 注释），`||` 守卫无冲突面。实例面仍走 `__zwHtmlTagIface` tag→接口名字符串表——完整接口对象/instanceof 同一性**挂账**（本刀只补静态 API）。

### 刀②：type 归一化 ASCII whitespace 收窄（engine extract.rs）

新 `trim_ascii_html_ws`（`trim_matches` 五字符类），`script_type_is_javascript`/`script_is_module` 的 `str::trim` 换用；调用方仅 extract.rs 脚本提取循环两处（grep 定谳），变更面收敛。附带单测 `script_type_strip_is_ascii_whitespace_only`：ASCII 五空白照剥（text/javascript classic / module 两形态）+ 五个非 ASCII 空白例不提取。

## 3. 验证

- **定向**：`FILTER=script-supports` 5/5 全绿、`FILTER=script-type-whitespace` 5/5 全绿（含基线已绿的 \u0085 例仍绿）。
- **全子树复扫**（`FILTER=scripting-1`，test-guard 包裹，同 ZW_TESTHARNESS_RAF_FRAME_DRIVEN）：/tmp/s93-final-scan.txt 540 行 **211P/282F/25T + 22 NotRun**。与基线 diff（name 键分类）：
  - **9 行 F→P**：script-supports 5 + script-type-whitespace 4（\u0085 例基线已绿不在列）——两面全数兑现，**script-supports 族清零、script-type-whitespace 族清零**。
  - **23 行非 Diff 逐行归因（零附带定谳）**：①1 行 noscript Pass = FILTER 口径加宽（the-script-element → scripting-1 多覆盖 the-noscript-element，该页本来就绿）；②22 行（21 NotRun + modulepreload origin-policy 1 Fail）= 超时页 teardown 粒度——**stash 本刀变更、以 s92 代码重建复跑 load-error 族 bitwise 同现（46 行含 22 NotRun，diff 空）**，且当前二进制全扫与单跑该族行集 bitwise 一致；基线文件缺这些行系 s92 扫描会话后台负载使页面早停（file-level Timeout 行两轮同键存在，超时前完成的 subtest 粒度不同）。上轮基线按历史口径重述：the-script-element 子树内 **210P/282F/25T（517 行恒等，+9P −9F）**；域账自此按 **211P/282F/25T** 记（raw 210P + 惯例 +1）。
- **单测**（test-guard `--per-proc-mem 4 --total-mem 8 --time-limit 1500`）：zero-engine **3531/3531**（lib 3527 = s92 3526 + 本刀 1 新测；engine 自 s92 零外来变更，`git log 2cf2a5c5c..HEAD -- crates/engine` 空）。本刀仅 engine 变更，runner/webview/script-sandbox 零改动不另跑腿。
- **fmt / clippy**：zero-engine `--all-targets -- -D warnings` clean（ulimit -v 24G + timeout 900 shell 兜底，同 s91/s92）。
- reftest 未跑：无 render/layout 面（脚本 type 判定 + shim 静态 API；type data block 化只影响脚本执行与否），沿 s91/s92 影响面口径。

## 4. 挂账残面（本轮不动）

- **完整接口对象面**：`HTMLScriptElement` 只有静态 supports——constructor/prototype 链/instanceof 同一性缺（`document.createElement('script') instanceof HTMLScriptElement` 仍 false；proto 表字符串名行为不变）。同族接口对象基建面（`__zwHtmlTagIface` 全表无真实 class）一并挂账。
- **module 尾片 43F 簇**（css-module 20 + json-module 11 + text-module 8 + import-attributes 4）：wptserve .py 执行语义缺口封顶——runner 实现 CGI 执行器（serve-custom-response.py/headers.py 等）前不可达。含 propagate-nonce-inline-module 1F 与 module 路径动态 import CSP 检查点缺位（s92 挂账承袭）。
- **execution-timing 80F / 顶层 65F**：numbered 套件 scheduler 语义（既有账）。
- **runner 报告粒度抖动**：超时页 teardown 粒度受会话负载影响（基线扫描建议空载跑）——本刀已将 s93 终扫定为下轮新基线（/tmp/s93-final-scan.txt，FILTER=scripting-1 口径，540 行 211P/282F/25T + 22 NotRun）。
