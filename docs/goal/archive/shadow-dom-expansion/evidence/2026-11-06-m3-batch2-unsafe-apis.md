# M3 Batch 2 — 簇 A unsafe API 三件套（setHTMLUnsafe / getHTML / parseHTMLUnsafe）

**日期**: 2026-11-06
**范围**: gethtml.html 主力翻转 + opt-in 页 parseHTMLUnsafe / DOMParser / XHR 面 + canvas 1F 顺带
**基线**: `2026-11-06-m2-baseline-after.json.gz`（M2 收口后全 WC 通道 392 文件逐子测试身份）

## 改动面

### zero-dom

- `node.rs`: `ShadowRootData` 新增 `declarative` / `delegates_focus` / `clonable` / `serializable` 四状态位
  （`ShadowRootData::new` 全 false）。
- `document/shadow.rs`: `attach_shadow_full(host, mode, df, cl, srl, declarative)`——校验同旧
  `attach_shadow`，状态位落 `ShadowRootData`；`attach_shadow` 降为薄包装（全 false）。
  `materialize_declarative_shadow_roots` 从属性 presence 读三状态位传入。
- `parser.rs`: `parse_html_fragment_with_scripting_materialized`——unsafe 双 API 消费面
  （片段解析 + 物化，whatwg/dom#831）。
- `serializer.rs`: ShadowRoot 布尔属性回写，顺序按 HTML 序列化 spec
  （shadowrootmode → delegatesfocus → serializable → clonable，`=""` 空值形态）。
  单测抓出首版 bug：mode 值闭引号缺失致整 tag 形态错（`shadowrootmode="closed
  shadowrootdelegatesfocus=""`）——修复为 mode 值闭引号后追加，无旗标位输出不变。

### zero-engine

- `js_dom_bridge.rs`:
  - `DomMutation` 新增 `SetInnerHtmlUnsafe{selector,html}` / `SetInnerHtmlOnHandleUnsafe{handle,html}`；
    `AttachShadow*` 增 `delegates_focus` / `clonable` / `serializable` 三旗标。
  - `replace_inner_html_unsafe`：remove children + 物化片段解析（shadow root 子 arm 用
    `attach_shadow_full` 深拷贝）重建。
  - `parse_html_subtree_deep_json_unsafe` / `deep_entry_json_unsafe` / `shadow_root_entry_json`：
    物化深臂 JSON（`sr` 字段 = `{m,d,c,s,nodes}`，ShadowRoot 子从 kids_src 过滤）。
  - `parse_html_element_json_full`：无 URL 查询视图（DOMParser / XHR responseXML / detached doc）
    翻转为 **unmaterialized**（opt-in 语义——模板存活）；URL 变体（iframe 子文档）维持物化。
- `callbacks.rs`: `__zw_set_inner_html_unsafe` / `__zw_set_inner_html_handle_unsafe` /
  `__zw_parse_html_subtree_unsafe` + attachShadow 旗标 wire（`'1'/'0'`）。

### shim（js_dom_shim part03/04/05）

- part03: `_zwMBuildBodyTree` 第 4 参 `materialize`（缺省 falsy = 旧行为，物化深臂换
  `__zw_parse_html_subtree_unsafe`）；`ShadowRoot.prototype.getHTML/setHTMLUnsafe`；
  detached doc `open/close` 最小面 + write 物化臂（`__zwUnsafeMaterialize` 印记）；
  静态 `Document.parseHTMLUnsafe`（复用 detached 工厂 + 印记，spec
  dom-document-parsehtmlunsafe）。
- part04: 元素/容器双域 `getHTML` / `setHTMLUnsafe` trap 臂；Shadow proxy
  serializable / clonable getter；innerHTML getter **shadow echo 剔除**；outerHTML getter
  引擎臂/handle 臂 echo 剔除。
- part05: `_zwAttachDeclarativeSRView`（深树 sr 消费 → 本地视图声明式根）；hydrate d/c/s
  状态位下发；`_attachShadow` / 工厂 attachShadow **声明式重挂接分支**（TypeError 校验先于
  NotSupportedError，全成员相等返现有根，命令式恒抛）；`_zwTransferDeclarativeSR`
  （setHTMLUnsafe 重建后 `__ds{n}` 句柄 + `__zw_declare_fragment` 引擎绑定）；
  `_zwFragmentAddedUnsafe` / `_zwSetHtmlUnsafeImpl`（setHTMLUnsafe 主路径：旧子快照移除
  + 物化重建 + SCRIPT 递归盖章 + t8r rebuild + 声明式转移）；`_zwGetHtmlOpts180` /
  `_zwSrQualifies180` / `_zwResolveRoot180` / `_zwSerNode180` / `_zwGetHtmlKids180` /
  `_zwGetHtmlImpl`（getHTML 序列化域：资格根模板形按 spec 属性序注入宿主开标签后，
  未资格根不回写）。
