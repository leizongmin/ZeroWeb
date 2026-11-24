# DC-2 s10-registries 刀⑧（registry 创建时点 own-stamp 模型＋initialize 子树盖印＋DSR clonable 根克隆传播＋`shadowrootcustomelementregistry` 序列化回写＋SR 解析上下文域印）判读收账（2026-11-24）

k7 收账后同轮续刀：custom-elements/registries/element-mutation.html 15F 簇
（5 形 × append/appendChild/prepend 三移动面，scoped registries「移动不重钉
＋initialize 盖印＋克隆传播」三语义缺位）收敛至 0F（+15 面）；配套
element-mutation-null-registry-removal.html 保持 Pass。

## 范围与根因（探针定谳链）

**移动不重钉（Face A own-stamp）**：spec 模型＝element 的 custom element
registry 在**创建时点**钉定（createElement/createElementNS → 创建文档的
registry），移动（append/appendChild/prepend 入出 scoped shadow 树）不重钉。
旧形无创建时点印——getter 全靠祖先走查，scoped SR 内的 plain 元素误读 SR 域。
修＝part06 createElement/createElementNS 双工厂臂 plain 路径盖 `_zwCeElRegSlots`
own 印（doc 默认＝global；option 臂既有 `_zwCeBindRegistry` 不动）。

**initialize 条件盖印**：spec「initialize」步 3＝root 的 shadow-including
子树逐元素 set——**仅 custom element registry 为 null 的元素**（own 槽 null
印记／no-BC 域印 `_zwCeNoBc`，即 template contents owner document 产物）；
显式非 null 槽不触碰（initialize-upgrades「only upgrades」面 registry2
存活断言＝语料锚）。首装版误作无条件盖印（含覆盖他实例/全局印），电池判读
initialize-upgrades 8P/4F→5P/7F（三 doc 味「only upgrades」全翻 P→F、零
F→P）→ 按语料条件收窄重装（`_zwCeStampSubtreeRegistry`，part03：own 槽
null 印记或 no-BC 域印→盖 inst；先序栈式，子解析所见父槽已定；嵌套 SR 不
盖印——根 SR 关联由既有 root 臂承载）。复盘要点：四 k7 残留 F
（tree-order Document 味／already-initialized／declarative/imperative SR
upgrade）全在**升级臂**（define/initialize 时点 upgrade gate、ctor 桥），
非盖印面——无条件盖印零 F→P 即此信号。

**DSR clonable 根克隆传播**：`<template shadowrootmode shadowrootclonable>`
宿主 `cloneNode(true)` 须传播物化根。克隆深臂（part04 trap cloneNode）走
`_zwFragmentAdded` 非物化臂——plain 视图零根。**引擎真相门**：序列化串含
`shadowrootmode` ＝ engine 已在 `__zw_set_inner_html_handle` 重解析时物化克隆
树声明式根（serializer 仅对 clonable 根回注模板形态，串标记即引擎真相）——
marker-gated 切物化孪生 `_zwFragmentAddedUnsafe`（`sr` 字段经
`_zwAttachDeclarativeSRView` 挂工厂域根视图，顶层子重建经 `_wcRebuildAsHandle`
→ `_zwTransferDeclarativeSR` 按 clonable 位转登 handle 域注册表，shadowRoot
读走既有命中臂）。无标记克隆零 delta（WC-M1 切片 2a 哨兵保持）。

**传输面择串**：融合序列化（`_zwFusedInnerHtmlSel`）JS 臂按 innerHTML 语义剥
物化根 echo（宿主 JS 视图非空时服务）——模板标记消失致 marker 门漏判。修＝
融合串缺 `shadowrootmode` 标记且引擎串含标记时切引擎串。代价：该组合形态
（子树带物化根＋融合桶独有 pending 子）下 pending 子不入克隆——传输完整性
让位根完整性（FIXME: 逐子融合择串切片）。

**`shadowrootcustomelementregistry` 序列化回写**（crates/dom/serializer.rs
ShadowRoot 臂，上注〔未落〕补齐）：缺失使序列化→重解析回环保不住独立
registry 槽声明（materialize 两臂 get_attribute 捕获 → sr JSON `r` 键 → JS
视图 DECL_NULL 粘滞槽印记断链）——克隆树 initialize 前 registry getter 误读
global（element-mutation test2/test3 前置面）。配套 part05
`_zwResolveRoot180` 双臂 `cerEligible` 判据＋`_zwSerNode180`/`_zwGetHtmlImpl`
两处 JS 侧回写。

