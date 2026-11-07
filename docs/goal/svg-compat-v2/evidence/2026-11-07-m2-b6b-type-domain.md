# M2/B6b — 采样引擎类型域（2026-11-07）

B6b 一件事：SMIL 值采样引擎类型域落地——`_sampleAnimStr` token 采样
（数值路径零触碰）+ animVal 反射消费面接线（enum/bool/integer/string/
angle/list/rect 六族）+ 列表域 from-by/by-only 合成 + invalid-value 负
向面 + additive=sum 三明治。全通道 + web-animations 双通道零回归复验
落档。

## 1. 实现落点

### 1.1 part09.js（token 采样器，数值路径 `_sampleAnim` 零触碰）

- **`_sampleAnimStr`**：attributeName 匹配 → interval/frac/iter 解析
  （同数值域含 freeze 迭代边界浮点钳制）→ raw token 值表 → calcMode
  （linear 缺省/discrete/spline/paced）→ kt/ks 校验（同数值域文法）→
  段定位 → 域分类输出：
  - ②角度域：度数域插值 + 段端点单位回写（SVGAngle.value 恒为度数；
    animVal 单位段内半程前取 from、后半程取 to——unitType 面 deg→200grad
    t=2.0 DEG / t=3.999 GRAD 定谳；90 度按 grad 表达写 100grad）。
  - ③数值列表域：逐项 user units 折算（`_zwListNumItems`/`_parseLenToken`
    px/cm/mm/in/pt/pc/q）+ 段线性；任一端列表形态（多 token）而任一项不
    可解析 → 整动画无效（SMIL invalid-value 负向面，不落 discrete 兜底）；
    to 端项数多于 from → 常值 to 列表不插值（svglengthlist-3 定谳）；
    to 端更少按 from tile 逐项插值（svgnumberoptionalnumber-3 面）。
  - ④不可插值（enum/bool/string/URI）：SMIL #InterpolationDefaults 离散
    语义整值表持有；无 keyTimes 均匀段边界取严格 >（harness setCurrentTime
    精确 seek 半程——svgboolean-1 t=2.0 归前段 false 定谳，ceil-1 与
    WebKit 虚拟 keyTimes 语义一致）。
  - **from-by/by-only 合成**：`_zwListSumTokens` 逐项 user units 求和
    （SMIL #FromByAnimation f(t) = from + by·p——svglengthlist-4/
    svgpointlist-2/svgrect-2 面）；by-only 起点取目标属性现值。
  - **to-only**：起点取目标属性现值，属性缺席保 null（采样返 null 时消
    费面回落 base 反射即类型缺省值——svgboolean preserveAlpha 缺席面）。
- **`_zwSmilAnimToken` 导出**（globalThis 契约）：文档序动画元素最后非
  空胜出（replace）；**additive=sum 三明治**逐项 user units base+采样
  delta（svglengthlist-5 面）；fast-path tickEverRan/smilSeen 与数值域
  同口径。
- **`_chainOf` 跨族 wrapper 键回落**：querySelector 产物根 svg 的
  children 是 sel 族 wrapper（'#id\0\0smil'）、scan 产物是 handle 族
  （'null\0__n0\0smil'），两侧 key 不同源致链查落空——svgrect viewBox
  根目标面探针实证（挂账 #32 邻接）。链 rec 存有元素 id 属性：id 非空
  时按 id 对齐（语料动画元素恒带 id——runAnimationTest 按
  getElementById 寻元素）。数值路径同享。

### 1.2 part03.js（animVal 反射消费接线，八处 + 一新分支）

- **get-trap target 分支**（SVGAElement，svg ns a 专用）：get trap 中间
  分支直答 + SameObject per-key 缓存（口径同 className 分支）——原型侧
  accessor 对 proxy 不达（R93 尾部回落前被通用反射分支先吞，同
  part03 toString 探针注记口径，先行落码的 prototype accessor 版本
  探针实证无效后改道）。animVal 经 SMIL token 钩子。