- **echo 剔除语义**（spec：物化根不入 light 子序列化面）：快照视图 unmaterialized 解析让
  已物化根以 `<template shadowrootmode>` 形态存活于融合 childNodes/引擎串——宿主有注册根
  时（`_zwHasRoot180`）剔该形态子/串段（`_zwIsShadowEcho180` / `_zwStripEchoHtml180`）。
  引擎子树内带 shadowrootmode 的 template 只能来自真根（物化解析消费一切合法声明式模板），
  判定按属性 presence 精确。FIXME 挂账：根内容嵌套 template 时字符串非贪婪止于内层闭 tag
  （引擎物化不达 shadow 内部模板，与嵌套 DSD Batch 1 同刀）。
- standalone canvas 补 `append`（委托同 handle trap 代理，R57 innerHTML/outerHTML 同款桥）——
  gethtml.html 命令式臂 `element.append(...)` 曾 TypeError。

### runner 基建

- `testharness.rs`: `ZW_TIMEOUT_MULTIPLIER` env → runner 案 deadline 与 harness 页预算同乘
  （wptrunner `--timeout-multiplier` 同义面，testharness.js 原生 `setup({timeout_multiplier})`
  消费；缺省 1.0 零行为变化）。mega-case 页预算放大完整上报用。

## 单测（zero-dom）

- `materialized_fragment_parse_attaches_flags`：物化片段解析消费 template + 四状态位读入
  （首版 serializer 引号 bug 由此抓出）。
- `attach_shadow_full_flags_serialize_in_spec_order`：布尔属性 spec 序回写 + 全 false 对照
  + attach_shadow 薄包装默认位。
- 全包 `cargo test -p zero-dom`：885+2+1+2+1+2 全绿 0F。

## gethtml.html 结果（solo 复跑 ×3 + 全通道复跑，默认页预算）

solo 复跑（run1 05:18 / run2 05:27 / run3 05:31，均为 canvas append 修复前二进制）逐轮
一致：**4F 全部为 canvas `element.append is not a function`**——run3 2,239 行 =
2,234P/4F/1T（页级 T）。全通道复跑（05:33，修复后二进制）：

| 文件 | 子测试 | Pass | Fail | Timeout |
|---|---:|---:|---:|---:|
| declarative/gethtml.html | 2,211 | 2,210 | 0 | 1（页级，预算截断） |
| declarative/gethtml-ordering.html | 3 | 3 | 0 | 0 |
| reference-target/tentative/gethtml-serialization.html | 3 | 0 | 3 | 0 |

- canvas ×4 append 修复经全通道复跑确认翻绿：预算内真实执行行 **0F**。
- 修正：初稿表格「2,231P/7F」为误计（run3 实测 2,234P/4F）；referenceTarget ×3 不在
  gethtml.html 内，而在独立文件 gethtml-serialization.html（3F，簇 C
  `shadowrootreferencetarget` 序列化，挂账不变，Batch 3 候选）。
- gethtml.html 长尾 = 页预算截断：oracle 潜在 ~5,000 子测试，long=60s 预算内真实执行
  ~2,210 行；基线 4,506F+1T 为 fail-fast 瞬报（未真实执行）。截断与 per-subtest 成本
  归因挂账（研删探针 zz-cost/zz-divx/zz-trunc 追踪中）。
- （注：×4 乘数未放大子测试上报量——`setup({timeout_multiplier})` 注入时机与 Tests
  构造次序疑点，挂账 runner 基建跟进；本轮 A/B 用默认预算与基线同条件。）

### 截断归因：时间成本模型排除，楔死（hang）假设成立中

定量排除「per-test 成本随计数增长」（无论线性外推还是二次总量）：

- 300 测试探针（zz-cost 四象限）实测 ~4ms/test 线性；若成本恒定，2,210 × 4ms ≈ 8.8s
  远低于 60s 页预算——页面不该截止。
- 若 per-test 成本随累计计数线性涨（总量二次）：60s→240s（×4）应给 ~2× 行数
  （N ≈ √(2D/c)），实测仅 2,210 → 2,245（+1.6%）。拟合矛盾：任取 a+b·i 模型，
  两组冻结点 (60s, 2210) 与 (240s, 2245) 无解。
- div×10 同构探针（3,840 潜在）冻在 1,915 行，gethtml.html 冻在 2,210/2,245——
  冻结点随**内容位置**变、不随预算变，两文件尾部均为页级 Timeout 行。

