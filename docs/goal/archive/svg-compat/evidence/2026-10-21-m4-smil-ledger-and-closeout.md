# M4 SMIL 分类账定稿 + 挂账归口 + goal 收口（2026-10-21）

M3 S3（evidence/2026-10-21-m3-s3-script-load-landing.md）后 goal 收口切片。
M4 定位按契约是「SMIL 评估 + 挂账定稿」——**不实现 SMIL 时钟**；以 S3 后
口径重跑 animations/ 分类账（prep 底稿 evidence/
2026-10-21-m4-smil-assessment-prep.md 的重跑承诺），量化真实时序面，把
「可收的接口存在性面」收干净，其余全部归口为显式挂账清单。

## 分类账（svg/animations/ S3 后口径，数据源 m4 前基线 r2 定稿日志）

基线对比（M1 v2 → M4 前）：Pass 4 子 → **48 子**（脚手架释放：S3 的
pauseAnimations/setCurrentTime stub 后 SVGAnimationTestCase setup 不再
即死，「is not a function」接口缺失死面 **0 残留**）；Timeout 23 → 32
（F→T 分类移动：断言面暴露后等待 SMIL 时钟的案转文件级等待）。

Fail 228 子分桶（按错误消息归类）：

| 桶 | 子数 | 归口 |
|---|---|---|
| SMIL 时序/值断言（calcMode/from-by/additive/accumulate 等 assert 失配） | 155 | **SMIL 时钟挂账**（goal 契约排除面） |
| path 'd' 属性插值（`no animated path data` 33 案） | 33 | SMIL 时钟挂账（d 插值依赖动画采样） |
| getCTM 接口缺失（animateMotion 断言载体 15 + rotate-angle 7 同根） | 22 | **rendering-compat 协同**（getCTM 族既有挂账，非 SMIL 面） |
| clock-value-parsing 文法（`dur="00:30:01"` 越界时分秒） | 7 | SMIL 时钟挂账（时钟文法解析） |
| SVGLength 上下文单位（cap/ch/ic/lh/rlh unitType） | 7 | S2a FIXME 上下文换算批（master.md 计划 4） |
| color-interpolation（getComputedStyle rgb 解析 null） | 7 | 渲染/style-system 协同 |
| SVGAnimationElement 接口面 | 3 | exceptions 2 子 **M4 已收**（见下）；getStartTime interval 切换 1 子挂 SMIL 时钟 |
| 杂项（reinserting 重插续跑 / trailing-semicolon / svgcolor-invalid） | 4 | SMIL 时钟 + 渲染协同 |
| a.ping 导航域 | 2 | 导航域挂账（既有） |

Timeout 32 案归桶（SMIL 时钟等待面定性——挂账量化）：

| 桶 | 案数 |
|---|---|
| seeking（setCurrentTime 驱动 seek 事件族） | 8 |
| eventbase（事件基 begin/beginevents/event-listeners） | 6 |
| begin/repeat/first-interval interval 模型 | 7 |
| syncbase/cyclic（时间基解析） | 5 |
| 属性文法 → SMIL 采样断言（keyPoints/keyTimes/animateTransform） | 3 |
| 渲染/交互态（currentcolor-visited / conditional-processing / slider-switch） | 3 |

**SMIL 挂账总量定稿**：Fail 155+33+7+4 = 199 子 + Timeout 29 案（seeking/
eventbase/interval/syncbase/采样 5 桶）——需要 SMIL 时钟模型（interval
解析/事件基/seek/采样应用），体量显著，goal 契约明文排除；本分类账即
挂账量化底稿。

## M4 切片：SVGAnimationElement.getStartTime/getSimpleDuration 异常语义

- **语义定谳**（WPT SVGAnimationElement-exceptions 两子 + spec SMIL
  Animation）：无 active interval → `InvalidStateError`；simple duration
  未定义 → `NotSupportedError`。SMIL 时钟未接入 → interval 恒 unresolved、
  dur 恒未解析 → 恒抛（FIXME 记档时钟接入后按 interval/dur 返回）。
