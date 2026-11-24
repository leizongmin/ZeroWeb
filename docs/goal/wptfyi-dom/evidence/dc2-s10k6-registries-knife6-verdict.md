# DC-2 s10-registries 刀⑥（define() 参数校验矩阵＋cbi extends 排除集）判读收账（2026-11-24）

k5 收账后同轮续刀：define.html 22F（`define()` 参数校验矩阵）＋
define-customized-builtins 1F（bgsound extends）双双翻绿；接口页
CustomElementRegistry.html 8F→8P 同矩阵外溢翻面。

## 范围与根因（预研＋探针定谳）

**22F 矩阵**（define.html，探针页 zz-k6-probe.html 五迭代全绿后删）：
`_ceDefine` 现序 `validName → typeof ctor` 与 spec 相反且缺校验——①零参形
name=undefined 走 validName 错抛 SyntaxError（spec WebIDL 必选参数 TypeError）；
②arrow/method `typeof==='function'` 但 IsConstructor=false（现 typeof 门漏过，
名含空格等非法形时错抛 SyntaxError）；③prototype Get 无 rethrow、
Type(prototype) 非 Object 无检查（undefined〔bound 函数形〕/原始值直通注册）；
④四 lifecycle callback 无 Get rethrow、无 callable 校验（null/object/integer 直通）。

**bgsound 1F**（define-customized-builtins extends 段）：extends 校验查
`__zwHtmlTagIface`，bgsound 引擎 legacy 渲染表内（HTMLElement）——但 HTML spec
该名无已定义接口（HTMLUnknownElement 形，Chromium 同形），不可作 customized
built-in 宿主。blink/isindex/multicol/nextid/spacer 表外已正确拒。

## 修复面（单文件 part03.js 零 Rust 行为面）

1. **IsConstructor 门前置**（spec 步 1）：`typeof !== 'function'` TypeError 保留＋
   探针 `Reflect.construct(Object, [], ctor)`——被构造的是 Object（用户 ctor 代码
   零执行），newTarget 侧 IsConstructor=false 即 TypeError；prototype getter 抛错
   沿构造步 Get 显形＝spec 步 5 rethrow 同形（异常对象原样透传）。
2. **prototype/lifecycle 校验块**（spec 步 5–8，置于重复检查后、extends 前）：
   Get 透传 rethrow → Type(prototype) 必须 Object（ECMAScript Type 面函数亦
   Object——valid-custom-element-names 语料 `prototype=HTMLElement` 构造器形
   合法，首版 `typeof !== 'object'` 判据误拒该页 222 测试已修）→ 四 callback
   Get 透传 rethrow → 非 undefined 非 callable TypeError（null 亦 TypeError）。
3. **cbi extends 排除集**：`_zwCeCbiUnknownIfaceNames`（bgsound/blink/isindex/
   multicol/nextid/spacer）命中即 NotSupportedError；仅 CE 注册域，
   createElement legacy 面不动。
4. **arity 门双 facade**（WebIDL 必选参数）：主实例 `customElements.define` 与
   子 realm 工厂 `_zwMakeCERegistry().define` 各置 `arguments.length < 2 →
   TypeError`（CustomElementRegistry.prototype.define 转发经主 facade 单点）。

常驻钉测 **part152_s10k6_define_validation_pins.rs** 单针全矩阵（同提交）：
arity×2／IsConstructor arrow+method／序交叉（arrow＋非法名 TypeError）／
prototype=HTMLElement 合法／bound undefined＋原始值 TypeError／lifecycle
null+object TypeError＋getter rethrow 恒等／bgsound NotSupportedError／
define+createElement+append 升级面（ctor+connectedCallback 触发计数）。

## 字节绑定链

| 阶段 | md5 |
|---|---|
| 终态 runner（电池判定二进制） | `c1bbef0f9955b2ae2da032192ec96e04` |
| part03.js（四针） | `57138b20ca41433a28a7c8ce54adb7b1` |
| part152 钉测 | `1a5ce31456feb44194deaa66d0966c66` |
| js_dom_bridge_tests.rs | `3ae4f43122b66816d13de527e95fa9d0` |
| 电池发射器（.acceptance/dps 本地） | `3d29896c6c348464623de8cc9923e940` |

前置钉测：`cargo test -p zero-engine --lib s10k`＝part148/149/150/151/152
**五代全绿 5 passed 0 failed**。拼接 29 分片 node --check SYNTAX_OK
（9,976,177 字节）；fmt --all 零 diff；clippy -p zero-engine 唯一 warning＝
cssa_registry.rs:262 identical-if（3e8cdfc12 css 流域既有，零新 warning）。

