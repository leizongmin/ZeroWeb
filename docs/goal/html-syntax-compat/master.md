# HTML 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../html-syntax-compat.md](../html-syntax-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-07（M4 片 e——ARIA Element 反射特性片 99.86%）

## 当前状态

**M4 片 e 已落地（2026-10-07）**：ARIA Element 反射特性片 + 尾簇扫尾（+48
passes）：`_ZW_ARIA_EL_ATTRS` 8 属性（activedescendant 单元素 + 7 FrozenArray
面）——树链有效性 walker（shadow host 跳跃/分离文档 ownerDocument 区分）、
树内 id DFS 解析（proxy 同步视图修复 host 索引同 turn stale）、显式引用存储
（removeAttribute 解除/setAttribute 覆盖/attr 空串面）、FrozenArray 身份缓存、
TypeError 面、sel 子挂 handle 容器同步父记录（`_zwAriaSyncParent`）；
aria-element-reflection 23F→3F + disconnected 2F→0 + aria-attribute-reflection
21F→0（role/aria nullable getter 面——set-then-remove 标记）+ historical 2F
（applets 空集合/computed float 'none'）+ fragment-parse-form-in-template 1F
（input.form TEMPLATE 祖先门）。全通道 **61216/61304 = 99.86%**（片 d 收尾
99.78%，+48），M1 基线逐案 **0 回归**；DC-4 实测全绿（test 68 suites / fmt /
clippy / reftest 704/704）。残差 88F：aria-element 2F（同 turn sel 移树 host
视图 stale——引擎 async-apply 架构缝）、render-blocking 行为 ~13F（管线集成
挂账）、document.all 1F、M2 残差（math-parse/svg-script/html5lib/
ambiguous-ampersand）。证据：
[evidence/2026-10-07-m4e-aria-element.md](evidence/2026-10-07-m4e-aria-element.md)。

**M4 片 d 已落地（2026-10-07）**：reflection 混合尾簇**系统面**修齐（+1381
passes）：值 getter 遮蔽门（part03 通用 dirty-cache value getter 撤出 BUTTON/LI/
METER/PROGRESS/PARAM/OPTION——li.value/meter.value 串化全族根因）+ standalone
canvas 手工属性面三修（attr 名 ASCII 小写/getAttribute has 门/逐字 DOMString）+
枚举面（inputMode/enterKeyHint 入 MAP、crossOrigin 扩 SCRIPT/LINK、referrerPolicy
补 A、全枚举 getter 换 ASCII 小写防 U+212A 误判）+ URL/串反射表调（meta.content/
scheme、codeType/acceptCharset/encoding 映射、name 17-tag 门——name-content
123F 修齐、nonce-hiding 面）+ 数值面（size/width/height/cols/rows/start spec 解析
与 limited/fallback 语义、ol.start 撤 throwOnZero、progress.max/meter setter）+
forms 尾（form.action 缺省文档 URL、formMethod/Enctype 缺省 ''、autocomplete 缓存
归一）。全通道 **61168/61304 = 99.78%**（片 c 收尾 97.53%，+1381），M1 基线逐案
**0 回归**；DC-4 实测全绿（test 68 suites——2 单测按 WPT corpus 更新 / fmt /
clippy 干净 / reftest 704/704）。残差 136F：aria-element-reflection 25F（Element
反射特性——片 e）、render-blocking 行为 ~18F（管线集成挂账）、M2 残差（math-parse/
svg-script/html5lib）等。证据：
[evidence/2026-10-07-m4d-tails.md](evidence/2026-10-07-m4d-tails.md)。

**M4 片 c 已落地（2026-10-03 提交 0e49d074c + d7e8aaa21）**：obsolete 尾簇
（403F → 2F——marquee scrollAmount/scrollDelay setter、FRAMESET cols/rows string
双侧 tag 门、bgColor LegacyNull tag 门扩 table 族、frame.frameBorder attr 名
小写化、FRAME 入 URL 表、font/dir.compact 面）+ forms 深水面（636F → 259F——
INPUT numeric width/height、maxLength/minLength limited-long 负值抛、METER double
反射、progress.max 浮点前缀解析、FLAT/MAP 扩表）。全通道 96.39% → **97.52%**
（+691），M1 基线 0 回归；DC-4 全绿（test 68 suites / fmt / reftest 704/704）。
证据：[evidence/2026-10-06-m4c-forms.md](evidence/2026-10-06-m4c-forms.md)。

