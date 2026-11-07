# M2/B6c — animateTransform 域 + href 显式定靶（2026-11-07）

B6c 一件事：animateTransform 域采样落地 + SVGAnimatedTransformList.animVal
消费接线 + href 显式定靶面（三消费钩子 children 隐式定位之外的 SMIL
#TargetElement 面，探针逐层定位 additive-type-by -45 回落 base 根因）+
连带补缺（SVGFilterPrimitiveElement.result、SVFFEConvolveMatrixElement.
kernelMatrix 反射，SVGTransform.angle 原始角度态，by-only 角度域合成）。
part09 超 2000 行准则切分 part09b。全通道 + web-animations 双通道零回归
落档。

## 1. 实现落点

### 1.1 part09.js → part09.js + part09b.js（2000 行准则切分）

- 切分：1299/1300 行界字节精确切分（R2963 先例）——part09 1,303 行
  （时间线状态/clock 文法/interval 链/数值域 B6a/类型域 B6b 采样器），
  part09b 789 行（B6c transform 域 + 三消费钩子 + 事件泵 + 元素接口）。
  同一自持 IIFE 续段（part09 开 `(function(){`、part09b 收 `})();`），
  桥接层 part09 后紧接 include——拼接字节序零变化。
- **`_sampleAnimStr` by-only 角度域合成回落**（前半）：`_zwListSumTokens`
  不达单位化角度 token（-45deg）时 `_zwAngleDegOf` 度数合成段端点
  （SMIL #FromByAnimation——additive-type-by orient by-only -45deg +
  45deg·p 定谳，插值共用②角度域路径）。
- **`_smilPhase`**：active 简单时间 repeat 取模 / freeze 冻结 activeEnd
  浮点钳制（同数值域精度口径；仅 B6c 新采样器用，前两采样器路径不回改）。
- **`_zwTrVecOf`/`_zwTrStateOf`**：type 域组件向量（translate/scale 1-2、
  rotate 1-3、skewX/skewY 1；分量越界 → null）→ {type,a..f,ang} 矩阵态；
  rotate 中心点 e = cx(1−cos)+cy·sin。
- **`_sampleAnimTransform`**：attributeName 匹配 + animatetransform 门 +
  type 解析（缺省 translate SVG1.1、未知无采样）+ values/from-to/
  from-by/by-only（零向量起点——SMIL #AnimateTransformElement by 语义，
  additive-type-by 定谳 rotate(45) base 上 by="-45" 采样 0→−45 不基于
  base）+ calcMode（linear 缺省/discrete/spline；paced 对 transform 未
  定义回落 linear FIXME M3）+ keyTimes/keySplines + accumulate 迭代传播
  k×(last−first) 逐分量。to-only/set 元素无语料定谳面 → 无采样 FIXME M3。
- **`_zwSmilAnimTransform` 钩子**（globalThis 契约）：animVal 表项级
  三明治——replace 重置 items / additive=sum 追加；clearBase =
  attr!=='transform'（gradientTransform/patternTransform replace 清 base
  ——animate-gradient-transform 定谳 animVal 单项；transform 属性 replace
  恒与 base 复合——animVal = base 项 + 动画项 = 2 项，additive-type-by
  定谳）。
- **href 显式定靶面（本批主修）**：三消费钩子（`_animatedValue` 数值 /
  `_zwSmilAnimToken` 类型域 / `_zwSmilAnimTransform` transform）原只遍历
  目标元素 children（隐式父级定位）——additive-type-by 等 19 案动画元素
  挂 svg 根用 `xlink:href="#id"` 指 defs 内目标，钩子不可见恒回落 base。
  新增 `_zwAnimHrefOf`（href 优先、xlink:href 回落——markup 冒号名归
  local、setAttributeNS 字面量保 qname，两名并查）+ `_zwHrefAnimList`
  （`_scanDoc` 全文档动画元素按 `#id` 对齐 el id；无 id 靶不可被指 →
  零扫描回落）。children pass 跳过带 href 的动画元素（显式指靶优先 +
  additive=sum 双计防护），href pass 后跑覆盖（两类定位语料不混用同属性
  ——文档序近似 FIXME M3）。fast-path：tickEverRan&&!smilSeen 同口径跳过
  全文档扫描（非 SMIL 文档零开销）。读时解析——change-target-while-
  animating setAttributeNS 改靶即时生效定谳。

### 1.2 part03.js（接线 + 补缺）

