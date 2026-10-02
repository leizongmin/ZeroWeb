# M2 树构造簇（b）— foreign context CDATA + NUL charref 读回

**日期**: 2026-10-02 | **对应缺口**: P2 解析树一致性（树构造边缘簇第二片）
**前置**: [2026-10-02-m2b-foreign-ns-fidelity.md](2026-10-02-m2b-foreign-ns-fidelity.md)

## 簇 1：cdata-in-integration-point-fragment（1/10 → 10/10）

**根因（双层）**：spec `markup-declaration-open-state`——`<![CDATA[` 在「adjusted
current node 非 HTML ns」时为 character data。html5ever 0.29.1 的该门只看 open_elems
（fragment 模式仅合成 html 根），不 consult context_elem（`tree_builder/mod.rs:542`
与树规则 1407 行的 fragment 特判不一致——0.39 已修 adjusted_current_node）。

- **宿主层**：`parse_html_fragment` context 非 HTML ns 时把 CDATA 段预变换为转义文本
  （`translate_cdata_sections`——首个 `]]>` 终止 + EOF 内容照收，与 spec
  cdata-section-state 一致）；FIXME 注记升级 0.39 后移除。已知边界：integration
  point 内嵌 HTML 元素后的 CDATA 不做树状态跟踪（语料面为 top-level context）。
- **shim 本地视图层**：`_zwFragmentAdded`/`_zwMBuildBodyTree` 恒按 body context 解析
  （`<body>` 包裹 + 全文档 parse）——createElementNS 容器（handle + `_nsHandles` ns
  记录）的本地视图绕过 foreign context。修法：`__zw_parse_html_child_nodes` 增可选
  arg[2] = context ns（`child_nodes_json_ctx` 非 HTML ns 走 parse_html_fragment 顶层
  子提取），shim 从 `_nsHandles[hostHandle].namespace` 传入。

## 簇 2：zero.html NUL-in-charref（8/14 → 14/14）

**诊断反转**：zero-dom 探针实证 html5ever 0.29.1 对 NUL-in-charref 的 tokenizer 语义
**完全正确**（text 上下文 NUL 丢弃 `&\0auml;`→`&auml;`、attribute 上下文 NUL→FFFD
`&notin\0;`→`&notin\uFFFD;` 全对）——6 个失败全在**读回层**：`<span title=...>`
赋值同 turn 读 `firstChild.title` 命中 JS 本地视图解析节点（`_zwMEl` 工厂），其属性
面无 `title` 反射 IDL → undefined。修法：`_zwMEl` 补 HTMLElement 全局反射字符串
三元组 title/lang/dir（spec 单元，getter 读 getAttribute）。

## 结果（全通道复跑逐案对账，分母恒 113 案）

| 案 | 修复前 | 修复后 |
|---|---|---|
| cdata-in-integration-point-fragment | 1/10 | **10/10** |
| zero.html | 8/14 | **14/14** |
| **全通道** | 46168/61304 = 75.31% | **46183/61304 = 75.33%** |

回归案数：**0**。

## 门禁

- `make test` 68 suites 全绿（串行）；fmt 干净（zero-dom/engine）；clippy 零警告
- zero-dom 新单测 `translate_cdata_sections_escapes_and_terminates` +
  `foreign_context_cdata_is_character_data`（HTML context bogus-comment 回归护栏）
- 定向 bench-gate（zero-dom + zero-engine 35 指标）**GATE PASS**

## 记账

- html5ever 0.29 → 0.39 升级（其 adjusted_current_node 已含 fragment 特判，可移除
  CDATA 预扫描）为候选架构片——爆炸半径覆盖全解析栈（TreeSink API 跨版迁移），
  另立 slice 评估。
- the-end/DCL-defer（5）生命周期时序 + html5lib 三案 30s 案预算 Timeout 维持记账。
