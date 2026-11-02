# M2 s250 — text-module integrity matches 臂 + load-error-events 四页 src 臂族 + s100 拦截面 module 型豁免 + 动态 src module 执行通道 rev3（12 键翻绿，第九个连续完美 A/B）

日期：2026-11-03。切片号 s250（承 s249）。根：s249 下一步候选
两项——text-module integrity matches 臂 + load-error-events 非
inline 臂勘根；施工中扩及 s100 拦截面 module 型误伤定谳（本切
最深根）。

## 入场定刀位

s249 收口后 the-script-element 数据模块族残余形态：text-module
integrity.html matches 臂 F（`import text from "./file" with
{ type: "text" }` → 404）；load-error-events 四页（text-module/
json-module/css-module/module 五变体）inline 4 臂已翻绿、src 臂
（200/404 × parser/dynamic）F 或 NotRun、module/ 变体文件级
Timeout（仅 4 动态臂 Pass，静态臂 pending 不可见）；另有
`inline, 200, not parser-inserted` ×3（text/json/css）NotRun。

## 勘根（代码链逐环定谳）

- **text 型 MIME 校验缺口（integrity matches 臂根）**：s249
  fetcher `__zw_mtype` 臂对 text 型误套 `text/plain` essence 校验。
  spec fetch-a-single-module-script step 7.2：「moduleType 为 text
  → **无条件**创建 text module script」——css←text/css、json←JSON
  MIME type、js←JavaScript MIME type 三型才校验响应 MIME，text 无
  校验。上游判据页 text-module/ignore-content-type.any.js 同口径
  （裸 `file` wptserve 实测交付 application/octet-stream 仍可
  import；Edge 158 stable 2/2 PASS 佐证）。误校验把 integrity
  matches 臂打成网络错。修正一行（essence 校验仅 css 分派，text
  直通）。
- **语料缺口**：`text-module/file`（无扩展名语料文件）上游存在、
  本地缺失——fetch-wpt 按 MIME 扩展名白名单过滤漏收无扩展名文件
  （承 s249 redirect-target.css 同坑）。从上游 wpt 同路径原件补入
  10 字节 `text file\n`；update-wpt-data.sh 挂账注记（升级新 ref
  后需重新补）。
- **load-error-events.py 端点缺失（四页 src 臂根）**：上游
  wptserve CGI handler 不在语料 pull 面，运行时取回落通用缺文件
  404。上游四风味（solo diff 四页定谳）：
  - `text-module/load-error-events.py`：`*_load` 体 = `import
    "./file.txt" with { type: "text" }; N.executed = true;`、
    `*_error` 体 = import `./not_found.txt`（本体 200，error 由
    内部 import 取回失败触发，体不执行）；
  - `json-module/load-error-events.py`：module.json /
    not_found.json + `type: "json"` 同构；
  - `css-module/resources/load-error-events.py`：basic.css /
    not_found.css + `type: "css"` 同构；
  - `the-script-element/resources/load-error-events.py`：素模块
    风味——`*_load` = `"use strict"; N.executed = true;`、
    `*_error` **本体直接 404**（上游 return status 字面）。
  `test` 查询参数校验 `^[a-zA-Z0-9_]+$` 与上游 assert(re.match)
  同口径，缺席/违例 → 500 语义（fetcher Err）。runner fetcher 单
  臂建模（serve-with-content-type.py s115 同法——.py 端点不在磁
  盘，fetcher 内按路径+查询分派回体）。
