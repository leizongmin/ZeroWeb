# 刀 CSINTERP-KA：sizing 族动画 keyword round-trip＋max-* computed none 归一

日期：2026-11-21
face：crates/engine/src/animation.rs（apply_single_property width/height/min/max
六臂 keyword round-trip＋`parse_sizing_anim_value` 单源 helper＋清除 parse_px
无条件 Px(0) 形变）＋ crates/engine/src/js_dom_bridge/computed_style.rs
（serialize 侧 max-width/max-height 槽内 Auto→`Px(INFINITY)` 归一
`max_size_computed_norm`，串化经 sizing_length_to_css 出 "none"）
判别：FACE-MARKER `KA-FACE csinterp-keyword-roundtrip`（b0=0/b1=1）＋M2（bin
md5 异）＋S1/S3 ride census（RIDE 2 docs 文件零扰动）
电池：`.acceptance/wptfyi-css/knife-csinterp-ka-battery.sh`（launcher
knife-csinterp-ka-launcher.sh，systemd zw-knife-csinterp-ka-launcher-2020）；
语料 values/transitions/sizing/flexbox/typed-om 五套件 ×双臂
bin：b0 `cd7e1e9d5b32191391f004b173328b71`（20:18:19 build）/b1
`39caef1f1f85827f4780e26aba28b502`（20:19:08 build）；PATCH-MD5
`c34d2a0057a6e16eaf27c77e825ec5b9` 绑定
交叉验证：KA b0-runner md5 ＝ CLIENTWIDTH-PADBOX v2 b1-runner md5
（cd7e1e9d…）——两独立构建内容一致（v2 b1＝part04.js face 落树态、KA b0＝
该 face 已入库 HEAD＋同批 ride），基线同源互证

## 机制（规范锚）

- css-sizing-3 §5.1/§5.2：`auto`（min-*）/`none`（max-*）计算值 keyword 需在
  动画 apply 回程保形。旧径 `parse_px(value)` 无条件 `Px(0)`（width/height 臂）、
  `parse_px_strict` 失败即弃（min/max 臂）→ canonicalize None → consult 回落
  cascaded 终值（echo to 值）→ discrete/keyword 臂全页成立 `expected "none "
  but got "20px "`。
- 修复：keyword 端点 round-trip——`none`→`Px(INFINITY)`（仅 max-*，none 槽
  表示与 gCS `sizing_length_to_css` 无穷臂同口径）、max-* `auto` 拒收保级联
  （css-sizing §5.2 max 无 auto 计算值）、其余 keyword/长度走 css-parser
  `parse_length` 单源（R4086 stretch/R1018 bare fit-content/裸零在库）；
  解析失败保级联（不再落 0px 污染 gCS echo）。
- serialize 侧：max-* 槽内 `Auto` 变体（级联 auto 声明写入径）computed 语境
  归一 `none`（§5.2）；min-* auto 保 `"auto"`（§5.1）。

## 预注册（电池头）

- 直接靶：六 interpolate-size-{height,width,max-height,max-width,min-height,
  min-width}-interpolation.html allow-discrete/keyword 族 60F（9/15/12/12/6/6）
  ＋interp 块 keyword 臂（register 早退 canonicalize None 病灶同修）；
  which-value 6F 观察位逐案归因（判别消费＝G1a/S2-symbolic 域，本刀不预期
  翻）；interpolate-size-computed 2F 不预期翻（G1a 域）。
- 数值臂（used px lerp＝K-B 域）与符号臂（min-* calc-size 串捕获＝K-C 域）
  本刀不预期翻；composition 五页 130F（S3 域）不预期翻；logical-properties
  144F（S1b 域）不预期翻。
- 哨兵：css-transitions 零 P2F（keyword 径只影响 keyword 值）；css-sizing/
  css-flexbox＝Px(0) 清除审计窗（auto→animated→清除后回 auto 不得落 0px，
  任何 P2F 逐案归因）；css-typed-om SET-IDENTICAL（face 不触 shim）。
- b0 census：values 6317P/3549F/256 页；transitions 1054P/478F/126 页；
  sizing 3149P/2560F/168 页；flexbox 1429P/3284F/369 页；typed-om
  12280P/126F/359 页。六页 fail：height 191/width 183/max-h 97/max-w 97/
  min-h 118/min-w 118；which-value 6F；computed 2F。

## 电池结果（v1 fail-stop：P2F>0 不入账）

