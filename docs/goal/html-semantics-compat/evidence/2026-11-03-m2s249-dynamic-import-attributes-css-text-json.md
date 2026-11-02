# M2 s249 — 动态 import 属性三通道（json/css/text）+ runner 数据模块 intent 臂 + CSSOM 颜色序列化归一（css/text/json 动态面 13 键翻绿，第八个连续完美 A/B）

日期：2026-11-03。切片号 s249（承 s248）。根：s248 挂账 scripting-1
module 面——css-module redirect 对 3F 勘根入場，分型扩及
css-module/text-module/json-module 动态面全族。

## 入场定刀位

s248 收口后 scripting-1/the-script-element 模块族形态：css-module
10F+3T（redirect 对 3F + parse-error/integrity/dynamic 等）、
text-module 5F+3T、json-module 5F+1T、module/dynamic-import 基面
~10F。solo 签名——`import(x, { with: { type: "css" } })` 拒绝
`Failed to fetch module`，静态面同型断言（content-type-checking/
integrity/parse-error）全挂。

## 勘根（代码链逐环定谳）

- **动态 import 属性丢弃根（全族单点）**：① prelude
  `__zw_dynamic_import = function(spec)` 单参——attributes 对象在
  调用点即丢弃，`import(x, { with: … })` 与 `import(x)` 对宿主不可
  分；② classic 上下文 V8 原生回调 `dynamic_import_callback` 签名带
  `import_attributes: v8::Local<v8::FixedArray>` 但
  `#[allow(unused_variables)]` 弃用——两通道同病。故 css/text/json
  数据模块动态面无法记账（module type map 校验、类型合成臂均无入
  口）。
- **fetcher 契约缺口**：`ScriptSourceFetcher = Fn(page_url, src)`
  无类型形参——数据模块类型无法穿到 runner fetcher。定谳以内部标记
  查询参数 `__zw_mtype=<css|text>` 携带（s124 `__zw_lp` 同法）：runner
  fetcher 消费即剥离，真实 HTTP 宿主忽略未知查询参数零影响；json 面
  不加标记（既有 .json / serve-with-content-type intent 通道已覆盖，
  加标记反致双通道分叉）。
- **MIME 派生序**（wptserve 语义，runner 侧建模）：serve-with-
  content-type.py `ct` 查询 → `pipe=header(Content-Type,…)` →
  `.headers` sidecar → 扩展名规范映射（.css→text/css、.txt→
  text/plain）。type↔MIME 匹配校验（HTML module type map：css←
  text/css、text←text/plain）不匹配 → Err（网络错误语义，
  content-type-checking `ct=application/css` 臂判据）。
- **css-module redirect 对 3F = 语料缺口非引擎缺陷**：
  `css-module/resources/redirect-target.css` 上游存在、本地语料缺失
  （404 → module 网络错误 → load 应用/同对象/分 map 条目三断言永
  假）。从上游 wpt 同路径原件补入（3 行 CSS）；`load-relative-url.css`
  经核与上游一致未动。redirect.py 终跳链路 s248 已真 302，零施工。
- **JSON parse 失败语义改判**：旧版 parse 失败按网络错误（元素 fire
  error、不 fire load、体不执行）；spec 口径取回成功仅 internalize
  失败 → 以 SyntaxError 报 window（s102 report 面）、元素照常 fire
  load（s102 模块执行期异常同口径：模块失败非 classic 体抛错不聚合
  abort）。WPT json-module/parse-error 判据 = window error event
  `error instanceof SyntaxError`；crossorigin-import-parse-error 双页
  无类型断言，语义变更零位移。
- **CSSOM 颜色序列化根（bonus 面）**：css-module 断言读
  `sheet.cssRules[0].cssText` 期望 Blink 形 `rgb(255, 0, 0)`——shim
  `_parseDeclarations` 值原文透传（hex/命名色不归一）且
  `declsText` 末声明无分号。CSSOM §serializing CSS values：color 按
  parsed Color 序列化。
- **charset 面定性**：runner fetcher 全量 UTF-8 lossy 交付，charset
  转换/BOM 嗅探未建模——css/text charset ×6 文件 Timeout 在册维持，
  本切不动。

## 施工（shim 1 + rust 4 + runner 1 + scripts 1）

- **script-sandbox es_module.rs**：prelude `__zw_load_module(spec,
  parent, typeHint)` 第三参穿 `__zw_compile_module(spec, parent,
  type)`；`__zw_dynamic_import(spec, attributes)` 按 TC39 import
  calls phase 2 解析——attributes 在场非对象 TypeError、`with` 非
  对象 TypeError、`type` 值 ∉ module type map（json/css/text，与
  import_attributes.rs VALID_MODULE_TYPES 同集）TypeError；同步
  throw 由 try 捕获转 rejected promise（ImportCall 恒返 promise）。
  https://tc39.es/proposal-import-attributes/#sec-import-calls
- **script-sandbox v8_runtime.rs**：`DynamicImportResolver` 增第四
  参 `type_attr`；`dynamic_import_callback` 去 `#[allow(unused_
  variables)]`，import_attributes FixedArray 平铺 [key, value, …]
  对提取 `type`（try_cast 失败按缺省空串）+ module type map 校验，
  违例 TypeError reject 不触 resolver；单测
  `test_native_dynamic_import_attributes_threading_s249`（穿参 +
  分派 + bogus type reject）。lib.rs trait 默认签名/文档同步。
