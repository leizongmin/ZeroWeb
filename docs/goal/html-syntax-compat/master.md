# HTML 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../html-syntax-compat.md](../html-syntax-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-02（M2 片 c 落地）

## 当前状态

**M1 + M2 三片已落地（2026-10-02）**：基线 71.6% → 首簇 charref 直通修齐 75.23%
（[m2-ncr-charref](evidence/2026-10-02-m2-ncr-charref.md)）→ 片 a foreign ns 保真
75.31%（[m2b-foreign-ns-fidelity](evidence/2026-10-02-m2b-foreign-ns-fidelity.md)）
→ 片 b CDATA + NUL 读回 75.33% → **片 c P5 document.characterSet 嗅探
46185/61304 = 75.34%**（恒定分母逐案零回归；quotes/meta 全绿）。片 c 另含案预算
工具（ZW_CORPUS_CASE_TIMEOUT_SECS）与 make test 栈加固（32MiB——管线深嵌套测试
栈边际随 zero-dom 重编译抽签，渲染流域碰头项已记账飞书）。片 b 根因
双层：html5ever 0.29.1 fragment 模式 CDATA 门不 consult context_elem（宿主预变换
+ 本地视图 ns 通道双层修齐，FIXME 记 0.39 升级可移除）+ `_zwMEl` 无 title 反射
（补 HTMLElement 全局 title/lang/dir 三元组；探针实证 html5ever 的 NUL charref
语义本就正确）。证据：
[evidence/2026-10-02-m2c-cdata-nul.md](evidence/2026-10-02-m2c-cdata-nul.md)、
[evidence/2026-10-02-m2d-p5-cherset.md](evidence/2026-10-02-m2d-p5-cherset.md)。

**M2 剩余缺口**：the-end/DCL-defer（6，生命周期事件时序——runner 尾部派发缺
bubbles/target 序语义 + defer 任务序，engine 域设计片）；html5lib 三案案预算复评
（首轮 Timeout 系递归 bug 冤案，修后未复评）。serializer 面归 M3。html5ever
0.29→0.39 升级（fragment CDATA 门正解、可移除预扫描）记候选架构片。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | html/syntax + html/dom corpus 导入 + 基线 | ✅ M1（2026-10-02，113 案 71.6%） |
| P2 | 解析树一致性（innerHTML/outerHTML/DOMParser）逐簇修齐 | 🔄 首簇 NCR 表 ✅（2231/2231）+ 片 a foreign ns ✅（foreign 4 案全绿）+ 片 b CDATA ✅（10/10）+ zero 读回 ✅（14/14）+ 片 c P5 characterSet ✅（quotes/meta 全绿，2026-10-02）；⏳ the-end 生命周期片 |
| P3 | 序列化边缘（XMLSerializer/HTML serializer） | ⏳ M3（escaping 0/9 + XML 面 0.9%） |
| P4 | html/dom 接口语义 + createContextualFragment 补面 | ⏳ M3（reflection 74.7% 边缘簇 + elements 12.1% + render-blocking IDL 面） |
| P5 | 文档级编码嗅探（encoding-compat M4 转入） | 🔄 testharness 通道可执行面首片 ✅（sniff→Document label→characterSet 管线，2026-10-02）；📊 通道外案面仍记账：charset/ 7 案（reftest 形态）+ xmldecl/ 3 案（iframe 形态）+ the-input-byte-stream 9 案（HTTP 头形态）= 19 案 testharness 通道外记账；encoding/ 域 bom-handling/eof-*/sniffing 案面（已在 wpt-data，encoding 通道按 document.characterSet 规则 skip）尚未基线，落地通道（reftest import / 通道扩展）随 M2+ 定 |
| — | render-blocking 机制面（62 案 19.3%） | 📊 M1 已基线；`blocking=render` 与渲染管线耦合，是否本 goal 修齐随 M3 碰头定 |

## 已完成切片

- **M2 片 c（2026-10-02）**：P5 首片——`<meta charset>` 预扫描（spec 引号/失败/
  续扫语义 + 序列化快照实体解码，zero-dom 增 encoding_rs）→ Document label →
  原生 getter + `__zw_get_character_set` 回调 → shim 主文档面；quotes/meta 全绿，
  全通道 75.34%；ZW_CORPUS_CASE_TIMEOUT_SECS 工具 + make test 栈加固（碰头记账）。
- **M2 片 b（2026-10-02）**：foreign context CDATA 双层修齐（zero-dom
  `translate_cdata_sections` + `child_nodes_json_ctx` ns 通道）+ `_zwMEl`
  title/lang/dir 反射；CDATA 10/10、zero 14/14，全通道 75.33%。
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

1. **M2 续**：① the-end/DCL-defer 生命周期时序设计片（DCL bubbles/load
   target=document/pageshow 先序 + defer 任务序——runner 尾部派发与 shim 生命周期
   面改造）；② html5lib 三案 ZW_CORPUS_CASE_TIMEOUT_SECS 复评。按 evidence 失败
   聚类逐簇销账。

**待用户决策清单**：（空）