**SR 解析上下文域印（s10k5② 回归根因，面包屑定谳）**：刀⑤ 钉面
（null 域 SR `innerHTML='<span>'` 后 querySelector 读 null）在 Face A 上线后
回归——`_wcRebuildAsHandle`（innerHTML/setHTMLUnsafe 重建器）以
`document.createElement` 重铸解析产物 wrapper → Face A doc 默认印误盖 global
（fragment 解析 appropriate registry 应取目标 SR 容器 registry 槽）。定谳三段：
①lossy 探针打印把 `_ZW_CE_NULL_REG` marker 误读作 "inst"（修探针非引擎）；
②手工 walk 读错店（元素印在 `_zwCeElRegSlots` WeakMap 非 `_zwCERegistry` 属性）；
③resolver 面包屑（`__k8rz`，`enter:nt1:(sl=G)`）实证 span 自身 WeakMap 槽＝
global。修＝`_wcRebuildAsHandle` 第 2 参 `ctxReg`（undefined＝缺省域零
delta；null 印记/scoped 实例＝盖 own 印，深子递归传播），t8r innerHTML 臂
（part04）＋`_zwSetHtmlUnsafeImpl` 臂（part05）对 SR 目标容器
（`_shadowHandles[handle]` 门）经 `_wrapHandle(handle)._zwCERegistry` 归一
（`_zwCeIsRegNullMark` → `_ZW_CE_NULL_REG`）传入。克隆/adopt 重建面不传
（undefined 零 delta）。FIXME: createElement 本体的全局升级副作用（defined
tag 在 null 域解析仍走全局 ctor）未抑制——语料无钉面。

**元素容器解析上下文域印（v2 收尾，电池回归定谳）**：首电池判读
Element-innerHTML「disconnected 元素 simple HTML 形」与 exceptions「innerHTML
创建 failed element」双面 P→F——容器为**带 scoped own 印的 disconnect 元素**
（非 SR），重建 Face A doc 默认印阻断 walk-up。修＝t8r＋setHTMLUnsafe 两臂
补元素容器支：非 SR 容器经 `_zwCeResolveElementRegistry`（own→祖先→doc
默认）解析域，null/印记归一 `_ZW_CE_NULL_REG` 盖 own 印；主文档全局域值恒
等零 delta（k7 未印 walk-up 同值）。

**TEMPLATE 容器跳过（v3 收尾，PEEK 插桩定谳）**：元素容器支上线后
Element-innerHTML「insertAdjacentHTML with beforebegin」面 P→F——根因链
定谳（registry `__k8id` 身份插桩＋setter 位点面包屑 `__k8Mark`＋PEEK
`__k8OwnSlot`）：语料 face「innerHTML on a template with a scoped registry
…」（L163 形）把 outer_element append 到 body **无 cleanup**，残留
`<new-element>` 留在 body；beforebegin 面断言的 `div.previousSibling`
读到的正是这份残留（own=S id=7，而面自身 registry id=9——PEEK
`own=S ownId=7 divId=9 same=false`）。v2 元素支把残留按 TEMPLATE 自身
scoped 印重盖→面 F；v1/首装版残留无印→走查→global→面 **恰好通过**（
k7 即此假 P 形：面断言读到的是残留而非新插 sibling，IAH 视图补偿缺口
使新 sibling 不可达＝恰好免检）。spec 语义＝fragment 解析对 template
context 用 contents
owner document（无 BC → null 域），主文档模板 contents 语义面＝**文档
registry**——template 自身 option 印不传播。修＝t8r＋setHTMLUnsafe 两臂
元素支前置 `_realTag(null, handle) === 'TEMPLATE'` 跳过（语料锚：
Element-innerHTML「innerHTML on a template with a scoped registry should
use the scoped registry of the document」）。
https://html.spec.whatwg.org/multipage/parsing.html#parsing-html-fragments
v3 单页核验＝Element-innerHTML 12 行逐面恒等 k7（Fail Fail Pass Fail Fail
Fail Fail Fail Fail Pass Fail Timeout）——simple HTML P 保留＋beforebegin
P 复原双兑现。