- **s100 拦截面 module 型误伤（module/ 变体文件 Timeout 根，本切
  最深）**：runner `inline_local_scripts` 预处理对
  `the-script-element/resources/load-error-events.py` 无条件 s100
  经典族仿真回体（判定先于 `is_module_type` 计算）→ module/ 变
  体 4 静态 src 臂被**内联投递**为 type=module 单元，s100 投递尾
  `(function(){ var _cs = document.currentScript; if (_cs && …)
  _cs.dispatchEvent(new Event('load')) })()` 在 module 单元内
  `document.currentScript` 恒 null（spec currentScript getter 对
  module script 返 null）→ `if (_cs && …)` 静默空转 → load/error
  事件均不派发 → 四臂 pending → 文件 Timeout。**probe 三态实验
  定谳**：①src 上溯出界（`../resources/` → scripting-1/resources/
  不匹配 s100 精确路径）→ 真外链 404 → error 事件正常派发（排除
  事件机制故障）；②同结构探针改指 text-module 端点 → 双臂 Pass
  （排除页面结构与 helpers 问题）；③s100 精确路径 → 双臂 pending
  复现。修复 = s100 判定加 `!is_module_type` 豁免（判定上移到
  s100 前计算复用；serve-with-content-type 臂既有同款门先例）→
  module 单元维持外链 → 引擎静态模块链 + fetcher 四风味臂 → 引
  擎原生模块事件机制派发。经典族 load-error-events-1.html（无
  type 属性）不受豁免影响零回归；css-module 页 src 指自身
  `css-module/resources/` 不匹配 s100 精确路径，无暴露面。
- **动态内联 module 成功体执行上下文（inline,200 臂残余根，未修
  挂账）**：`inline, 404` 臂（编译期依赖失败）经本切 Leg C 状态机
  翻绿；`inline, 200` 成功体（`onExecute(test4_dynamic_load)` 引
  用严格页面 var）在 shim eval 上下文 ReferenceError。8 轮探针定
  谳：严格模式页面脚本 `var` 绑定活在 runner 共享 replay 作用
  域——shim 侧任何 eval（直接/间接）与 window 属性通道均不可见；
  静态路径（parser 内联/外链 module）可用是因 host 在脚本阶段上
  下文执行编译体。详见残余挂账。
- **动态 src module 执行通道设计迭代（A/B 回归驱动的三版收敛，
  rev3 定稿）**：rev1（setTimeout 延迟编译 + shim 裸 eval +
  `__moduleCache` 直写）首轮全树 A/B 揭 4 P→F——string-
  compilation-base-url-external-module ×3（`import()` 解析基漂
  移：旧「沿用既有通道」= classic fetch+eval 经
  `_zwExecuteFetchedScript` 换 referrer/base 槽位，rev1 裸 eval
  丢槽位 → 模块体 import() 相对 spec 解析到文档基）+ execorder
  dynamic async-ordered（async=false in-order 队列未建模按 asap
  破序）。rev2（同步编译 + 直经执行器/队列）修基与队列但**同步
  时序**翻转 load-after-execution 交错断言（css/json/text/module
  五臂 src,200 动态 F）——classic 取回时序 = setTimeout 延迟、
  执行器收尾派 load，同步化改变事件与执行交错。rev3 收敛：**队
  位调用时刻入队（spec list 位次 = 插入序）+ setTimeout 内延迟
  编译 + 编译体经 `_zwExecuteFetchedScript`（asap）/
  `_zwInOrderScriptQueue` 条目 ready 置位（async=false）**——基、
  队列、时序三面同时复位，且 text-module src,200 动态臂（rev1
  NotRun）随执行器上下文一并翻绿。缓存直写移除：同一 URL 的
  script 元素插入与 import() 模块记录独立（spec 每元素新记录），
  `__zw_load_module` 按需自取回。

## 施工（runner 4 + webview 1 + shim 3 处 + scripts 1 + 语料 1）

