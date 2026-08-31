# R135 — dom/nodes name-validation spec regex 族全 100%（5F→0F）

**日期**: 2026-08-20
**里程碑**: M4（WPT dom 上游基线建立与扩展）
**Driving 用例**: `dom/nodes/name-validation.html`（5 subtest，WPT 315976933870b34d6ea30e3f6643403edae678ba）
**运行入口**: `make testharness-dom FILTER=name-validation`（polyfill）/ `ZW_NATIVE_DOM=1 ...`（native）

## 背景

R134 轮 master.md「下一步计划 (a)」候选簇之一（name-validation 5F）。上轮 session
429 中断，工作区遗留 4 个 `js_dom_shim` 文件未提交改动（本轮开头核对后确认为本
轮 WIP：spec regex 校验族重写的完整实现）。本轮收口：补 detached 路径遗漏 +
修回归 + 单测 + 全量验证。

## 根因（校验器三缺陷 + 一处 detached 路径遗漏）

1. **JS `/\s/` 与 ASCII 空白五字符集不一致**——`\x0B`（垂直制表）/`\x85`/` `
   是 JS 空白但非 spec 的 ASCII whitespace（0x9/0xA/0xC/0xD/0x20），
   旧 `/[\s>]/` 把 `A\x0B` 误判 invalid（WPT 断言 valid）。
2. **NUL 漏校验**——`\0` 非 `/\s/` 成员，旧整名 `/[\s>]/` 对 `null\0` local
   不抛（WPT 断言 InvalidCharacterError）。
3. **element / attribute / NS-prefix / NS-local 四名单语义未分**——attribute 名
   无首字符限制但禁 `=`；NS prefix 段禁 `:` 允 `=`；NS local 段禁 `=` 允 `:`；
   element 名走 spec valid-name regex（首字符 ASCII 字母 → 后续任意 / `:` `_`
   `>=0x80` → 后续 NameChar 集）。
4. **detached 路径遗漏（本轮新发现）**——`createDocument` 经 detached doc 的
   `createElementNS`（part03 `_makeDetachedDocument`）建 root，该校验器未升级
   spec regex，使 `p:null\0`/`p::soh\x01` local 不抛——WPT「createElementNS and
   createDocument」subtest 持续 Fail 的最后根因（主文档 createElementNS part06
   已修但 createDocument 走 part03 独立实现）。

## 修复（五处，全部 `js_dom_shim` JS 侧）

1. **part01b**：`_r135NameRegex`（spec valid-name regex 直引）+
   `_r135IsValidName` + `_r135IsValidQualifiedNameSpec`（NS 族两段拆分）+
   `_r135AttrNameRegex`（`/^[^\0\t\n\f\r />=]+$/u`，无首字符限制）+
   `_r135IsValidAttrQNameSpec`（prefix 禁 `:` / local 禁 `=`）。
   `_zwIsValidHtmlElementName` 改走 spec regex。
2. **part03**（createElementNS detached）：显式 invalid 字符集
   （`/[\u0000\u0009\u000A\u000C\u000D\u0020/>]/`）+ `_r135IsValidName` 段校验
   （local 段走 spec regex）。
3. **part04**（setAttribute）：`_r135IsValidAttrName` 名单校验（toggleAttribute
   同函数路径）；setAttributeNS：显式字符集 + `_r135IsValidAttrQNameSpec` 段语义
   （单测抓到的遗漏——`p:a=b` 不抛，补后抛）。
4. **part06**（主文档 createElementNS / createAttribute(NS) /
   createProcessingInstruction / createDocumentType）：同款显式字符集 +
   spec regex / attr 名单校验；doctype `\x0B` valid + NUL invalid。
5. **关键实证修正（prefix 段从宽）**——spec regex 对 `>=0x80` 首字符的 prefix 段
   有 NameChar 限制，但 WPT `validNamespacePrefixes` 含全码点（`\x01` 等）×
   valid local 组合都须**不抛**（浏览器实证宽松）。element NS 的 prefix 段
   **不校验**（仅整名禁 NUL/空白/`/`/`>`），只校验 local 段。attribute NS
   的 prefix 段保留 `=` 允许 + `:` 禁止（名单实测）。

