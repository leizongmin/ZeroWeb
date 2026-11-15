# `target` expando shim 刀落地——插值 harness 全域解锁 A/B

2026-11-15。执行 [刀规格](2026-11-15-target-expando-interpolation-unlock.md)
§4。提交 8ee02114e（3 shim 文件 +34/-1：part01 helper、part05 set 守卫、
part04 get 回读）。

## 1. 实现（最小切片，镜像 `name` 臂先例）

- **part01.js**：`_REFLECTED_TARGET_TAGS = ' a area form link base '` +
  `_reflectedStringAttrOn(tag, prop)`——`target` 仅真反射 tag 返 'target'，
  其余 tag 返 null（caller 回落 expando）；其他名与 `_reflectedStringAttr`
  恒等。
- **part05.js set**（R3042 expando 守卫）：`_reflectedStringAttr(p) === null`
  → `_reflectedStringAttrOn(_realTag(sel, handle), p) === null`。非反射 tag
  上对象值落 expando；真反射 tag R5007 ToPrimitive 反射不变；原始值面不动
  （R3069 另臂，spec 明示不动）。
- **part04.js get**（R3037 反射读臂头部）：`target` 非反射 tag 先查
  `_expando[key]` own 命中即返（对象值 `.style` 链活），miss 回落属性读
  （保 R3069 原始值 round-trip 与 setAttribute 面）。SVG 臂不属本刀（spec
  §SVG 分支语义正确未触）。
- 已知窄偏差（接受，无语料面）：非反射 tag 同名先对象后原始值写
  （`div.target={}` 再 `div.target='x'`）读回旧对象（attr 写不清 expando，
  real browser 同 JS 槽后写胜）。masking harness 每案新建容器不触发。

## 2. 验证链

### zz-mprobe-expando 17 项矩阵（探针页留 wpt-data 不入库）

全钉：primitives/plain object/元素/自引用 回读 ✓；**`div.target=b` 回读
`object`（刀前 `string`）+ `.tagName`=DIV + `.style.setProperty`=0.5**
（crash 链 `:277 set → 回读 String → .style undefined` 断链）；
`a.target='x'` 反射 `attr=x` ✓；`a.setAttribute('target')` → `.target`=
_blank ✓；`div.setAttribute('target','y')` → 读回 y（R3069 原始值面
divergence 保持）✓。

### masking 组 A/B（B0=results-texp-b0 / B1=results-texp-b1，zz- 探针页剔除）

| | pages | Pass | Fail | Timeout |
|---|---|---|---|---|
| B0（刀前） | 61 | 126 | 417 | 1 |
| B1（刀后） | 61 | **615** | 5725 | 1 |

**净 +489P、0 页回归（逐页 Pass 集不降，61/61 页保序）**。Fail 膨胀为
21 页 setup 崩（`declared tests: 0`）转真实子测的诚实分母——子测声明数与
chrome summary 逐页精确对齐（21 崩页合计 **5,818 上游子测解锁**）。

### 跨组抽验（evidence §3 预测三组全实证）

| 页 | chrome 子测 | B1 声明 | Pass |
|---|---|---|---|
| transforms/animation/transform-interpolation-004 | 288 | 288 ✓ | 12 |
| motion/animation/offset-path-coord-box-interpolation | 144 | 144 ✓ | 0 |
| css-backgrounds/animations/background-position-interpolation | 196 | 196 ✓ | 7 |

三页刀前同签名 crash（evidence §3），刀后全量声明真实子测。转化率
（12/288、0/144、7/196）为插值引擎能力面残差（WAAPI/插值语义支撑），
按 §3 分布逐组实测另轮推进。

### provenance

B1 二进制 mtime 09:17:36（> 我刀编辑 09:13-09:14，含刀；peer g50′
7bc31687d 同树内容先于构建落定——其 M 文件 mtime ≤09:11:34，09:19 提交
内容恒等）。masking/transforms/motion/backgrounds 插值页与 g50′ 面
（vertical-align 长度臂/border-spacing 视口真值）无交集。探针页在
wpt-data（gitignore），结果 JSON 在 `.acceptance/wptfyi-css/results-texp-b1/`。

## 3. 后续（下轮）

按 evidence §3 的 337 页分布逐组实测转化率：transforms 32 / backgrounds
30 / motion 22 / gaps 21 / masking 21（本轮已全量）/ values 18 / fonts 16
/ grid 16 / sizing 16 / text 12 / filter-effects 12 / ui 11（余长尾）。
每页 B0 均为 declared 0 崩溃 → B1 转化率 = Pass/上游子测（chrome summary
`c[1]` 分母）。残差大头是引擎插值能力（WAAPI support 门、interpolate
语义），解锁后逐组按失败签名分桶开刀。
