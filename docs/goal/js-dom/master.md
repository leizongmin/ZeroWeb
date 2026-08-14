# JS/DOM 原生化 — 主控面板（master.md）

**入口文档**: [../js-dom.md](../js-dom.md)（长期 Mission / Done Criteria / 执行协议 / 文档治理规则）
**关联 RFC**: [../../specs/p1b-v8-native-bindings-rfc.md](../../specs/p1b-v8-native-bindings-rfc.md)
**创建日期**: 2026-08-13（goal 拆分 bootstrap）
**本轮**: R25 — native MouseEvent/KeyboardEvent `view` + KeyboardEvent `which` 补全（dom_bindings event.rs，经 runner 实测诊断确认 native_dom=1 下 `new MouseEvent()` 走 native 覆盖 polyfill，native 缺 UIEvent view 父链属性 + KeyboardEvent which legacy）；`set_ui_view` helper（缺省 null/init dict 对象）+ KeyboardEvent which（回退 keyCode）；native 正确性净正（单测验证，default-on 后合规），**双路径差未缩**（6.45pp 保持——WheelEvent 子类链断/SubclassedEvent class 语义等多点分散缺口，转高 ROI 切片）

> **本文件由执行 agent 于 2026-08-13 按入口文档「首轮进入检查清单」逐项核实重写**，替换 bootstrap 占位符。
> 所有状态带证据（commit hash / 文件路径 / 行号 / 测试命令）。并行双流下 main 随时漂移（run-rules §10），每轮开工先 `git pull --rebase`。

---

## 当前状态（2026-08-13 核实，基线 commit `f7219b2c`）

| 项 | 状态 | 证据 |
|----|------|------|
| P1a（event loop / fetch / Observer） | ✅ 实质完成 | 入口文档基线块（fetch/定时器/MO e2e 在 `apps/browser/src/tab_js_worker.rs`）；非阻塞 follow-up 见下「缺口」 |
| P1b **V8** native bindings S0–S5 | ✅ 已 land，默认关 | `crates/engine/src/dom_bindings/` 19 文件；`mod.rs:55 native_dom_enabled()` 读 `ZW_NATIVE_DOM` env，`OnceLock` 缓存 |
| L1 Live Document 共享（V8） | ✅ 已 land | `webview.rs user_actions.rs:367` `config.native_dom` + `install_native_dom_bindings()`；`mod.rs` `parse_and_install`（`Rc<RefCell<Document>>`，注释「read-only 快照」未完全 live） |
| **V8 native API 面（核实后更完整）** | ✅ 超基线 | `mod.rs:558-624` 已注册：`__zw_native_query_selector/_all`、`create_element/text_node/comment/document_fragment`、`documentElement/body/head`、`get_element_by_id`——比入口文档基线描述更完整 |
| L2 polyfill-live 合一（V8） | ❌ 未做（M1） | `js_dom_bridge/callbacks.rs:119-137` `__zw_query_match/_all` 仍 `with_query_doc(snap, …)` 经 `parse_html(dom_html)` 重解析快照；`callbacks.rs:1454` 注释明示每次查询 re-parse |
| S6 高层 API 去字符串（V8） | ❌ 未做（M2） | Fetch/Observer/FontFaceSet 仍经 `__zw_*` String ser/deser（part02.js / part01.js） |
| **QuickJS 原生 DOM 绑定（DC-7）** | ❌ **完全真空（核实确认）** | `script-sandbox/src/lib.rs:167 install_native_bindings` 仅 `#[cfg(feature="v8")]` 存在，QuickJS trait 方法**不存在**；`quickjs_runtime.rs:357 register_callback` 走 `__zw_*` polyfill 桥（与 V8 共用 `js_dom_shim`） |
| S7 死代码清理 + shim 萎缩 | ❌ 未做（M5/M7） | `js_dom_shim/part01-06.js` 共 ~815KB（part01 111KB+part01b 28KB+part02 149KB+part03 148KB+part04 127KB+part05 150KB+part06 103KB） |
| **双引擎** default-on + 删 kill-switch | ❌ 未做（V8=M5, QuickJS=M7，改 Mission 级单向门） | `WebViewConfig.native_dom` 默认 `false`（`webview_builder.rs:79`） |
| 真实 SPA/WC 端到端验收 | ❌ 无资产（M3） | 无 React/Vue/Svelte/lit 端到端 fixture |
| WPT dom 上游基线 | ✅ **dom/nodes polyfill 55.63% / native 54.98% 双基线对等**（178 用例 / 4502 subtest；Element-classlist.html 100%）+ **dom/events polyfill 42.58% / native 36.13%**（81 用例 / 319 subtest，R24 事件子类父链继承后；双路径差 6.45pp——native dom_bindings event.rs 待 R25 对齐） | `testharness-dom`（polyfill）+ `testharness-dom-native`（ZW_NATIVE_DOM=1）双入口；R24 事件子类 init 属性父链继承（Event-subclasses-constructors polyfill 42P/49）。失败聚类：dom/nodes iframe.contentDocument（深结构 html-compat）/querySelector-mixed-case（selector 域）；dom/events native 事件构造器对齐（R25）/三阶段分发/EventListener（~44 个 0-pass 用例） |
| **Canvas path-objects JS 侧 API（DC-8, v1.2 接手）** | ⚠️ 用例**完全缺失**（须重新导入） | `wpt-data/html/canvas/element/` 目录本地不存在（不止 path-objects，整个 canvas element 子树未 fetch）；`testharness.rs:26 CANVAS_TEST_SUBDIRS` 8 个目录无 path-objects；`testharness-canvas` 子命令已就绪（`main.rs:220`） |
| `make test` / clippy / coverage（含 quickjs 矩阵） | ✅ 基线全绿（入口文档） | workspace ~13,000+ 测试，行覆盖 95.46%，clippy 零警告；Makefile `QUICKJS_TEST_CRATES`/`QUICKJS_CLIPPY_CRATES` CI 强制 `--features quickjs` |
| dom_bindings 独立 coverage 口径 | ❌ 待补（M0 项 4） | `scripts/check-coverage.sh` 仅 workspace 全量，无单 crate/子模块口径；`cargo-llvm-cov` **本地未安装**（环境前提，见下） |

**核心缺口**（本目标要消除，按优先级）：
1. ~~**WPT dom 上游用例 0 导入**（DC-3）~~ → ✅ **R1 首切片已建**：dom/nodes 141 用例，41.25% pass 基线。**剩余**：扩展子目录（events/collections/...）+ native 路径对照（`ZW_NATIVE_DOM=1`）。基线暴露的真实差距已重排见下。
2. ~~**DOMException 抛出语义未实现（classList 部分）**~~ → ✅ **R2 已修**：classList token 校验抛真 DOMException（空→SyntaxError、空白→InvalidCharacterError），双路径同步 + native DOMException 构造器。dom/nodes **41.25% → 56.08%（+14.83pp）**。✅ **R3 已修**：createElement 非法标签名抛 InvalidCharacterError（双路径 + spec Name production 校验 helper），dom/nodes **56.08% → 56.45%（+0.37pp）**。✅ **R4 已修（native 路径）**：appendChild/insertBefore/removeChild/replaceChild 的 DomError 转 DOMException（WouldCreateCycle/CannotInsertDocumentRoot→HierarchyRequestError，NotAChild→NotFoundError）。**注**：R4 仅 native（node.rs），polyfill 路径架构限制（mutation 延迟批处理 + shim 无 live 祖先链）无法同步抛——待 M1 L2 polyfill-live 合一。testharness-dom 基线走 polyfill 故 R4 不提升基线数字（native 修复经单测验证，是 default-on 生产路径合规）。**剩余**：testharness-dom native 路径对照（让 R2/R3/R4 native 修复基线可见）；createElement XML/XHTML iframe 上下文（独立缺口）。
3. **polyfill vs native A/B 对照门**（DC-4）→ ✅ R0 读路径骨架 + R2 异常路径扩展（classList 抛 DOMException 等价）
3. **dom_bindings 独立 coverage 口径缺失**（DC-4）——`check-coverage.sh` 无子模块报告（M0 项 4）
4. V8 native 路径默认关 → 生产仍走 polyfill 字符串桥（DC-1，M5）
5. V8 polyfill 桥仍 re-parse String 快照，三方 Document 未合一（DC-1，M1 L2）
6. V8 高层 API 仍经 `__zw_*` String ser/deser（DC-1，M2 S6）
7. **QuickJS 页面引擎 native 完全真空**（DC-7，M6）
8. canvas path-objects 用例需重新导入 + roundRect panic/精度（DC-8，M8，但当前 canvas 流活跃碰撞）

---

## Active Milestone

**当前活跃里程碑**: **M0 — 基线建立 + polyfill-live 合一起刀**（见入口文档「Single Active Milestone」）