- **runner testharness.rs**：
  1. `__zw_mtype` text 臂 MIME 校验豁免——essence 校验仅 `css`
     分派（`text/css`），text/js 直通（spec step 7.2 判据 + 注释
     修正 s249 text/plain 误注）；
  2. `load-error-events.py` 四风味端点臂——file_name 门 + test
     参数校验（空/非法 → Err）+ 四 flavor 路径包含分派回体
     （text/json/css 数据模块体逐字镜像上游；素模块 `_load` 回
     体、`_error` 直接 Err 404）；
  3. s100 拦截面 `!is_module_type` 豁免 + `is_module_type` 判定
     上移复用（s115 serve-with-content-type 门同款）；
  4. 单测 ×2：`script_fetcher_text_module_mime_agnostic_s250`
     （裸 `file?__zw_mtype=text` Ok + 原文交付）+
     `script_fetcher_load_error_events_endpoint_s250`（text
     `_load`/`_error` 体镜像、素模块 `_error` Err、非法 test 名
     Err）。
- **webview webview.rs**：`__zw_compile_inline_module` 回调——
  动态内联 type=module prepare 按静态面同链：属性校验（s105）→
  静态 import 收集（s107 带属性）→ 依赖图递归（is_dep=true，
  s103 失败分类 DepResolution/网络错分派）→ `registry.register`
  入口键 `{parent}#!zw-dyn-inline`（s94 内联缓存键同形）→
  compile_dependency_iife。契约四态（`\u{1f}` 分隔）：""
  依赖图取回失败 / `1\u{1f}` specifier 解析失败 / `2\u{1f}<msg>`
  属性校验违例 / `1\u{1f}<iife>` 编译成功。
- **engine part03.js**：①s109 type 门 `_zwScriptTypeClassify`
  classify 化（删 `ty.indexOf('module')` bail）；②s109 内联 module
  分派臂——动态插入内联 type=module → run-once 记账 → 基 URL
  （`_zwDocBaseURL()` 回落）→ 同步调回调 → 四态状态机（空→
  setTimeout trusted error；tag 2→`_zwReportListenerError`
  SyntaxError + load；tag 1 空码→load（FIXME 报告面）；tag 1
  体→eval，catch → `_zwReportListenerError`）；③
  `_zwPrepareSrcScriptGate` module 分支（rev3 定稿，迭代见勘根）
  ——动态 src type=module（此前整类 classic fetch+eval 通道）→
  让位旗标/run-once 门 → s110 队列分支判定（async 属性/
  force-async 表同口径）→ 队位调用时刻入队（async=false）→
  setTimeout(0) 内 `__zw_compile_module(url, base, '')` 三态分派：
  空→队列 failed/直派 trusted error；空码→队列空码/直派 load；
  成功→队列条目 ready+code 后 `_zwDrainInOrderScriptQueue`（复用
  classic FIFO 头排水）/asap 直经 `_zwExecuteFetchedScript`（换
  referrer/base 槽位 + load 收尾），求值抛错 catch → window 报告
  仍 load（队列深层抛错归因不保 FIXME）。
- **scripts/update-wpt-data.sh**：text-module/file 缺失挂账注记
  （承 redirect-target.css 先例格式）。
- **语料**：`text-module/file`（10 字节 `text file\n`，wpt-data
  侧本地补入，主仓不落）。

## 验证

- solo（FILTER=scripting-1/the-script-element，RAF frame-driven
  + test-guard，evidence/logs/s250-solo.log）：**979P/79F/25T/
  10N**（对 s249-solo-postpull 同滤波 963P/86F/26T/15N：ΔP=+16、
  ΔF=−7、ΔT=−1、ΔN=−5）。逐族：module/ 变体 8 臂全 P（4 静态臂
  pending→Pass + 文件 Timeout 解除）；text-module 全 8 臂 8P
  （src,200 动态臂随 rev3 执行器上下文 NotRun→P + integrity
  matches F→P）；json-module/css-module src 臂全 P（含 src,200
  动态臂）；四页仅余 inline,200 动态 NotRun（挂账）；module/
  dynamic-import string-compilation-base-url-external-module ×3
  + execorder dynamic async-ordered rev2 回归在 rev3 全部复位；
  经典族 load-error-events-1/2/3 形态不变（s100 豁免零波及）。