**adopt 走查非嫌疑（面包屑排除）**：scoped-registry-append 页 Global-gain
形（scoped SR 内直/嵌 append new-foo → iframe adopt 后须 win 全局）单页假
F 嫌疑——`__k8ad` 面包屑插桩＋corpus 逐字复刻探针（diagC）定谳
`a1IsWin=true a2IsWin=true`（直子＋x-foo 嵌套双面 adopt 后均 win 全局），
电池 v2 页级对账该页零 delta——走查正确，插桩全拆（grep `__k8ad` 计 0）。

## 常驻钉测（同提交）

**part154_s10k8_registries_knife8_pins.rs** 单针六面：
① plain createElement own-stamp（创建时点印＋双向移动恒 global）；
② initialize 条件盖印对非 null own 槽不触碰（plain 元素 initialize 后
SR 内/移出/跨根恒 global）；
③ initialize 条件盖印双臂（own null 印记→盖实例；显式 option 他实例→
不触碰，语料「only upgrades」registry2 存活锚）；
④ DSR clonable 根克隆传播（fixture 初始种子 → cloneNode 后 shadowRoot 物化
＋clonable 真＋shadow 子可达＋initialize 盖印移出恒等）；
⑤ createElementNS plain 臂同面＋option 臂不受影响；
⑥ 显式 option 槽保持（inst→inst、null→null）。

## 字节绑定链

| 阶段 | md5 |
|---|---|
| 终态 runner（电池判定二进制，v3） | `ebf4ca0e20d7ac7b4c04cc51616d89a5` |
| part03.js | `1fd8f2c6777bf8113044744cc272e965` |
| part04.js | `1c233ceb0c791ac3a686ee1d2cee2719` |
| part05.js | `2731034b588b4ff181146f88de76fbee` |
| part06.js | `eeed2b4a7ac88a228bcf6ac1bf1cfbfa` |
| serializer.rs | `f75b3718932bd3407d8f6d8a9c581841` |
| part154 钉测 | `b959138bae29de1a6d3389cc99b70605` |
| js_dom_bridge_tests.rs | `2b76168a78e88980115732da6a619e6d` |
| 电池发射器（.acceptance/dps 本地，v3 绑定） | `89c44d91e0b69c54166c9e5c526d5c7f` |

中间态电池（判读用，已被 v3 取代）：v2 runner
`960205266547f4018715d6d33f6f8f2f`、part03
`29ca81d3c297ce35bbeef2ecdefdd5f7`、part04
`4e4430c3aa6fc3f6d3b198aac6896e6e`、part05
`580e1f6fd21209525527dba78f2374fc`、发射器
`8d5d19362d4cf8590c2c13c768618cdd`；v1（首装无条件 initialize＋无元素支）runner
`0dc089c2d93e82e44527dac1205907ce`、part03
`3413fae2363486a0b38f9d096550c85a`、part04
`d15a1d2c269c3a4b2169005b6b617c86`、part05
`d07f9aef4246d4ac9c061fb753961261`、part154
`bc6eede368f1ef80a358119440398e0b`。

前置钉测：`cargo test -p zero-engine --lib s10k`＝part148–154 **七代全绿
7 passed 0 failed**。fmt --all 唯一 diff＝computed_style.rs（既有 HEAD 漂移
e134036a5，本刀未触，零新 drift）；clippy -p zero-engine/-p zero-dom 唯一
warning＝cssa_registry.rs:262 identical-if（既有，零新 warning）。