**M0 must-complete 入口动作进度**（入口文档「首轮进入检查清单」）：
- [x] 1. 探索 `dom_bindings/` + `js_dom_bridge.rs` + `js_dom_shim/` 事实，核实 RFC L1/L2 与现状一致 ✅（本文件即产出）
- [x] 2. 创建/重写 `master.md`（本文件）
- [x] 3. 确认 `archive/` + `evidence/` 目录存在（均空，待首轮证据追加）
- [ ] 4. 补齐 dom_bindings 独立 coverage 口径（**环境前提 `cargo-llvm-cov` 本地未装**，本轮记入「未解决问题」，待装后补）
- [x] 5. 建 polyfill vs native A/B 对照门骨架（双 feature 可参数化）✅（R0，`tests_ab_compare.rs`）
- [x] 6. 选定首切片并直接动手推进 ✅（R0：A/B 门；R1：WPT dom 基线）

**M0 状态**: 项 1/2/3/5/6 ✅ 完成；项 4（dom_bindings coverage 口径）待 `cargo-llvm-cov` 安装（记「未解决问题」）。M0 核心目标（基线 + A/B 门 + 首切片 land）已达成，**转入 M4 推进**（入口文档允许 M4 与 M1–M3 早期并行）。

**当前推进: M4 — WPT dom 上游基线 + 按聚类驱动修复（R5 已 land）**
- R1 已建：`testharness-dom` 子命令 + `fetch-dom-subset.sh` + `DOM_TEST_SUBDIRS=["dom/nodes"]`；基线 41.25%
- R2 已修：classList token 校验抛 DOMException（双路径）+ native DOMException 构造器。polyfill 41.25% → 56.08%
- R3 已修：createElement 非法标签名抛 InvalidCharacterError（双路径）。polyfill 56.08% → 56.45%
- R4 已修（native）：node mutation（append/insert/remove/replace）DomError→DOMException。native 路径合规
- R5 已建：`testharness-dom-native` 入口 + native DOMException constructor 修复（部分）；建立 polyfill 56.45% vs native 41.25% 双基线，定位 native 落后根因
- R6 已修（native 追平 polyfill）：DOMException identity 三重根因修复。native 41.25% → 56.08%
- R7 已做：① fetch-dom-subset 补齐 .js 依赖（.html + .js + dom 根共享 .js + testharnessreport.js）——基线真实化（polyfill 56.45%→51.12%、native 56.08%→50.79%，双路径对等，差 0.33pp；此前虚高因用例 .js 缺失被跳过）② native createProcessingInstruction API（factories invoke + document.rs 注册 + node.rs PI target/data getter + node_name PI→target 修正 + spec 校验 invalid target/data→InvalidCharacterError）+ native 单测
- R7 发现：PI 用例（Document-createProcessingInstruction.html）超时——用例外层 `test()` 嵌套多个 `test()` + `pi instanceof ProcessingInstruction`（ProcessingInstruction 构造器未装）→ testharness completion callback 未调。需 ProcessingInstruction 构造器（下轮）
- R9 已做：① polyfill document.createProcessingInstruction 桥接（`js_dom_bridge.rs` 新增 `DomMutation::CreateProcessingInstruction` + apply + query 分支；`callbacks.rs` 注册 `__zw_create_processing_instruction`；shim part06 方法 + part01 `_piHandles` + part04 PI 节点包装 nodeType=7/nodeName=target/target/data getter + part03 ProcessingInstruction 构造器占位）② **DOMException identity 对等修复**（createElement/PI 校验抛错改用 `globalThis.DOMException`，避免 native 叠加路径 wrong-global；顺带修 R3 createElement 既存对等 bug）③ polyfill PI 单测 `test_create_processing_instruction_r9`。基线 polyfill 37.82%→38.03%、native 37.59%→37.81%（双路径对等差 0.22pp）；PI 用例双路径 1P/11F→6P/6F。engine v8 2076 / quickjs 1407 / wpt-runner v8 168 / quickjs 103 单测全绿，双矩阵 clippy 干净
- R9 关键发现：**用例侧 `document` 始终是 polyfill document（即使 ZW_NATIVE_DOM=1）**——native document template（R7 PI API）用例访问不到。故 PI 必须在 polyfill document 实现，双路径用例才能过。DOMException identity：native 路径 shim 裸 `new DOMException` 抛的异常 → testharness assert_throws_dom "wrong global"，改 `globalThis.DOMException` 修复（R6 教训）
- R10 已做：① polyfill Proxy `_makeProxy` handler 加 `getPrototypeOf` trap（part05 handler 闭合处）——element→HTMLElement.prototype（链 Element→Node）、PI→ProcessingInstruction、fragment→DocumentFragment、text/comment→Node；构造器缺失回落 Object.prototype。仅影响 instanceof/getPrototypeOf/原型链查找，不影响 get/set ② DOM 原型方法不可枚余（part03 cloneNode/addEventListener/removeEventListener 改 Object.defineProperty enumerable:false，修 getPrototypeOf 副作用——for-in 枚举到原型方法污染 expando）③ **顺带修并行 canvas 流 2 个回归**：CSS.escape/supports 合并（canvas 流 part05:920 先建 CSS={percent,deg} 致 part06 `||` 短路，escape/supports 丢失 → CSS.escape is not a function；改 part06 合并模式）+ testharness.rs fetch_handler 编译错误（`wpt_root.and_then` → `wpt_data_fetch_handler(wpt_root)`，&Path 无 and_then）④ instanceof 单测 `test_instanceof_prototype_chain_r10`。基线 polyfill 38.03%→39.23%、native 37.81%→38.96%（双路径对等差 0.27pp）；cloneNode 用例 0P→51P。engine v8 2071 / quickjs 1407 单测，双矩阵 clippy 干净
- R10 归因：**6 个 fetch/response 测试既存失败**（clean R9 tree 同样存在，并行流引入）——`instanceof Response`=false（fetch 结果非 Response 实例）+ fetch abort/binary/stream/signal/forbidden-headers。归因 fetch/net 域，非 js-dom DOM 桥工作面，记未解决问题不硬解（run-rules §9）。本切片修了同源 CSS 回归（part06 工作面内）
- R11 已做：① element.localName getter（part04 get trap：HTML 元素 = tagName 小写；带 prefix 限定名去 prefix；非 Element→null）② **HTML 元素子类 instanceof**（part03 注册 ~64 个 HTML*Element 构造器 prototype→HTMLElement.prototype + `__zwHtmlTagIface` tag→接口映射表 div→HTMLDivElement 等覆盖 spec HTML 元素接口全表；part05 getPrototypeOf element 分支按 tag 查映射返对应子类 prototype）③ localName 单测 + 子类 instanceof 单测。基线 polyfill 39.23%→40.89%、native 38.96%→40.63%（双路径对等差 0.26pp）；cloneNode 用例 51P→121P（+70）。engine v8 2082 / quickjs 1408 单测，双矩阵 clippy 干净
- R11 已知边缘：`createElement('canvas') instanceof HTMLCanvasElement` 仍 false——canvas 经 `_zwMakeCanvas()` 特殊 proxy（canvas 流专用路径），不走 _makeProxy/getPrototypeOf。记未解决问题
- R12 已做：① element.prefix getter（part04 get trap：限定名冒号前；无冒号→null；非 Element→null。注：_realTag 大写化致 prefix 大写，case.js abc/Abc 态仍 fail，待 createElementNS 保留原 tag 深改）② getElementsByTagNameNS（part04 元素级 + part06 document 级，忽略 ns 按 localName 查，元素级支持 `*` 通配，返 HTMLCollection）③ prefix+getElementsByTagNameNS 单测。基线 polyfill 40.89%→41.71%、native 40.63%→41.45%（双路径对等差 0.26pp）；polyfill 净 +37 pass。engine v8 2083 / quickjs 1408 单测，双矩阵 clippy 干净
- R12 评估：**iframe.contentDocument 是深结构跨面改**（createElementNS XML/XHTML 路径需 iframe src 真实解析 + 独立 Document + 独立 window（defaultView.DOMException）+ 跨文档节点归属 ownerDocument===doc，完整 iframe 子文档 = html-compat 域），记未解决问题待评估，转零碰撞面
- R13 已做：① classList token 有序去重（part03 `_classListProxy` cur() 加 Object.create(null) seen 表，spec DOMTokenList 有序去重 + ASCII 空白分隔，`"a a a"`→["a"]）② contains() 空串/含 ASCII 空白 token → false 不抛（spec `dom-domtokenlist-contains`，区别于 add/remove/toggle/replace 的 check 抛；原 contains 误用 check 抛）③ classlist 单测。基线 polyfill 41.71%→46.60%、native 41.45%→46.33%（双路径对等差 0.27pp）；classlist 用例 1140P/280F→1360P/60F（+220，迄今最大单切片提升）。engine v8 2084 / quickjs 1408 单测，双矩阵 clippy 干净
- R14 已做：① 注册 5 缺失 event 子类构造器（part05 `_defineEventSubclass`：BeforeUnloadEvent/DeviceMotionEvent/DeviceOrientationEvent/TextEvent/TouchEvent）② createEvent map 全覆盖（part06：扩全集 alias 含复数 Events/HTMLEvents/SVGEvents→Event、MouseEvents→MouseEvent、UIEvents→UIEvent、custom→CustomEvent + 缺失子类）③ 未知 type 抛 NotSupportedError（spec `dom-document-createevent`，原 lenient 回落 Event 改 spec 合规抛，globalThis.DOMException 保 identity）④ 更新 test_event_subclasses2_r2812（UnknownEvent 改期望抛）+ 新增 createEvent alias/NotSupported 单测。基线 polyfill 46.60%→50.33%、native 46.33%→50.07%（双路径对等差 0.26pp，**dom/nodes 突破 50%**）；createEvent 用例 96P/183F→264P/15F（+168）。engine v8 2085 / quickjs 1408 单测，双矩阵 clippy 干净
- R15 已做：① implementation.createDocumentType 返 DocumentType（part06 主 document + part03 detached doc，spec `dom-domimplementation-createdocumenttype` 不校验，返 name/nodeName=qualifiedName、publicId、systemId、nodeType 10、ownerDocument、nodeValue/textContent=null；原 return null stub）② detached doc（_makeDetachedDocument）加 implementation 块（hasFeature + createDocumentType，ownerDocument 指 detached doc；用例 doTest(doc,...) 经 doc.implementation.createDocumentType）③ **顺带修并行 canvas 流 2 个 clippy 红灯**（main 既有：crates/canvas types.rs:199 `.or_else(||Some)`→`.or(Some)` + js_dom_bridge/canvas.rs:873 setWordSpacing 嵌套 if 合并，机械修正无逻辑变化）④ createDocumentType 单测。基线 polyfill 50.33%→52.11%、native 50.07%→51.84%（双路径对等差 0.27pp）；createDocumentType 用例 1P/81F→80P/2F（+79）。engine v8 2086 / quickjs 1408 单测，双矩阵 clippy 干净
- R16 已做：① classList write 加 runUpdate 比较（spec DOMTokenList update 算法：新 token 集合序列化 vs 原 attribute 原始值，相同不 setAttribute；add/remove/replace 总经此，原值含尾空格/重复时规范化重写）② toggle force 分支 no-op（force 与现状一致直接 return 不 write，保持 attribute 原样；spec toggle(token,force)，WPT checkToggle 保持原样非规范化）③ replace 顺序 + 同名 runUpdate（oldT===newT 存在→runUpdate 规范化；replace 在 oldT 位置换 newT + 移除后续重复有序去重保位置，WPT checkReplace("c b a","c","a")→"a b"）。基线 polyfill 52.11%→53.00%、native 51.84%→52.73%（双路径对等差 0.27pp）；classlist 用例 1360P/60F→1400P/20F（+40）。engine v8 2086 / quickjs 1408 单测，双矩阵 clippy 干净
- R17 已做：① createEvent 移除 9 个 non-createable modern interface（part06 map：wheelevent/pointerevent/popstateevent/progressevent/transitionevent/animationevent/pagetransitionevent/clipboardevent/errorevent，按 WPT someNonCreateableEvents 列表；spec createEvent 仅支持 legacy event interface，modern 走 `new XxxEvent()` 构造器）→ 对其抛 NotSupportedError ② 更新 2 受影响单测（test_event_subclasses2 ProgressEvent 改断言抛、test_window_onerror ErrorEvent 改 new 构造）③ **核实 event target null gap 实际不存在**（_makeEvent part03:1063-1064 已设 target/currentTarget=null，createEvent 初始化测试已 Pass，R14 误记）。基线 polyfill 53.00%→53.20%、native 52.73%→52.93%（双路径对等差 0.27pp）；createEvent 用例 264P/15F→273P/6F（+9）。engine v8 2086 / quickjs 1408 单测，双矩阵 clippy 干净
- R18 已做：① createElementNS 改经**新回调** `__zw_create_element_ns` → host `doc.create_element_ns`（DomMutation::CreateElementNS + apply + callback + shim part06 createElementNS 调本回调），**保留原 qualifiedName 大小写 + prefix + namespace**（spec createElementNS 不小写，区别 createElement HTML 小写）② shim `_nsHandles`（part01，与 `_piHandles` 对称）存 `{qualifiedName, namespace}` 原值 ③ part03 大小写敏感解析 helper `_nsLocal`/`_nsPrefix`/`_nsQualified` ④ part04 get trap：tagName/nodeName/localName/prefix isNs 走 `_nsHandles` 原值读回（不经 `_realTag` 大写化）；**新增 namespaceURI getter**（isNs 读 ns，普通 createElement 元素恒 XHTML）⑤ 更新 R12 prefix 断言（`'SVG'`→`'svg'`）+ 新增 R18 单测（abc/Abc/ABC 三态 prefix + 裸名无 prefix + localName 大小写敏感 + namespaceURI SVG/null/HTML）。基线 polyfill 53.20%→55.11%、native 52.93%→54.46%（双路径对等差 0.65pp）；case.html createElementNS abc/Abc/ABC 三态全 Pass（R12 仅 ABC）。engine v8 2087 / quickjs 1408 单测，双矩阵 clippy 干净
- R19 已做：① classList replace **校验顺序特殊**（两 token 空串 SyntaxError 先于两 token 空白 InvalidCharacterError，区别 add/remove 逐参先空后空白；`replace(" ","")`→SyntaxError）② replace **去重算法重写**（splice(i,1,newT) + 全局有序去重 seen 表，统一覆盖 newT 在 oldT 前后所有情形；`"a b c" replace("c","a")`→`"a b"`）③ `write(arr, force)` 加 force 参数（replace 返 true 时 `write(p,true)` 强制 setAttribute+notify，绕过 runUpdate「值相同 return」；spec replace 返 true 必触发 mutation）④ classList set trap readonly 分支（return true no-op，早于 className/generic fallthrough）⑤ `_clsProxyCache` per-element 缓存（part01+part03，同 `_proxyCache` 模式；spec classList cached accessor identity，`e.classList===e.classList`）+ R19 单测（校验顺序+去重+mutation+assignment no-op+identity）。基线 polyfill 55.11%→55.55%、native 54.46%→54.91%（双路径对等差 0.64pp）；**Element-classlist.html 全量 1400P/20F→1420P/0F（100%）**。engine v8 2088 / quickjs 1408 单测，双矩阵 clippy 干净
- R20 已做：① testharness `map_harness_results` status 映射精确化（原 `0=>Pass,2=>Timeout,_=>Fail` 把上游 status 3(NOTRUN)/4(PRECONDITION_FAILED) 误计 Fail）② `HarnessStatus` 新增 `NotRun`/`PreconditionFailed` 中性变体（精确映射 3→NotRun/4→PreconditionFailed，未知 5+ 保守回落 Fail）③ 通过率统计改 WPT 标准口径 pass/(pass+fail)（中性从分母排除，与上游 dashboard 一致）④ R20 单测（6 种 status 编码精确映射）。解 createEvent 剩 6F（TouchEvent `assert_implements_optional` 失败 = optional legacy touch API 不支持，spec 中性非 Fail；runner exit 1 判定不变仍作 rally 推进信号）。dom/nodes：polyfill 55.55%→55.63%（2501P/1995F+6中性）、native 54.91%→54.98%（2472P/2024F+6中性，双路径对等差 0.65pp，各 6 fail→中性）。wpt-runner v8/quickjs 单测全绿，双矩阵 clippy 干净
- R21 已做：① 扩展 `DOM_TEST_SUBDIRS` + `fetch-dom-subset.sh` SUBDIRS 加 `dom/events`（81 .html + 16 .js，jsdelivr CDN 拉取 0 失败，零生产逻辑改动）② 建立 polyfill dom/events 基线 **31.61%（98P/212F/9timeout，319 subtest，81 cases）**③ 失败聚类分析（Event 对象缺属性[30]/事件分发断言[25+12]/EventListener handleEvent[12]/returnValue[4]/incumbent-global 测试设施[5]；54 个 0-pass 用例）④ 核实 dom/nodes 双路径零回归（native classlist 1420P/0F）。**误报 native events dispatchEvent hang**（R22 推翻——非系统性 hang，是单用例 timeStamp 死循环）。wpt-runner 双矩阵 clippy 干净
- R22 已做：① 定位修复 native dom/events 死循环（**更正 R21 误报**：经诊断插桩 native dispatchEvent+polyfill __zw_parent 均 0 调用→用例走 polyfill dispatch；直接 binary 单用例 Event-dispatch-click 10P 正常；逐用例 timing 定位唯一真死循环 `Event-timestamp-safe-resolution`）② 根因 native `Event.timeStamp` 恒 0（event.rs 硬编码，注释「沙箱无 perf timer」）→ WPT `do-while(e2.timeStamp-e1.timeStamp==0)` 死循环（polyfill 用 `__zw_performance_now` 单调 timer 不 hang）③ 修复 native Event.timeStamp 改 `perf_now_ms()`（OnceLock<Instant> origin elapsed，DOMHighResTimeStamp，Number f64 保子毫秒精度）+ R22 单测（非0/有限 + 连续创建差值可收集非零）④ 建立 **native dom/events 基线 31.29%（97P/213F/9timeout，96s，从 570s 超时→96s）**，双路径对等差 0.32pp。dom_bindings v8 单测全绿，双矩阵 clippy 干净
- R23 已做：① polyfill Event eventPhase 常量补全（part05：Event 构造器 + Event.prototype 各挂 NONE=0/CAPTURING_PHASE=1/AT_TARGET=2/BUBBLING_PHASE=3，Object.defineProperty enumerable:false，guard 幂等；实例经原型链继承，CustomEvent.prototype=Object.create(Event.prototype) 链继承）② R23 单测（4 对象 Event/Event.prototype/createEvent('Event')/createEvent('CustomEvent') × 4 常量 = "0,1,2,3"×4 + 不可枚举）。Event-constants.html 双路径 0P→4P/4（100%）；dom/events：polyfill 31.61%→32.90%（102P/208F）、native 31.29%→32.58%（101P/209F，双路径各 +4 pass，对等差 0.32pp 不变）。engine v8 单测全绿，双矩阵 clippy 干净
- R24 已做：① polyfill `_defineEventSubclass` 父链继承（`_eventSubclassProps` 注册表记录 [ownProps, parentName]，构造器沿父链收集全部 props 设值——MouseEvent extends UIEvent 实例缺 view/detail 根因；子类先父类后，子类覆盖父类 spec 一致；null/undefined 用默认）② KeyboardEvent 改用工厂（extends UIEvent，补 EventModifierInit + key/code/location/repeat/isComposing/charCode/keyCode/which + getModifierState 复用 MouseEvent）③ R24 单测（MouseEvent 默认/设定 + KeyboardEvent 默认含父链 + WheelEvent 三层父链）。Event-subclasses-constructors polyfill 0P→42P/49（native 24P/49）；dom/events：polyfill 32.90%→42.58%（132P/178F，+30 pass）、native 32.58%→36.13%（112P/198F，+11 pass，**双路径差扩至 6.45pp**——native dom_bindings event.rs 旧实现缺父链继承/KeyboardEvent which/MouseEvent instanceof/UIEvent view 校验，R25 对齐）。engine v8 单测全绿，双矩阵 clippy 干净
- R25 已做：① 经 runner 实测诊断（`MouseEvent.toString()` 探 native/polyfill + forced-fail message 带属性状态）确认 native_dom=1 下 `new MouseEvent()` 走 native（覆盖 polyfill），native MouseEvent 缺 UIEvent `view` 父链属性 + KeyboardEvent 缺 `which` ② `set_ui_view` helper（设 view：缺省 null，init dict 对象原样）+ MouseEvent/KeyboardEvent 调之 ③ KeyboardEvent which（缺省回退 keyCode，spec legacy）④ R25 单测（MouseEvent view 缺省/设定 + KeyboardEvent view + which 缺省/显式）。**双路径差未缩**（6.45pp 保持，Event-subclasses native 仍 24P/49——剩余多点分散缺口：WheelEvent 子类链断[父 native MouseEvent 不在 polyfill 注册表]/SubclassedEvent class 语义/MouseEvent 属性细节/UIEvent view 校验）。R25 view/which 是 native 正确性净正（单测证明，default-on 后合规），转高 ROI 切片。dom_bindings v8 单测全绿，双矩阵 clippy 干净
- 剩余聚类（按 ROI，R25 后重排）：① **polyfill 三阶段分发 capture/bubble/stopPropagation**（Event-dispatch 系列 ~44 个 0-pass 主力，R26 高 ROI）② EventListener handleEvent（listener 对象调 .handleEvent）③ Event-cancelBubble setter 语义 ④ 双路径差 6.45pp 收口（WheelEvent 子类链/SubclassedEvent class 语义/native MouseEvent 属性细节/UIEvent view 校验，分散低 ROI）⑤ iframe.contentDocument（深结构 html-compat 域）⑥ querySelector-mixed-case（selector 域）⑦ polyfill appendChild 闭环（M1 L2）⑧ native namespaceURI getter 独立化（dom/nodes 双路径差 0.65pp）⑨ 扩 DOM_TEST_SUBDIRS（dom/collections 等）