- **AnimatedNumber**：baseVal read() 多 token 拆项（X/Y 尾缀 + 单值
  tile 镜像——number-optional-number 单值双映 spec 语义）；animVal token
  回落拆 X/Y 项（M1 面已有，本轮补 baseVal 侧）。
- **AnimatedInteger**：read/readAnim 同款拆项+tile 镜像（order 数值对
  orderX/orderY 映射同名内容属性——svginteger-2 baseVal/animVal 面）。
- **AnimatedEnum/Bool**：readAnim token 关键词表直查/直比，null 回落。
- **SVGAngle**：readOnly（anim 实例）分支 token 先行解析。
- **SVGRect**：anim 实例 token 置 raw（viewBox 四元组插值——svgrect-1/2
  面）。
- **SvgList**：parse() readOnly 分支 token 先行（lengthlist/numberlist/
  pointlist/stringlist 全覆盖）。

## 2. 验证（test-guard 4G/8G 包裹，`--json` stdout 落档本目录）

### 2.1 svg 全通道（`2026-11-07-m2-b6b-type-domain-channel.json`）

- **486 案 1,710 子：1,319P / 384F / 7T = 77.1%**（B6a 1,278P/425F/7T
  = 74.7%：**+41P / −41F**，Timeout 7 案恒等）。
- **零 P→F**（逐子 flip 矩阵核；中途两轮 P→F 当场修复后终捕：列表
  invalid-value 负向面 9 案 + initial-values tile 镜像 10+ 案 + numopt-4
  by 面）。
- **F→P ×41（FIXED）分类**：
  - svgenum ×13（全数转 P——含 5 值 values 表离散均匀持有）；
  - svgangle/marker-orient 角度域 ×11（跨单位四案 + unitType +
    from-angle/to-angle/auto 族——单位回写 + to-only 缺省 0）；
  - 列表域 ×11（lengthlist ×5、numberlist ×2、pointlist ×2、rect ×2——
    单位折算 + from-by 合成 + 数对不齐规则 + additive sum）；
  - numopt ×2、integer ×2、bool ×1、string ×1（拆项 tile 镜像 + to-only
    缺省回落 + 离散边界 + get-trap target 接线）。

### 2.2 web-animations 依赖通道（划界约束）

`testharness-web-animations --json`（终版二进制，同 guard 口径，本目录
`2026-11-07-m2-b6b-type-domain-web-animations.json`）：**506P/444F/9T
逐子恒等零 flip**。

## 3. B6b 批次记账（DC-3 FIXED/REGRESSED）

- **FIXED +41P**（§2.1 分类）。
- **REGRESSED 0**。
- M0 数值门进度：通道 P 1,319，对 DC-4 全量分母 15,342 = 8.60%（门
  8.6%——严格值 8.598% 差约 1 子，余量由 B7 path d 与滚动新面兑付）。

## 4. 挂账门（本批不改，跨域记账）

- **B6c/animateTransform 域**：svgtransform ×2、additive/non-additive-
  type-by ×3（「all XML property types」面）——SVGAnimatedTransformList
  消费面未实现（non-additive-type-from-by 探针 `undefined.animVal` 实
  证），矩阵插值/accumulate 叠加独立批。
- **#4 getComputedStyle SMIL 表现值门**：slider-switch 等五案（B5 §4
  在案）；svgstring fallback-to-discrete 同门。
- **animateMotion ×25**：getCTM 缺失（挂账 #2 邻接）。
- **path d 插值 ×31**：B7/M3。
- **composition 族 helper ×8**：runner inline extras 缺
  test_composition（查 SVGAnimationTestCase 上游新版本）。
- **活跃前置动画 underlying 快照**：B6a §4 在案。

## 5. 下一步（M2 续 → M3）

B7 path d 插值（M3）+ 滚动新面导入 + 挂账 #2/#3 归口 → M4 coverage 复
测收口（DC-4 svg topdir ≥ 8.6% 严格达线）。
