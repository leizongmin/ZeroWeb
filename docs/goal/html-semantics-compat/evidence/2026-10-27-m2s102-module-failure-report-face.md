# M2 切片 102：module 失败报告面——编译/求值失败向 window 报异常 + 元素 load + 页面继续

日期：2026-10-27。范围：`crates/webview/src/webview.rs`（编译失败面 + 执行期失败面 narrow + 外链取回失败元素 error 投递）+ `crates/engine/src/js_dom_bridge/script_gen.rs`（`script_report_module_failure` helper）。上轮：[M2 切片 101](2026-10-27-m2s101-external-module-import-base.md)。

## 1. 入场定谳

- s101 终扫 73 行 `page script threw`，35 页 declared=0——module 编译/求值失败在 strict 下整页 abort，掩盖页面其余用例（runner「空洞」面）。
- spec 语义（run a module script）：fetch **成功**后的编译/链接/求值失败 → 向 window 报异常（window error 事件，ev.error 为抛出值；parse 失败为 SyntaxError）+ 元素照常 fire **load** + 页面继续；仅**网络级取回失败** → 元素 fire **error**。WPT 判据页：module/error-type-1（parse error → window SyntaxError + 元素 onload；network error → 元素 onerror）、instantiation-error-3/4/5（链接错误 → window SyntaxError + 元素 load）。
- 旧实现三面缺失：①编译失败 strict abort（`return Err`）；②执行期异常 strict abort（`combined_err` 面）；③外链 classic/module 取回失败仅 warn 静默跳过——classic 臂注释已声称 spec error 面但实现未投递。

## 2. 接线（四刀）

- ①engine `script_report_module_failure(msg, ordinal)`：`reportError(new SyntaxError(msg))`（shim reportError part01——ev.error 携 Error 实例）+ `getElementsByTagName('script')[ordinal].dispatchEvent(new Event('load'))`（文档序 script 序号 = extract_script_csp_info 同口径，适配无 src 的内联模块；外链模块走既有 R2944 src 匹配派发）。
- ②webview 编译失败臂（compile_module_script Err）：strict abort → 报告面 + warn + continue（strict 亦不 abort——prepare 阶段失败面非内联体抛错，与外链取回失败同类）。
- ③webview 执行期失败臂（`combined_err`）：`strict && !is_module` 才 abort；module 执行期异常（求值抛错/变换后代码 parse 失败——compile_module_script 变换为词法面不做全量 parse，error-type-1 的 syntaxerror.js `%` 体过变换、sandbox parse 期炸实证）→ 同①报告面 + 继续。
- ④webview 外链取回失败（classic + module 两臂）：补 `script_dispatch_script_event(&abs, "error")` 元素 error 投递（R2944），warn + continue 不变。

## 3. 验证

- 单测：zero-webview lib **804/804**（+2：`module_compile_failure_reports_window_error_and_continues_s102`——strict 下 window error 收 SyntaxError 实例 + 元素 onload 达 + 失败模块后 classic 照跑；`external_module_fetch_failure_fires_element_error_s102`——取回失败元素 onerror 达 + 页面继续）；zero-engine lib **3533/3533**（含他流并入用例）。
- 定向：FILTER=error-type——error-type-2 **1F→1P**、error-type-1/3 从整页 abort（declared=0）转为子测试级 Fail（用例真正运行断言）；FILTER=instantiation-error——**3/4/5 三页 1F→1P**、1/2/6/7 子测试级 Fail、8 Timeout；FILTER=specifier-error——子测试级 Fail（bare specifier 运行时 TypeError 面残余，归尾片）。
- 全子树终扫（scripting-1 口径 590 行）：**342P/180F/41T + 27 NotRun**（s101：326P/220F/14T+18N）；行级 diff：**F→P 12 页**（choice-of-error-1/2、error-type-2、fetch-error-1、instantiation-error-3/4/5、nested-imports、parse-error、error-and-slow-dependency、css-module/script-element-css-src、text-module/script-element-json-src）、**零 P→非P**（comm 校验 /tmp/s102-diff.txt）。
- **F→T 27 页归因**（行为面变化，非回归）：原 declared=0 abort 族（css/json/text-module charset/integrity/referrer/http-accept + module/referrer-*.sub + import-subgraph-404 + slow-cycle + instantiation-error-8 + module-import-referrer）——页面从「首个失败模块处整页 abort」变为「报告后跑完」，测试体因 feature 缺失（import attributes `with{type:"css"}`、.py CGI 端点、referrer 语义）从未注册 → runner「completed without reporting registered tests (tests:0)」映射 Timeout。状态更保真（window error 已报、页面继续），行形由信息量低的 abort Fail 变 Timeout；翻回需 import attributes/.py 执行器（尾片）。
- fmt clean；clippy `-D warnings`（engine + webview，ulimit 兜底 s91 口径）；runner testharness.rs 零变更；reftest/product-smoke 未跑（无 render/layout 面）。

## 4. 范围与挂账

- 触碰面：webview.rs（两失败臂 narrow + 取回失败投递 + 2 单测）、script_gen.rs（helper）；shim 零变更（reportError/ErrorEvent 既有面直接复用）；语料不可改维持。
- 残余（本面已暴露、归尾片）：①dep 级取回失败仍空存根兜底（error-type-1 的 404.js 臂需「图中任一取回失败 → 元素 error」语义——动 M2-S4E 存根契约，需独立切片全扫审计）；②非 Error 抛出值经 reportError 置 ev.error=null（error-type-3 对象身份断言）；③bare specifier 运行时 TypeError 面（specifier-error）；④动态 import() 对已错模块的 reject 形（instantiation-error-1 的 promise_rejects_js TypeError vs SyntaxError）。
- 承袭挂账：module 尾片（.py 执行器前置 + import attributes + 上述残余）、116 frameset 文档语义缺口、063 Timeout 承袭、charset 族（现 Timeout 形）、FIXME 组、接口对象基建面。
