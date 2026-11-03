# M2 s267 — serve-custom-response.py 端点仿真（案级 stash + 交付缓存 + 动态 import 改写守卫）

日期：2026-11-03。切片号 s267（承 s266 下一步候选 #1）。runner 车道
2 文件（testharness.rs + es_module.rs），repeated-import 三页
**3F→3P**（8 job 并行 5/5 子测试全绿）。

## 勘根

1. **端点语义**（上游 pin 31597693
   html/semantics/scripting-1/the-script-element/serve-custom-response.py
   ，gh api 取回，**语料端点文件保持缺席**——runner 仿真）：wptserve
   `server.stash` 按 `key` 查询参数记账；`action=stat` 读数不计数、
   `reset-stat` 清零、`set` 更新 network-error/status-code/
   content-type；无 action 的模块取回——network_error → 断连 + 计
   数，否则按 content-type 回体（JS MIME→`export default 'hello';`
   、application/json→`{"hello": "world"}`、text/css→CSS 背景规则
   、其余→`"42"`）+ 计数。
2. **投递拓扑**：页面 `fetch()` action 面（stat/reset-stat/set）落
   runner fetch handler；运行时插 script 元素装载 + 动态 import 落
   script fetcher（webview 经 `ScriptSourceFetcher`）——两端点消费
   同一 stash，**须共享案级状态**。
3. **跨案竞态实证**（探针 + strict solo 复现）：runner 8 job 为
   **同进程线程**，进程级全局表下页 B/C 语料共用 fixed token
   （页面内硬编码 uuid），并行走线互相污染配置（页 B error 步拿到
   页 C 先行写入的 success 配置误 Ok；solo B 四步全过实证归因）。
   上游 wpt 顺序跑单浏览器会话，全局 stash 无此形态——本切案级
   （per-case Arc）作用域复刻该隔离。
4. **`__zw_mtype` 臂序**：s249 intent 臂在 fetcher 内先行，css 型
   动态 import 端点 URL 被其按扩展名 MIME 派生接管（.py 非 css
   MIME → Err 不计数，页 A test1 stat "0"——solo 复现）。端点臂须
   前置 + 门按 mtype 感知。
5. **改写器标识符边界 bug**：`rewrite_dynamic_imports` 的
   `import(` 匹配吃进标识符内部——页 B classic 脚本
   `removeScriptReimport(` 被改写为
   `removeScriptRe__zw_dynamic_import(` → ReferenceError（语料红态
   实证）。两改写点（顶层 + TemplateExpr）同缺前置续字符守卫。
6. **取回序债务（t8j-r2 转池候选同源）**：`__zw_load_module` 先调
   `__zw_compile_module`（host 取回）后查 `__moduleCache`——动态
   import 每次都重新取回，页 C step5 复 import stat "1"≠"0" 红态
   实证。元素装载面同理无跨元素缓存（R5182 预热仅静态文档图）。

## 施工（2 文件）

1. **testharness.rs**：
   - `ServeCustomResponseState`（network_error / run_count /
     status_code / content_type / **delivered_ok**）+
     `ServeCustomResponseStash = Arc<Mutex<BTreeMap<String, State>>>`
     ；生产路径在 case 配置构建点创建案级 Arc，fetch handler 与
     script fetcher 两闭包共享；单测经 None 回落进程级静态表（两
     个既有签名 wrapper 转 `#[cfg(test)]`）。
   - **端点模块取回臂**（script fetcher，置于 `__zw_mtype` intent
     臂之前）：delivered_ok 缓存先查（命中回缓存体不计数）；
     network_error / status≠200 → 计数 + Err；成功路径计数 + MIME
     准入门（**mtype 感知**：`__zw_mtype=text` 无校验步——s250
     口径；`__zw_mtype=css` 按 module type map essence==text/css；
     无标记按 JS MIME ∪ json ∪ css 数据模块意图集——s107 同集）+
     按 content-type essence 回上游同形回体 + `delivered_ok` 记账。
   - **fetch handler action 臂**（stat→run_count 回体 /
     reset-stat→清零 / set→解析三参 / 其余→空体）切案级 stash。
