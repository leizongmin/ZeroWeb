# M2-S338：动态 import credentials 透传——模式管线全链收口 + iframe 域 window 语义缺口定谳（承 s337）

日期：2026-11-08。控制文档：`docs/goal/html-semantics-compat.md`（L5 链 s338 段）。
本切片收口 s337 挂账的 `dynamic-imports-credentials` 簇（s337 记「10 子测」实勘为
**12 子测**：classic/module × same/cross × 无/anonymous/use-credentials）。施工
目标：module 元素 `crossorigin` 属性 → credentials mode（`use-credentials` →
`include`，其余 → `same-origin`）→ 动态 import() 取回继承定义模块的模式
（spec：fetch a single module script 的 credentials mode 步）。
https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script

## 定谳（本切片终态：管线全链通、corpus 面被引擎 iframe 域 window 语义缺口阻塞）

### C1：credentials 模式管线（本切片施工，契约级全绿）

- 元素级：`crossorigin` 属性逐 script 提取（`extract.rs`
  `extract_script_credentials`，与 `extract_script_referrer_policies` 同构遍历）；
  非法值按 spec 落 `same-origin`（anonymous/非法均非 include）。
- 编译级：per-IIFE `_importCred` 绑定——`build_module_script`/`build_dep_iife`
  顶部发射 `var _importCred = '<mode>';`，动态 import 发射引用词法名
  （零引号字符，t8j-r2 合规）。不穿参改写链（8 处 call site 不动）。
- 取回级：宿主 shim `__zw_compile_module` 第 4 参 credentials →
  `compile_module_contract` → 静态图继承入口模式（嵌套依赖同批标注）；
  **仅非缺省模式**追加 `__zw_cred=<mode>` 查询标记（same-origin 零 src 漂移，
  精确 src fetcher 钉不受扰；registry/cache 键不含标记）。
- runner 级：credential cookie stash（`(cookie host, name)` → 值，M3/M4 cookie
  stash 同型 Arc 共享）；`set-cookie.py` 臂写 stash、`check-cookie.py` 臂按
  attach 判定（`credentials == include || 同源`）读 stash 回
  `window.<id> = 'found' | 'not found' | 'different value: <v>'`；
  check-cookie 臂置 `strip_wpt_host_prefix` 之前（自处理 www2 host）。

### C2：corpus 面阻塞根因（引擎既有缺口，非本切片回归——探针定谳）

corpus 载体 `dynamic-imports-credentials.sub.html` 经
`runTestsFromIframe`（helper .js，页源无 iframe 字面避 case-skip）在 **iframe
子文档**内注册 12 个 promise_test。子测试体 `assert_equals(window[id], expected)`
读的是合并 Function 形参遮蔽的 **iframe win**；而动态 import() 的模块体求值落
**isolate 全局作用域**——`window` 解析主窗 global，`window.<id>` 写到主窗，
iframe 续延读形参恒 `undefined` → 12F。

根因链（代码级）：

1. 子文档脚本执行：`_zwRunIframeScripts`（`js_dom_shim/part01.js` R206 通道）
   以 `new Function('window','self',…,'document',…, body)` + `fn.call(win, win,
   …, doc, …)` **形参遮蔽** iframe 域——同源同步面（`window.iSync`）、
   promise_test 体内写、自身续延读均一致（探针 Pass）。
2. 动态 import 模块体求值：classic 原生回调 `settle_dynamic_import`
   （`v8_runtime.rs`）与 prelude helper `__zw_load_module` 的 eval 臂
   （`es_module.rs`，`__zwRealEval('('+code+')')`）均在**全局作用域**编译求值
   IIFE——`window` 恒主窗。
3. 实证矩阵（探针，gitignored wpt-data，证据已删、结论在此存账）：顶层页
   set-cookie stash 写 → check-cookie import → `window.p0='found'` classic/
   module 双车道全通（含 `__zw_cred` 标记路径）——**credentials 管线全链为真**；
   iframe 内同链 import 解析成功但 `window.i0` 于后续微任务 `undefined`——
   写落主窗（window 身份错位），与 credentials 无关（无 credentials 的普通
   mod.js 探针同败）。

次级缺口（Gap-B）：iframe 子文档 script 元素的 `crossorigin` 属性**不入**
`extract_script_credentials`（该遍历只走主文档 HTML；子文档 HTML 由 shim JS
regex 路径解析）——即使 window 身份修复，2 个 use-credentials cross 子测
（预期 found）仍不可达。

### C3：setTimeout 变体文件（file Timeout，同族根因）

`dynamic-imports-credentials-setTimeout.sub.html`：字符串
`import("${url}").then(window.resolve)` 经 step_timeout 字符串求值落全局作用域，
`window.resolve`（resolve 落在 iframe 形参 win）不可见 → 外层 promise 永挂 →
文件级 Timeout。同 C2 window 身份族，本切片零变化（承基线）。

