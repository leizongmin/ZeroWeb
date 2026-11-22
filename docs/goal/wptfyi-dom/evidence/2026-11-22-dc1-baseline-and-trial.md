# DC-1 · 基线与试跑（06 DOM 与 Shadow DOM）

日期：2026-11-22。WPT 语料 pin：`315976933870b34d6ea30e3f6643403edae678ba`（lib.sh 共享 pin）。ZeroWeb 基线 commit：`6d7407da2`（本轮开工 pull --rebase 后）。

## 口径

- 分母口径：[coverage-2026-11-22 快照](../../../compat/wptfyi/coverage-2026-11-22.md)（wpt.fyi aligned stable 三家并集 @`a3830cd8f7`，2026-11-20 runs；子测试取 `c[1]` 最大值）。11-07 快照同口径；两快照间其他流落的修复已体现在 11-22 数字里，本 goal 以 11-22 为起点基线。
- 语料 revision 差：本 goal 语料 pin `31597693`，快照分母 @`a3830cd8f7`。净增按 pin 不变口径计；上游漂移（post-pin 新增面）单列（见「漂移」节）。
- 试跑入口：`make testharness-dom` / `testharness-web-components` / `testharness-shadow-dom` / `testharness-quirks` / `testharness-domparsing` / `testharness-domxpath`（test-guard 包裹），日志见本目录 `dc1-*.log`。
- 运行环境注意：语料 fetch 需先 `source ~/use-proxy.sh`（直连 raw/CDN 超时，2026-11-22 首射实证）；fetch 撞网络超时先查代理再重试。

## 基线（coverage-2026-11-22）

| 目录 | 分母子测试 | 通过 | 缺口 | 通过率 | 覆盖文件 |
|---|---:|---:|---:|---:|---:|
| dom | 67,148 | 57,247 | 9,901 | 85.25% | 359/863 |
| shadow-dom | 15,368 | 8,519 | 6,849 | 55.43% | 172/387 |
| quirks | 6,984 | 40 | 6,944 | 0.57% | 6/55 |
| custom-elements | 4,854 | 2,842 | 2,012 | 58.55% | 175/224 |
| sanitizer-api | 3,834 | 0 | 3,834 | 0% | 0/51 |
| trusted-types | 3,764 | 0 | 3,764 | 0% | 0/249 |
| domparsing | 3,757 | 121 | 3,636 | 3.22% | 25/100 |
| domxpath | 1,172 | 21 | 1,151 | 1.79% | 32/37 |
| webidl | 838 | 0 | 838 | 0% | 0/63 |
| xml | 252 | 0 | 252 | 0% | 0/30 |
| **合计** | **107,971** | **68,790** | **39,181** | 63.71% | — |

对 11-07 账面（64,947 通过 / 缺口 41,603）：shadow-dom 33.30%→55.43%（对端 shadow-dom-rendering 流连带收益，按「收益归测试路径所在 goal」规则计入基线）；custom-elements 64.57%→58.55%、dom 85.75%→85.25% 回落为上游分母漂移（post-pin 新增面入分母）。

## 缺口分解（DC-1 核对结论）

用 pin 树（GitHub trees API @`31597693`）× wpt.fyi 清单 × 本地语料三方交叉对账后，缺口按可行动面分三类：

### 一、引擎工作 bound（主体，~34,500）

| 簇 | 缺口 | 性质 |
|---|---:|---|
| shadow-dom/reference-target | 6,233 | `referenceTarget` API 未实现（33/6,266；CI 语料已转正 15/16 文件） |
| dom/nodes 失败面 | ~3,100 | 语料在 pin 已齐（本地 208 den 文件全覆盖 pin 静态面），缺口为已覆盖文件引擎失败 |
| quirks/unitless-length | 4,784 | 样式系统 quirks 解析（rendering 流写面，协作划界）；excluded-properties 三大文件在跑、99% 失败；mode 判定 R5346 已在 |
| quirks/hashless-hex-color | 2,060 | 同上（样式解析）＋ 3 文件皆 iframe 形态（test-ref-iframe.js），runner 无 iframe 文档管道 |
| sanitizer-api | 3,834 | API 未实现；语料 pin 在（50 文件可 fetch）、通道未建 |
| trusted-types | 3,764 | 同上（246 文件） |
| domparsing XML 面 | 3,636 | DOMParser .xhtml/XML 文档管道缺失（runner 页锚只认 .html；covered 3.22%） |
| dom/ranges 失败面 | ~2,500 | 语料 pin 已齐（59/59），已覆盖文件引擎失败 |
| xml | 252 | XML 管道 + 语料/通道（xslt 6 文件 193st） |
| webidl | 838 | idlharness 通道 + 语料（63 文件 pin 在） |
| domxpath | 1,151 | XPathEvaluator shim 轴覆盖面（R2981 实用子集在） |
| custom-elements 引擎面 | ~1,000 | registries static 2,779st（本地 31 文件已跑 1,976 过）＋ reactions/form-associated/parser 失败面 |
| dom/observable | 525 | Observable API 未实现（语料基本 post-pin 漂移面） |
| dom/events 失败面 | ~360 | scrolling/ iframe 面 + 捕获/组合事件失败 |

### 二、变体映射 / runner 包装面（runner 工程，~2,600）

