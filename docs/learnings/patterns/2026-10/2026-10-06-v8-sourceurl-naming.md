---
date: 2026-10-06
modules: script-sandbox,renderer,engine
---

# V8 无名脚本命名：sourceURL 注释的末行规则与失效形态

## 问题描述

ZeroWeb 的 sandbox 执行通路（`v8::Script::compile(try_catch, code_str, None)`，无 ScriptOrigin）下所有站点脚本无名，异常栈帧全显 `<anonymous>`——bilibili video 页 6× split TypeError 只有 `N @ <anonymous>:2:16812`，无法定位到 bundle，诊断停滞（siteopt r1 P-B3）。

修法是源尾追加 `//# sourceURL=<url>` 注释（V8 对无名脚本取 sourceURL 为脚本名），但生效有苛刻条件，且经典页面脚本的 eval wrapper 会静默破坏它。

## 根因分析

1. **V8 仅在源「真末行」取 sourceURL**。注释之后任何非注释文本都会顶掉它：
   - 注释后还有代码行 → 命名被污染（URL 仍可从 stack grep，行号错位）。
   - **strict 顶层 `var` 场景完全弃用**：`script_run_classic_page` 的 R201 accessor 导出后缀（`+';Object.defineProperty(...)'`）拼接在 eval 源上，注释被顶离末行后 V8 对该 Script **直接放弃 sourceURL 回 `<anonymous>`**（比污染更彻底；node vm 同款 V8 代码路径实测）。WPT strict 测试库（dom/common.js 等）正中此型。
2. **执行源行号映射**：js_worker Execute 臂有 `__zw_begin_script && __zw_begin_script();` 前缀占第 1 行 → 执行源行 N = 脚本文件行 N-1。列号不受影响，定位 bundle 内位置时必须做此映射。
3. **module 路径无效**：`es_module` split_statements 会剥离 `//#` 注释——module 的命名需走 ScriptOrigin（未实现），append 无害但无效。
4. QuickJS 忽略 `//# sourceURL`（优雅降级，零语义影响）；注释不改变脚本语义，仅命名。

## 解决方案

- **直接执行路径**（无 wrapper，如动态脚本 tick）：`page_scripts::append_source_url` 源尾置注释；URL 控制字符剔除（换行会把注释后文本变回可执行代码——页面自伤面）。
- **classic wrapper 路径**：URL 作为参数穿进 `script_run_classic_page(code, script_index, source_url)`，注释并入**导出后缀字符串字面量的末尾**（eval 源真末行）——不是在 fetch 点 pre-append（会被后缀顶离）。
- 回归钉：普通形态 + **strict var 形态都要钉**（后者才是真正会静默失效的路径，变异核验显示禁掉传参后 strict 钉 FAIL、普通钉仍 PASS——普通钉守不住这个缺口）。

## 如何避免

- 给 eval 串拼接任何后缀（导出、插桩、sourceURL 自身）时，先问「V8 还认得末行注释吗」；后缀生成器与注释放置必须同点决策，不能分层各自 append。
- 诊断「栈帧全匿名」先查执行通路是否有 ScriptOrigin / sourceURL，再查注释是否真在末行——`//# sourceURL` 写了但不在末行与没写一样（strict var 场景更糟）。
