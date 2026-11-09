# M2/B6a — 采样引擎数值域（2026-11-07）

B6a 一件事：SMIL 值采样引擎数值域落地——calcMode 四模式（linear 缺省/
discrete/spline/paced）+ keyTimes/keySplines 文法校验 + additive/accumulate
三明治合成 + 长度文法单位上下文解析（em/ex/%/物理单位/calc()）+ 链模型
restart=always 中途截断 + freeze 迭代边界浮点钳制 + by-only replace
underlying 起点。全通道 + web-animations 双通道零回归复验落档。

## 1. 实现落点（全在 `part09.js`，js shim 编译期内嵌）

- **值表组装**（spec #AnimationFunctionAmtxValSyntax 优先序）：values
  （分号表，尾分号/尾空段忽略、中段空→整动画无效）> from-to > from-by >
  by-only > to；全部 token 经 `_lenUserUnits` 解析，任一不可解析 → 整
  动画无采样（无效值负向面定谳）。
- **`_lenUserUnits`**：px/user/物理单位表/em/ex（font-size 属性→
  computed style→16px 兜底）/%（最近祖先 svg 视口——viewBox → svg
  width/height 属性 → 300×150 缺省；HTML 内嵌 svg 时
  documentElement=documentElement html 非 svg，须 parentNode 链上溯，
  svglength-additive-by-7 定谳 5%×800=40）；数字与单位间空白不容
  （invalid-value 1/5-9 负向面）；**calc() 项级加减求值**（数×单位项
  序列，% 项按视口、物理单位按 px 因子逐项折算——by-8
  calc(4% + 8px)=0.04×800+8=40 定谳；乘除/嵌套 FIXME B6b）。
- **calcMode**：元素缺省（set→discrete、animateMotion→paced、余
  linear）；未知值→无采样（属性枚举封闭）。discrete 按 keyTimes 在场
  值 i 自 kt[i] 起持有/缺省均匀 n 段；spline 段内 `_bezSolve` 牛顿 8 轮
  + 二分 24 轮兜底；paced 忽略 keyTimes（paced-value-overwrites-keyTimes
  定谳）按累计 |Δ| 均匀。
- **keyTimes/keySplines 校验**（SMIL #KeyTimesAttributeSyntax /
  #KeySplinesAttributeSyntax）：keyTimes 项数=值数、首 0、linear/spline
  末 1（discrete 末不限——animate-from-to-keyTimes "0;0.25" 定谳）、非
  降序 ∈[0,1]；keySplines 组数=段数、每组 4 数、**四值全部 ∈[0,1]**
  （SMIL 文法全值域约束，与 CSS cubic-bezier 仅限 x 不同——
  keysplines-x/y-limits 双面定谳）、组内分隔 comma-wsp（逗号空白混用
  合法——animate-keySplines 语料「0 ,0  1 , 1」防回归修复）。违例→整
  动画无效。
- **additive/accumulate 三明治**（`_animatedValue` 文档序合成，spec
  #AdditiveAnimation sandwich model）：additive="sum" 累加（首个 sum 无
  前置 replace 时以 base 起底）、缺省 replace 重置；accumulate="sum"
  迭代传播 k×(last−first)；by-only replace 起点取三明治运行值
 （underlying）非裸属性 base（svglength-additive-by-2 定谳 75=an1 冻结
  50+by×0.5；FIXME：spec 取 interval 起点快照，现按采样时运行值近似——
  前置动画冻结/常值场景恒等，活跃前置待 B6b）。
- **链模型 restart=always 中途截断**（`_docChains` 时间序 pick 循环，
  置 whenNotActive 门后仅缺省通道触达）：新实例 b 落入已建活跃区间→
  旧区间截断至 b（SMIL #Timing-RestartMechanisms「restarted」——
  multiple-begin-additive 定谳面）。
- **freeze 迭代边界浮点钳制**：span≈k·sd 残差 <1e-9·sd 取整迭代末
 （frac=1——animate-end-attribute-numeric-precision 定谳：begin/dur 尾
  数浮点残差致 span%sd≈1e-16 冻结值塌向迭代起点）。