| 面 | 缺口 | 机制 |
|---|---:|---|
| dom idlharness window 面 | 1,986 | `idlharness.window.html?include/exclude` 虚拟路径；上游无静态 .html，需 wrapper 生成（any_js_window_wrapper 先例） |
| dom idlharness worker 面 | 660 | `.any.worker/.sharedworker/.serviceworker.html` 虚拟路径 |
| dom/nodes .window.html 面 | （含在漂移） | post-pin 新增虚拟形态为主；wrapper 机制与上一致 |
| custom-elements/registries .window 面 | ~150 | scoped registry 虚拟形态（多半 post-pin） |

### 三、上游漂移（pin bump 前不可回收，单列）

post-`31597693` 上游新增面在我 goal 各目录约 12,000+ 子测试（dom 406 文件/10,949st、shadow-dom/declarative 与 reference-target 增量、CE registries/form-associated ~40 文件、quirks/unitless-length 三文件本地 67st vs 快照分母 4,784st＝上游按属性枚举扩写，试跑实证，修订本节初稿的「quirks 微量」）。bump 共享 `lib.sh WPT_REV` 是跨流共享面（所有 goal 脚本共用），不在本流单方权限内——记入待决，DC-3 前按组合口径统一处理。

### 语料补全可回收面（小，不构成主路径）

- dom/events 非 scrolling 面 +71 文件（~116st）；shadow-dom loose 事件 retargeting 4 文件；sanitizer-api/trusted-types/webidl/xml 语料 fetch 本身（随 DC-2 API 工作入通道）。
- dom/nodes、dom/ranges、dom/traversal、dom loose static：**pin 语料已齐**——「补语料」不是本 goal 的主杠杆（DC-1 开工前假设已被三方对账推翻，记录在此防后续轮重走）。

## 试跑结果（守卫下，commit 6d7407da2 干净树构建）

六腿串行（test-guard 包裹，日志 dc1-*.log，汇总 dc1-tally.txt）：

| 腿 | Pass | Fail | Timeout | 计 | 对快照（CI 11-20） |
|---|---:|---:|---:|---:|---|
| testharness-dom | 57,471 | 683 | 12 | 58,172 | 57,247（+224，引擎自 CI run 后推进） |
| testharness-web-components | 11,948 | 1,300 | 17 | 13,273 | shadow-dom+CE=11,361（差值 586 为 the-template-element 586st——归 05 html/ 计分，不入本 goal）；本 goal 两目录净面 11,362 vs 11,361 ✓ |
| testharness-shadow-dom（渲染补充池） | 11 | 90 | 0 | 101 | 池通道（45 号流所有） |
| testharness-quirks | 40 | 35 | 0 | 75 | 40 ✓ 逐项一致 |
| testharness-domparsing | 149 | 1,068 | 0 | 1,217 | 121（+28 引擎推进） |
| testharness-domxpath | 21 | 102 | 1 | 124 | 21 ✓ |

**一致性定谳**：本地实跑与 CI 记分管线逐目录对齐（registries 1,976、declarative 7,702、quirks 40、domxpath 21 与快照全等）——通道接线与语料完整，DC-1「缺失产物/变体映射」核对通过。残余差额全部归因清楚：引擎推进漂移（dom +224、domparsing +28）与 template 计分归属（586st 归 05）。

**reference-target 试跑切片**：16 文件全入扫，本地面 33P/106F（与快照 33P 一致）。property-reflection 三大矩阵各只注册 ~30 子测试（helper 按 `reflected_property in createElement` 早退）——wpt.fyi 分母 6,266st 里 ~6,127 在我们侧是「未注册」而非「注册失败」：实现 `referenceTarget` 属性本身即让矩阵注册展开（后续行为面再逐簇收敛），这条会计路径写进 DC-2 首切片计划。写侧定位（探针 [dc1-wc-reftarget-probe.log](dc1-wc-reftarget-probe.log) 逐文件计数）：get 侧 `property-reflection.html` 30P/1F（IsNull 反射已通），`-idl-setters.html` 与 `-imperative-setup.html` 双双 0P/30F＝**写反射（setter→关联建立）是缺失引擎面**，首切片以此为靶。

**domparsing 试跑发现**：`new CustomElementRegistry()` 在 XML 解析面测试里抛 `Illegal constructor`（shim 构造器守卫误伤合法调用）——记 DC-2 domparsing 簇伴生修复点。

Timeout 面（dom 12 + wc 17）集中在多全局/incumbent/动画事件/跨域命名访问页，属已知 runner 边界，不另挂账。

## DC-2 优先序（按净增/成本比）

1. Reference Target API（6,233，最大单簇，DOM 语义面在本流写面内）。**首切片已甄别（2026-11-22）**：6,266st 中 88%（5,536st）集中在 property-reflection 三文件（property-reflection.html 1,846 / -idl-setters 1,845 / -imperative-setup 1,845）——纯 `Element.referenceTarget` / `ShadowRoot.referenceTarget` 属性 + `shadowrootreferencetarget` 声明式属性的 IDL 反射语义面，不依赖 testdriver/accname；其余 ~730st（aria-labelledby/label-for/form/commandfor 等）走 `test_driver.get_computed_label`，需 accname 管道，后置。setHTMLUnsafe 已实现（shim part05.js s180），dom-mutation 面（15st）可随批。
2. dom/nodes + dom/ranges 已覆盖失败面收敛（~5,600，按失败分类逐簇）。
3. sanitizer-api / trusted-types：语料 fetch + runner 通道 + API 实现（7,598）。
4. domparsing XML 文档管道（3,636；连带 xml/ 252）。
5. dom idlharness wrapper 面（2,646）。
6. quirks 样式解析（6,844；与 rendering 流协作划界后动手）。
7. domxpath 轴扩展（1,151）、CE 引擎面（~1,000）、dom/observable（525）。
8. webidl 通道（838）。