结论：特定子测试**楔死**（页面停摆空转至预算截止），非累积成本爆炸。divx 冻结点
反推楔死候选 = 第 5 个 div 的第 ~379 号象限（declarative / mode=closed /
clonable=true / serializable=undefined / empty=false / light=''，Element 臂）。
逐测试计时探针（zz-knee-probe，div×8 = 3,072 测试，名字嵌 `at+<ms>` 注册耗时）
待二进制重建后跑：若行计时平坦至末行、下一索引永远缺失即坐实楔死，并给出精确
测试身份。引擎侧已排除：arena 不回收死节点（remove_child 只摘 children 链）但
shadow_root_ids/query_selector_including_shadow 的调用面均不在子测试热路径；
find_by_selector 为 live-tree 根遍历 + 有界 memo。

**后续定案（05:45 成对复跑，重建二进制，`-- env ZW_TIMEOUT_MULTIPLIER=4` 注入
生效——页面 ~240s 超时而非 60s，两跑尾部均为 harness 页级 Timeout 且
`pending:1`）**：

- ×1（gethtml-check）2,056P + 页级 T vs **×4（final-gethtml-x4）2,035P + 页级
  T**——预算 ×4 行数平坦（±1%），楔死**坐实**；楔死子测试 = 超时时刻 harness
  `pending` 的那一索引（最后 Pass 行的下一序位，两跑冻结点随负载微漂 ~20 行）。
- zz-accum 探针（A 附着不清理 / B 脱离销毁 / C 单 wrapper 复用，各 300 op，
  setHTMLUnsafe+attachShadow+getHTML）：300 op 内无一楔死；per-op 成本随 op 数
  上升（A 2.9→12.5ms、C 13.1→18.5ms）——累积减速面存在（registry/树增长），
  300 op 量级不足以楔死；与 oracle 冻结点 ~2,0xx 的量化衔接挂 knee probe。
- 残余挂账（Batch 3）：楔死子测试精确身份（zz-knee-probe 逐测试计时）→ 引擎
  侧修复；修复后 gethtml.html 预期全量 ~4,5xx 行真实上报，M4 复测以彼口径取门。

## 探针法证（研删，未入库）

- echo 双发形态：`<h1><template…><slot></slot></template><span>light</span><template…></template></h1>`
  ——mid（shadow 形态）+ 融合视图 echo 子并发；`_zwSerKids180` 闸门修复。
- innerHTML 泄漏：closed 根宿主 `wrapper.innerHTML` 含 `<template shadowrootmode=closed>`——
  getter 剔除 + outerHTML 引擎臂剔除后 clean（`getHTML()===innerHTML` equality 面闭合）。

## 全通道 A/B（WC 392 文件 vs M2 基线）

复跑 393 文件（392 基线 + 1 研删探针页）：9,053 行 = 7,412P/1,624F/9T + 6 Unsupported
+ 2 PreconditionFailed（次要状态面，两轮一致）。
公共行（两轮均上报）逐子测试身份比对：

- **P→F = 0**（Pass→Timeout/Timeout 变体全零）——零回归门通过。
- 唯一转移 Fail→Pass，合计 2,839，逐文件归因：

| 文件 | F→P | 前后 |
|---|---:|---|
| declarative/gethtml.html | 2,210 | 预算内全真实执行翻绿（canvas 修复确认 0F） |
| declarative/declarative-shadow-dom-attachment.html | 484 | 0P/654F → 484P/170F |
| declarative/declarative-shadow-dom-opt-in.html | 108 | 1P/117F → 109P/9F |
| reference-target/tentative/property-reflection.html | 30 | 0P/31F → 30P/1F |
| shadow-dom/shadow-root-clonable.html | 2 | 0P/6F → 2P/4F |
| declarative/declarative-shadow-dom-serialization.html | 2 | → 2P/0F |
| basic / repeats / adoptedstylesheets-serialization | 3 | 各 +1 |

- attachment.html +484 为 Batch 2 附加面：「child of \<X\> should be disallowed」
  校验臂（片段解析不物化的 opt-in 语义面）整族翻转；余 170F 为「should be
  safelisted」物化臂，挂账下批甄别。property-reflection +30、clonable +2
  同为物化/旗标面外溢。
- opt-in 余 9F 逐条：canvas 1、DOMParser historical 1、CFR 1、parseHTMLUnsafe
  document.open 1、document.write allowed 1、iframe 2、XHR document.open 2
  （Batch 1 挂账的查询路径快照闭包 4 由本轮 innerHTML echo 剔除消解）。
- BASELINE-ONLY 2,298 全部为 gethtml.html 截断残差（基线 fail-fast 瞬报 4,507
  行 − 现实执行 2,210 行），未真实执行、不计回归。
- custom-elements/template/sd-top 既有账逐案零变化（STF 全保留，无翻转）；
  reftarget 旧面仅 property-reflection 正向翻转。DC-4 的 custom-elements
  零回归门维持。