**M4 片 b 第三波已落地（2026-10-05）**：URL/枚举 setter 分支前移到 expando
判定之前（`IDL set to ""`/undefined/对象 attr 残留根因——expando 先行拦截吞写）
+ IMG.src/IFRAME.src 排除出统一分支（专面 fetch/error 派发与导航钩子依赖，
r3284/r388 单测 bisect 实证）。embedded 尾 408F → 394F，全通道维持
**59091/61303 = 96.39%**，M1 基线逐案 **0 回归**；DC-4 实测全绿（test 68
suites——首跑 send_keys 3F 为负载敏感 flake 隔离复跑绿+全量复跑确认 / fmt /
reftest 704/704）。残差归 M4 片 c：embedded 尾 394F（setAttribute undefined
串写面）、obsolete ~400F、其余域尾。证据：
[evidence/2026-10-05-m4c-tails.md](evidence/2026-10-05-m4c-tails.md)。

**M4 片 b 第一波已落地（2026-10-05）**：reflection 混合尾簇首批——数值 setter
面（img/video width/height/hspace/vspace IDL set > maxInt → "0"）+ media falsy
实例同步（video/audio.loop/defaultMuted——R122 短路根因）+ track 三面（kind
missing/空串 has_attr 区分 subtitles/metadata、src 空 → ''、label/srclang 逐字）
+ FLAT/MAP/BOOL 扩表（scrolling/frameBorder/archive/code/standby/codeType/
width/height string 面、valueType、declare/noHref）。全通道 **58638/61303 =
95.65%**（+363），M1 基线逐案 **0 回归**；DC-4 实测全绿（test 68 suites——R388
track missing 单测按 has_attr 区分更新 / fmt / reftest 704/704）。残差：embedded
860F（iframe/embed/object width-height string 读侧遮蔽、URL 面 expando 泄漏）、
obsolete ~400F、其余尾 ~900F——逐簇续片 b。证据：
[evidence/2026-10-05-m4b-tails.md](evidence/2026-10-05-m4b-tails.md)。

**M4 片 a 已落地（2026-10-04）**：reflection 尾残双域全绿——reflection-tabular
**6116/6116**（串/映射扩表 + scope 枚举 + col/colgroup.span clamped 口径三轮
收敛 + colSpan/rowSpan max clamp + 撤 min===1 全表 0 抛仅留 start throwOnZero）+
aria-attribute-reflection-enumerated **1722/1722**（`_ZW_ARIA_ENUMS` 20 属性
kw/inv/d 表 + gated getter + set-then-remove null 语义 `_zwAriaExplicit` +
setter null/undefined → removeAttribute）。全通道 **58275/61303 = 95.06%**
（+3387），M1 基线逐案 **0 回归**。DC-4 实测全绿（test 68 suites——首跑
vue_e2e 3F 为 aria getter Symbol prop 未守卫，补 typeof 后复跑全绿 / fmt /
reftest 704/704）。残差：embedded 尾 1180F、obsolete 403F、forms 尾 374F
（均为混合个体面，非系统簇）；M4 片 b 逐簇收口。证据：
[evidence/2026-10-04-m4a-tails.md](evidence/2026-10-04-m4a-tails.md)。

**M3 片 e 已落地（2026-10-04）**：reflection 尾簇续——form 枚举/URL 面
（form.action/formAction URL 解析、formMethod/formEnctype/form.autocomplete
枚举、progress.max double、li.value long）+ URL 反射表（`_REFLECTED_URL_TAGS`：
img.lowsrc/longDesc、object.data/codeBase、video.poster、script.src、cite 族
等 ~500F）+ embedded 枚举（referrerPolicy/decoding/loading，A 门补录）+
inputMode/enterKeyHint 全局枚举 + canvas standalone 属性方法面 + iframe/embed
width/height string 面收窄 + marquee 数值面。全通道 **54888/61303 = 89.54%**
（+3508），M1 基线逐案 **0 回归**；sections 100%、text 99%、forms/misc/grouping/
metadata/weekmonth 94-95%、embedded 86%、obsolete 82%。DC-4 实测全绿（test 68
suites + R2839/R3037 单测按 spec 更新 / fmt / reftest 704/704）。残差归 M4：
tabular 68%（table 章节集合面）、aria-enumerated 30%、obsolete 尾 82%。
证据：[evidence/2026-10-04-m3e-tails.md](evidence/2026-10-04-m3e-tails.md)。

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
| P4 | html/dom 接口语义（createContextualFragment 已落 ✅） | 🔄 **M4 片 d ✅（2026-10-07，99.78%）**：reflection 混合尾簇系统面（值 getter 遮蔽门/canvas 面/枚举 ASCII/URL 串表/数值 limited 面/forms 尾——name-content 123F 修齐）；历史：M3 片 c/e + M4 片 a/b/c 递次收口；残差：aria-element-reflection 25F（Element 反射——片 e）+ render-blocking 行为 ~18F（挂账） |
| P5 | 文档级编码嗅探（encoding-compat M4 转入） | 🔄 testharness 通道可执行面首片 ✅（sniff→Document label→characterSet 管线，2026-10-02）；📊 通道外案面仍记账：charset/ 7 案（reftest 形态）+ xmldecl/ 3 案（iframe 形态）+ the-input-byte-stream 9 案（HTTP 头形态）= 19 案 testharness 通道外记账；encoding/ 域 bom-handling/eof-*/sniffing 案面（已在 wpt-data，encoding 通道按 document.characterSet 规则 skip）尚未基线，落地通道（reftest import / 通道扩展）随 M2+ 定 |
| — | render-blocking 机制面（62 案 19.3%） | 📊 M1 已基线；`blocking=render` 与渲染管线耦合，是否本 goal 修齐随 M3 碰头定 |