**M0 首切片（R0）**: **polyfill vs native A/B 对照门骨架（must-complete 项 5）**
- 理由：入口文档明列 must-complete；纯新增测试文件，零生产代码改动、零碰撞；为后续 M1(L2)/M6(QuickJS) 所有迁移切片提供「行为不退化」安全网（DC-4）；双 feature 可参数化设计为 M6 提前铺路。
- 设计：对同一 HTML + 同一可观测 DOM 操作（如 `getAttribute`/`textContent`/`tagName`/`querySelector` 命中），断言 native 路径（`install_dom_bindings` + `__zw_native_*`）与 polyfill 路径（`generate_js_dom_shim` + `register_dom_callbacks` + `__zw_*`）返回一致。聚焦**行为等价**（可观测结果），不强求 API 形态同构（native=真对象 vs polyfill=Proxy）。

**为何本轮不选 L2-read-only / canvas path-objects 作为首切片**（决策记录）：
- **L2 完整改 polyfill 桥是深结构跨面改**：`register_dom_callbacks` 签名收 `Arc<Mutex<String>>`（dom_html 快照），改读 live Document 需改签名为 `Rc<RefCell<Document>>`，触及 renderer/browser/reftest 三处调用点（callbacks.rs:18-21 注释明示）→ 触发入口文档「深结构护栏」，不宜作首切片。L2 的最小只读子集（仅 getElementById/querySelector getter）虽可行，但缺 A/B 对照门前无法证明行为等价，风险高于先建安全网。
- **canvas path-objects 当前是热碰撞面**：`git log` 核实 canvas 流最近非常活跃（`f7219b2c` 刚改 `js_dom_shim/part05.js` + `canvas.rs` + `testharness.rs`），part05.js 的 canvas 段与本目标 v1.2 接手段并发编辑 → 按入口文档护栏「碰 canvas 共享面前先 git log，有活跃编辑则转零碰撞面」，本轮转 A/B 对照门（DOM 面，零碰撞）。canvas path-objects 待 canvas 流告段落再接手（记「未解决问题」）。

