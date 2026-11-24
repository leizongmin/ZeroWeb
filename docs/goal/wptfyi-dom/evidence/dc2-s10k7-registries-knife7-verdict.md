# DC-2 s10-registries 刀⑦（define() spec 步序重构＋Get/转换 rethrow 矩阵＋running flag＋whenDefined 恒等＋shadow-including 升级序）判读收账（2026-11-24）

k6 收账后同轮续刀：CustomElementRegistry.html 19F（接口页 Get 序/转换/迭代/
running flag/升级序/whenDefined 族）收敛至 1F（残面＝iframe 文档内升级路由，
R364 在册后续片，见残面段）。

## 范围与根因（预研＋探针定谳）

**Get 序与 rethrow 矩阵**（探针页 zz-k7-probe.html 五迭代全绿后删）：
`_ceDefine` 旧形 disabledFeatures 走 swallow try/catch、observedAttributes 走
`try { void ctor.observedAttributes; } catch {}` 吞错热读、四 lifecycle callback
在 entry 后置读——与 spec define() 步 14 的 Get 链（prototype → 五 lifecycle
callback〔含 connectedMoveCallback〕→ observedAttributes 条件 Get〔仅
attributeChangedCallback 非 undefined〕→ disabledFeatures → formAssociated →
ToBoolean → 四 form callback）序与 rethrow 语义不符。sequence<DOMString> 转换
缺 iterable 门（Get(@@iterator) 非 callable TypeError）与迭代 rethrow。

**running flag**（spec 步 8-9，per-registry 单飞）：旧形无 flag——ctor prototype
Get 内嵌套同 registry define 不抛 NotSupportedError；finally 清旗缺位。

**whenDefined 恒等**（spec「resolve 已存在 promise」）：pending 表存裸
[resolve] 数组——重复 whenDefined 每次新建 promise，identity 断言失败；
reshape 为 {promise, resolvers} capability 后同 promise 复用。

**shadow-including 升级序**（spec「upgrade elements in the shadow-including
tree order」）：旧形候选枚举用 payload 扁平序——serializer 对 shadow host 用
replace-children 语义，host light child 不在其文档位（探针 r2 身份标记定谳：
childOfHost 落 after 之后）；解析树包装属 selector 世界（`__zwHandle` 缺席，
`_handleChildren` 镜像只在 handle 世界）——真树 walk 须走 `childNodes` live
视图（R140 融合视图，sel/handle 双世界通吃）。

**dup-name 零可观察 get**：刀⑥ 的 `Reflect.construct(Object, [], ctor)`
IsConstructor 探针经 newTarget 侧 Get(newTarget,'prototype') 触发 proxy get
trap（dup 名二次 define calls==[] 断言失败）——换 gOPD 探针（own
'prototype' descriptor 在即判可构造，零 trap；Reflect.construct 仅 fallback
arrow/method/bound 无 own prototype 形）。

## 修复面（单文件 part03.js 零 Rust 行为面）

1. **IsConstructor gOPD 探针**（spec 步 1）：`typeof !== 'function'` TypeError
   保留；gOPD(ctor,'prototype') 存在且 value 为 object/function 即判可构造；
   缺 own prototype（arrow/method/bound）走 Reflect.construct(Object,[],ctor)
   fallback（Object 构造零用户代码执行面）。
2. **spec 步序重构**：extends 校验（含 7.1 scoped registry → NotSupportedError）
   前置于 prototype Get；per-registry running flag（`{v}` 槽位对象）步 9 置
   true、try/finally 步 14 全链（prototype Get → Type 检查 → 五 lifecycle
   callback Get＋callable 校验 → observedAttributes 条件 Get＋sequence 转换
   （`_ceToDomStringSequence`：Get(@@iterator) rethrow/非 callable TypeError/
   迭代 rethrow/元素 String 转换）→ disabledFeatures Get＋转换 → formAssociated
   Get＋ToBoolean → 四 form callback 条件 Get＋callable 校验）后 finally 清旗。
3. **entry 字段扩展**：observedAttributes/formAssociated 入 entry（升级侧
   `_ceFireInitialAttrChanges` live 读面保持）。
4. **whenDefined capability reshape**：pending[name] = {promise, resolvers}；
   define 完成后逐 resolver 注入 ctor；重复 whenDefined 返同一 promise。
