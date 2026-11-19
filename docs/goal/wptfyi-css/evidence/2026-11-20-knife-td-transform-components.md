# 刀 TD：typed-om transform 组件槽位校验 + is2D 序列化（2026-11-20）

## 池与 face

池＝residue survey 批三候选① transform 半池 ~101F（stylevalue-subclasses 七组件页
cssRotate/cssScale/cssTranslate/cssSkew/cssSkewX/cssSkewY/cssPerspective +
cssTransformValue 容器 + serialization/normalization 两页）。face＝`part10-typed-om.js`
单文件 +127/−60：

- `_zwOmTransMatch`/`_zwOmTransSlot`：槽位型门（`<number>`/`<angle>`/`<length>`/
  `<length-percentage>`），型判定复用 `_zwOmTypeOfVal` 的 math 消解（negate 保形、
  invert 指数取负、product 指数累加、sum 逐对加法），单位消去随指数算术自然成立；
  setter 失败保旧值（css-typed-om-1 §cssrotate/§cssscale/§csstranslate/§cssskew/
  §cssperspective）。
- 七组件构造器与 accessor 校验 + 坐标归一（裸数→CSS.number/px/deg；skew/perspective
  槽位不收裸数——语料 'a double' TypeError 钉）；CSSPerspective 'none' 前置转换臂；
  CSSTransformValue 空表 TypeError。
- is2D 槽化：rotate/scale/translate 可写（store prop，按参数量初始化 2 参 true /
  3 参 false），skew 族/perspective 固定形 no-op setter。
- toString 按 §serialization 以 `_is2D` 槽定形（v2 修复，见下）。

## v1 电池（knife-td-battery-0006，00:04:47 build → 00:28:47 GATE-DONE rc=0）：外溢判读

发射后主 session 死于 API 限额（00:22 本地），结果未消费；本 rollover 轮接管判读。
`TOTAL: gained=133 lost=1 regressed=655 pages_gain=10 pages_loss=1`——按签名差粗看
FAIL，逐层分解后定谳为**单根因外溢**：

1. **655 "regressed" 全为改名对（pass 中性）**：testhelper 用组件对象序列化生成
   子测试名（`<transform>` 百分比示例＝`CSSTransformValue([new CSSTranslate(50%,50%)])`
   的 toString）。v1 将 CSSTranslate 的 z 槽从裸数 `0` 归一为 `CSSUnitValue(0,'px')`，
   而 toString 仍以「z 是否 number(0)」（`_zwOmIsZeroNum`）决定是否输出第三参 →
   2 参构造的示例串 `translate(50%, 50%)`→`translate(50%, 50%, 0px)`、
   `translate(0px, 1px)`→`translate(0px, 1px, 0px)`，the-stylepropertymap/properties
   ~335 页 ×2 子测试名漂移；逐页 gained=lost（accent-color 2/2、logical 92/92 等），
   子测试 Pass→Fail 计数实测为 **0**。
2. **真回归 1 条**：transform.html `Can set 'transform' to a transform: translate(50%,
   50%, 0px)`（P 31→30）——改名后的示例串经引擎 set/get 通道解析失败
   （'Specified value must be a CSSStyleValue expected true got false'）。

## 根因修复与 v2 电池（knife-td-battery2-022529，02:25:29 → 02:49:07 GATE-DONE rc=0）

toString 改按 css-typed-om-1 §serialization 以 `_is2D` 槽定形：is2D true → 两参形、
false → 含 z（translate/scale 两处同 pattern）。scale 3 参含 number(1) z 由「省 z」
改「含 z」（is2D false 规范形）——顺带兑现 serialization 页 +12（v1 +11）。

`TOTAL: gained=134 lost=0 regressed=0 pages_gain=10 pages_loss=0`：

| 页 | ΔP |
|---|---|
| stylevalue-normalization/transformvalue-normalization.tentative | +22 (2→24) |
| stylevalue-serialization/cssTransformValue.tentative | +12 (0→12) |
| stylevalue-subclasses/cssPerspective.tentative | +15 (10→25) |
| stylevalue-subclasses/cssRotate.tentative | +23 (4→27) |
| stylevalue-subclasses/cssScale.tentative | +9 (13→22) |
| stylevalue-subclasses/cssSkew.tentative | +16 (14→30) |
| stylevalue-subclasses/cssSkewX.tentative | +11 (8→19) |
| stylevalue-subclasses/cssSkewY.tentative | +11 (8→19) |
| stylevalue-subclasses/cssTransformValue.tentative | +1 (7→8) |
| stylevalue-subclasses/cssTranslate.tentative | +14 (9→23) |

哨兵 css-properties-values-api / css-animations / css-transitions / css-masking 四语料
页级 reconcile 零条目（face 不越界实证）。b0 md5 `85d2eb02` / b1 md5 `fad090d9`（M2
异 md5），RIDE 空（porcelain 除 face 外零修改），ride census S1/S3 零扰动。node 冒烟
105 断言（`/tmp/td-smoke.js`，fixed concat 重跑全 PASS）+ 序列化 13 断言
（`/tmp/td-ser-check.js`：testhelper 示例串与 b0 逐字节一致 + is2D setter 往返 +
scale 3 参规范形）。v1 现场存档 `results-knife-td/r1/`（b0/b1 json + 日志 + v1 face
patch），v1 判读脚本对账见本文件§v1。

## 挂账

- `_zwOmParseTransform` 'translate' 分支静默丢第 3 参（`nums[0..1]`）——`translate(x%,
  y%, 0px)` 非法 CSS 语法，语料不达；挂 FIXME 观测，parse 通道刀再修。
- v1 判读方法论沉淀：**testhelper 子测试名含组件对象 toString 产物，typed-om 组件
  序列化变更会以「全语料改名」形态伪装大面积回归**——签名差 A/B 判读须先分解
  gained/lost 配对再定谳（本例 655 REG 仅 1 真回归）。已入 docs/learnings。