**本轮/本切片进度**: ✅ **完成**——polyfill vs native A/B 对照门骨架已实现并验证（`crates/engine/src/dom_bindings/tests_ab_compare.rs`）。
- 9 条读操作用例（tagName/nodeType/getAttribute/hasAttribute/querySelector(All)/getElementById/descendant/反射 id）+ querySelectorAll 索引读 + 2 个 sanity → **native 与 polyfill 读路径全等价**（4 测试函数全绿）
- 双 feature 验证：v8 矩阵 zero-engine 2063 测试全绿（含新 4 个）；quickjs 矩阵 zero-engine 1405 全绿（A/B 模块 `#[cfg(feature="v8")]` 排除，不影响 quickjs）；双矩阵 clippy 零警告
- 关键结论：**实证 native 读路径与 polyfill 读路径行为等价**——为 M1 L2-read-only 切片（polyfill 桥改读 live Document）提供了可直接复用的 A/B 验收门

---

## 测试基线

| 基线 | 命令 | 当前值 |
|------|------|--------|
| zero-engine 测试（v8） | `cargo test -p zero-engine --features v8 --lib` | ✅ 2088 passed（含 R19 classList replace+assignment+identity 单测，R19 实测） |
| zero-wpt-runner 测试（v8/quickjs） | `cargo test -p zero-wpt-runner --features v8 --lib` | ✅ 181 passed（含 R20 status 映射单测，R20 实测） |
| dom/nodes 通过率口径 | pass/(pass+fail)（中性 NotRun/PreconditionFailed 从分母排除） | polyfill 55.63% / native 54.98%（R20，WPT 标准口径） |
| zero-engine 测试（quickjs） | `cargo test -p zero-engine --no-default-features --features quickjs --lib` | ✅ 1406 passed（A/B 门 + dom_exception 均 cfg(v8) 排除，R2 实测） |
| zero-webview 测试（v8） | `cargo test -p zero-webview --features v8` | ✅ 17 passed（native_dom 接线回归） |
| clippy（v8 + quickjs 双矩阵） | `cargo clippy -p zero-engine ...` | ✅ 零警告（双矩阵） |
| workspace 全量 `make test` | `make test` | ⚠️ 本轮单次超时未跑完（workspace 全量编译+测试+quickjs 矩阵 >580s）；聚焦验证已覆盖变更面（纯测试新增，无生产代码改动） |
| 行覆盖率（全量） | `scripts/check-coverage.sh` | 95.46%（基线；**本地缺 `cargo-llvm-cov`，本轮无法实测**） |
| dom_bindings 覆盖率（独立） | 待补口径（M0 项 4） | ❌ 无口径 |
| product-smoke | `make product-smoke` | 本轮非渲染热路径变更，A/B 骨架（纯测试）豁免 |
| bench-gate | `make bench-gate` | 本轮非 JS 桥热路径代码变更，豁免 |