## A/B 验证

- **name-validation**：5F→**0F（5P 双路径 100%**，polyfill + native 同步）。
- **dom/nodes 全量**：polyfill 8438→**8459P（+21）**；fail 集 diff 核实零真回归
  （见下「回归核对」）；native 7658→**7679P（+21 同步）**。
- **回归核对（stash A/B 全量 fail 集 diff）**：baseline 151 行（被 450s test-guard
  截断，realm 族 57 fail 未跑完）vs 本轮 210 行——新增 60 行中 57 = realm 族
  本轮跑完（基线超时截断非通过）、2 = Attr-prefix SVG/MathML（基线预存，探针
  实证 baseline `getAttributeNodeNS(xlink, href)` 同返 null）、1 =
  query-target-in-load-event 超时抖动（隔离复跑仍 Timeout，与改动无关）。
  移除 1 行 = name-validation createElement（本轮修复）。
- **相邻校验域**：Document-createElement 754P / createElementNS 596P /
  DOMImplementation-createDocumentType 82P / MutationObserver-attributes 41P+Timeout
  （基线同）全零回归；Document-createProcessingInstruction 9P/3F（3F 基线预存，
  stash 实证：`·A`/`×A`/`A×` 多字节名，非本轮面）。
- **events/collections/traversal**：events 423P/28F/10T、collections 49P、
  traversal 1589P/15F/1T 与 R134 逐项一致（events Timeout 波动为既知 flake 面）。
- **单测**：engine `test_name_validation_spec_regex_r135`（createElement
  `\x0B` valid/NUL·空白·slash·gt invalid；NS localName NUL/`\x01` 抛 +
  createDocument 同步；attribute 无首字符限制禁 `=`；NS prefix 禁 `:` local 禁
  `=`；PI target regex；doctype NUL invalid——7 断言组，首跑即抓到
  setAttributeNS `p:a=b` 遗漏）。
- `make test` 双矩阵全绿（66 套件；第一轮 network_loading::stale_etag flake
  隔离复跑 + 全量重跑均绿）；fmt 无 diff；clippy v8 + quickjs 矩阵零警告。

## dom/ranges 现状记录（本轮全量盘点，非本轮修复面）

ranges 全量因 `Range-mutations-dataChange`/`insertBefore`（历史 >420s 慢用例，
R51c/R52 归因 M1 L2）无法单次跑完；分文件跑合计（不含 2 慢文件）：
**P=4108 F=32927 T=5**。F 大头是 mega-case（Range-compareBoundaryPoints 9313F /
Range-set 8005F / Range-isPointInRange 5694F / Range-intersectsNode 2356F /
comparePoint 5580F）——「Set up range N」setup 簇失败（foreignDoc/detachedDoc
容器），属 M1 L2 深结构域，与 name-validation 无关。后续轮次候选：
ranges setup 簇（对比 R96 后基线判升降）。

## 教训

1. **JS `/\s/` 与 spec ASCII 空白集不重合**——校验器字符集一律显式枚举
   （`\u0009\u000A\u000C\u000D\u0020`（ASCII 空白五字符）+ `\u0000`），不用 `\s` 简写。
2. **spec regex 是仲裁但不是全貌**——WPT 名单是浏览器实证，regex 对 prefix 段
   的 NameChar 限制与实证宽松冲突时实证优先（element NS prefix 从宽）。
3. **同名 API 双实现（主文档 part06 / detached part03）**——校验器升级要两处
   同步；「主路径修了 createDocument 仍 Fail」先查调用链是否走另一实现。
4. **全量 fail 集 diff 要考虑 test-guard 截断**——基线跑超时被杀的文件不是
   「通过」，新增 fail 行要先核对基线是否真的跑完了该文件。
