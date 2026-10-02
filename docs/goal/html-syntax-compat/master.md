# HTML 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../html-syntax-compat.md](../html-syntax-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-02（M1 基线落地）

## 当前状态

**M1 已落地（2026-10-02）**：html/syntax + html/dom corpus 通道建成 + 基线
**113 案 43884/61304 = 71.6%**。通道：`make testharness-html-syntax`（runner 子命令
+ HTML_SYNTAX_CORPUS_SUBDIRS + skip 规则 + test-guard 包裹，fetch 走 goals/40 编号
脚本含 depth-2 support 资产补拉）。证据：
[evidence/2026-10-02-m1-baseline.md](evidence/2026-10-02-m1-baseline.md)（分域通过率 +
通道外 48 案分账 + 失败聚类）。

基线形状：html/dom reflection 面强（74.7%，js-dom 遗产）；**html/syntax/parsing
1.5% 为 M2 主缺口**——named-character-references NCR 查表面（~2231 子测试）+
树构造边缘（the-end/zero/cdata/foreign 族）；serializer 面 escaping 0/9 +
serializing-xml-fragments 1/112 归 M3。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | html/syntax + html/dom corpus 导入 + 基线 | ✅ M1（2026-10-02，113 案 71.6%） |
| P2 | 解析树一致性（innerHTML/outerHTML/DOMParser）逐簇修齐 | ⏳ M2（NCR 表 → 树构造边缘簇） |
| P3 | 序列化边缘（XMLSerializer/HTML serializer） | ⏳ M3（escaping 0/9 + XML 面 0.9%） |
| P4 | html/dom 接口语义 + createContextualFragment 补面 | ⏳ M3（reflection 74.7% 边缘簇 + elements 12.1% + render-blocking IDL 面） |
| P5 | 文档级编码嗅探（encoding-compat M4 转入） | 📊 html/syntax 域案面已随 M1 入账：charset/ 7 案（reftest 形态）+ xmldecl/ 3 案（iframe 形态）+ the-input-byte-stream 9 案（HTTP 头形态）= 19 案 testharness 通道外记账；encoding/ 域 bom-handling/eof-*/sniffing 案面（已在 wpt-data，encoding 通道按 document.characterSet 规则 skip）尚未基线，落地通道（reftest import / 通道扩展）随 M2+ 定 |
| — | render-blocking 机制面（62 案 19.3%） | 📊 M1 已基线；`blocking=render` 与渲染管线耦合，是否本 goal 修齐随 M3 碰头定 |

## 已完成切片

- **M1（2026-10-02）**：goals/40 fetch 脚本（DIRS + depth-2 support 补拉）+ runner
  通道（testharness-html-syntax 子命令 + html_syntax_case_skipped）+ Makefile
  fetch-wpt-html-syntax / testharness-html-syntax（test-guard）+ 基线 113 案 71.6%
  + wpt-suites.csv 数据行回填。

## 下一步计划

1. **M2**：解析树一致性逐簇修齐——先 NCR 查表面（最大单簇 ~2231 子测试），再树构造
   边缘（the-end/zero/cdata-in-integration-point/foreign getElementsByTagName 族），
   按 evidence 失败聚类逐簇销账。

**待用户决策清单**：（空）