---

## Coverage 矩阵

| crate/模块 | 行覆盖率 | 趋势 | 备注 |
|------------|----------|------|------|
| dom_bindings（zero-engine 子模块） | ❌ 待补口径 | — | `mod.rs` + 14 子模块 + 5 测试文件（5457 行测试）；M0 项 4 待 `cargo-llvm-cov` 装后补 `check-coverage.sh` 子模块口径 |

**覆盖率口径规则**：不缩范围伪造达标；新代码必带测试；持续提升、不退化。

---

## Latest Evidence

| 日期 | 轮次 | 证据 | 结果 |
|------|------|------|------|
| 2026-08-13 | goal 拆分 bootstrap | 入口文档基线事实块 | 框架占位 |
| 2026-08-13 | R0 核实 | 代码实测（commit `f7219b2c`）：dom_bindings 19 文件 / QuickJS native 真空确认 / L2 仍 re-parse 确认 / canvas wpt-data 缺失确认 / cargo-llvm-cov 未装确认 | 本文件重写，勘误见下 |
| 2026-08-13 | R0 首切片 | A/B 对照门骨架 `tests_ab_compare.rs`（9 读用例 + 索引读 + 2 sanity）；v8 2063 + quickjs 1405 + webview 17 全绿，双矩阵 clippy 干净 | **native 读路径 ≡ polyfill 读路径**（行为等价实证），M0 项 5+6 完成 |
| 2026-08-13 | R1 | WPT dom/nodes 基线：`testharness-dom` 子命令 + `fetch-dom-subset.sh` + `DOM_TEST_SUBDIRS` + Makefile；141 用例 / 2696 subtest / **41.25% pass** | DC-3 首切片达成；暴露 DOMException 抛出语义（~474）/ createProcessingInstruction（44）/ XML doc 模型（98）为最高 ROI 修复方向 |
| 2026-08-13 | R2 | classList token 校验抛 DOMException（双路径）+ native DOMException 构造器（`dom_exception.rs`）+ A/B 门异常路径扩展；v8 2065 / quickjs 1406 / wpt-runner 167 全绿，双矩阵 clippy 干净 | **dom/nodes 41.25% → 56.08%（+14.83pp，400 subtest 净 pass，0 回归）**；Element-classlist.html 单用例 80.3% |
| 2026-08-13 | R3 | createElement 非法标签名抛 InvalidCharacterError（双路径 + spec Name production 校验 helper `is_valid_qualified_name` native / `_zwIsValidQualifiedName` polyfill）+ A/B 门 createElement 异常路径扩展；v8 2068 / wpt-runner 167 全绿，双矩阵 clippy 干净 | dom/nodes 56.08% → **56.45%（+0.37pp，createElement HTML 上下文 invalid 全转 pass，0 回归）** |
| 2026-08-13 | R4 | native node mutation（append/insert/remove/replace）DomError→DOMException 映射（dom crate 已有校验，dom_bindings 此前吞错）+ `dom_error_exception` helper + 3 单测；v8 2072 全绿，双矩阵 clippy 干净 | native 路径规范合规（default-on 生产路径）；**基线不提升**（testharness-dom 走 polyfill，polyfill appendChild 闭环架构限制待 L2）；net≥0 |
| 2026-08-14 | R5 | `testharness-dom-native` 入口（ZW_NATIVE_DOM=1）+ native DOMException constructor 修复（throw_dom_exception 取全局构造器 new + 构造器 invoke 用 This；prototype.constructor 属性 V8 Fatal 回退留下轮）；dom_bindings 197 全绿，双矩阵 clippy 干净 | **建立 polyfill 56.45% vs native 41.25% 双基线**（DC-3 native 对照达成）；定位 native 落后 15.2pp 根因（DOMException prototype.constructor 缺失 → assert_throws_dom "wrong global" ~414） |
| 2026-08-14 | R6 | DOMException identity 三重根因修复（prototype.constructor 补齐 + build_and_register 幂等 + shim part03 check 用 globalThis.DOMException）+ webview overlap 诊断测试；zero-engine v8 2073 / webview_coverage 17 全绿，双矩阵 clippy 干净 | **native dom/nodes 41.25% → 56.08%（+14.83pp，追平 polyfill 56.45%，差仅 0.37pp）**；classList 单用例 native 80.3%（=polyfill） |
| 2026-08-14 | R7 | fetch-dom-subset 补齐 .js 依赖（.html+.js+dom 根共享.js+testharnessreport.js）+ native createProcessingInstruction API（factories invoke + document.rs 注册 + node.rs PI target/data getter + node_name PI→target + spec 校验）+ native PI 单测；dom_bindings 199 全绿，双矩阵 clippy 干净 | **基线真实化**（用例 .js 缺失致跳过→真跑暴露 gap）：polyfill 56.45%→51.12%、native 56.08%→50.79%（双路径对等差 0.33pp）。native PI API 净正（单测证明），PI 用例待 ProcessingInstruction 构造器 |
| 2026-08-14 | R8 | testharness 运行时本地 .js 内联（`inline_local_scripts` + `extract_script_src` + `normalize_relative`）+ `wpt_root`/`case_path` 贯穿 `prepare_harness_html`/`run_testharness_html`/`run_canvas_testharness_html`/`run_testharness_html_inner`；与并行 canvas 流（G5 image fetcher）合并（统一 `&Path`）；wpt-runner v8 168 / quickjs 103 单测全绿，clippy 干净 | **基线进一步真实化**（用例 .js 运行时内联→用例真正跑起来）：subtest ~2696→4490（+1794），polyfill 51.12%→37.82%、native 50.79%→37.59%（双路径对等差 0.23pp，**非回归**——暴露真实 gap：createElementNS 596/createEvent/createProcessingInstruction instanceof 89 等）。完整 JSON 快照入 evidence |
| 2026-08-14 | R9 | polyfill document.createProcessingInstruction 桥接（DomMutation::CreateProcessingInstruction + __zw_create_processing_instruction callback + shim part06 方法/part01 _piHandles/part04 PI 节点包装/part03 构造器占位）+ DOMException identity 对等（createElement/PI 用 globalThis.DOMException，顺带修 R3 既存对等 bug）+ PI 单测；engine v8 2076 / quickjs 1407 / wpt-runner v8 168 / quickjs 103 全绿，双矩阵 clippy 干净 | **基线提升 + PI 双路径对等**：polyfill 37.82%→38.03%、native 37.59%→37.81%（双路径对等差 0.22pp）；PI 用例双路径 1P/11F→6P/6F。关键发现：用例侧 document 始终是 polyfill document（即使 native_dom=1）。完整 JSON 快照入 evidence |
| 2026-08-14 | R10 | polyfill Proxy getPrototypeOf trap（part05 _makeProxy handler：element→HTMLElement.prototype 链、PI→ProcessingInstruction 等）+ DOM 原型方法不可枚举（part03，修 for-in 副作用）+ 顺带修并行 canvas 流回归（part06 CSS.escape/supports 合并 + testharness fetch_handler 编译）+ instanceof 单测；engine v8 2071 / quickjs 1407 全绿，双矩阵 clippy 干净 | **基线提升**：polyfill 38.03%→39.23%、native 37.81%→38.96%（双路径对等差 0.27pp）；cloneNode 用例 0P→51P（+51，instanceof 直接解锁）。归因：6 fetch 既存失败（clean R9 同样）非本切片引入。完整 JSON 快照入 evidence |
| 2026-08-14 | R11 | element.localName getter（part04：HTML 小写/去 prefix/非 Element→null）+ HTML 元素子类 instanceof（part03 注册 ~64 HTML*Element 构造器 + __zwHtmlTagIface tag→接口映射；part05 getPrototypeOf 按 tag 返子类 prototype）+ localName/子类 instanceof 单测；engine v8 2082 / quickjs 1408 全绿，双矩阵 clippy 干净 | **基线提升**：polyfill 39.23%→40.89%、native 38.96%→40.63%（双路径对等差 0.26pp）；cloneNode 用例 51P→121P（+70）。完整 JSON 快照入 evidence |
| 2026-08-14 | R12 | element.prefix getter（part04：限定名冒号前/无冒号→null）+ getElementsByTagNameNS（part04 元素级 + part06 document 级，忽略 ns 按 localName 查）+ 单测；engine v8 2083 / quickjs 1408 全绿，双矩阵 clippy 干净。iframe.contentDocument 评估为深结构跨面改（html-compat 域），记待评估 | **基线提升**：polyfill 40.89%→41.71%、native 40.63%→41.45%（双路径对等差 0.26pp）；polyfill 净 +37 pass。完整 JSON 快照入 evidence |
| 2026-08-14 | R13 | classList token 有序去重（part03 cur() seen 表，`"a a a"`→["a"]）+ contains 空串/含 ASCII 空白→false 不抛（区别于 add/remove/toggle/replace 抛）+ 单测；engine v8 2084 / quickjs 1408 全绿，双矩阵 clippy 干净 | **基线大幅提升**：polyfill 41.71%→46.60%、native 41.45%→46.33%（双路径对等差 0.27pp）；classlist 用例 1140P/280F→1360P/60F（+220，迄今最大单切片提升）。完整 JSON 快照入 evidence |
| 2026-08-14 | R14 | createEvent alias 全覆盖（注册 5 缺失 event 子类 BeforeUnloadEvent/DeviceMotionEvent/DeviceOrientationEvent/TextEvent/TouchEvent + map 扩全集含复数别名）+ 未知 type 抛 NotSupportedError（spec 合规，原 lenient 改抛）+ 更新 event_subclasses2 测试 + 新增 createEvent 单测；engine v8 2085 / quickjs 1408 全绿，双矩阵 clippy 干净 | **基线提升 + 突破 50%**：polyfill 46.60%→50.33%、native 46.33%→50.07%（双路径对等差 0.26pp）；createEvent 用例 96P/183F→264P/15F（+168）。完整 JSON 快照入 evidence |
| 2026-08-14 | R15 | implementation.createDocumentType 返 DocumentType（part06 主 + part03 detached doc，spec 不校验）+ detached doc 加 implementation + 顺带修并行 canvas 流 2 个 clippy 红灯（main 既有，机械修正）+ 单测；engine v8 2086 / quickjs 1408 全绿，双矩阵 clippy 干净 | **基线提升**：polyfill 50.33%→52.11%、native 50.07%→51.84%（双路径对等差 0.27pp）；createDocumentType 用例 1P/81F→80P/2F（+79）。完整 JSON 快照入 evidence |
| 2026-08-14 | R16 | classList toggle force no-op（无变化不 write 保持原样）+ write runUpdate 比较（新集合序列化 vs 原 attribute，相同不 setAttribute）+ replace 顺序/同名 runUpdate（有序去重保位置）；engine v8 2086 / quickjs 1408 全绿，双矩阵 clippy 干净 | **基线提升**：polyfill 52.11%→53.00%、native 51.84%→52.73%（双路径对等差 0.27pp）；classlist 用例 1360P/60F→1400P/20F（+40）。完整 JSON 快照入 evidence |
| 2026-08-14 | R17 | createEvent 移除 9 个 non-createable modern interface（WPT someNonCreateableEvents，spec createEvent 仅 legacy）→ 抛 NotSupportedError + 更新 2 单测（ProgressEvent/ErrorEvent）+ 核实 event target null gap 不存在；engine v8 2086 / quickjs 1408 全绿，双矩阵 clippy 干净 | **基线提升**：polyfill 53.00%→53.20%、native 52.73%→52.93%（双路径对等差 0.27pp）；createEvent 用例 264P/15F→273P/6F（+9）。完整 JSON 快照入 evidence |
| 2026-08-14 | R18 | createElementNS 改经 `__zw_create_element_ns` → host `create_element_ns`（保留大小写+prefix+namespace）+ shim `_nsHandles` + part03 helper + part04 getter（tagName/nodeName/localName/prefix/namespaceURI 大小写敏感）+ 更新 R12 断言 + 新增 R18 单测（三态 prefix + namespaceURI）；engine v8 2087 / quickjs 1408 全绿，双矩阵 clippy 干净 | **基线提升**：polyfill 53.20%→55.11%（2481P/2021F）、native 52.93%→54.46%（2452P/2050F，双路径对等差 0.65pp）；case.html createElementNS abc/Abc/ABC 三态全 Pass（R12 仅 ABC）。`querySelector-mixed-case` 既存失败非回归（R14 同失败，selector 域）。完整 JSON 快照入 evidence |
| 2026-08-14 | R19 | classList replace 四 bug 修复（校验顺序两空串先于两空白 + 去重算法重写 splice+seen + write(force) 强制 mutation + classList set trap readonly + `_clsProxyCache` identity 缓存）+ R19 单测（校验顺序+去重+mutation+assignment no-op+identity）；engine v8 2088 / quickjs 1408 全绿，双矩阵 clippy 干净 | **Element-classlist.html 全量 100%**（1400P/20F→1420P/0F）。dom/nodes：polyfill 55.11%→55.55%（2501P）、native 54.46%→54.91%（2472P，双路径对等差 0.64pp，各 +20 pass）。完整 JSON 快照入 evidence |
| 2026-08-14 | R20 | testharness `map_harness_results` status 精确化（`HarnessStatus` 新增 NotRun/PreconditionFailed 中性变体，3→NotRun/4→PreconditionFailed，原 `_ => Fail` 误计修正）+ 通过率口径改 WPT 标准 pass/(pass+fail) + R20 单测；wpt-runner v8/quickjs 全绿，双矩阵 clippy 干净 | createEvent 剩 6F（TouchEvent `assert_implements_optional` 失败 = optional legacy touch API 不支持）从 fail→中性。dom/nodes WPT 标准口径：polyfill 55.55%→55.63%（2501P/1995F+6中性）、native 54.91%→54.98%（2472P/2024F+6中性，双路径对等差 0.65pp，各 6 fail→中性）。完整 JSON 快照入 evidence |
| 2026-08-14 | R21 | 扩展 DOM_TEST_SUBDIRS + fetch-dom-subset.sh 加 dom/events（81 .html + 16 .js，jsdelivr CDN 0 失败，零生产逻辑）+ polyfill dom/events 基线 31.61%（98P/212F/9timeout）+ 失败聚类 + dom/nodes 双路径零回归核验；wpt-runner 双矩阵 clippy 干净 | **polyfill dom/events 基线 31.61%**（54 个 0-pass 事件分发缺口：Event 属性/三阶段/EventListener/returnValue）。**误报 native events dispatchEvent hang**（R22 推翻：单用例 timeStamp 死循环）。不阻 CI（make test 不含 testharness-dom-native）。完整 JSON 快照入 evidence |
| 2026-08-14 | R22 | 定位修复 native Event.timeStamp 死循环（更正 R21 误报：诊断插桩+直接 binary+逐用例 timing 三步定位到 `Event-timestamp-safe-resolution` 的 do-while(timeStamp差==0) 死循环，根因 native timeStamp 恒 0）+ 修复 native timeStamp 改 perf_now_ms()（OnceLock<Instant>，DOMHighResTimeStamp）+ R22 单测；dom_bindings v8 全绿，双矩阵 clippy 干净 | **native dom/events 基线建立 31.29%**（97P/213F/9timeout，96s，从 570s 超时→96s）。双路径对等差 0.32pp（polyfill 31.61%）。完整 JSON 快照入 evidence |
| 2026-08-14 | R23 | polyfill Event eventPhase 常量（part05 Event 构造器+prototype 挂 NONE/CAPTURING_PHASE/AT_TARGET/BUBBLING_PHASE，enumerable:false，实例经原型链继承）+ R23 单测（4 对象×4 常量 + 不可枚举）；engine v8 全绿，双矩阵 clippy 干净 | **Event-constants.html 双路径 100%**（0P→4P/4）。dom/events：polyfill 31.61%→32.90%（102P/208F）、native 31.29%→32.58%（101P/209F，双路径各 +4 pass，对等差 0.32pp 不变）。完整 JSON 快照入 evidence |
| 2026-08-14 | R24 | polyfill 事件子类 init 属性父链继承（_defineEventSubclass 沿父链收集 props，_eventSubclassProps 注册表）+ KeyboardEvent 改用工厂（extends UIEvent，补 EventModifierInit+key/code/location/repeat/isComposing/charCode/keyCode/which）+ R24 单测（MouseEvent 默认/设定 + KeyboardEvent 父链 + WheelEvent 三层）；engine v8 全绿，双矩阵 clippy 干净 | **Event-subclasses-constructors polyfill 0P→42P/49**（native 24P/49）。dom/events：polyfill 32.90%→42.58%（132P，+30 pass）/ native 32.58%→36.13%（112P，+11 pass）。**双路径差扩至 6.45pp**（native dom_bindings event.rs 待 R25 对齐）。完整 JSON 快照入 evidence |
| 2026-08-14 | R25 | native MouseEvent/KeyboardEvent view（set_ui_view helper，缺省 null/init dict 对象）+ KeyboardEvent which（回退 keyCode）+ R25 单测；经 runner 实测诊断确认 native_dom=1 下 MouseEvent 走 native 覆盖 polyfill；dom_bindings v8 全绿，双矩阵 clippy 干净 | native 正确性净正（view/which，default-on 后合规）。**双路径差未缩**（6.45pp 保持——Event-subclasses native 仍 24P/49，剩余 WheelEvent 子类链/SubclassedEvent class 语义/MouseEvent 属性细节/UIEvent view 校验多点分散缺口，转高 ROI 切片）。dom/events 基线不变（polyfill 42.58% / native 36.13%） |

