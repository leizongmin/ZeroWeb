# M0 差集甄别 + declarative 重审 + 数值门（DC-1，2026-11-06）

## 对账方法

- 上游 pin `315976933870b34d6ea30e3f6643403edae678ba`（fetch 脚本既有 pin，
  不动）shadow-dom 子树（tree `57adfcfc`）git trees API `recursive=1` 全量
  blob 清单：**392 blobs**（366 .html + 9 .js + 4 .css + 2 .xhtml + 9 .yml
  + 1 .md + 1 readme），其中可执行面 377（.html/.js/.xhtml）。
- 内容级甄别：逐文件读头部——`testharness.js` 引用 → TH 用例；`rel=match`
  → reftest 对页；无断言崩溃页单独归类。
- 引擎能力探针：grep 绑定注册面 + `declarative-shadow-dom-basic.html` 案头
  审读（setHTMLUnsafe/template IDL/clone 物化需求面）。
- 子测试分母引用 coverage-2026-11-02 对账 JSON
  （docs/compat/wptfyi/coverage-2026-11-02.json，wpt.fyi aligned run
  @ 2e4dc257cf，与本 pin 文件数微差不影响分档）。

## 目录分档表（392 blobs → 通道准入判定）

| 域 | blobs | TH 用例 | wpt.fyi 子测试分母 | 判定 |
|---|---:|---:|---:|---|
| 顶层 | 116 | 64（62 已入账 + 2 新候选） | 781（已 621P） | 2 新案入通道；其余差集全为渲染级/ref/崩溃页 |
| declarative/ 顶层 | 20 | 20 TH | 7,793（declarative 全域） | **转正**（parser 层已落） |
| declarative/tentative/shadowrootadoptedstylesheets/ | 39 | 36 TH + 1 崩溃页 | （同上） | **转正**，M2 分档 |
| reference-target/tentative/ | 18 | 16 TH + 1 helper | 6,266 | **转正**（JS 语义可修档） |
| untriaged/ | 67 | 54 TH | 251 | **转正**（reftest 对页 5+5 不入，resources 2 页 fixture 拉取不扫描） |
| focus/ | 42 | 0 TH | 109 | 排除：焦点导航交互面（45 号/环境受限） |
| focus-navigation/ | 49 | 0 TH | 134 | 排除：焦点导航交互面 |
| crashtests/ | 26 | 0 | 0 | 排除：无断言崩溃面（无子测试分母） |
| leaktests/ | 5 | 0 TH | 16 | 排除：leak harness，GC 可观测性环境受限 |
| reference/ | 1 | 0 | 0 | 排除：空支撑页 |
| resources/ | 4 | — | — | helper 资产（fetch 不扫描） |

**顶层差集 52 案逐档**：62 已入账 vs 上游 114 顶层（.html 112 + .xhtml 2）
——52 个缺失文件 = reftest 对页 16（`-ref.html` 等）+ 渲染级 composed tree
页面 ~34（slot-fallback-content-00{1..8}、layout-*、directionality-*、
restyle-*、shadow-style-*、invalidate-sibling-*、manual-slot-assignment-*、
imperative-slot-layout-invalidation-001、nested-hover-*、focus-within-shadow、
scroll-*、offset*、touch/wheel/drag 命中与 related-target 面、命中测试四面
caretPositionFromPoint/elementFromPoint/highlightsFromPoint/offsetX-offsetY、
accesskey.tentative）→ 全部归 45 号 shadow-dom-rendering；+ 无断言崩溃页 4
（user-agent-shadow-root-crash、restyle-on-disconnected-child-after-attach-crash、
directionality-003-crash、host-with-namespace.xhtml/innerHTML-setter.xhtml
XML 形态）→ 排除；**JS 语义新候选恰 2 案**：`input-element-list.html`、
`input-type-radio.html`（form 语义跨 shadow 边界：datalist 关联 + radio
group）→ 入通道。

## declarative 重审结论（DC-1 关键项）

- **parser/template 层已落**：`crates/dom/src/document/shadow.rs` s168
  `materialize_declarative_shadow_roots`（`<template
  shadowrootmode=open|closed>` 解析期物化）+ `crates/dom/src/parser/
  declarative_shadow.rs` s173（html5ever 0.29.1 `shadowrootmode` 值精确
  匹配缺陷绕道）。fetch 脚本旧「declarative/（declarative shadow DOM，
  范围外）」排除条款**过时**，收窄为「渲染级消费归 45 号」（45 号划界
  条款明文：declarative parser/template 层归 42 号重审范围）。