**判定：FAIL（净 +425/−173 页级；case 级 F2P=565/P2F=313，页级
gained/lost 对冲掩盖 P2F 池）——v1 face 不入库，按收账纪律 P2F>0 直接
归因后收窄重发。**

- values corpus case 级（b0 6317P → b1，20:38 落盘）：
  - F2P=230/P2F=203；页级 changed 13 页。gains：max-h/max-w interpolation
    +47/+47（none 臂 25×2＋keyword 臂 31×2 discrete 族）＋四 composition 页
    +14（全 keyword round-trip 臂，cssa registry add 组合径，零 P2F）＋
    min-h/min-w interp +26/+22（keyword discrete 族）；calc-size-width
    +38（calc-size(auto,…) 臂诚实化）。
  - P2F 池三species（逐案归因）：
    1. **width/height 槽 keyword echo 破 used-size 巧合**（calc-size-height
       53＋calc-size-width 46＋height 页 24＋width 页 13＋which-value 1）：
       b0 `parse_px` 无条件 Px(0) 恰合 harness expected 侧 used-size 读回
       "0px"（空元素 auto=0）；KA 保 keyword → 动画槽回显裸 keyword
       （"auto"/"min-content"/"max-content"）≠ used-size 期望。修需动画槽
       used-px echo（＝K-B/G1c 机器，未建）。
    2. **min/max interp 臂 calc-size 符号期望**（max-h 9＋max-w 9＋min-h 24
       ＋min-w 24，"should be [calc-size(...)]"）：b0 经 s2b slot 串形巧合
       过；KA round-trip 以 LengthValue 变体覆写 max_*/min_* 字段、clobber
       s2b CalcSizeSlots 符号存储链 → 期望 calc-size(...) 臂翻红。修＝
       keyword 动画值路由 calc-size slot 机器（＝K-C/S2-symbolic 域）。
    3. neutral/px 端点 interp 臂连带（同 2 池）。
- 裁决要点：v1 的 60F 直接靶（allow-discrete 族）确实翻面（max-h/max-w
  discrete 族为主），但 keyword round-trip 的 P2F 代价（203 raw）不可入账；
  双错相消揭穿型——b0 的 Px(0)/slot 巧合池被诚实化揭穿，与
  CLIENTWIDTH-PADBOX v1→v2 同构（learnings pair-cancelled-bugs 复用）。
- 后续：KA v2 收窄重设计需先做 arm-species 映射（discrete/interp/composition
  ×none/keyword ×registry 类型），或与 K-B（used-px echo）/K-C（calc-size
  slot 插值）合刀排期；v1 数据留档为三池基准。
- transitions 腿（20:46:14 b1 落盘）：**case 级 F2P=0/P2F=0 全 SET-IDENTICAL**
  ＝「css-transitions 零 P2F」红线保持（keyword apply 径对该 corpus 零扰动）。
- sizing 腿（20:47:58 b1 落盘）：F2P=335/P2F=110（净 +315/−90）。F2P 池
  ＝普通 max/min 页 294（keyword echo 诚实化增益，零对冲）＋其余 41；
  P2F 池＝height-interpolation 78、width-interpolation 27、
  height-composition 5（物种 1/2 同 values 池）。
- flexbox 腿（20:49:23 b1 落盘）：**F2P=0/P2F=0 全 SET-IDENTICAL**＝
  Px(0) 清除审计窗零扰动（face 不触 flexbox 轴）。
- typed-om 腿（20:50:28 b1 落盘）：**F2P=0/P2F=0 全 SET-IDENTICAL**
  ＝哨兵保持（face 不触 shim）。
- GRAND（judge-knife-casediff 全五套件）：gained=425 lost=173
  F2P=565 P2F=313 ASYM=2（calc-size 双页断言数漂移，与 keyword 臂诚实化
  同源非执行截断）。电池 rc=0，RECONCILE 输出与 judge 全一致。
- **at(0) 端点直接实证**（内置 RECONCILE 补充）：`CSS Animations from
  [fit-content] to [calc-size(50px, size)] at (0) should be [fit-content]`
  翻红——at(0) 即 from 端点回显，keyword round-trip 保形不完整
  （动画槽回读形 ≠ 静态侧 used-size echo 形），v1 的「端点保形」设计
  目标本身未达成，进一步支持 fail-stop 不入库。
- 收尾：KA face revert（两文件 checkout，`git apply --check -R` 已预验
  干净）；results 目录留档 `results-knife-csinterp-ka/`；v2 收窄设计
  待 K-B（used-px echo）/K-C（calc-size slot 路由）定排期后合成。
