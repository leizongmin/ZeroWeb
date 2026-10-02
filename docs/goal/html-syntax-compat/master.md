# HTML 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../html-syntax-compat.md](../html-syntax-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-02（M2 片 a 落地）

## 当前状态

**M1 + M2 两片已落地（2026-10-02）**：基线 71.6% → 首簇 charref 直通修齐 75.23%
（[m2-ncr-charref](evidence/2026-10-02-m2-ncr-charref.md)）→ **树构造片 a foreign
ns 保真 46168/61304 = 75.31%**（恒定分母逐案零回归）。片 a 根因 = shim sel 代理四
身份 getter（namespaceURI 硬编码 XHTML / tagName 无条件 ASCII 大写）——R120 DOM
语义匹配核心被误折叠为 HTML；修法 = `_zwSelNs`（host NS_MEMO + gen 印章 memo）+
四 getter foreign 臂。
证据：[evidence/2026-10-02-m2b-foreign-ns-fidelity.md](evidence/2026-10-02-m2b-foreign-ns-fidelity.md)。

**M2 剩余缺口**（按 evidence 失败聚类）：cdata-in-integration-point-fragment（9，
fragment 模式 CDATA）+ zero NUL-in-charref（6，tokenizer 边缘）+ quotes-in-meta/
meta-inhead（2，文档编码面）；the-end/DCL-defer（5，生命周期事件时序——脚本执行
管线面）；html5lib 三案 30s 案预算 Timeout（需切片跑法）。serializer 面归 M3。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | html/syntax + html/dom corpus 导入 + 基线 | ✅ M1（2026-10-02，113 案 71.6%） |
| P2 | 解析树一致性（innerHTML/outerHTML/DOMParser）逐簇修齐 | 🔄 首簇 NCR 表 ✅（2231/2231）+ 树构造片 a foreign ns ✅（foreign 4 案全绿，2026-10-02）；⏳ cdata/zero-NUL/quotes 片 + the-end 生命周期片 |
| P3 | 序列化边缘（XMLSerializer/HTML serializer） | ⏳ M3（escaping 0/9 + XML 面 0.9%） |
| P4 | html/dom 接口语义 + createContextualFragment 补面 | ⏳ M3（reflection 74.7% 边缘簇 + elements 12.1% + render-blocking IDL 面） |
| P5 | 文档级编码嗅探（encoding-compat M4 转入） | 📊 html/syntax 域案面已随 M1 入账：charset/ 7 案（reftest 形态）+ xmldecl/ 3 案（iframe 形态）+ the-input-byte-stream 9 案（HTTP 头形态）= 19 案 testharness 通道外记账；encoding/ 域 bom-handling/eof-*/sniffing 案面（已在 wpt-data，encoding 通道按 document.characterSet 规则 skip）尚未基线，落地通道（reftest import / 通道扩展）随 M2+ 定 |
| — | render-blocking 机制面（62 案 19.3%） | 📊 M1 已基线；`blocking=render` 与渲染管线耦合，是否本 goal 修齐随 M3 碰头定 |

## 已完成切片

- **M2 片 a（2026-10-02）**：sel 代理四身份 getter foreign ns 保真（part03 `_zwSelNs`
  host NS_MEMO + gen 印章 memo + part04 四 getter foreign 臂）——foreign gET 4 案
  全绿 + math-parse +4，全通道 75.31%；goals/40 补拉 parsing/resources。
- **M2 首簇（2026-10-02）**：character reference 直通双向修齐——engine 三处 setter
  快速路径删除 + shim 纯文本本地视图同语义；named-character-references 0/2231→全绿
  + zero.html +7，零回归；全通道 75.23% 回填 wpt-suites.csv。
- **M1（2026-10-02）**：goals/40 fetch 脚本（DIRS + depth-2 support 补拉）+ runner
  通道（testharness-html-syntax 子命令 + html_syntax_case_skipped）+ Makefile
  fetch-wpt-html-syntax / testharness-html-syntax（test-guard）+ 基线 113 案 71.6%
  + wpt-suites.csv 数据行回填。

## 下一步计划

1. **M2 续**：① cdata-in-integration-point-fragment（fragment 模式 CDATA——查
   html5ever parse_fragment 的 integration-point CDATA 面）+ zero NUL-in-charref
   tokenizer 边缘；② the-end/DCL-defer 生命周期时序（脚本执行管线，跨域碰头评估）；
   ③ html5lib 三案切片跑法评估。按 evidence 失败聚类逐簇销账。

**待用户决策清单**：（空）