- 类型化域（color/transform/enum/string/lengthlist 等）= B6b FIXME，
  现势不采样无回归面。

## 2. 验证（test-guard 4G/8G 包裹，`--json` stdout 落档本目录）

### 2.1 svg 全通道（`2026-11-07-m2-b6a-sampling-channel.json`）

- **486 案 1,710 子：1,278P / 425F / 7T = 74.7%**（B5 1,239P/464F/7T =
  72.5%：**+39P / −39F**，Timeout 7 案恒等）。
- **零 P→F**（逐子 flip 矩阵核）。
- **F→P ×39（FIXED）分类**：
  - calcMode 采样 ×8：spline from-to/from-by/by/to/values + discrete
    ×2 + paced-overwrites-keyTimes；
  - keyTimes/keySplines 文法 ×3：keysplines-x-limits、keysplines-y-
    limits、animate-from-to-keyTimes；
  - additive/accumulate 三明治与 restart 截断 ×10：additive-from-to-
    width、additive-values-width、accumulate-values-width、
    multiple-begin-additive、svglength-additive-by-1/2/7/8、
    svglength-additive-from-by-1/3；
  - 数值文法与单位上下文 ×10：invalid-value 1/5/6/7/8/9、
    svglength-animation-px-to-ems、LengthModeWidth/Height、
    smil-values-empty-segments；
  - 时钟边界与连带 ×8：animate-end-attribute-numeric-precision、
    cyclic-syncbase、animate-elem-14/15/16/17/18/19-t-drt（渲染参照复
    制案动态采样连带）。
- 中途验证轮一次 P→F（animate-keySplines comma-wsp 误拒）当场修复，
  终轮 JSON 零 P→F 为准；animate-keySplines 本身在 B5 即 P（comma-wsp
  修复为防回归性质，不计入 FIXED）。

### 2.2 web-animations 依赖通道（划界约束）

`testharness-web-animations --json`（终版二进制，同 guard 口径，本目录
`2026-11-07-m2-b6a-web-animations.json`）：**506P/444F/9T = 959 子，对
B5 逐子恒等零 flip**。批内一轮该通道曾显 1 例 P→F
（DocumentTimeline/constructor origin-time 精度断言），隔离复跑 3 连
全 P 定谳为负载下时序 flaky（WAAPI 域，与本批 SMIL 改动无涉），以空闲
终捕为准。

## 3. B6a 批次记账（DC-3 FIXED/REGRESSED）

- **FIXED +39P**（§2.1 分类）。
- **REGRESSED 0**。
- M0 数值门进度：通道 P 1,278，对 DC-4 全量分母 15,342 = 8.33%（门
  8.6%，B5 时 8.08%）；余量由 B6b 类型域、B7 path d、滚动新面导入兑付。

## 4. 挂账门（本批不改，跨域记账）

- **B6b 类型域采样**：attributeTypes 族、svgenum/svginteger、
  LengthMode/unitType 面（JS harness 错域）、marker-orient 角度插值、
  color/transform/enum/string 值域、additive/non-additive-type-by 族
  （「all XML property types」面）。
- **B7 path d 插值**：animate-path-* 族 33F（tentative 含）——本批
  by/accumulate path 案仍 F 无回归。
- **#4 getComputedStyle SMIL 表现值门**：slider-switch 等五案（B5 §4
  在案）；animate-currentcolor 渲染交互面。
- **活跃前置动画 underlying 快照**：by-only replace 的 interval 起点语
  义（§1 FIXME，语料现无定谳面）。

## 5. 下一步（M2 续）

B6b 类型域采样（M0 账 animations 域次大簇：attributeTypes/color/
transform/marker-orient）→ B7 path d 插值（M3）+ 滚动新面导入 + 挂账
#2/#3 归口 → M4 coverage 复测收口（DC-4 svg topdir ≥ 8.6%）。
