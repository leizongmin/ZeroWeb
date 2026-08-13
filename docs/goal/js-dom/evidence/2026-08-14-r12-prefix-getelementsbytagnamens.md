# R12 — element.prefix getter + getElementsByTagNameNS（M4 / DC-3）

**日期**: 2026-08-14
**轮次**: R12
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**commit**: 见 `git log`（feat(js-dom): element.prefix getter + getElementsByTagNameNS）

## 背景

R11 后评估剩余聚类。iframe.contentDocument（createElementNS/case/cloneNode 大头 ~390 subtest）核实为**深结构跨面改**（需为每个 iframe 维护独立 Document + window + createElementNS 子文档工作 + 跨文档节点归属，html-compat 域），记待评估，转零碰撞面。

选 case.html 第二大失败块的聚焦根因：
- `element.prefix` getter 缺失（返 undefined/null，~90 subtest）
- `getElementsByTagNameNS is not a function`（document + element 两级，~20 subtest）

## 改动

### 1. element.prefix getter（part04 get trap）

`element.prefix`（spec `dom-node-prefix`）：限定名冒号前部分；无冒号 → null；非 Element → null。
**注**：polyfill `_realTag` 强制大写（HTML 语义），故 prefix 返大写——case.js 测 abc/Abc/ABC 三态时仅 ABC 态匹配（abc/Abc 仍 fail，待 createElementNS 保留原 tag 大小写的深改，记 master.md）。

### 2. getElementsByTagNameNS（part04 元素级 + part06 document 级）

spec `dom-element-getelementsbytagnamens` / `dom-document-getelementsbytagnamens`。polyfill 无 ns 概念（HTML 单 ns），忽略 ns 按 localName 查（同 getElementsByTagName 模式，委托 querySelectorAll/__zw_query_all_sub）。元素级支持 `*` 通配（客户端递归下降）。返 HTMLCollection。

### 3. 单测

`test_prefix_and_get_elements_by_tag_name_ns_r12`：prefix（限定名/无冒号 null）、getElementsByTagNameNS document 级（命中数）、element 级子树作用域、`*` 通配。

## iframe.contentDocument 评估结论（深结构，记待评估）

createElementNS 用例 XML/XHTML iframe 路径需求：① `doc.documentElement.textContent` == "Dummy XML document"（需真实解析 iframe src）② `doc.createElementNS` 子文档工作（独立 Document）③ `doc.defaultView.DOMException`（独立 window）④ `element.ownerDocument === doc`（跨文档节点归属）。完整 iframe 子文档 = html-compat 域深结构（跨文档 + iframe 解析），远超轻量切片。记入未解决问题，转零碰撞面。

## 基线结果（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R11 | R12 | Δ |
|------|----|----|---|
| polyfill | 40.89% | **41.71%** | +0.82pp |
| native | 40.63% | **41.45%** | +0.82pp |

双路径对等差 0.26pp。polyfill 净 +37 pass。完整 JSON 快照入 evidence。

## 验证

engine v8 2083 / quickjs 1408 单测；fmt + clippy（v8 + quickjs）零警告。

## 下一步

- iframe.contentDocument（深结构，html-compat 域，需独立切片评估或转 html-compat 流）。
- Element-classlist（280）/ createEvent（183）/ DOMImplementation-createDocumentType（81）等其余聚焦 API 精度。
- createElementNS 保留原 tag 大小写（解 case.js prefix abc/Abc 态，深改 host tag 存储）。