## 施工

| 面 | 位置 | 内容 |
|---|---|---|
| 元素提取 | `extract.rs` | 新 `extract_script_credentials(html)`——crossorigin `use-credentials` → `Some("include")`，其余/缺席 → `None`（镜像 referrer-policy 遍历） |
| 编译 | `es_module.rs` | `compile_module_script_with_credentials`（+credentials 参）；`build_module_script`/`build_dep_iife`/`inline_dep_once`/`transform_import`/`transform_export` 穿参；per-IIFE `var _importCred` 发射（两处 emission 同步）；prelude `__zw_load_module` 4 参 `credHint`、`__zw_dynamic_import` 读 `this.credentials` |
| SW 宿主 | `service_worker.rs` | classic 面全局预置 `_importCred = "same-origin"`（发射引用词法名；对象字面量求值先于 stub TypeError） |
| webview | `webview.rs` | 编译位按脚本序取模式（缺省 same-origin）；shim `__zw_compile_module` 第 4 参 → `compile_module_contract`（`None` = same-origin/空，不追加标记，FIXME(s339) classic-lane 捕获面挂账）；`collect_module_deps_recursive` 穿参 + `fetch_spec` `__zw_cred` 追加（`__zw_lp` 之后，仅非缺省） |
| runner | `testharness.rs` | `CredentialCookieStash` 类型 + `wpt_data_script_fetcher_with_stash` check-cookie 臂（strip 前置、authority 同源判定、attach 门）+ `wpt_data_fetch_handler_with_stash` set-cookie 臂；生产位 stash 双 builder 注入 |

不采用（备选否决，防 mock 绕变更边界）：runner 合成 check-cookie 体广播写
sibling iframe windows（`querySelectorAll('iframe').contentWindow`）——准则 6
症状压制；断言将不再核引擎 window 语义（恰是缺处），且 Gap-B 两子测仍红。

## 验证

- **契约**：sandbox `test_module_credentials_emission_s338`（发射串钉：
  `var _importCred = "include"`、`credentials: _importCred`、缺省
  `"same-origin"`、prelude 第 4 参）+ `test_dynamic_import_credentials_passthrough_s338`
  （V8 stub 捕 `__zw_compile_module` 4 参透传 `include`）；runner
  `dynamic_import_credentials_cookie_roundtrip_s338`（同源+include → found、
  同源缺省 → found、cross+include → found、cross 缺省 → not found、
  cross+same-origin → not found 五态全钉）。sandbox lib **404P/0F**、runner
  bins **291P/0F**、webview lib **844P/1F**（红 = `proxy_mode_decodes_png_correctly`
  环境红，s168c/s256 在案）、engine lib **3807P/6F**（已知六红精确）。
- **clippy/fmt**：`-D warnings` 四 crate（script-sandbox/webview/wpt-runner/engine）
  全绿；rustfmt 全绿。
- **顶层探针**（gitignored wpt-data，结论存账）：credentials 管线全链（stash
  写 → check-cookie → `__zw_cred` 标记 → window 写读）双车道全通。
- **the-script-element 域 A/B**：`logs/s338-script-element-after.log`
  **1058P/54F/11T** vs `s337-script-element-after.log` 1058P/54F/11T——**数果
  全等**；全行 comm（state+case+subtest 归一键，LC_ALL=C sort -u）双向零差
  （1122 = 1122 行）——credentials 簇 12F 与 setTimeout 文件 Timeout 承基线
  零变化，P→F 零，模块发射加线 corpus 零漂移。

## 挂账

- **s339 候选（根因修复）**：iframe 域动态 import 模块体 window 身份（C2 根因
  链）+ iframe 子文档 script credentials 提取（Gap-B）。可commit 面勘过三向：
  ①宿主 testharness.js 打标 `_zwRemoteWindow`（fetch_tests_from_window 事件源
  捕获）→ body 运行窗置全局 override——**不可 commit**（wpt-data 全树
  gitignore，refresh 即失）；②classic 面 import() 源码改写穿窗——blast radius
  全 classic 面（否决）；③per-document realm/global——架构级（独立切片勘根）。
  window 身份修复后预期簇面 12 子测中 10 可绿（use-credentials cross 2 子测
  需 Gap-B 同批收口）。
- `window.onerror = fn` per-fire `window.event` 语义偏差（承 s336/s337）。

## 证据索引

- `logs/s338-credentials-solo.log`——credentials 簇 solo 面（12F + setTimeout
  文件 Timeout，承基线零变化）
- `logs/s338-script-element-after.log`——the-script-element 域全域终态
  （A/B vs s337）