- **SVGAnimatedTransformList.animVal parse** readOnly 分支查
  `_zwSmilAnimTransform`（动画在册：clearBase 时空表 / 否则 base 条目取
  `__zwTransformState` 副本 + 动画项追加）。
- **SVGTransform 角度态**：`_zwSvgTransformParse` rotate/skewX/skewY 存
  原始角度 `ang`（矩阵反解 atan2 丢 0-360 环绕——rotate(200)→−160）+
  `setAll` 回落计算 + `angle` getter + setRotate/setSkewX/setSkewY 存取。
- **SVGFilterPrimitiveElement `result`** 反射（SVGAnimatedString；animVal
  经 token 优先 null 回落 base）——non-additive-type-by/from-by 探针
  `undefined.animVal` 实证缺补。
- **SVFFEConvolveMatrixElement `kernelMatrix`** 反射（SVGAnimatedNumberList）
  ——additive-type-by kernelMatrix.animVal 面（divisor/order/targetX/
  kernelUnitLengthX/Y 既有反射零改动）。

## 2. 验证（test-guard 4G/8G 包裹，`--json` stdout 落档本目录）

### 2.1 svg 全通道（`2026-11-07-m2-b6c-transform-domain-channel.json`）

- **486 案 1,710 子：1,329P / 374F / 7T = 77.7%**（B6b 1,319P/384F/7T
  = 77.1%：**+10P / −10F**，Timeout 7 案恒等）。
- **零 P→F**（逐子 flip 矩阵核，other 0）。
- **F→P ×10（FIXED）分类**：svgtransform ×2（accumulate=sum scale +
  calcMode=discrete）+ svgtransformlist-invalid-value ×3 + non-additive-
  type-by / from-by ×2（result 反射）+ animate-gradient-transform +
  animateTransform-pattern-transform + change-target-while-animating-SVG-
  property（href 定靶读时解析——sample2@2s 改靶后 2.001s rect1 回 base、
  rect2 按现值起点接续）。
- **additive-type-by 案级仍 F（1 子，预期记账）**：采样断言面全过——
  orientAngle（角度域 by-only）、divisor/order/targetX/kernelUnitLengthX/Y
  （数值 href）、kernelMatrix（numberlist href）、rect.y（length）、dy
  （lengthlist）、viewBox（rect）、points（pointlist 逗号对 tile）、
  transform animVal 2 项 angle −22.5 十一域全绿；余卡 expectFillColor
  （getComputedStyle().fill 解析——#4 getComputedStyle SMIL 表现值门，
  跨域记账见 §4）。

### 2.2 web-animations 依赖通道（划界约束）

`testharness-web-animations --json`（同 guard 口径，本目录
`2026-11-07-m2-b6c-transform-domain-web-animations.json`）：**506P/444F/9T
= 959 子逐子恒等零 flip**。注册抖动备注：updating-the-finished-state
（rAF 驱动）同二进制连跑注册数 5/5/8 实证非确定（960 一次为抖动多注册，
终通道以 959 恒等录）。

## 3. B6c 批次记账（DC-3 FIXED/REGRESSED）

- **FIXED +10P**（§2.1 分类）。
- **REGRESSED 0**。
- **DC-4 数值门现势达线**：通道 P 1,329，对 DC-4 全量分母 15,342 =
  **8.66% ≥ 8.6%**（门余量 +10 子；M4 复测收口以滚动新面导入后现势复核）。

## 4. 挂账门（本批不改，跨域记账）

- **#4 getComputedStyle SMIL 表现值门**：additive-type-by expectFillColor
  （fill 域，本批新增）+ slider-switch 等五案（B5 §4 在案）+ svgstring
  fallback-to-discrete 同门。
- **animateMotion ×25**：getCTM 缺失（挂账 #2 邻接）；
  animatetransform-type-missing-value-default ×4 同门。
- **path d 插值 ×31**：B7/M3。
- **composition 族 helper ×8** + **活跃前置动画 underlying 快照**（B6a
  §4 在案）。
- 本批内 FIXME(M3)：transform calcMode paced 回落 linear、to-only
  transform/set 元素采样、混用定位（children+href 同属性）文档序归并。

## 5. 下一步（M2 收口 → M3）

M2 六批（B3/B4/B5/B6a/B6b/B6c）全收口。M3：B7 path d 插值 + 滚动新面
导入 + 挂账 #2/#3 归口 → M4 coverage 复测收口（DC-4）。