**本轮勘误**（vs 入口文档基线块）：
1. dom_bindings native API 面**比基线描述更完整**：除 S0–S5 基线外，`mod.rs:558-624` 已注册 querySelector 族 + createElement/Text/Comment/Fragment + documentElement/body/head 全套工厂（注释「R3098/R3131/R3136」）。入口文档「19 文件」清单未列全这些工厂——native 写能力实际比「读 ~15.6x」更广。
2. canvas path-objects 缺口**比基线描述更严重**：本地 `wpt-data/html/canvas/element/` 整个 canvas element 子树都不存在（不止 path-objects），`make fetch-wpt-data` 拉的 wpt-data repo（v1.10）目前只含 reftest 数据（css/fonts/images/quirks），**不含 html/ testharness 用例**。canvas testharness 用例须从上游 wpt 仓库单独导入（非 fetch-wpt-data）。

---

## 下一步计划

1. **R25（本轮，已完成）**：native MouseEvent/KeyboardEvent view + KeyboardEvent which 补全（native 正确性净正，双路径差未缩 6.45pp 保持，转高 ROI） → land
2. **下轮候选（按剩余 ROI，R25 后重排）**：
   - **(a) polyfill 三阶段分发 capture/bubble/stopPropagation**（Event-dispatch 系列 ~44 个 0-pass 主力，R26 高 ROI——DOM 事件桥核心能力，批量解锁）。
   - **(b) EventListener handleEvent**（listener 是对象时调 .handleEvent）。
   - **(c) Event-cancelBubble setter 语义**（独立小切片）。
   - **(d) 双路径差 6.45pp 收口**（WheelEvent 子类链/SubclassedEvent class 语义/native MouseEvent 属性细节/UIEvent view 校验，分散低 ROI，按需）。
   - **(e) native namespaceURI getter 独立化**（dom/nodes 双路径差 0.65pp）。
   - **(f) querySelector-mixed-case**（selector 域）。
   - **(g) iframe.contentDocument**（深结构 html-compat 域）。
   - **(h) dom_bindings coverage 口径**（M0 项 4）：装 `cargo-llvm-cov` 后补。
   - **(i) 主线里程碑推进**：M1 L2 / M6 QuickJS native——均为深结构，评估切片化可能。
