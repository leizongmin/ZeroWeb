# HTML 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../html-syntax-compat.md](../html-syntax-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-03（M3 片 d 第一波——per-interface IDL 83.83%）

## 当前状态

**M3 片 d 第一波已落地（2026-10-03）**：per-interface IDL 清单首批簇——spec 严格
解析面（`_zwParseSpecInt/Nonneg`：UINT 表 getter、input/select.size、PRE width
long、limited-unsigned setter 0 抛 IndexSizeError + "-0" 归一）+ URL/枚举反射
（BASE/LINK.href、crossOrigin nullable 枚举、link.as 关键字表、nonce 撤出反射面）
+ hr.noShade 布尔。全通道 **51391/61303 = 83.83%**（片 c 82.18%，+1011），M1 基线
逐案 **0 回归**；forms 88%、grouping 94%、metadata 85%、text 97%。DC-4 实测全绿
（test 68 suites / fmt / reftest 704/704）。残差归片 e：form.enctype/formMethod
枚举 IDL-set 面（198 簇）、formAction URL 解析反射（32）、embedded 尾簇（2599F）、
tabular（1992F）、aria-enumerated（1201F）。证据：
[evidence/2026-10-03-m3d-interfaces.md](evidence/2026-10-03-m3d-interfaces.md)。

**M3 片 c 已落地（2026-10-03）**：reflection IDL 反射**系统面**修齐（一改千测）：
R3042 expando 豁免（反射 DOMString 属性收 null/对象仍走 WebIDL 转义）+
`_zwAttrInstanceRemoveKey` 实例层同步（boolean IDL 移除后 hasAttribute 不再短路）+
autofocus/inert handle 真移除 + title/lang/accessKey 归普通 DOMString（null→"null"，
R3185 单测按 spec 更新）+ tabIndex "-0" Int32 归一 + `_REFLECTED_STRING_NULL_EMPTY`
（body 颜色族 LegacyNull）+ legacy 反射串扩表 + document 颜色/dir 别名。全通道
**50380/61303 = 82.18%**（片 b 75.55%，+3833），M1 基线逐案 **0 回归**；
reflection-sections **5604/5604 全绿**、text 97%、forms 85%、misc 87%。DC-4 实测：
make test 68 suites 全绿 + clippy 干净 + fmt + reftest **704/704**。残差归片 d：
per-interface IDL 清单（base/link.href URL 解析反射、table 章节族、ol.start/
reversed/li.value 数值面、ARIA 枚举反射）。证据：
[evidence/2026-10-03-m3c-reflection.md](evidence/2026-10-03-m3c-reflection.md)。

