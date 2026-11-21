# 刀 OM-18 · typed-om reify 路由/规整族 19 靶（2026-11-22）

## 认领与恰集

- 池＝OM-16 电池 b1r css-typed-om 残池 post-OM-17 余量
  （results-knife-om17-census/census.json 全靶状态 12406 entries）＋探针
  zz-om18probe.html P1-P21（b1r-runner 秒级复跑，gitignored corpus 不入库）。
- 恰集 **19F/6 页**（NOTES-knife-om18-reify-claim.md 预注册）：atf/ttf step
  4F＋clip-path box keywords 7F＋column-rule-width 6F（-3.14em 1＋
  does-not-support 5）＋offset-rotate 2F（reverse 基类＋3.14rad→deg）。
- 挂账不入：scroll-snap-align 4F（结构性不可赢，见下）；clip unsupported
  1F（style-system validate 白名单缺 clip inset 形，跨 crate 另刀）；
  engine-lane 两笔（clip-path gCS 双叠序列化 bug 本体、ttf declared 序列化
  canonicalize bug 本体——face 侧规整绕过）。

## 探针定谳（P1-P21）

1. **裸引用调用形＝结构性不可赢（P11/P13）**：scroll-snap-align.html
   `computed: assert_is_unsupported` 裸函数引用全仓唯一（grep 实证；
   border-radius/border-image 族全为箭头双参形）。testsuite.js:365 以
   `(computed)(example.input, computedResult)` 调用——单参形参绑定
   example.input，断言落输入值内部类：keyword input 恒 [object CSSKeywordValue]
   （P13 protoTag 实证、P11 helper 对 input 抛错）。实现侧无解。
2. **clip-path computed 双叠（P14/P15）**：引擎 gCS 对 box keywords 输出
   'fill-box fill-box'（decl 单倍）——序列化 bug 本体 engine-lane；parse
   不命中表回 opaque → computed 面基类。
3. **crw 写者路由（P8/P9）**：IDL 置 'thin' 后 get=CSSKeywordValue（does-not-
   support 臂期望基类）；同文本 keyword 臂期望 CSSKeywordValue——写者身份
   不可从声明文本判别。border-image-repeat 页绿之因＝unsupported 列表全是
   多 token 串（'stretch repeat'）parse 自然 opaque，无路由需求；crw 5 靶
   全单词条且表可 parse＝路由标记唯一消费者。
4. **offset-rotate（P3/P4/P16）**：reverse computed 被既有 keyword 臂错误
   reify CSSKeywordValue（页箭头双参形断言落计算 face 期望基类）；3.14rad
   computed 透传 rad 原单位（期望 input.to('deg')）。
5. **timing（P17/P20/P21）**：atf 引擎保 'step-start' 原文（parse keyword →
   computed 臂 canonicalize 缺）；**ttf 引擎 CSSOM 写把 step-start/step-end
   归一 'steps(1, start)'/'steps(1)' 落 declared 文本**（atf 不归一——两属性
   序列化不一致即 bug 面；双路由双属性四探实证）→ parse 函数形不在 keyword
   表回 opaque → specified 默认 equal 先红（computed toString 臂未执行到）。

## face（part10-typed-om.js 单件 +127/−2，patch md5 `dd561fe02cc74b468494c980a8bb00cc`）

1. **写者路由标记**：`_zwOmMakeStyleMap(readable, getStyleObj, markHost)`
   第三参；仅 attributeStyleMap getter 传 element（SameObject `__zwAttrStyleMap`
   缓存配 element expando `__zwOmRouteMarks`；rule.styleMap/computedStyleMap
   两参不接）。mapSet 成功路径落 `{text, opaque}`（opaque 判定＝单值输入
   `_zwOmKind()==='opaque'`）；delete/clear 清；mapGetAll 对 column-rule-width
   declared 面：标记失配/opaque 标记 → 基类 opaque 回落（css-wide/var 文本
   排除）；命中非 opaque → 既有表 parse。近似边界：CSSOM 写同文本残留标记
   误命中——语料每例 fresh element 无此序列（OM-17 读回等式近似门同型）。
2. **crw StoreUnchecked slot**：OM-17 Arm A 读回对账直推通道复用（-3.14em
   拒收签名直推；Sum 包裹 `_zwOmNonNegativeProps` 已含）。
3. **canonicalize 三臂**（computed 面专属）：
   - clip-path opaque 双叠 `^(\S+) \1$` 且首词在表 → CSSKeywordValue(首词)；
   - offset-rotate：'reverse' keyword → 基类 opaque（toString 'reverse'）；
     Unit rad → `_zwOmNumClean(v * (180/Math.PI))` deg、turn → ×360 deg
     （与 CSSNumericValue.to 因子表 rad=['angle',180/Math.PI] deg=['angle',1]
     同式 bit-equal）；'auto' 不动（页 try/catch 自适应臂两形皆绿）；
   - atf/ttf 'step-start'/'step-end' keyword → CSSKeywordValue('steps(1,
     start)'/'steps(1)')（toString 断言类不敏感）。