5. **shadow-including 候选枚举**（`_ceShadowWorldActive()`＝`_shadowRoots`
   非空门，零 shadow 页走 `_ceDefineCandidates` 既有快道零变化）：主序＝
   documentElement 起 `_ceShadowSubtreeWalk`（自身 → shadow 子树〔
   `__zwShadowRootForHost` handle + `_handleChildren[rootH]` 镜像，identity
   保持〕→ light 子〔childNodes live 视图〕）真 pre-order；payload 扁平列表
   降级兜底 sweep（seen 去重，镜像漏网按 payload 序追加，零漏面）；候选终序
   按 (stamp, nameLc) 缓存（`_ceShadowCandCache`，定义密集页 stamp 未变直取）。
   FIXME: cbi 分支（`_zwDocAllElements` 世界）shadow interleave 未接——语料
   order 页为 autonomous 形，随 cbi shadow 语料补。
6. **Element.prototype.moveBefore**（R5258 形，R194b own-method 偏好＋守卫
   `!('moveBefore' in Element.prototype)`）：参数校验序（param1 非 Node
   TypeError；param2 非 Node 非 null TypeError）＋ `__zwScMoveMode` 窗口＋
   委托 `this.insertBefore`——语料 interleave 断言
   `("moveBefore" in Element.prototype)` 为 TRUE 分支自洽世界（FALSE 分支
   constructorCalls 期望表缺 counter 5＝vestigial 不可解），五 callback Get
   列表唯一自洽。

常驻钉测 **part153_s10k7_registries_knife7_pins.rs** 单针八面（同提交）：
dup 名零 get／running flag NSE＋finally＋跨 registry 不互斥／ctor Get 序／
observedAttributes 四形／disabledFeatures＋formAssociated Get·转换·迭代·
九 lifecycle get·form callback 中止／whenDefined 恒等＋微任务 resolve（旗标
可观察，part14 pump 先例）／shadow-including 序（closed＋嵌套）／scoped
extends NSE。

## 字节绑定链

| 阶段 | md5 |
|---|---|
| 终态 runner（电池判定二进制） | `cfe8ae9ef412c9851cc4fd451bf0e7fd` |
| part03.js（六针） | `acf8ffe19d04856361d2afa1afc8989f` |
| part153 钉测 | `b8eadbe26bea38fd9bd061c443aeaf1e` |
| js_dom_bridge_tests.rs | `caf14d74681fc365da77e857f7836c3a` |
| 电池发射器（.acceptance/dps 本地） | `23c441a1ac203ec261d9cec1efbe04d3` |

前置钉测：`cargo test -p zero-engine --lib s10k`＝part148/149/150/151/152/153
**六代全绿 6 passed 0 failed**。fmt --all 零 diff；clippy -p zero-engine 唯一
warning＝cssa_registry.rs:262 identical-if（3e8cdfc12 css 流域既有，零新
warning）。

中间迭代事件（防重走）：①探针 dup-setup 误对首次 define 断 NSE（首 define 本
应成功）；②候选序首版用 payload 扁平序——探针 r2 身份标记实证 host light
child 落位错误；③镜像 walk 首版从 documentElement 走 `_handleChildren`——
r4 面包屑（`__k7walkLog`）实证 documentElement 无 `__zwHandle`（selector
世界），childNodes live 视图换源后 r5 全绿；④钉测三处 authored bug（attrCb
误挂 ctor 级 trap〔引擎从 prototype 读〕／disabledFeatures='shadow' 字符串
可迭代非 TypeError／`nm()` 自增致 whenDefined 名漂移）＋`String(promise)`
恒 "[object Promise]" 可观察面选错（改旗标）——引擎行为 runner 探针已证
正确，修钉测面非引擎面；⑤**首电池发射后单页抽查抓获 isScoped 误标回归**：
child factory（13281 `_zwMakeCERegistry`）define 无条件 `isScoped: true`，
iframe per-realm registry 的 cbi extends（pseudo-class-defined-customized-builtins
页 line 63 `w.customElements.define(..., {extends: tag_name})`）撞 spec 步
7.1 误抛 NSE→onload handler 死→打印面 16→4（wholesale 假败签名）——刀②
已建 `__zwCeIsScoped` 判据（仅 markActive 即显式 new 为 scoped，per-realm
iframe 实例非 scoped），child define 改消费同印记后该页与 k6 逐测试恒等
（逐名对账 same=16）；scoped 族页逐名对账 zero flip（initialize 双页混计
伪信号澄清）；主面 CER 复验 45P/1F 保持。终态电池以修复后二进制重发。