**M3 片 b 已落地（2026-10-03）**：**serializing-html-fragments 域 137/137 全绿**
（template.html 2 面 + processing-instructions.html 3 面 + outerHTML canvas 面
修齐）+ **serializing-xml-fragments 域 112/112 全绿**（`_zwXMLSerialize`
DOM-Parsing §3.2.1 语料面 + `_zwMSerialize` XML 域分支三接线）。全通道
46316/61303 = **75.55%**，M1 基线逐案 0 回归。架构首块：`__zw_get_inner_html`
切视图文档（解析插入子树同 turn 读一致性片落地——R57 view doc 读链对齐，空闲期
同一 live fast path）。DC-4 实测：make test 68 suites 全绿 + clippy 干净 + fmt +
reftest **704/704**。残差收窄：ambiguous-ampersand 另一半（`__zw_child_nodes`
live-aware + char-by-char 流语义）仍挂账。证据：
[evidence/2026-10-03-m3b-serializing.md](evidence/2026-10-03-m3b-serializing.md)。

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
| P3 | 序列化边缘（XMLSerializer/HTML serializer） | ✅ **M3 片 b 落地（2026-10-03）**：serializing-html-fragments 域 **137/137 全绿**（template/processing-instructions/outerHTML canvas 面）+ serializing-xml-fragments 域 **112/112 全绿**（`_zwXMLSerialize` XML 序列化 + 读链视图文档对齐）；escaping.html 9/9（片 a）；残差：ambiguous-ampersand 读链 seam（挂账） |
| P4 | html/dom 接口语义（createContextualFragment 已落 ✅） | 🔄 **M3 片 c 落地（2026-10-03）**：reflection 系统面修齐（全通道 82.18%，reflection-sections 全绿、text 97%、forms 85%）；残差归片 e：form 枚举/URL 面、embedded/tabular 尾簇、ARIA 枚举反射；**片 d 第一波 ✅（2026-10-03，83.83%）**：spec 严格解析 + BASE/LINK.href/crossOrigin/as/nonce + noShade |
| P5 | 文档级编码嗅探（encoding-compat M4 转入） | 🔄 testharness 通道可执行面首片 ✅（sniff→Document label→characterSet 管线，2026-10-02）；📊 通道外案面仍记账：charset/ 7 案（reftest 形态）+ xmldecl/ 3 案（iframe 形态）+ the-input-byte-stream 9 案（HTTP 头形态）= 19 案 testharness 通道外记账；encoding/ 域 bom-handling/eof-*/sniffing 案面（已在 wpt-data，encoding 通道按 document.characterSet 规则 skip）尚未基线，落地通道（reftest import / 通道扩展）随 M2+ 定 |
| — | render-blocking 机制面（62 案 19.3%） | 📊 M1 已基线；`blocking=render` 与渲染管线耦合，是否本 goal 修齐随 M3 碰头定 |

## 已完成切片

- **M3 片 d 第一波（2026-10-03）**：spec 严格解析（`_zwParseSpecInt/Nonneg` +
  UINT/size/PRE-width 面换装 + limited setter 0 抛）+ BASE/LINK.href/
  crossOrigin/as/nonce URL 与枚举反射 + noShade；83.83%（+1011），0 回归；
  DC-4 全绿。证据 evidence/2026-10-03-m3d-interfaces.md。
- **M3 片 c（2026-10-03）**：reflection IDL 反射系统面——R3042 expando 豁免 +
  实例层移除同步（`_zwAttrInstanceRemoveKey`）+ autofocus/inert handle 真移除 +
  title/lang/accessKey 归普通 DOMString（R3185 单测按 spec 更新）+ tabIndex
  "-0" 归一 + `_REFLECTED_STRING_NULL_EMPTY` + legacy 反射串扩表 + document
  颜色/dir 别名；82.18%（+3833），0 回归；DC-4 全绿（test/clippy/fmt/reftest
  704/704）。证据 evidence/2026-10-03-m3c-reflection.md。
- **M3 片 b（2026-10-03）**：serializing 双域全绿——serializing-html-fragments
  137/137（template contents 序列化双面 + PI 三序列化环 + canvas standalone
  outerHTML）+ serializing-xml-fragments 112/112（`_zwXMLSerialize` DOM-Parsing
  §3.2.1 + `_zwMSerialize`/outerHTML/`_zwParsedDoc` 三接线）；`__zw_get_inner_html`
  切视图文档（读一致性架构片首块）；75.55%，0 回归；DC-4 实测全绿
  （test 68 suites / clippy / fmt / reftest 704/704）。
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

1. **M3 片 e（下一片）**：reflection 尾簇续——form.enctype/formMethod 枚举
   IDL-set 面（198 簇）、formAction URL 解析反射（32）、embedded 尾簇（img/
   iframe 维度与 URL 族，2599F）、tabular 章节族（1992F）、aria-enumerated
   枚举反射语义（1201F）；render-blocking IDL 面随碰头定。
2. **挂账随行**：bench-gate 等让复评（静窗）；html5ever 0.39 升级评估（顺带静态
   template noscript 解析面）；P5 通道外 19 案落地通道（reftest import 优先）；
   读一致性架构片余量（`__zw_child_nodes` live-aware——ambiguous-ampersand
   char-by-char 流语义另叠一层，独立片评估）；XHR XML MIME 解析挂账。

**待用户决策清单**：（空）