4. **ttf declared 反查**（v2 增量）：segs 循环 parse 后，transition-timing-
   function opaque 段精确两串 'steps(1, start)'→CSSKeywordValue('step-start')、
   'steps(1)'→CSSKeywordValue('step-end')；computed 面随后经 canonicalize 臂
   折回 canonical 形——specified==input 与 computed toString 双面闭合。其余
   steps 形（'steps(4, end)'）不匹配精确串保持 opaque。

## 电池（承 knife-om17 v2c 配方）

b0r（HEAD 247d1b0ff 基线剥除 patch——发射时 HEAD=6dae130a2，收账前
pull --rebase 已并入远端 8 提交，face 文件无冲突面）/b1r（face 重放）双臂，
patch md5 断言＋双向校验＋trap 归一＋test-guard 包裹＋M2 异 md5＋
FACE-MARKER 双 token（`_zwOmRouteMarkSet`/`__zwOmRouteMarks` b0r=0/b1r>0，
OM-17 tokens 已入库不可复用）。腿 {css-typed-om 主靶, css-values,
css-sizing, cssom} × 双臂；judge：19 靶恰集 F2P＋GRAND P2F=0＋new/gone=0
＋恰集外 F2P 出列；judge v2 排除域＝zz-* 探针诊断页（v1 P2F 污染源——
zz-om17probe probe D 断言随 face 演进过时）＋css-values random flake 靶
（doc_seed 进程地址采样，已沉淀 docs/learnings/bugs/2026-11/
2026-11-22-random-docseed-process-lottery.md）。

- **v1**（battery-r1.log 01:40）GATE FAIL：TARGET-GAIN 17/19——atf 2＋
  clip-path 7＋crw 6＋offset-rotate 2 全翻，恰集未翻＝ttf step-start/step-end
  两靶（declared 反查臂缺）。P2F=2 全数定谳非恰集回归（zz-om17probe 探针
  过时断言＋random flake）。NON-TARGET-F2P 1＝探针页 clip-path 同噪声。
- **v2 face 收窄**：declared 反查臂如上；恰集 19F 不变；judge 排除域增补。
- **发射链教训**：对端 ZeroWeb-2 r3 逐页批长尾占用 preflight（40 tick 上限
  ABORT 循环两次）；load 1.95 轻负载窗＋对端单页串行＋主靶轻页语料裁决转
  直发（flock＋patch md5 自守卫保留），P2F 假红经 FAIL 分支可 solo 复跑
  仲裁、不会假 PASS。发射器 python replace 断言失败后 launcher 仍执行的
  同型错误三犯——replace 断言失败必须中止整链（已在本节记档）。

## 判读（v3b 电池 02:31 PREFLIGHT-PROBE busy=5 load 1.95 → 02:50 GATE PASS rc=0）

二进制绑定 b0r=`1bfdd934f20aad8530a06e07e3852a93` / b1r=
`57f12d4570f093bedcff802d2d35975b`（M2 异 md5，md5sum 见 battery-r3.log）：

- [css-typed-om] 12475 keys **P 12386→12405（+19）**：F2P=19 P2F=0 new=0
  gone=0——19 靶恰集全翻，零恰集外增益；
- [css-values] 1385 keys SET-IDENTICAL（P 985→985 b1 域内 flake 靶入排除
  域，excluded=3 全局）；
- [css-sizing] 5700 keys SET-IDENTICAL——尺寸族/路由标记零外溢；
- [cssom] 1630 keys SET-IDENTICAL——CSSOM setProperty/IDL 写面零漂移；
- GRAND F2P=19 P2F=0 new=0 gone=0 excluded=3；TARGET-GAIN om18-reify **19/19**。

## 预注册对账

- 19 靶全 F2P ✓；GRAND P2F=0 ✓；new=0/gone=0 ✓；三哨恒同 ✓；
  FACE-MARKER 双 token 增量兑现 ✓；恰集外无增益（v1 探针页噪声入排除域）。

## 工件（本 clone .acceptance/wptfyi-css/，未入库）

v1/v2 patch（knife-om18-reify.patch 现 v2 态）、电池脚本
knife-om18-reify-battery.sh、judge knife-om18-reify-judge.py、认领 NOTES-
knife-om18-reify-claim.md（v1 判读与 v2 收窄增补）、results-knife-om18-
reify/ 双臂 JSON、电池日志 battery-r1/r2/r3.log、探针 zz-om18probe.html
（gitignored corpus，P1-P21 诊断）。

## 残池（typed-om 通道后续候选另刀）

clip unsupported 1F（style-system validate 白名单 clip inset 形）、engine-
lane：clip-path gCS 双叠序列化＋ttf declared 序列化 canonicalize 本体修复、
get-auto-min-size 6F、transform/序列化/IDL 族维持前账；scroll-snap-align
4F 结构性不可赢已闭环挂账（防后续轮重复消耗）。
