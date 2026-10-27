# HTML 文档面兼容 — 运行时控制面板（master.md）

> **已归档（模式 A，2026-10-20）**：当前终态、历史验证与挂账保留；入口见 [归档目标](../html-syntax-compat.md)。

**入口文档**: [../html-syntax-compat.md](../html-syntax-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-20（P10 document.write 流同 turn 读收敛，ambiguous-ampersand 2F→0F，通道 61347/61422）

## 当前状态

**P10 document.write 流同 turn 读收敛（2026-10-20）**：ambiguous-ampersand
**2F → 0F 清零**（P9 遗留挂账的 async-apply 缝 document.write 子集收口），
全通道 **61347P/61F/13T/1U = 61422 = 99.91%**（+2P −2F 零漂移，Fail 集构成
逐域核对不变）。R5206 续二五点：写流重放门（`__zwDocWriteReplay`）+ innerHTML
setter 门内 removed 记账只留补偿代理（基线重放非真移除——static div 从 gEBT
消失根因）+ 创建站点打标 `_zwSelPendingParent.inViewReplay`（IAH 站恒真/写流
门内 innerHTML 站真/**普通 innerHTML 站不打标**——SIH 不被视图回放，代理是其
同 turn 文档级枚举唯一可见面，初版无差别过滤被探针证伪 len 1→0）+ part05 两处
文档级并入剔打标代理（顺带修复 plain IAH 既有双计 gEBT 3 vs 2）+
`__zw_child_nodes` raw 缺席 sel 回落查询视图（written div childNodes 恒空根因）。
常驻测试 part24 双案（写流四断言 + 打标两侧语义）。**余账收敛**：本 goal 通道
63F → 61F，全为既有挂账域（58 render-blocking + 2 svg-script tokenizer 缝 +
1 document.all）；R3029 SIH 不回放的 stale base 限制维持（无案主张）。
证据：[evidence/2026-10-20-p10-write-stream-same-turn-read.md](evidence/2026-10-20-p10-write-stream-same-turn-read.md)。

**P9 ARIA 跨文档 scope + 视图回放修复（2026-10-20）**：`aria-element-reflection`
**2F → 0F**（M4 片 e 起挂账的 async-apply 缝 aria 子集全部翻绿）——R5206 三修：
`_zwAriaTreeChain` nodeType-9 文档身份落桶（主/分离文档同串 bucket 根因 +
WeakMap.size 'docNaN' 死代码激活）、sel 父 plain 子 parentNode/parentElement
重指（R368 盖章后 R84 gated 跳过洞）、getElementById `_r125AncestorRemoved`
先消费 `_zwAriaSyncParent` 同步父记录（shadow 移树后 gEBI stale 短路）。续查
ambiguous-ampersand 时发现共享路径双缺陷并修复（callbacks.rs
`with_query_view_doc`）：**R342 id 去重离一**（`start + 4` 应为 `start + 3`——
截断分片 panic 且带引号 id 去重自落地起从未生效）+ **同 turn 重置覆盖
latest-wins**（R5001 write 流每步 SIH+IAH 入队，全量重放堆 N 个截断分片子树——
host 视图 160 divs → 收敛 base + 满缓冲）。全通道 **61345P/63F/13T/1U =
61422**（+2P −2F vs P8，两行为变更零回归）；ambiguous-ampersand 2F 维持挂账
（残差在 shim 侧读路径 overlay 多层积累——host 层已收敛，探针
`_zwApplyGeneration()=0` 证实整页单程序无 apply，归 async-apply 缝
document.write 子集）。常驻测试 part21 r5206 四断言 + callbacks
view_doc_replay_tests 三案。证据：
[evidence/2026-10-20-p9-aria-crossdoc-gEBI.md](evidence/2026-10-20-p9-aria-crossdoc-gEBI.md)。

**P8 xmldecl 收编（2026-10-20）**：`html/syntax/xmldecl` **88/88 全绿**，
全通道 **61343P/65F/13T/1U = 61422**（vs P7 基线 61255P/65F/13T——**+88P
零新增 Fail/Timeout**，Fail 集仍全落 render-blocking 58F + syntax/parsing
4F + dom 3F 既有挂账域）。四层落地：① WHATWG prescan 算法
`_zwFramePrescanCharset`（1024 窗 meta 归纳 + get-an-XML-encoding 兜底，
字节/字符串双视图）；② iframe 解码链 BOM → transport charset → prescan →
重解码（replacement 标签保留降级 1252；plain wire 合法 UTF-8 同走嗅探）；
③ kind 按 Content-Type text/html 判 HTML 解析语义（trickle.py 动态端点）；
④ runner trickle.py 静态等价 + `.headers` sidecar CT + `.htm` MIME。
死循环教训（prescan 单 position 游标）沉淀
[learnings/bugs/2026-10](../../../learnings/bugs/2026-10/2026-10-20-prescan-position-cursor.md)。
评估关闭两项：XHR XML MIME final encoding（corpus 零断言无验证面）；
the-input-byte-stream 9 案复制（上游自述式测试依赖 wptserve 管道，复制需
发明 oracle）。证据：
[evidence/2026-10-20-p8-xmldecl-iframe-charset.md](evidence/2026-10-20-p8-xmldecl-iframe-charset.md)。


**P7 XHR document 编码嗅探（2026-10-20）**：html/syntax/charset `xhr.html`
**22F → 22P 清零**，全通道 **61255/61333 = 99.89%**（rebase cb8f59044 后组合
态；同日三独立跑全等 61255P/65F/13T——确定性确认，Fail 87→65 全为既有挂账
域零新增）。XHR set-a-document-response final encoding 接线：BOM →
override/MIME charset → meta 嗅探 → UTF-8，非 UTF-8 重解码 + `_zwEncoding`
槽（characterSet/charset/inputEncoding 三别名）。嗅探语义定谳 WebKit
HTMLMetaCharsetParser（tokenizer 感知 + head 区段追踪 + 1024 停扫 + 属性值
字符引用解码——spec 纯字节 prescan 被 ncr/in-script 案证伪）；runner fetch
通道非 UTF-8 .html 原字节直供（原 from_utf8_lossy 内联路径先于 JS 解码损失
字节面）。补 `rel` IDL 反射。余账：XHR XML MIME 分支 final encoding 接线
（FIXME 记账，非 22 案面）。证据：
[evidence/2026-10-20-xhr-responsexml-charset.md](evidence/2026-10-20-xhr-responsexml-charset.md)。

**P6b escaping noscript 漂移修复（2026-10-20）**：`escaping.html` **8/9 → 9/9
全绿**，全通道 **61229/61333 = 99.83%**（rebase 4bc987928 后组合态复测
**61233/61333**——+4P/−9F/+5T 全落 render-blocking 挂账域，本 goal 域零回归，
见证据组合态节）。归因（bisect 有效落点）：9dcd16661
（2026-10-10）innerHTML 产物物化真 handle 树改读路径，part04 融合序列化对文本子
无条件转义缺 noscript 分支。R5197：融合序列化补 literal/escape 判据（子节点
`__zwTreeScripting` 印章 + ownerDocument `__zwScriptingEnabled` 回落）+
`_wcRebuildAsHandle` text node 印章传递。**vs-M4 漂移账清零**（script-inserted
双案归 rendering-compat 域维持）。常驻测试
`test_noscript_serialization_literal_escape_r5197` 双面。证据：
[evidence/2026-10-20-p6b-escaping-noscript.md](evidence/2026-10-20-p6b-escaping-noscript.md)。

**P6 form-pointer 边界修复（2026-10-19）**：`fragment-parse-form-in-template`
**1F → 1P**，全通道 **61228/61333 = 99.83%**。R5196：`_zwMEl` 自有 `form`
accessor 祖先/root 上行命中 TEMPLATE 即断（inline content 视图 parentNode 指
模板 proxy 是视图捷径，宿主 form 不越界关联；同树 form= 关联保持）。**账目
修正**：该案非 post-M4 漂移——M4 收口「QSA 补面翻绿」记述不实（补面只消
'not a function'，form 断言自补面起一直红，bisect + 机制探针证伪），vs-M4
漂移余 2 案（escaping noscript 真实漂移待归因 + script-inserted 双案归
rendering-compat 域）。常驻测试 `test_template_content_form_owner_boundary_r5196`
三面。证据：[evidence/2026-10-19-p6-form-pointer.md](evidence/2026-10-19-p6-form-pointer.md)。

**P5 落地面已收口（2026-10-19）**：全通道复测 **61227/61333 = 99.83%**（+1
Unsupported 通道记账）。四修落地：R5003 reftest 字节级 decode 载入（charset/
编码案入通道）+ R5004 annotation-xml HTML integration point（math-parse01/03
5F→0F）+ R5005 SVG script href 外链（unclosed-svg-script +1P）+ R5006 prescan
注释跳过（charset meta-* 8 案 8F→8P）；另 **corpus 断补修复**——wpt-data 8 案页
+ 2 资产脱落（成因不明，fetch 脚本无删除路径）致通道总分较 M4 虚减 2595 子测，
已按 pin 补拉 + fetch 脚本显式记账。**html5ever 0.39 升级评估关闭**：升级可闭合
0F（complete_script 在 0.29 与 main 均为死 API 实证；definitionurl 调整两侧逐字
相同且 DOM 存储正确），残差改记「html5ever tokenizer 能力缝 2F」。P5 reftest
基线 18/22（4 案 0 像素差归因渲染端字体 notdef 回退，通道层挂账）。vs M4 漂移
4 案记账（script-inserted 双案 / fragment-parse form-pointer 回红 / escaping
noscript 序列化）——均 post-M4 main 多流 merge 区间形成，非本轮改动触及；
escaping/noscript 案与并行流活动工作面相邻，记碰头信号。M1 账逐案对账仅
render-blocking script-inserted 双案 −1P（该域既挂账 rendering-compat）。
DC-4 终态实测：fmt / guarded clippy / make reftest **712/712** 全绿；make test
75/76 suites 全绿——余 1 suite 2F 为 GPU filter 单测 luma 断言滞后于渲染流域
当日提交 8e7ba040f（着色器已切 709、单测未同步），碰头信号记账不硬解。
/tmp 共享 tmpfs 另见 cand-target 20G 并行流构建物占满（36K 余量）——共享资源
碰头信号记账。证据：
[evidence/2026-10-19-p5-landing.md](evidence/2026-10-19-p5-landing.md)。

**M4 已收口（2026-10-07）**：全通道冻结 **61218/61304 = 99.86%**（M1 基线
71.6% → +7334），M1 基线逐案 **0 回归**。收口前末波 2 小修：
template.content.querySelectorAll 补面 + video/IMG width/height "-0" 缓存
归一。**DC-1~4 全满足**（判定明细见
[evidence/2026-10-07-m4-close.md](evidence/2026-10-07-m4-close.md)）；DC-4
终态实测（make test 68 suites EXIT=0——首跑 service_worker 单进程内存超限
guard 杀为已知负载敏感 flake 型，复跑 0F 确认 / clippy / fmt / reftest
704/704）。
残差 86F 全分类挂账：render-blocking 行为面 ~67F（**渲染管线域 →
rendering-compat 挂账**——M1 碰头决策点落定：IDL 面已修齐，行为面非本 goal
桥接/解析域）、html5ever 0.29 core 8F（0.39 升级账）、读一致性缝 2F、
async-apply 缝 2F、document.all 1F、html5lib 3F、通道支撑 1F。

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
| P4 | html/dom 接口语义（createContextualFragment 已落 ✅） | ✅ **M4 片 e ✅（2026-10-07，99.86%）**：ARIA Element 反射（8 属性表/树链有效性/树内 id 解析/显式引用/FrozenArray 缓存）+ aria nullable getter 面；历史：M3 片 c/e + M4 片 a/b/c/d 递次收口；通道残差全分类挂账（render-blocking 管线域/html5ever 升级账/架构缝） |
| P5 | 文档级编码嗅探（encoding-compat M4 转入） | ✅ **P5 落地面收口（2026-10-19）**：R5003 字节级 decode 载入（charset/ reftest 22 案入通道，基线 18/22）+ R5006 prescan 注释跳过（meta-* 8 案 8F→8P）+ testharness 通道 charset 9 案新入（30 子测，8P）；通道外余量维持：xmldecl/ 3 案（iframe 形态，2026-10-20 R115 收编）+ the-input-byte-stream 9 案（HTTP 头形态）；XHR responseXML characterSet 22F 新记账（XHR/shim 域）；**testharness 通道语料读取接文档解码管道面 2026-10-28 由 encoding-legacy-mb goal DC-1 落地销账**（`read_corpus_to_string` 统一入口，legacy-mb 文档族解锁——docs/goal/encoding-legacy-mb/evidence/） |
| P6 | fragment-parse form-pointer 边界（账目修正） | ✅ **P6 修复（2026-10-19）**：R5196 `_zwMEl` 自有 form accessor TEMPLATE 边界（1F→1P，通道 61228/61333）；账目修正——非 post-M4 漂移，M4 冻结起既有 1F（「QSA 补面翻绿」记述不实） |
| P6b | escaping noscript 真实漂移 | ✅ **P6b 修复（2026-10-20）**：bisect 归因 9dcd16661（innerHTML 物化改读路径）→ R5197 融合序列化 noscript 分支 + 物化印章传递（8/9→9/9，通道 61229/61333，组合态复测 61233/61333）；vs-M4 漂移账清零 |
| P7 | charset XHR responseXML characterSet 面（22F 新账） | ✅ **P7 修复（2026-10-20）**：WebKit HTMLMetaCharsetParser 语义嗅探 + final encoding 接线 + runner 非 UTF-8 .html 字节保真（22F→22P，通道 61255/61333）；余账 XML MIME final encoding（FIXME） |
| P9 | aria-element 跨文档 scope + gEBI 同 turn shadow 移树（async-apply 缝 aria 子集 2F） | ✅ **P9 修复（2026-10-20）**：R5206 三修（树链 nodeType-9 文档身份/sel 父 plain 子重指/`_zwAriaSyncParent` 消费）2F→0F + 附带修 callbacks 视图回放 R342 离一 panic 与 latest-wins 重置覆盖（通道 61345/61422，零回归） |
| P10 | async-apply 缝 document.write 子集（ambiguous-ampersand 2F——shim 侧读路径多层积累） | ✅ **P10 修复（2026-10-20）**：R5206 续二五点（写流重放门/removed 记账门/创建站点打标/文档级并入剔打标代理/childNodes raw 缺席回落视图）2F→0F，顺带修 plain IAH 既有双计；通道 61347/61422 零漂移 |
| — | render-blocking 机制面（62 案 19.3%） | 📊 M1 已基线；**rendering-compat 域挂账维持（2026-10-19 归因收口：SyntaxError 41 案 M1 起既有 + post-M4 漂移 2 案）** |

## 已完成切片

- **P10（2026-10-20）**：document.write 流同 turn 读收敛——ambiguous-ampersand
  2F→0F（R5206 续二：写流重放门 `__zwDocWriteReplay` + innerHTML 门内 removed
  记账只留补偿代理 + 创建站点打标 `inViewReplay`（普通 innerHTML 不打标保留）
  + part05 文档级并入剔打标代理（顺带修 plain IAH 既有双计）+ childNodes raw
  缺席回落视图）；通道 61347/61422（+2P −2F 零漂移），plain IAH 同 turn gEBT
  3→2 双计修复附验证。常驻测试 part24 双案。证据
  evidence/2026-10-20-p10-write-stream-same-turn-read.md。
- **P9（2026-10-20）**：ARIA 跨文档 scope + gEBI 同 turn shadow 移树
  2F→0F——R5206 三修（`_zwAriaTreeChain` nodeType-9 身份落桶 +
  `_zwAriaDocIdSeq` 替代 WeakMap.size / sel 父 plain 子 parentNode 重指 /
  getElementById 先消费 `_zwAriaSyncParent`）；附带 R5206 续修 callbacks
  视图回放——R342 离一（`start+4`→`start+3`，截断分片 panic + 带引号 id
  去重激活）+ 窗口内 latest-wins 重置覆盖（写流 N 分片堆积收敛）+
  增量链 SIH 前置退全量；通道 61345/61422（+2P −2F 零回归）。常驻测试
  part21 r5206 四断言 + callbacks view_doc_replay_tests 三案。证据
  evidence/2026-10-20-p9-aria-crossdoc-gEBI.md。
- **P6b（2026-10-20）**：escaping noscript 漂移归因修复——bisect 落点
  9dcd16661（innerHTML 物化真 handle 树，融合序列化无条件转义缺 noscript
  分支）；R5197 融合序列化 literal/escape 判据 + `_wcRebuildAsHandle` 印章
  传递；8/9→9/9 通道 61229/61333（组合态复测 61233，render-blocking 域
  有利漂移逐桶核账）；vs-M4 漂移账清零。常驻测试 part21 r5197
  双面。证据 evidence/2026-10-20-p6b-escaping-noscript.md。
- **P6（2026-10-19）**：fragment-parse form-pointer 边界修复——R5196 `_zwMEl`
  自有 form accessor 祖先/root 上行 TEMPLATE 即断（inline content 视图捷径
  逃逸面）；1F→1P 通道 61228/61333；账目修正（M4 冻结起既有 1F 非 post-M4
  漂移——bisect + 机制探针证伪「QSA 补面翻绿」记述）。常驻测试
  part21 r5196 三面。证据 evidence/2026-10-19-p6-form-pointer.md。
- **M4 收口（2026-10-07）**：DC-1~4 全满足判定 + 残差终账（86F 全分类：
  render-blocking ~67F 归 rendering-compat 域、html5ever 8F 归升级账、
  架构缝 4F、其余 7F）+ 末波 2 小修（template.content.querySelectorAll/
  video "-0" 缓存）；99.86% 冻结，0 回归；DC-4 终态实测全绿。
  证据 evidence/2026-10-07-m4-close.md。
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

Goal 已收口（DC-1~4 全满足；P5 落地面 2026-10-19 收口）。后续账目（均不阻收
口，归各自域）：
1. ~~charset XHR responseXML characterSet 面（22F，新账）~~ ✅ P7 修复清零
   （2026-10-20，22F→22P，通道 61255/61333）。余账：XHR XML MIME 分支
   final encoding 接线（FIXME 挂账，非 22 案断言面）。
2. **vs-M4 漂移账清零（2026-10-20）**：escaping noscript 已 P6b 归因修复
   （9dcd16661 物化读路径回退，R5197）；form-pointer 已 P6 修复并改判 M4 既有
   账；余 script-inserted 双案维持 rendering-compat 域挂账。
3. ~~同 turn sel 移树同步视图（async-apply 架构缝，aria-element 2F）~~ ✅
   P9 修复（2026-10-20，R5206 三修 2F→0F，未动 async-apply 整体架构）。
   ~~缝的 **document.write 子集**：ambiguous-ampersand 2F~~ ✅ **P10 修复
   （2026-10-20，R5206 续二 2F→0F 清零）**——shim 侧读路径三层缝（raw wrapper
   误入 removed 记账/补偿代理与视图回放双计/raw 缺席容器子视图恒空）全部
   收敛，R3029 SIH 不回放的 stale base 限制维持（无案主张）。async-apply 缝
   在本 goal 的挂账面至此**全部关闭**。
4. **维持既有挂账**：render-blocking 行为面 → rendering-compat 域；html5ever
   升级评估已关闭（残差改记 tokenizer 能力缝 2F，升级无收益）；bench-gate
   等让复评（静窗）；P5 通道外 12 案（xmldecl 3 + input-byte-stream 9）维持
   通道外记账；XHR XML MIME 解析挂账；document.all exotic 对象（1F——
   [[IsHTMLDDA]] 不可实现性 P9 轮评估关闭：truthy JS 对象破真实站
   `if (document.all)` IE 探测产品兼容，无 V8/QuickJS 公开 API）；charset
   reftest 4 案字体回退像素语义（随渲染通道复评）。

**待用户决策清单**：（空）