3. **后续主线**：M1 L2（polyfill-live 合一，解 polyfill appendChild 闭环限制）→ M2 S6 → M3 SPA/WC → M4 WPT dom 持续扩 → M5 V8 default-on（待用户决策）→ M6 QuickJS native → M7 双引擎 default-on + 收尾；M8 canvas path-objects 待 canvas 流告段落接手

---

## 待用户决策清单（深结构护栏）

| 事项 | 触发条件 | 状态 |
|------|----------|------|
| V8 `ZW_NATIVE_DOM` default-on（改 Mission 级单向门，M5） | M1–M4 完成、V8 native 路径生产就绪 | 待 M5 启动前征询 |
| QuickJS `ZW_NATIVE_DOM` default-on（改 Mission 级单向门，M7） | M6 QuickJS native 移植完成 | 待 M7 启动前征询 |

> 本轮 A/B 对照门骨架为纯测试新增，不触发上述门禁。

---

## 未解决问题

1. **`cargo-llvm-cov` 本地未安装** → dom_bindings 独立 coverage 口径（M0 项 4）本轮无法实测落地。方案：`cargo install cargo-llvm-cov` + `rustup component add llvm-tools-preview`，再扩 `check-coverage.sh` 加 `cargo llvm-cov -p zero-engine` 子模块报告。本轮记入，下轮处理（安装非阻塞）。
2. **canvas path-objects 是热碰撞面**：canvas 流（`f7219b2c` 等）正在活跃编辑 `part05.js` canvas 段 + `canvas.rs`。M8 path-objects 接手须等 canvas 流告段落，或确认 part04/05 path-objects 段无并发编辑后再动。碰撞信号点：`git log --since="14 days" -- crates/engine/src/js_dom_shim/part05.js crates/canvas/src/`。
3. **canvas wpt-data html 子树缺失**：canvas testharness 用例（含 path-objects）需从上游 `web-platform-tests/wpt` 仓库单独导入到 `wpt-data/html/canvas/element/`，`make fetch-wpt-data` 不提供。M8 接手第一动作。
4. **polyfill appendChild/insertBefore 闭环校验架构限制**（R4 发现）：polyfill 桥的 mutation 经 `__zw_append_child` 回调延迟批处理（`apply_dom_mutations` 在脚本执行后 apply），且 shim 层 `_makeProxy` 只有 selector/handle 无 live 祖先链——无法在 `appendChild` 调用点同步抛 HierarchyRequestError。native 路径已修（R4），polyfill 待 M1 L2 polyfill-live 合一（shim 改读 live Document 后才有祖先链）。
5. **testharness-dom 仅测 polyfill 路径**（R4 发现，**R5 已解**）：R5 加 `testharness-dom-native` 入口（ZW_NATIVE_DOM=1）。
6. ~~**native DOMException identity**（R5 定位）~~ → ✅ **R6 已修**：三重根因全部修复。native dom/nodes 41.25% → 56.08% 追平 polyfill（注：R7 .js 依赖补齐后基线真实化为 50.79%，双路径对等）。
7. **PI 用例超时 + ProcessingInstruction 构造器缺失**（R7 发现，R8/R9 持续缓解）：Document-createProcessingInstruction.html 外层 `test()` 嵌套多个 `test()` + `pi instanceof ProcessingInstruction`（构造器未装）→ testharness completion callback 未调 → 超时。**R9 已交付 polyfill createProcessingInstruction + ProcessingInstruction 构造器占位**（PI 用例 1P/11F→6P/6F，超时不再），但 `instanceof ProcessingInstruction/Node` 仍 false（polyfill Proxy instanceof 需 getPrototypeOf 深改，属 instanceof 89 块，下轮候选 a）。
8. **基线真实化**（R7→R8→R9 持续）：R7 补 .js 文件 → 51.12%/50.79%；R8 让 .js 运行时内联执行 → 37.82%/37.59%；**R9 PI 桥接 + DOMException identity 对等** → polyfill 38.03% / native 37.81%（双路径对等差 0.22pp）。
9. **用例侧 document 始终是 polyfill document（R9 发现）**：即使 `ZW_NATIVE_DOM=1`，顶层 `globalThis.document` 仍是 polyfill shim 装的（part06.js）。native document template（dom_bindings 装的方法）用例访问不到。故 polyfill document 必须实现所有 document.* 方法，native dom_bindings document 方法仅作生产路径（default-on 后）能力。**对等含义**：driving 用例经 polyfill document 跑，native 路径修复须经 polyfill document 同步才能基线可见（R4 注释的架构限制的另一面）。
10. **DOMException identity 对等**（R9 修复）：native_dom 叠加路径下，shim 裸 `new DOMException(...)` 抛的异常 → testharness `assert_throws_dom` "wrong global"（词法作用域 part01b DOMException ≠ 全局 native DOMException）。R9 已修 createElement/PI 校验路径（改 `globalThis.DOMException`）。**R3 createElement 既存对等 bug 同步修**（但其 0P/147F 主因是 instanceof Element + iframe，非 DOMException）。其余 shim 抛 DOMException 路径（如需）应同样用 `globalThis.DOMException`。
11. **6 个 fetch/response 测试既存失败**（R10 归因，并行流引入）：`test_fetch_abort_signal_r3044`/`test_fetch_forbidden_headers_r3221_r3222`/`test_fetch_response_binary_body_r3021`/`test_request_signal_passthrough_r3045`/`test_response_body_readable_stream_r2967`/`test_response_request_constructors_r2968`。clean R9 tree 同样失败（非 js-dom 流引入）。根因 `instanceof Response`=false（fetch 结果非 Response 实例，`_makeResponseFromWire` 路由问题）+ fetch abort/binary/stream/signal/forbidden-headers。**归因 fetch/net 域**（并行 canvas/net 流引入），非 js-dom DOM 桥工作面（run-rules §9 工作面不重叠）。修复需深入 fetch 桥接，留给引入它的流或专项切片。注：本切片修了同源的 CSS.escape 回归（part06 工作面内，canvas 流 part05 CSS.percent/deg 破坏 part06 CSS 定义顺序）。
12. **instanceof 具体元素子类剩余**（R10 部分解，**R11 已修**）：R10 解了 instanceof Element/HTMLElement/Node（cloneNode 0P→51P）；**R11 注册 ~64 HTML*Element 构造器 + tag 映射，getPrototypeOf 按 tag 返子类 prototype**，cloneNode 51P→121P。**剩余边缘**：`createElement('canvas') instanceof HTMLCanvasElement` 仍 false——canvas 经 `_zwMakeCanvas()` 特殊 proxy（canvas 流专用路径），不走 _makeProxy/getPrototypeOf。canvas proxy instanceof 待 canvas 流或专项协调。
13. **iframe.contentDocument 是当前最大失败块，深结构跨面改（R12 评估确认）**：createElementNS/case/cloneNode 用例 XML/XHTML iframe document 路径 `Cannot read properties of undefined (reading 'documentElement')`（~390 subtest）。R12 评估：完整需求 = iframe src 真实解析 + 独立 Document + 独立 window（`doc.defaultView.DOMException`）+ 跨文档节点归属（`element.ownerDocument === doc`）+ createElementNS 子文档工作。属 **html-compat 域深结构**（跨文档 + iframe 解析），远超 js-dom 轻量切片。待评估为独立切片或转 html-compat 流。R12 转 prefix/getElementsByTagNameNS 零碰撞面切片。
14. **prefix getter 大写化限制（R12）**：polyfill `_realTag` 强制大写（HTML 语义），createElementNS('x','Abc:local') 的 prefix 返 'ABC'（大写）。case.js 测 abc/Abc/ABC 三态 prefix，仅 ABC 态匹配，abc/Abc 仍 fail。正确修复需 createElementNS 保留原 qualified name 大小写（深改 host `__zw_create_element`/tag 存储，目前 createElement 统一小写、_realTag 统一大写）。待 createElementNS 大小写深改切片。（注：R18 createElementNS 改经 `__zw_create_element_ns` 已解 case.js 三态，本条仅影响 createElement 非 NS 路径的 prefix，低优先级）
15. ~~**native dom/events dispatchEvent hang（R21 发现）**~~ → ✅ **R22 已修（更正 R21 误报）**：非系统性 dispatchEvent hang。R22 经诊断插桩（native dispatchEvent+polyfill __zw_parent 均 0 调用）+ 直接 binary 单用例（Event-dispatch-click 10P 正常）+ 逐用例 timing，定位到**唯一真死循环** = `Event-timestamp-safe-resolution` 的 `do-while(e2.timeStamp-e1.timeStamp==0)`——根因 native `Event.timeStamp` 恒 0（event.rs 硬编码）vs polyfill 用单调 perf timer。R22 修复 native timeStamp 改 `perf_now_ms()`（OnceLock<Instant>）。native dom/events 全量 570s 超时→96s，基线建立 31.29%（双路径对等差 0.32pp）。**注**：`make testharness-dom-native` 现可跑完（~96s events + nodes），10 个慢用例 ~10s each 命中 CASE_TIMEOUT 正常超时（非缺陷）。