- 单测：runner 281 + webview 835 全绿（image_decoder proxy 案首
  轮失败为 zero-image-decoder 二进制不在 target/debug——
  target-disk-guard 分级清理环境面，`cargo build -p
  zero-image-decoder` 后 4/4 绿，非代码回归）；fmt --check 干
  净；clippy -p zero-webview -p zero-wpt-runner 干净。
- 全树 A/B（evidence/logs/s250-ab.log vs s249-ab.log）：行级
  7757P/458F/78T/21N → **7773P/451F/77T/16N**（ΔP=+16、ΔF=−7、
  ΔT=−1、ΔN=−5）。全键 join（子测试整键，last-wins + LC_ALL=C
  sort/join/comm 同序）**8313/8313 对，LOST 13 / GAINED 16 全部
  归因**：LOST = 12 翻绿键旧态形（Fail/NotRun）+ module/
  load-error-events.html 文件级 Timeout 键；GAINED = 同 12 键新
  Pass 形 + module/ 变体 4 静态臂新报键（s249 文件 Timeout 态
  pending 不可见，Timeout 解除后 8 子测全报——4 动态键 s249 已在
  （Pass 保持）不入 GAINED）。翻转 **7 F→P + 5 N→P，P→F 0**——
  第九个连续完美 A/B：
  - 7 F→P = css-module/json-module/text-module 三页
    `src, 200, parser-inserted` + `src, 200, not parser-
    inserted` ×6 + text-module/integrity.html matches 臂；
  - 5 N→P = css-module/json-module/text-module 三页
    `inline, 404, not parser-inserted` ×3 + module/
    load-error-events-inline.html `src, 404` 动态两臂（defer/
    async 变体名）。
  迭代过程记账：rev1 shim 全树 A/B 曾揭 4 P→F
  （string-compilation-base-url-external-module ×3 + execorder
  dynamic async-ordered），rev2 同步时序换形 5 臂 F，rev3 收敛后
  全部复位——定稿 A/B 零回归（勘根节详）。

## 残余挂账（本切不动）

- **动态内联 module 成功体执行上下文**（`inline, 200, not
  parser-inserted` ×3：text/json/css）：shim eval 不可见严格页面
  var（replay 作用域），8 probe 定谳三通道全败。修复方向 = host
  侧执行编译体（镜像静态链机制）；卡点 = 回调契约
  `Fn(&[String]) -> String` 无 sandbox 捕获 + v8/quickjs 回调内
  重入执行两端 runtime 无先例，需沙盒重入 API 设计（compile+run
  一体回调返状态串方案已草案）。404 臂（编译期依赖失败）本切已
  翻绿，事件语义正确。
- **prelude `__zw_load_module` 直接 eval 同型潜伏面**：动态
  import（非 script 元素路径）模块体 eval 同处 shim 上下文——引
  用严格页面 var 的动态 import 模块预期同病，勘根时一并核。
- **动态 src module specifier 解析失败 window TypeError 报告
  面**（shim tag 1 空码臂 FIXME）与 in-order 队列（async=false
  动态模块按 asap FIXME）维持。
- 历史挂账延续：charset ×6 Timeout、http-accept-header ×2、CSS
  module 基 URL/复用面（relative-urls 余 1、redirect-relative-
  urls、repeated-import ×3、script-element-css-src/json-src）、
  referrer-policies ×3、module/dynamic-import 基面 ~10F、s97
  insertBefore inline-module 面；customizable-select 177F 挂起待
  用户拍板（GB-20261031 征询在途）。

## 下一步候选

1. 动态内联 module host 侧执行通道：沙盒重入 API 设计（v8
   FunctionCallback 内 `Script::Run` 合法性 + 沙盒内部状态机—
   watermark/replay 记账——重入安全评估）→ `inline, 200` ×3 翻绿
   面 + prelude 同型潜伏面一并收口。
2. module/dynamic-import 基面 ~10F 勘根（动态 import 与 script
   元素面共享底层后自然暴露的下一层缺口）。