2. **es_module.rs**：
   - `is_identifier_continue_byte`（`[A-Za-z0-9_$]`，ECMA-262
     ID_Continue ASCII 子集）前置守卫，两改写点同补 + 钉
     `test_rewrite_skips_import_suffix_of_identifier`（标识符后缀
     保持 + 真 import 恰改写一次；模板串变体同证）。
   - `__zwModuleFetchKeys` 取回键缓存（键 =
     `spec␟parent␟typeHint`）：成功求值后记账，`__zw_load_module`
     入口先查缓存命中即回（**host 调用之前**）——动态 import 复取
     面收口；失败态不记账照旧重取（页 B step2/3 判据 stat 递增语
     义保留）。复位脚本与 prelude 初始化同步补键。

## 验证

- **单测**：端点状态机钉 `serve_custom_response_endpoint_state_
  machine_s267`（stat 起点 0 / reset / network-error Err+计数 /
  404 Err+计数 / text/plain 门 Err / application/javascript Ok 回
  体+计数 / text/css fresh key Ok+计数 / `__zw_mtype` css 门 Err +
  text 门 Ok / delivered_ok 复取不计数）绿；改写守卫钉绿。
- **沙箱**：`cargo test -p zero-script-sandbox --lib`
  **396P/0F**（含新钉，即基线 395+1）。
- **corpus**：solo 页 A `css-module/repeated-import` **3 子测试
  全 P**（error/404/MIME/成功/串行 ×5/并行 ×5 判据全过——s249
  CSSStyleSheet 合成路径经 mtype 门真实通路）；solo 页 B 5 步 P；
  **8 job 并行族跑 5/5**（A3+B1+C1，日志
  evidence/logs/s267-repeated-import.log——案级 stash 后跨案竞态
  消失）。
- **邻域回归**：FILTER="module" 域 330P/43F；对位 s261-ab 全域基
  线（7794P/439F）模块子集 37F diff——**REGRESSED=∅，FIXED=6**
  （3×s267 + 3×s266 MIME 页），残 31 页悉数挂账既有集。
- **runner bin 全量**：**284P/0F/76I**（283 基线 + s267 钉）。
- **fmt/clippy**：`cargo fmt --all --check` clean；clippy
  zero-wpt-runner + zero-script-sandbox `--all-targets` 零告警。

## 挂账更新

- repeated-import ×3 销案（本切收口）；s256 credentials/
  shared-dependency/并发慢 import dedup 挂账**部分收窄**——
  fetchKey 缓存 + delivered_ok 覆盖本端点与常规同 URL 动态 import
  复取面，**webview 元素装载面真实 module map 仍缺席**（delivered_ok
  为端点域仿真，非通用缓存），原挂账保留。
- 页 B/C fixed token 跨案共享为语料形态注记：上游顺序跑全局 stash
  无碍，runner 同进程线程并行须案级隔离——后续端点类挂账（
  referrer-checker 等）沿用案级 stash 形态。
- 承前不动：多张 map merge + 动态插入（s262 架构级缓议挂起待
  拍板）；microtask checkpoint 时序族（s255——本次残 31 页含
  execorder/single-evaluation 同族）；customizable-select 177F
  （GB-20261031 挂起）；renderer SetDomSnapshot entries 化（s174）
  ；engine lib 5 红 R-t2qq（该流自查，本流不入侵）；s265 声明块
  直读路径未预处（零 corpus face）。
- css-module relative-urls / redirect-relative-urls（CSS url()
  基面，渲染流车道交接）；http-accept-header ×2（Accept 头穿参
  面）；referrer-policies.sub（referrer-checker.py +
  `{{domains[www1]}}` 替换）；NotRun 微面（json `;boundary` ×3、
  css `;params` ×2）。

## 下一步候选

1. referrer-policies.sub 端点 + `{{domains[www1]}}` 替换面（runner
   车道，案级 stash 形态复用）。
2. NotRun 微面收尾（json `;boundary` ×3、css `;params` ×2——
   serve-with-content-type 端点参数解析）。
3. css-module relative-urls CSS url() 基面→渲染流车道交接注记
   （本流不动渲染）。