- 落点：part03.js S4c 控制面 stub 块内追加（beginElement/endElement/
  beginElementAt/endElementAt 同口径）；`_zwDomException` 构造
  DOMException。
- 验证：探针 m4-probe.html **5/5 Pass**（接口全族存在性 + 两异常 name/
  DOMException 实例面 + S4c beginElementAt NaN TypeError 不回归）；
  靶面 SVGAnimationElement-exceptions **2/2 全绿**（基线 Fail）。
- 划界：SVGAnimationElement-getStartTime.html 的 interval 切换断言
  （begin="1s;3s" 换档）需真实 SMIL 时钟——挂账（Fail→Fail，错误消息从
  `is not a function` 转 InvalidStateError，接口面已通）。

## 全 corpus 对账

对账 r1（基线 = M3 S3 收口 1238P/489F/40T/1767 = 70.1%）：

- 原始全量 **1245P/487F/40T/1772 = 70.3%**（+7P/−2F；分母 +5 = m4-probe
  5 子）。
- 转移矩阵：1238 Pass→Pass（**零 Pass 回归**、零 DOWN）、487 Fail→Fail、
  40 Timeout→Timeout、**2 Fail→Pass = 靶面 SVGAnimationElement-exceptions
  2 子**（getStartTime/getSimpleDuration 异常语义 stub）。
- 探针剔除同分母（1704 恒等）：**1175P/489F/40T = 69.0% → 1177P/487F/40T
  = 69.1%**（恰 +2P/−2F，全部落靶面）。
- 分母漂移：仅本轮 +5 = m4-probe（探针口径）；基线零行消失。

## DC-4 全量门禁

- `cargo fmt --all --check` **EXIT=0**。
- `make guarded-clippy`（workspace --all-targets -D warnings）**EXIT=0**。
- `make reftest`（上游全量，test-guard 包裹）**EXIT=0**——Layout 508/508 +
  Text 204/204 全绿零回归。
- `make test`（全 workspace，TIME_LIMIT=3000，test-guard 包裹）**EXIT=0**
  ——76 个 test suite、**22346 passed / 0 failed**。
- **DC-4 四项门禁全绿**（fmt / clippy -D warnings / reftest / make test）。

## DC 满足判定

- **DC-1 ✅**（M1）、**DC-2 ✅**（M2 S1–S4c）、**DC-3 ✅**（几何接口随
  M2 S4a + 事件目标化随 M3 S1/S2/S3 + SMIL 评估记账随 M4 分类账定稿）、
  **DC-4 ✅**（本节四项门禁）。
- 挂账定稿见上节七项清单——**goal 完成判定成立**（2026-10-22 终验）。

## 挂账定稿清单（goal 收口口径）

1. **SMIL 时钟模型**（199 子 + 29 案 Timeout，见上分类账）——goal 契约
   排除面，量化底稿即本文档。
2. **getCTM/getScreenCTM 族**（22 子 animateMotion 载体 + M2 S3 划界面）
   ——rendering-compat 协同（CSS transform 感知布局合成）。
3. **SVGLength 上下文单位**（cap/ch/ic/lh/rlh 7 子 + viewport/percentage
   14 子）——与 getBBox 几何上下文同批（master.md 计划 4）。
4. **文本几何**（tspan/text/getComputedTextLength/text-hittest 族）、
   **image 内在尺寸**、**CSS cascade 命中面**（pe-bbox 19 子）、
   **CSS d property**——既有挂账不变（master.md 计划 6）。
5. **script-load-error-events `.py` 动态端点** 2F——runner/infra 域。
6. **a.ping + a.rel-\***——导航域。
7. **color-interpolation 7 子**（getComputedStyle rgb 中间值解析 null）
   ——渲染/style-system 协同。
