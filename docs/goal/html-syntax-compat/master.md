# HTML 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../html-syntax-compat.md](../html-syntax-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-03（M3 片 a 收口——escaping 9/9）

## 当前状态

**M3 片 a 已收口（2026-10-03）**：escaping.html **3/9 → 9/9**（serializing 通道
首个全绿用例），全通道 46205/61309 = **75.36%**，对 M1 基线 per-case 逐案比对
**0 回归**。六失败面修齐：template content 解析期 disabled 标记 +
`Range.createContextualFragment`（DC-3 明确项，host `__zw_parse_fragment_children`
深 JSON 通道）+ detached doc.write + IAH afterbegin plain 节点 insertBefore 分支
+ 主文档 write 隐式 open 流（基线快照 + 累计重放）+ XHR data: URL responseXML
（HTML MIME 复用 `_zwParsedDoc`）。DC-4 门禁实测：make test 68 suites 全绿 +
clippy `-D warnings` 干净（顺修上轮遗留 unused import）+ fmt + reftest **704/704**
零不一致。残差记账：ambiguous-ampersand 写入子树读链 hit 查询视图/live doc 分裂
seam（R57 view doc vs `__zw_child_nodes` live-aware——解析插入子树同 turn 全 API
面读一致性归架构片）；XHR XML MIME 解析挂账；静态 `<template><noscript>` 文档
解析面待 html5ever 0.39 评估。证据：
[evidence/2026-10-03-m3a-closing.md](evidence/2026-10-03-m3a-closing.md)。

**M1 + M2 已收口（2026-10-02）**：基线 71.6% → M2 五片（charref 直通 75.23% →
foreign ns 保真 75.31% → CDATA/NUL 75.33% → characterSet 嗅探 75.34% → 生命周期
时序 75.35%）；DC-2 判定成立（可执行面全绿 + 残差全分类：P5 通道外 19 案 +
html5lib 3 案 document.write 管线面）。证据：
[evidence/2026-10-02-m2-verdict.md](evidence/2026-10-02-m2-verdict.md) 及同日
各片 evidence。挂账：bench-gate 复评（兄弟流活跃污染等让）；html5ever 0.29→0.39
升级候选架构片。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | html/syntax + html/dom corpus 导入 + 基线 | ✅ M1（2026-10-02，113 案 71.6%） |
| P2 | 解析树一致性（innerHTML/outerHTML/DOMParser）逐簇修齐 | ✅ **M2 收口（2026-10-02）**：五片落地（charref/foreign ns/CDATA/characterSet/生命周期），通道可执行面全绿 75.35%，DC-4 门禁实测齐（reftest 700/700）；残差分类记账（P5 通道外 19 案 + html5lib 管线面 3 案） |
| P3 | 序列化边缘（XMLSerializer/HTML serializer） | 🔄 **M3 片 a 收口（2026-10-03）**：escaping.html 9/9（noscript scripting 旗标双面 + 6 缺失 API/视图/XHR 面全绿）；残差：XMLSerializer XML 面 0.9%、ambiguous-ampersand 读链 seam |
| P4 | html/dom 接口语义（createContextualFragment 已落 ✅） | ⏳ M3 续（reflection 74.7% 边缘簇 + elements 12.1% + render-blocking IDL 面） |
| P5 | 文档级编码嗅探（encoding-compat M4 转入） | 🔄 testharness 通道可执行面首片 ✅（sniff→Document label→characterSet 管线，2026-10-02）；📊 通道外案面仍记账：charset/ 7 案（reftest 形态）+ xmldecl/ 3 案（iframe 形态）+ the-input-byte-stream 9 案（HTTP 头形态）= 19 案 testharness 通道外记账；encoding/ 域 bom-handling/eof-*/sniffing 案面（已在 wpt-data，encoding 通道按 document.characterSet 规则 skip）尚未基线，落地通道（reftest import / 通道扩展）随 M2+ 定 |
| — | render-blocking 机制面（62 案 19.3%） | 📊 M1 已基线；`blocking=render` 与渲染管线耦合，是否本 goal 修齐随 M3 碰头定 |

## 已完成切片

- **M3 片 a 收口（2026-10-03）**：escaping 3/9→9/9——六面修齐（template 解析期
  disabled 标记 / createContextualFragment / detached doc.write / IAH afterbegin
  plain 分支 / 主文档 write 隐式 open 流 / XHR data: URL responseXML）；75.36%，
  0 回归；DC-4 实测（test 68 suites + clippy + fmt + reftest 704/704）；残差
  记账（读链 seam / XML MIME / 静态 template noscript）。
- **M3 片 a 续二（2026-10-03）**：detached 旗标通道贯通——child_nodes_json_full
  （arg[3]）+ `_zwParseEl._ensureMutTree` 桥 inert 印章 + `_zwMBuildNode` 递归旗标
  + `_zwMEscapeText` 补 nbsp；escaping 1/9→3/9；rebase 撞兄弟流 slice19 merge
  （import 冲突取并集）后组合树复验绿。
- **M2 收口（2026-10-02）**：DC-2 判定成立（可执行面全绿 75.35% + 残差全分类）；
  DC-4 实测（reftest 700/700 零不一致补齐最后一块）；html5lib 300s 复评定性
  document.write 管线面；bench 复评挂账（兄弟流活跃污染）。
- **M2 片 d（2026-10-02）**：生命周期时序——runner 尾单 timer 任务严格序
  DCL→load→pageshow + Event/子类 toStringTag；the-end 4/4、DCL-defer 1/1，
  全通道 75.35%。
- **M2 片 c（2026-10-02）**：P5 首片——`<meta charset>` 预扫描（zero-dom 增
  encoding_rs）→ Document label → characterSet 管线；quotes/meta 全绿，75.34%。
- **M2 片 b（2026-10-02）**：foreign context CDATA 双层修齐（html5ever 0.29.1
  fragment CDATA 门不 consult context_elem——宿主预变换 + 本地视图 ns 通道，
  FIXME 记 0.39 升级可移除）+ `_zwMEl` title/lang/dir 反射；CDATA 10/10、
  zero 14/14，75.33%。
- **M2 片 a（2026-10-02）**：sel 代理四身份 getter foreign ns 保真；foreign gET
  4 案全绿 + math-parse +4，75.31%。
- **M2 首簇（2026-10-02）**：character reference 直通双向修齐；
  named-character-references 0/2231→全绿，75.23%。
- **M1（2026-10-02）**：goals/40 fetch 脚本 + runner 通道 + Makefile target +
  基线 113 案 71.6% + wpt-suites.csv 回填。

## 下一步计划

1. **M3 片 b（下一片）**：serializing 邻面残差——serializing.html（解析上下文
   iframe 形态，M2 已记通道外）/ outerHTML.html / processing-instructions.html /
   serializing-lt-gt.html 逐案诊断；XMLSerializer XML 面（0.9%）随 dc-3 接口面
   一并评估。
2. **M3 片 c 起**：html/dom 接口语义——reflection 74.7% 边缘簇 + elements 12.1%
   逐簇修齐（DC-3 主面）。
3. **挂账随行**：bench-gate 等让复评（静窗）；html5ever 0.39 升级评估（顺带静态
   template noscript 解析面）；P5 通道外 19 案落地通道（reftest import 优先）；
   解析插入子树同 turn 读一致性架构片（R57 view doc vs `__zw_child_nodes`
   live-aware 分裂——ambiguous-ampersand 残差根因）。

**待用户决策清单**：（空）