- **webview.rs**：`compile_module_contract` 增 `type_attr` 形参穿
  `collect_module_deps_recursive`（动态 resolver 与原生 import 两通
  道同源）；递归内——`__zw_mtype` 标记注入（仅 css/text）+ 三合成
  臂：json parse 失败 `throw new SyntaxError(<serde 消息>)`（入
  module map 记账，s87 ③ 同口径）、css `export default (function(){
  var s=new CSSStyleSheet(); s.replaceSync(<literal>); return s;})()`
  （sheet 构造/CSS 解析复用页面 shim CSSOM——R2809 constructable
  sheet 链，CSS parse 错误不拒模块）、text `export default
  <literal>;`（serde_json 字符串转义内嵌）。单测 ×3：json parse
  failure 改判重钉（SyntaxError + 元素 fire load 不 fire error + 体
  不执行）、dynamic css module → CSSStyleSheet（断言
  `rgb(255, 0, 0)` Blink 归一化形态）。
- **runner testharness.rs**：fetcher 数据模块 intent 臂——标记解析
  + redirect.py 跟随 ≤4 跳（location 绝对值 strip_wpt_host_prefix
  名单映射/origin 相对值原样/其余相对值按 redirect.py 目录解析）
  + serve-with-content-type/pipe/sidecar/扩展名 MIME 派生序 +
  type↔MIME 匹配校验（不匹配 Err 网络错误）；单测
  `script_fetcher_data_module_marker_s249`（带标记取回 + 无标记
  .css 维持非 scripty Err 零回归面）。
- **shim part06.js**：`_zwNormalizeDeclValue` 声明值逐 token 颜色归
  一（hex #rgb/#rgba/#rrggbb/#rrggbbaa 展开 + CSS Color 4 命名色
  148 全集含 gray/grey 双拼 + transparent → rgba(0, 0, 0, 0)；
  alpha=1 收 rgb 形，其余去尾零；custom property `--` 豁免、无
  color token 原文返回零重排）+ `_zwSerializeDecls`（`name: value;`
  空格连接，末声明带分号 Blink 形）+ `_ruleFromText` cssText 经
  parse 重建（原文 passthrough 弃用）。
- **scripts/update-wpt-data.sh**：redirect-target.css 补语料挂账
  注记（升级新 ref 后需重新补）。

## 验证

- 全树 A/B（evidence/logs/s249-ab.log vs s248-ab.log）：行级
  7746P/469F/81T/21N → **7760P/458F/78T/21N**（ΔP=+14、ΔF=−11、
  ΔT=−3）。全键 join（子测试整键，last-wins + LC_ALL=C sort/join
  同序）**8314/8314 对，LOST 0 / GAINED 0**；翻转 **13 F→P + 1
  T→P + 2 T→F 预期换形，P→F 0——第八个连续完美 A/B**：
  - 12 通道 F→P = css-module 10 键（content-type-checking `text/css`
    + parse-error ×3 + import-css-module-dynamic + integrity +
    load-error-events inline + redirect ×3）+ text-module 2 键
    （load-error-events inline + module "Text modules"）；
  - 1 T→P = json-module/parse-error.html（文件级 Timeout → 子测试
    "JSON modules: parse error" Pass——文件从永挂转可判，改判语义
    直接兑现）；
  - 2 T→F 预期换形 = css-module/relative-urls + json-module/
    parse-error-location.tentative（同上文件级 Timeout → 真子测试
    Fail——测试可见性修复，余红为下一层缺口非回归）；
  - 1 F→P = checkable-active-onblur-with-click——s238/s239/s245/
    s246/s248 在册 checkbox `:active` 交互漂移族回归恢复（双向漂
    移，s248 轮 P→F 同键；本切通道 = 模块属性/合成臂，与指针键盘
    时序无涉），按在册先例销账。
- **post-pull 复核**（并行流合入后新基线，evidence/logs/
  s249-solo-postpull.log，FILTER=scripting-1/the-script-element）：
  963P/86F/26T/15N，对 s249-ab 同切片 join 1089 对零位移（lost 0/
  gained 0/异态 0）——12 通道键 + T→P/T→F 键逐键复现，webcrypto/
  渲染流合入零干扰证实。
- 单测：script-sandbox native import attributes threading 1 +
  webview css module stylesheet / json parse SyntaxError 2 全绿；
  runner bin 279 passed 0 failed（含 fetcher marker 新钉）；`cargo
  fmt` 干净；guarded clippy（script-sandbox + webview + wpt-runner
  --all-targets）零警告；shim core 7-part concat `node --check`
  SYNTAX_OK。

## 残余挂账（本切不动）

- **charset 面 ×6 文件**（css/text charset、charset-2、charset-bom
  3+3）Timeout——charset 转换/BOM 嗅探未建模（fetcher UTF-8 lossy
  交付，在册）。
- **text-module/integrity.html 残 F**：mismatch 臂未过（css 侧
  integrity 双臂已绿——text 侧 SRI 校验未接 text MIME 通道）。
- **http-accept-header.sub.html ×2**（json/text）：Accept 头协商
  未建模。
- **load-error-events 非 inline 臂**（not parser-inserted/外部错
  误臂）各族残 F + NotRun。
- **CSS module 基 URL/复用面**：relative-urls（CSS 文件 URL 解析
  余 1 断言）、redirect-relative-urls、repeated-import、
  script-element-css-src/script-element-json-src。
- **referrer-policies ×3** Timeout——referrer policy 面未建模。
- **module/dynamic-import 基面 ~10F**（credentials/nonce/延迟求值
  等）——s87 时代挂账族，独立勘根轮。

## 下一步候选

text-module integrity mismatch 臂 + load-error-events 非 inline 臂
勘根（数据模块族同层收口）；module/dynamic-import 基面 ~10F 分型；
customizable-select 177F（挂起待用户拍板，GB-20261031 征询在途）。