- **已知缺口**（M2 基线如实呈现，M3 逐簇归因）：
  1. `HTMLTemplateElement.shadowRootMode/DelegatesFocus/Clonable` IDL
     反射——engine 绑定 grep 零命中；
  2. `setHTMLUnsafe`/`getHTML`（fragment 解析路径 + getHTML 序列化）——零命中；
  3. template content clone/importNode 的声明式 shadow 物化语义
     （shadowrootclonable）；
  4. adoptedStyleSheets + module script 样式加载（shadowrootadoptedstylesheets
     簇 36 案主体）：`document/ShadowRoot.adoptedStyleSheets` shim 已有
     （js_dom_bridge/callbacks.rs 汇总规则文本下发），module script 进
     constructable stylesheet 的载入链路待验。
- 与 45 号交界：declarative shadow root 进渲染管线（样式/布局消费）不在
  本 goal；本批 TH 用例断言 JS 可观测面（shadowRoot 存在性、slot 分配、
  序列化、getHTML 文本），渲染不出不影响 JS 断言路径。

## reference-target 依赖面（M2/M3 输入）

`referenceTarget` 属性未实现（grep 零命中）。用例依赖：
`setHTMLUnsafe` + `shadowrootreferencetarget` 声明式属性 + testdriver
（runner 装配层对 testdriver script 标签置空注入，不阻塞执行）+
`wai-aria/scripts/aria-utils.js`（aria-labelledby 一案，已补拉）。
property-reflection 三案为排列生成式（labelable × 属性 × 断言行为表），
是 6,266 子测试分母主体。

## 地址池判定与 DC-4 数值门（决策规则落定）

地址池 = 导入即得 + JS 语义可修：

| 域 | 子测试分母 | 归池理由 |
|---|---:|---|
| declarative/ | 7,793 | parser 层已落；缺口为 JS 语义（IDL 反射/setHTMLUnsafe/module stylesheet） |
| reference-target/ | 6,266 | JS 可观测语义（属性反射/事件路径/序列化），非渲染耦合 |
| untriaged/ | 251 | v0 遗留 TH 面（部分断言 v0 语义，M2 基线甄别环境受限部分） |
| 顶层残余 + 新候选 | ~190 | 160（既有扫描域未覆盖）+ 2 新案 |

**合计 ≈ 14,500 子测试 ≥ 8,000 阈值 → 全量推进**（立项决策规则的 ≥8k 分档）。

**DC-4 数值门（初版）**：

- shadow-dom topdir 通过率：≥ (621 + 14,500 × 76.0%) / 15,350 ≈ **75.0%**
  （76.0% = web-components 收口时 JS 语义簇整体通过率 3,619/4,764，锚定
  archive/web-components/master.md；M2 基线分档后按簇实测细化）。
- custom-elements 既有账 64.67%（2,817/4,356）零回归；全量头条不回落。
- 修订规则：M2 基线如把部分池判为环境受限/渲染耦合（如 untriaged v0 语义
  断言、module stylesheet 域整簇不可跑），按实剔后重算门值并在此回填——
  池只减不增，门值只随池如实下调，不以调门替代收敛。
- **回填（同日 M2 基线）**：实测新面池 5,531 子测试（wpt.fyi 口径估计
  ≈14,500 的差值源于缺 API 早抛截断，untriaged 对照域 251=251 两侧一致
  为证）；门值修订为 **31.4%** = (621 + 5,531×76.0%)/15,350。全量推进
  决策维持（依据同批文件全实现口径可达面 ≥8k）。详见
  [M2 基线与失败簇归因](2026-11-06-m2-baseline-and-cluster-triage.md)。

## 通道接线核对（DC-2 前置）

- 四件套：fetch 脚本清单扩批（SD_DECL_* / SD_REFTARGET_* / SD_TOP_DELTA_*
  / SD_UNTRIAGED_*，两侧同源）+ `wc_case_skipped` 收窄（ declarative/
  reference-target/ 转正，新增 fixture/support 与崩溃页排除）+
  `WC_TEST_SUBDIRS` 递归扫描零改动 + Makefile `fetch-wpt-web-components`/
  `testharness-web-components` 既有 target 零改动。
- weekly：`html-semantics web-components` 手调分片 time_limit 10800s
  （ci-shards.yml:26-29），扩面后预算余量足；套件发现为 runner 源码
  match 臂自动提取，零 CI 编辑。
