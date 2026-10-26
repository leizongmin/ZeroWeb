# M2 切片 103：module graph 取回/解析失败面——元素 error + 体不执行 + 解析失败 window TypeError

日期：2026-10-27。范围：`crates/webview/src/webview.rs`（fetcher 路径 graph 失败分派替换空存根兜底 + 2 单测）+ `crates/engine/src/js_dom_bridge/script_gen.rs`（`script_report_module_failure` 加错误构造器形参 + 新 `script_dispatch_script_event_by_ordinal` helper）。上轮：[M2 切片 102](2026-10-27-m2s102-module-failure-report-face.md)。

## 1. 入场定谳

- s102 残余①：dep 级取回失败仍空存根兜底——module 图内任一 dep fetch 失败 `collect_module_deps_recursive` 返 Err，旧版 `registry.register(spec, "")` 空存根后模块体**照常执行**，网络级失败面丢失。
- spec 语义（fetch-a-single-module-script / resolve-a-module-specifier）：图内取回失败 → 元素 fire **error** + 体不执行 + 无 window 报告（网络错误优先于 parse 错误）；图内 specifier 解析失败 → 向 window 报 **TypeError** + 元素照常 fire **load** + 体不执行。
- 判据页：module/error-type-1「network error has higher priority than parse error」——两模块元素，第一（syntaxerror.js parse→window SyntaxError+onload=log.push(1)）s102 已达；第二（error-type-1.js `import './syntaxerror.js'; import './404.js';`）需图内 404 → onerror=log.push(2)、无 window 报告，log.length==3。module/specifier-error——bad-module-specifier.js 裸 specifier → window error ev.error=TypeError 实例 + log.length==1（体不执行）+ 元素不 fire error。
- 关键事实：prelude 急切求值非入口注册模块（es_module.rs R5182）——体不执行必须连 prelude 一并跳过，否则已取回依赖（error-type-1 第二图的 syntaxerror.js）经 prelude 抛 parse 错误多报一次 window error、且其元素 onload 误触 `unreachable()`。

## 2. 接线（三刀）

- ①engine `script_report_module_failure(msg, ordinal, error_ctor)`：加错误构造器形参（编译/求值面传 `"SyntaxError"`，解析面传 `"TypeError"`）；s102 两调用点同步。新 `script_dispatch_script_event_by_ordinal(ordinal, ty)`：无 src 内联模块的元素级事件派发（序号口径同 report helper）。
- ②webview fetcher 分支：收集循环 `is_dep=true`（此处 spec 均为被运行模块的依赖——模块自身已取回/内联不经此处；旧 `is_dep=false` 使裸 specifier 误作相对 URL 取回、失败误归 TopFetch），首 Err 捕获 `(ModuleGraphFailure, spec)` 即 break；有失败按类分派——`DepResolution` → report helper（TypeError 面，元素 load）；`TopFetch|DepFetch` → 元素 error（外链 `script_dispatch_script_event(&module_cache_key)` R2944 src 匹配——`module_cache_key`=src 绝对化；内联 ordinal 派发）后 `continue`（编译+prelude 一并跳过）。无 fetcher 路径与动态 import 契约路径（`compile_module_contract`，is_dep=false 语义由 JS 侧按取回失败处理）不变。
- ③webview +2 单测：`module_dep_fetch_failure_fires_element_error_and_skips_body_s103`（dep 404 → 元素 onerror + 体 undefined + 无 window error + 页面继续）、`module_dep_resolution_failure_reports_typeerror_and_skips_body_s103`（裸 specifier → window ev.error=TypeError + 元素 onload + 体 undefined + 页面继续）。

## 3. 验证

- 单测：zero-webview lib **806/806**（+2）；zero-engine lib **3533/3533**；fmt clean；clippy webview+engine `-D warnings` clean（ulimit s91 口径）。
- 定向：**error-type-1 1F→1P**（log=[SyntaxError,1,2] 链闭合——第二模块图内 404 只 fire 元素 error、syntaxerror.js 不经 prelude 二次报错）、error-type-2 保 P、error-type-3 残余 Fail（instantiation 优先级，尾片）；**specifier-error 1F→1P**。
- 全子树终扫（scripting-1 口径 593 行）：**348P/193F/37T + 15 NotRun**（s102：342P/180F/41T+27N；净 +6 Pass）。
- **P→非P 5 页逐页归因**（全部为 s102 存根假阳的诚实化，非契约回归）：
  - css/json/text-module load-error-events ×3：每页恰 `inline, 200, parser-inserted` 一子测试 P→F（import attributes 拉 basic.css/data.json/text.txt 的 load 期望臂）——同一 fetcher 同一语料下 s103 使元素 fire error ⇒ fetch 在 s102 同样失败被存根掩盖；`src, 200` ×2 两版同 Fail、404 臂全保 P。引擎无 CSS/JSON/text module + import attributes 支持，该臂诚实状态即 Fail。
  - module/nested-imports P→F：d.js 依赖 `./resources/delayed-modulescript.py`（服务端 CGI 端点）——s102 Pass 系存根使图平凡满足的假阳（测试本要求延迟加载服务端模块）；s103 如实报元素 error → `unreachable()` 断言失败。.py 执行器缺口重暴露。
  - json-module/parse-error P→T：`parse-error.json` **文件不在语料**——s102 Pass 系「空存根→export 检查失败→碰巧 SyntaxError 面」双重假阳；s103 元素 error + 体跳过 → 测试未注册 → Timeout。JSON module + import attributes 缺口重暴露。
- **连带 F→T 1 页归因**：json-module/parse-error-location.tentative F→T（同上体跳过、测试未注册——memory 坑 7 同类，状态更保真）。
- **增益**：F→P 3 页（error-type-1、specifier-error 目标 + fetch-error-2）、T→P 1 页（import-subgraph-404——s102 F→T 族翻回真 Pass）、N→P 2 行（module/load-error-events-inline）、T→Fail+Pass 3 页（css/json/text integrity——404 臂真 P）、N→F+P（css-module/content-type-checking 2P 出现）、T→F 2 页（http-accept-header.sub）、N→F 2+5 行（json/text module.html、valid-content-type）。

## 4. 范围与挂账

- 触碰面：webview.rs（fetcher 分支 graph 失败分派 + 2 单测）+ script_gen.rs（error_ctor 形参 + ordinal 派发 helper）；shim/语料/runner testharness.rs 零变更；语料不可改维持；`compile_module_contract` 动态 import 契约路径不变。
- 残余（归 module 尾片）：①. py 执行器（nested-imports/integrity 族 load 期望臂、http-accept-header）；②import attributes + CSS/JSON/text module 支持（load-error-events 200 臂、valid-content-type、content-type-checking Fail 面、parse-error 族）；③非 Error 抛出值 ev.error=null 身份（error-type-3）；④动态 import 已错模块 reject 形（instantiation-error-1）；⑤error-type-3 instantiation 优先级。
- 承袭挂账：116 frameset 文档语义缺口、063 Timeout、charset 族（Timeout 形）、svg-script-fragment-clone 3F、FIXME 组、接口对象基建面。
