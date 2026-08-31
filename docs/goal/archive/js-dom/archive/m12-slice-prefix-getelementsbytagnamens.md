# M4 R12 切片 — element.prefix getter + getElementsByTagNameNS

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**前置**: R11（element.localName + HTML 元素子类 instanceof）
**commit**: 见 `git log`（feat(js-dom): element.prefix getter + getElementsByTagNameNS）

## 背景

R11 后评估剩余聚类。iframe.contentDocument（createElementNS/case/cloneNode ~390 subtest）核实为深结构跨面改（独立 Document + window + 跨文档节点归属，html-compat 域），记待评估。转 case.html 第二大失败块的聚焦根因：element.prefix getter 缺失 + getElementsByTagNameNS 缺失。

## 改动（3 文件）

### 1. element.prefix getter（part04）

spec `dom-node-prefix`：限定名冒号前；无冒号 → null；非 Element → null。注：_realTag 大写化致 prefix 大写，case.js abc/Abc 态仍 fail（待 createElementNS 保留原 tag 深改）。

### 2. getElementsByTagNameNS（part04 元素级 + part06 document 级）

忽略 ns（polyfill HTML 单 ns），按 localName 查（同 getElementsByTagName，委托 querySelectorAll/__zw_query_all_sub）。元素级支持 `*` 通配。返 HTMLCollection。

### 3. 单测（part07）

`test_prefix_and_get_elements_by_tag_name_ns_r12`。

## iframe.contentDocument 评估结论（深结构，记待评估）

createElementNS 用例 XML/XHTML iframe 路径需：iframe src 真实解析 + 独立 Document + 独立 window（defaultView.DOMException）+ 跨文档节点归属（ownerDocument === doc）。完整 iframe 子文档 = html-compat 域深结构（跨文档 + iframe 解析），远超轻量切片。记未解决问题。

## 基线（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R11 | R12 | Δ |
|------|----|----|---|
| polyfill | 40.89% | 41.71% | +0.82pp |
| native | 40.63% | 41.45% | +0.82pp |

双路径对等差 0.26pp。polyfill 净 +37 pass。

## 验证

engine v8 2083 / quickjs 1408 单测；fmt + clippy（v8 + quickjs）零警告。

## 下一步

- iframe.contentDocument（深结构，html-compat 域）。
- Element-classlist（280）/ createEvent（183）/ createDocumentType（81）聚焦 API 精度。
- createElementNS 保留原 tag 大小写（case.js prefix abc/Abc 态）。