## 电池判读（锚＝已推 刀⑥ [dc2-s10k6-wc-post-knife6.log](dc2-s10k6-wc-post-knife6.log)；窗 2026-11-24 09:51:57Z–10:05:44Z solo 零并发 cargo）

| 腿 | 刀⑦ 终态 | 对已推 刀⑥ | 判读 |
|---|---|---|---|
| testharness-web-components | P 20,444 / F 1,135 / T 20 | P −126 / F −19 | **CER 19F→1F/27P→45P＝+18 面精确兑现**＋zz-k6-probe 残页 1F→0F（gitignored 刮擦页非 face）＋gethtml −144P 页预算截断落位（F/T 恒等，在册噪声族）＋其余页逐名恒等（pcd-cbi 页含）＝raw 口径 P −126 全由截断族构成，行为面真增益 **+18** |
| testharness-domparsing | P 1,264 / F 271 / T 1 | 全恒等 | SET-IDENTICAL（日志 md5 与 k6 逐字节恒等 492dc923…） |
| testharness-sanitizer-api | P 415 / F 341 / T 1 | 全恒等 | SET-IDENTICAL（日志 md5 与 k6 逐字节恒等 3e2fc2eb…） |
| testharness-dom | P 57,840 / F 496 / T 11 | P −51 / F·T 恒等 | Range-mutations 截断族 P-only 漂移（−51）；首电池 −731P/+1T → 终态电池 −51P/T 恒＝同族时序变差定谳（双采样零行为面） |

判读口径：行为面真实增益＝**18 面**（CustomElementRegistry.html 19F→1F，
唯一残 F＝跨全局 iframe 升级路由〔R364 后续片〕）；raw 口径 WC P 净 −126
（+18 − 144 gethtml 截断）、dom −51 截断不计。首电池（7a45326d…，isScoped
误标二进制）数据作废不采——其 WC +110P 形含 pcd-cbi 页 −12 打印面回归，
由单页抽查抓获并以终态二进制重发覆盖。

## 证据日志（同提交 add -f）

| 日志 | md5 |
|---|---|
| [dc2-s10k7-wc-post-knife7.log](dc2-s10k7-wc-post-knife7.log) | `feada23772041c2ea023f717ce661eea` |
| [dc2-s10k7-domparsing-post-knife7.log](dc2-s10k7-domparsing-post-knife7.log) | `492dc92343e774df4272e0e6a488e627` |
| [dc2-s10k7-sanitizer-post-knife7.log](dc2-s10k7-sanitizer-post-knife7.log) | `3e2fc2eb4651a8801a4535088f4345f7` |
| [dc2-s10k7-dom-post-knife7.log](dc2-s10k7-dom-post-knife7.log) | `687e2a4a30e626149f9cbbffdc5980aa` |

电池窗：WC 5m08s / DP 12s / SAN 17s / DOM 8m10s（09:51:57Z–10:05:44Z），四腿
rc=1 仅留痕（F 存量），腿门＝tally join 零漂移＋主面逐名对账。

## 残面（下刀挂账，序沿 master.md 下一步不变）

- CustomElementRegistry.html 残 1F：跨全局 iframe define-during-Get 的
  **iframe 文档内升级路由**（line 199 `iframe.contentDocument.createElement
  instanceof`——R364 在册「iframe 文档内升级路由 = 后续片」；child registry
  define 侧本刀已通，createElement 路由归「iframe 合成 doc」残面族）。
- element-mutation 15F、R90 trap scoped 原型臂 instanceof、parse-time attr
  区分〔live 读恒 parse 面 3 案反向〕、importNode/upgrade-order/criteria 页
  〔define-upgrade-criteria 页级 TO 停摆形在册〕、IAH handle-only、cbi shadow
  interleave（本刀 FIXME）——沿旧序。
- declarative-basic 2F／property-reflection 1F／html5lib-basics 3F（刀⑤
  残面承接）＋gethtml 页预算截断观察位。