## 已完成切片

- **M4 片 e（2026-10-07）**：ARIA Element 反射特性片（8 属性表/树链有效性/
  树内 id 解析/显式引用/FrozenArray 缓存）+ aria nullable getter 面 + 尾簇
  （applets/cssFloat/template form 门）；99.86%（+48），0 回归；DC-4 全绿。
  证据 evidence/2026-10-07-m4e-aria-element.md。
- **M4 片 d（2026-10-07）**：reflection 混合尾簇系统面——值 getter 遮蔽门 +
  standalone canvas 三修 + 枚举 ASCII 面 + URL/串表调 + name tag 门 +
  数值 limited/fallback 面 + forms 尾；99.78%（+1381），0 回归；DC-4 全绿
  （2 单测按 corpus 更新）。证据 evidence/2026-10-07-m4d-tails.md。
- **M4 片 c（2026-10-03）**：obsolete 尾簇 + forms 深水面；97.52%（+691），
  0 回归；DC-4 全绿；bgColor tag 门扩 table 族修正（d7e8aaa21）。
  证据 evidence/2026-10-06-m4c-forms.md。
- **M4 片 b 第三波（2026-10-05）**：URL/枚举 setter 前移 expando 前止损
  （embedded 408→394F）+ IMG.src/IFRAME.src 专面排除（r3284/r388 bisect）；
  96.39% 维持，0 回归；DC-4 全绿。证据 evidence/2026-10-05-m4c-tails.md。
- **M4 片 b 第二波（2026-10-05）**：canvas standalone 全局反射 + URL/枚举
  setter 统一分支（IFRAME.src 排除保留导航钩子）+ iframe/embed width-height
  string 面收窄 + numeric getter maxInt 面；96.39%（+453），0 回归；DC-4 全绿。
  证据 evidence/2026-10-05-m4b2-canvas.md。
- **M4 片 b 第一波（2026-10-05）**：数值 setter maxInt → 0 面 + media falsy
  实例同步 + track kind/src/label 三面 + FLAT/MAP/BOOL 扩表；95.65%（+363），
  0 回归；DC-4 全绿。证据 evidence/2026-10-05-m4b-tails.md。
- **M4 片 a（2026-10-04）**：tabular/aria 双域全绿——tabular 串/映射/枚举/
  clamped 全套 + `_ZW_ARIA_ENUMS` 20 属性枚举反射（default-slots 语义）；
  95.06%（+3387），0 回归；DC-4 全绿（vue_e2e Symbol prop 守卫修正后复绿）。
  证据 evidence/2026-10-04-m4a-tails.md。
- **M3 片 e（2026-10-04）**：reflection 尾簇——form 枚举/URL 面 + URL 反射表 +
  embedded 枚举 + 全局枚举 + canvas 属性方法面 + width/height string 收窄 +
  marquee 数值；89.54%（+3508），0 回归；DC-4 全绿。证据
  evidence/2026-10-04-m3e-tails.md。
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

1. **M4 收口评估（下一片）**：残差 88F 全分类记账后评估 DC-2/DC-3 判定与 M4
   收口（通道可执行面 99.86%）；可选深项——同 turn sel 移树同步视图（引擎
   async-apply 架构缝，aria-element 2F + 潜在收益面）、render-blocking 管线
   集成（~13F，随碰头定）、document.all exotic 对象（1F）。
2. **挂账随行**：bench-gate 等让复评（静窗）；html5ever 0.39 升级评估（顺带静态
   template noscript 解析面）；P5 通道外 19 案落地通道（reftest import 优先）；
   读一致性架构片余量（`__zw_child_nodes` live-aware——ambiguous-ampersand
   char-by-char 流语义另叠一层，独立片评估）；XHR XML MIME 解析挂账。

**待用户决策清单**：（空）