---

## 归档记录

> 已完成的 milestone/切片记录到 `archive/`。

- R0：M0 A/B 对照门骨架 → [archive/m0-slice-ab-gate-skeleton.md](archive/m0-slice-ab-gate-skeleton.md)
- R1：M4 WPT dom/nodes 基线首切片 → [archive/m4-slice-wpt-dom-nodes-baseline.md](archive/m4-slice-wpt-dom-nodes-baseline.md)
- R2：M4 classList DOMException 修复（双路径 + native DOMException 构造器）→ archive/m4-slice-classlist-dom-exception.md
- R3：M4 createElement 非法标签名校验（双路径 + spec Name production helper）→ archive/m4-slice-create-element-validation.md
- R4：M4 native node mutation DomError→DOMException（append/insert/remove/replace）→ archive/m4-slice-node-mutation-dom-exception.md
- R5：M4 testharness-dom native 路径对照 + native DOMException constructor 修复（部分）→ archive/m5-slice-native-baseline-domexception-constructor.md
- R6：M4 DOMException identity 三重根因修复（native 追平 polyfill 56.08%）→ archive/m6-slice-dom-exception-identity.md
- R7：M4 fetch .js 基线真实化 + native createProcessingInstruction API → archive/m7-slice-js-deps-createprocessinginstruction.md
- R8：M4 testharness 本地 .js 内联 + wpt_root 贯穿合并（基线真实化 178 用例 / 4490 subtest）→ archive/m8-slice-testharness-local-js-inline.md
- R9：M4 polyfill createProcessingInstruction 桥接 + DOMException identity 对等（PI 1P/11F→6P/6F）→ archive/m9-slice-createprocessinginstruction.md
- R10：M4 polyfill Proxy getPrototypeOf 解 instanceof + CSS 回归修复（cloneNode 0P→51P）→ archive/m10-slice-instanceof-prototype-chain.md
- R11：M4 element.localName getter + HTML 元素子类 instanceof（cloneNode 51P→121P）→ archive/m11-slice-localname-subclass-instanceof.md
- R12：M4 element.prefix getter + getElementsByTagNameNS（polyfill +37 pass）→ archive/m12-slice-prefix-getelementsbytagnamens.md
- R13：M4 classList token 去重 + contains 空白不抛（classlist +220 pass，迄今最大单切片）→ archive/m13-slice-classlist-dedupe-contains.md
- R14：M4 createEvent alias 全覆盖 + 未知 type 抛 NotSupportedError（createEvent +168 pass，dom/nodes 突破 50%）→ archive/m14-slice-createevent-aliases-notsupported.md
- R15：M4 implementation.createDocumentType + detached doc implementation（createDocumentType +79 pass）+ 修 canvas 流 2 clippy 红灯 → archive/m15-slice-createdocumenttype.md
- R16：M4 classList toggle no-op + write runUpdate + replace 顺序（classlist +40 pass）→ archive/m16-slice-classlist-toggle-replace.md
- R17：M4 createEvent 移除 9 non-createable modern interface（createEvent +9 pass）+ 核实 event target null gap 不存在 → archive/m17-slice-createevent-noncreateable.md
- R18：M4 createElementNS 大小写敏感（解 R12 case.js 三态）+ namespaceURI getter（polyfill +1.91pp / native +1.53pp）→ archive/m4-slice-createelementns-case-sensitive.md
- R19：M4 classList replace 校验顺序/去重/mutation + assignment readonly + identity 缓存（Element-classlist.html 100%，各 +20 pass）→ archive/m4-slice-classlist-replace-and-assignment.md
- R20：M4 testharness PRECONDITION_FAILED/NOTRUN 中性 status 精确化（createEvent 6 TouchEvent fail→中性，通过率口径 WPT 标准）→ archive/m4-slice-testharness-precondition-status.md
- R21：M4 导入 dom/events 子目录建立 polyfill 基线 31.61%（54 个 0-pass 事件分发缺口）+ 发现 native events dispatchEvent hang（R22 首要）→ archive/m4-slice-dom-events-baseline.md
- R22：M4 native Event.timeStamp 死循环修复（更正 R21 误报，timeStamp 恒 0→perf_now_ms）+ native dom/events 基线 31.29%（96s，双路径对等差 0.32pp）→ archive/m4-slice-native-event-timestamp-hang.md
- R23：M4 Event eventPhase 常量（NONE/CAPTURING_PHASE/AT_TARGET/BUBBLING_PHASE，Event-constants 双路径 100%，各 +4 pass）→ archive/m4-slice-event-phase-constants.md
- R24：M4 事件子类 init 属性父链继承 + KeyboardEvent 工厂化（Event-subclasses polyfill 42P/49，dom/events polyfill 42.58% +30P / native 36.13% +11P，双路径差扩至 6.45pp）→ archive/m4-slice-event-subclass-parent-chain.md
- R25：M4 native MouseEvent/KeyboardEvent view + KeyboardEvent which（native 正确性净正，双路径差 6.45pp 保持未缩——WheelEvent 子类链/SubclassedEvent 等多点分散，转高 ROI）→ archive/m4-slice-native-event-view-which.md