中间迭代事件（防重走）：①探针 lossy 打印误导——`_ZW_CE_NULL_REG` marker 对象
经 String 判型读作 "inst"，三态（undefined/null/instance）判据必须用
`__zwCeNullRegistry` 位非 typeof；②元素印店＝`_zwCeElRegSlots` WeakMap，
`_zwCERegistry` 属性仅 shadow root/fragment 容器持有——手工 walk 读错店
得出「未盖章」假象；③resolver 面包屑临时插桩（`__k8rz` 六行）实证定谳后
终态树零残留（grep 计 0）；④首电池发射含 gitignored 刮擦探针页
（zz-k8-probe/zz-k8-k5repro）→ wc 腿将污染 k7 对账（k5repro 刻意断言恒
Fail）——中止（pkill -f 自匹配本 shell exit 144、runner 孤儿按精确 pid
-9）后删页重发，终态电池为净语料；⑤adopt 走查嫌疑排除链——`__k8ad`
四点面包屑＋corpus 逐字复刻探针（diagC 直/嵌 new-foo 双断言同窗）
`a1IsWin=true a2IsWin=true`，页级对账零 delta 后全拆（grep 计 0）；
⑥probe rn() 助手 win 形参缺省把 win.customElements 误读 'inst'
（'inst' 标签对 win 恒假阴）——win 形参显式传；⑦testharness 单页探针
`add_cleanup` 不存在（换普通局部状态）、`makeIframe(t)` 形参 arity 死参
（改无参）；⑧首装 initialize 无条件盖印电池判读 +3F 零 F→P 即「错装」
信号——条件收窄重装前先逐面 diff（k7/k8 面清单）定 F 构成，不猜语义；
⑨元素容器支误伤 TEMPLATE——beforebegin P→F 面换即信号，插桩三件套
（`__k8id` 身份＋`__k8Mark` setter 位点＋`__k8OwnSlot` PEEK）逐层定谳
残留污染链后按 spec 语义跳过 TEMPLATE 容器（非「撤销元素支」——simple
HTML 面依赖它）；⑩终态收尾四件：全插桩拆除（grep `__k8` shim 计 0，
钉测自用 `__k8ra`/`__k8rb` fixture 不在拆除域）、四刮擦探针页删除
（zz-k8c/d/e/f，registries 目录 grep zz- 计 0）、重编译单页五腿核验
（element-mutation 15P/0F＋null-removal 1P/0F＋initialize-upgrades
8P/4F＝k7＋exceptions 2P/1F＝k7＋Element-innerHTML 逐面恒等 k7）、
发射器重绑 v3（EXPECT_MD5 前置断言，发射前 MD5-OK 留痕）；
⑪grep 对 wpt 日志二进制误判假空复发两次（k7 日志「无此页」结论即
假空产物）——日志计数一律 `LC_ALL=C awk`，见既有教训条目。

## 电池判读（锚＝已推 刀⑦ dc2-s10k7-*.log 四腿；窗 solo）

v3 电池（终态构建，发射 12:33:01Z→BATTERY-DONE 12:48:32Z，15m31s；四腿
EXIT=1 留痕，门信号用计数判读非 rc）：

| 腿 | k7 锚 | v3 终态 | 判读 |
|---|---|---|---|
| domparsing | 全日志 md5 `492dc92343e774df4272e0e6a488e627` | 同值 | **字节恒等** |
| sanitizer-api | 全日志 md5 `3e2fc2eb4651a8801a4535088f4345f7` | 同值 | **字节恒等** |
| web-components | element-mutation 0P/**15F**；gethtml 3622P/0F；opt-in 9F/109P+Timeout | element-mutation **15P/0F**；gethtml 6393P/0F；opt-in 7F/108P+Timeout | **主簇收敛 +15P/−15F 零真实回归**（gethtml +2771P＝页预算截断抖动，F=0 双侧；opt-in −2F−1P＝whole-page Timeout 截断缺尾——solo 单页复跑恒等 k7 形 9F/109P 三缺面全数在场）；其余 460 页逐页恒等 |
| dom | **496F** | **496F** | **F-恒等**；Pass −499＝Range-mutations 双页截断抖动（dataChange −212/replaceData −287，dF=0，已知噪声族；本窗 replaceData 记 Timeout 系争抢慢化） |

WC 全套 v3 总计 17687P/1118F/20T（k7 20444P/1135F/20T；Pass 差全在
gethtml/opt-in/Range 三截断页，真实 delta 仅 element-mutation ±15）。
v3 前单页核验链（中间态收尾证据）：element-mutation 15P/0F、
null-registry-removal 1P/0F、initialize-upgrades 8P/4F＝k7、exceptions
2P/1F＝k7、Element-innerHTML 12 行逐面恒等 k7（simple HTML P 保留＋
beforebegin P 复原）。钉测 s10k 七代全绿 7 passed 0 failed（终态树）。

## 证据日志（同提交 add -f）

- dc2-s10k8-wc-post-knife8.log ＝ v3 终态 wc 腿
- dc2-s10k8-domparsing-post-knife8.log ＝ v3 终态 dp 腿（字节恒等 k7）
- dc2-s10k8-sanitizer-post-knife8.log ＝ v3 终态 san 腿（字节恒等 k7）
- dc2-s10k8-dom-post-knife8.log ＝ v3 终态 dom 腿