中间迭代钉测事件（防重走）：钉测自身两处误用真构造器冒充 arrow（isctor-arrow
→no-throw 正确行为、isctor-order→SyntaxError 正确行为）＋upgrade 针漏
appendChild（connectedCallback 不触发）——引擎行为 runner 探针已证正确，修钉
测面非引擎面。首版 prototype Type 判据 `typeof !== 'object'` 误拒
valid-custom-element-names.html 222 测试（该页 `newClass.prototype =
HTMLElement` 构造器形），回归信号由簇 sweep 对 k5 锚逐页 diff 抓获，判据修正
为「object 或 function 皆过、null/undefined/原始值拒」。

## 电池判读（锚＝已推 刀⑤ [dc2-s10k5-wc-post-knife5.log](dc2-s10k5-wc-post-knife5.log)；窗 2026-11-24 08:20:46Z–08:34:12Z solo 零并发 cargo）

| 腿 | 刀⑥ 终态 | 对已推 刀⑤ | 判读 |
|---|---|---|---|
| testharness-web-components | P 20,570 / F 1,154 / T 20 | P −1 / F −30 | 双目标页精确兑现（define 22F→0F +22P、define-customized-builtins 1F→0F +1P）＋CustomElementRegistry.html 接口页 27F→19F/19P→27P＝**8 面同矩阵外溢翻面**（IsConstructor 序×2、prototype Get rethrow、prototype 非 Object、callback Get/转换 rethrow×2、名检查序——逐名 comm 对账全直属本刀校验面）＝gethtml −32P 页预算截断落位（F/T 恒等噪声带，非回归）＋zz-k6-probe +1F＝sentinel 探针页（gitignored 已删，不入 face）；逐页 fail/pass 集对账零附带损伤（其余页恒等） |
| testharness-domparsing | P 1,264 / F 271 / T 1 | 全恒等 | SET-IDENTICAL（0 differing keys，日志 md5 与 k5 逐字节恒等） |
| testharness-sanitizer-api | P 415 / F 341 / T 1 | 全恒等 | SET-IDENTICAL（日志 md5 与 k5 逐字节恒等） |
| testharness-dom | P 57,891 / F 496 / T 11 | P +22 / F 恒等 | Range-mutations-dataChange 1,157→1,179＝已知截断噪声带（±45P，F 恒等），非回归 |

判读口径：行为面真实增益＝**31 面**（define 22＋builtins 1＋接口页外溢 8，
全 F→P 翻面）；raw 口径 WC P 净 −1（+31−32 gethtml 截断）、dom +22 噪声不计。

## 证据日志（同提交 add -f）

| 日志 | md5 |
|---|---|
| [dc2-s10k6-wc-post-knife6.log](dc2-s10k6-wc-post-knife6.log) | `c2aa37fd7812f05f4fd0aa7dac1a293c` |
| [dc2-s10k6-domparsing-post-knife6.log](dc2-s10k6-domparsing-post-knife6.log) | `492dc92343e774df4272e0e6a488e627` |
| [dc2-s10k6-sanitizer-post-knife6.log](dc2-s10k6-sanitizer-post-knife6.log) | `3e2fc2eb4651a8801a4535088f4345f7` |
| [dc2-s10k6-dom-post-knife6.log](dc2-s10k6-dom-post-knife6.log) | `fc73790735e501e331a22ae2c6e1cd4e` |

电池窗：WC 5m06s / DP 12s / SAN 17s / DOM 7m51s（08:20:46Z–08:34:12Z），四腿
rc=1 仅留痕（F 存量），腿门＝tally join 零漂移。

## 残面（下刀挂账，序沿 master.md 下一步不变）

- CustomElementRegistry.html 残 19F：observedAttributes/disabledFeatures/
  formAssociated Get rethrow＋sequence 转换/迭代面（~12F）、element definition
  running flag 序面（~4F）、shadow-including 升级序/whenDefined promise 恒等
  （~3F）——observedAttributes 校验矩阵属本刀同族延伸，下刀候选。
- element-mutation 15F、R90 trap scoped 原型臂 instanceof、parse-time attr 区分
  〔live 读恒 parse 面 3 案反向〕、importNode/upgrade-order/criteria 页
  〔define-upgrade-criteria 页级 TO 停摆形在册〕、iframe 合成 doc 无 legacy
  `customElements` 读面、IAH handle-only——沿旧序。
- declarative-basic 2F／property-reflection 1F／html5lib-basics 3F（刀⑤ 残面
  承接）＋gethtml 页预算截断观察位。
