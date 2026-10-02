# HTML 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../html-syntax-compat.md](../html-syntax-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-02（M2 首簇落地）

## 当前状态

**M1 + M2 首簇已落地（2026-10-02）**：corpus 通道 + 基线 113 案 43884/61304 = 71.6%
→ **M2 首簇 character reference 直通修齐后 46122/61304 = 75.23%**（恒定分母逐案对账
零回归）。首簇根因双向：宿主三处 setter「无 `<` 直通」快速路径（innerHTML/
insertAdjacentHTML/outerHTML）+ shim innerHTML 纯文本本地视图注册原始串；修法 =
删快速路径走 html5ever charref 语义（零自维护实体表）+ shim 注册宿主解析展开文本。
证据：[evidence/2026-10-02-m2-ncr-charref.md](evidence/2026-10-02-m2-ncr-charref.md)。

**M2 剩余主缺口**（evidence M1 失败聚类序）：树构造边缘簇——the-end 0/4、
cdata-in-integration-point-fragment 1/10、foreign getElementsByTagName 4 案、
quotes-in-meta/meta-inhead-insertion-mode/html5lib_url|write 族；serializer 面
（escaping 0/9 + XML 面 1/112）归 M3。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | html/syntax + html/dom corpus 导入 + 基线 | ✅ M1（2026-10-02，113 案 71.6%） |
| P2 | 解析树一致性（innerHTML/outerHTML/DOMParser）逐簇修齐 | 🔄 首簇 NCR 表 ✅（0/2231→2231/2231，2026-10-02）；⏳ 树构造边缘簇（the-end/cdata/foreign/quotes 族） |
| P3 | 序列化边缘（XMLSerializer/HTML serializer） | ⏳ M3（escaping 0/9 + XML 面 0.9%） |
| P4 | html/dom 接口语义 + createContextualFragment 补面 | ⏳ M3（reflection 74.7% 边缘簇 + elements 12.1% + render-blocking IDL 面） |
| P5 | 文档级编码嗅探（encoding-compat M4 转入） | 📊 html/syntax 域案面已随 M1 入账：charset/ 7 案（reftest 形态）+ xmldecl/ 3 案（iframe 形态）+ the-input-byte-stream 9 案（HTTP 头形态）= 19 案 testharness 通道外记账；encoding/ 域 bom-handling/eof-*/sniffing 案面（已在 wpt-data，encoding 通道按 document.characterSet 规则 skip）尚未基线，落地通道（reftest import / 通道扩展）随 M2+ 定 |
| — | render-blocking 机制面（62 案 19.3%） | 📊 M1 已基线；`blocking=render` 与渲染管线耦合，是否本 goal 修齐随 M3 碰头定 |

## 已完成切片

- **M2 首簇（2026-10-02）**：character reference 直通双向修齐——engine 三处 setter
  快速路径删除 + shim 纯文本本地视图同语义；named-character-references 0/2231→全绿
  + zero.html +7，零回归；全通道 75.23% 回填 wpt-suites.csv。
- **M1（2026-10-02）**：goals/40 fetch 脚本（DIRS + depth-2 support 补拉）+ runner
  通道（testharness-html-syntax 子命令 + html_syntax_case_skipped）+ Makefile
  fetch-wpt-html-syntax / testharness-html-syntax（test-guard）+ 基线 113 案 71.6%
  + wpt-suites.csv 数据行回填。

## 下一步计划

1. **M2 续**：树构造边缘簇——先 the-end/cdata-in-integration-point-fragment/foreign
   getElementsByTagName（解析器插入模式边缘），再 quotes-in-meta/meta-inhead-
   insertion-mode/html5lib_url|write 族；按 M1 evidence 失败聚类逐簇销账。

**待用户决策清单**：（空）
