# M2 切片 378（s378）——dialog-form-submission t5 翻绿（image 按钮 selected coordinates 与 offsetX 读数链同构化）+ constructor-submitter-coordinate 既有红定性渲染流缺口

日期：2026-11-10（承 s377 挂账：流内剩余候选 form-submission 亚像素坐标）
域：html/semantics/interactive-elements/the-dialog-element（dialog-form-submission t5）+ xhr/formdata（波及面）
性质：**施工轮——t5 翻绿（interactive-elements 真案 16F→15F，纯 JS shim 双文件）；constructor-submitter-coordinate 基线同败定性 image input replaced sizing 渲染流缺口**

## 案面与根因

**t5（input image button should return the coordinates）**：click listener 读 `event.offsetX/offsetY` 拼串为期望值，dialog returnValue（image 提交链 selected coordinates）为实际值，断言两读数自洽（`assert_equals(returnValue, expectedReturnValue)` 动态，不钉上游字面值）。红态：expected `14.400000000000006,3.152000000000001` got `14,3`。

**三读数链不同构**（探针 zz_probe_s378 现场：RECT left=8 top=15.848 宽 29 高 8；EV clientX=22 offsetY=3.152；IMGCOORDS=[14,3]）：
1. **part03 R150 dispatch 期 offsetX 派生**（CSSOM View §dom-mouseevent-offsetx）：`offsetX = clientX − gBCR.left` 无 round 浮点 → listener 读数 `19 − 15.848 = 3.152`；
2. **part04/part06 recorder**（HTML image button activation behavior selected coordinates）：`Math.max(0, Math.round(clientX − rect 原点))` → 提交链 `round(3.152) = 3`；
3. trusted 注入 `Math.floor(detail.clientX)`（part06）——两通道整数化。

t5 红态=链 1 浮点 vs 链 2 整数。Chrome 双读数均浮点同构（上游案绿）。

## 施工（方案 W：floor 重锚 + round 退役，纯 JS 双文件）

1. **part06 trusted 注入 floor 保留但语义重锚**：与手动 ctor（part05 `_zwMouseCoordInit` WebIDL long truncate——UI Events init dict unsigned long，fractional untrusted +576P 依赖面）同构整数化 → 两通道 ev.clientX 恒等 → R150 派生、双 recorder 三面同源同式（submitter-coordinate-value 双 prong 一致断言面不变）。
2. **part06 offsetX 注入臂去 round**（R3999 兜底臂）：与 R150 同式无 round。
3. **part06 recorder 去 round**：`Math.max(0, _genFx − 原点)`——与 R150 同式（_genFx=floor 注入值=ev.clientX 同源）。
4. **part04 recorder 去 round**：手动通道 `ev.clientX（ctor floor 整数）− left（浮点盒原点）` 与 R150 同式同值恒等。

**首版缺陷（全域 join 抓获）**：首版同时去掉 trusted floor（浮点透传，模仿 Chrome trusted 内部浮点）→ submitter-coordinate-value 翻红（trusted 14.5 vs 手动 ctor truncate 14 两通道差异暴露——差异来源=ZW 的 image input 渲染 29×8 非 SVG 固有 200×200，中心 22.5 浮点化；Chrome 同页 200×200 中心 108 整数故 truncate 无影响恒一致）。stash 基线归因实证后回退 floor 改重锚（方案 W）——**教训：Chrome trusted 浮点透传对齐须以两通道一致性为前提，渲染缺口（replaced sizing）未修时整数化重锚是唯一鲁棒口径**。

## constructor-submitter-coordinate 既有红定性（非本刀，stash 基线同败实证）

案内 `clientX = Math.round((left+right)/7)`、期望 x=23——**SVG 固有尺寸 200×200**（root width/height=200，文件名 100px-green-rect.svg 误导）：期望 23 = round((8+208)/7) − 8、46 = round((8+208)/4) − 8。ZW image input 渲染 **29×8**（input 默认控件尺寸，replaced element sizing 未生效）→ clientX=round(45/7)=6 → 6−8=−2 → 负值钳 0 → got "0"。

**定谳：image input replaced sizing 渲染流缺口**（SVG 固有尺寸未进 replaced sizing 链）——挂账渲染流簇不修（run-rules 工作面纪律）。同缺口佐证：t5 探针 RECT 29×8 同面。

## 勘根过程坑（入册）

1. **html-semantics 通道探针 testdriver 装配**：TESTDRIVER_STUB 经 `replace_script_source("/resources/testdriver.js")` 装配——探针必须显式引用 `/resources/testdriver.js`（仅 testharness.js 引用不够，`test_driver is not defined`）。
2. **shim 无 dump 函数**：探针现场输出走 `assert_true(false, 现场)` 红态消息携带（dump is not defined）。

## 域口径变化

- interactive-elements 真案：**16F→15F**（t5 翻绿；R-s31 簇 13 + showModal 1 + corner-cases 1 上游单列）
- 跨流既有面 20 行单列维持（s377 归因 1bcfaac6d）；closed-details 出域 1 行维持
- sweep 446P→447P（+1 翻绿，join 纯移除零新增红零改名；探针行删除即消失）
- xhr/formdata 定向族 93 行 1F（constructor-submitter-coordinate 基线同败在册=渲染流缺口）零本刀新增红

## 挂账（移交下一切片）

1. **image input replaced sizing 渲染流缺口**（渲染流）：SVG 固有尺寸进 replaced sizing 链——constructor-submitter-coordinate 1F 解锁键，同簇佐证 t5 探针 29×8。
2. **R-s31 UA dialog 定位簇**（渲染流）维持；**customizable-select A2 簇**维持；跨流回归面 20 行维持（渲染流控制面处理）。
3. 流内剩余：closed-details-layout-apis（渲染流）、inert hit-testing 穿透近似面（无钉面不扩刀）、基建面 T=3+probe8/9/10 转正决策承继。

## 验证

- dialog-form-submission solo 10/10 全绿；submitter-coordinate-value solo 绿（首版回归→方案 W 修复）；constructor-submitter-coordinate solo 红=stash 基线同败（非本刀）
- interactive-elements 全域 sweep 447P/37 Fail 行：join 翻绿 1（t5）+"翻红"1=探针自身（assert_true(false) 恒红 dump，勘根毕删），零意外位移
- net-api 全域 sweep 12775P/1102F：formdata 定向族 93 行 1F=在册既有红；uievents offset 抽查 7P/0F
- engine lib 3827P/7F（+1P 组合态漂移）7 红逐名同构 s377 基线集零新增红
- shim 22 part 拼装 node --check PASS（方案 W 后复验）；纯 JS 双文件轮 fmt/clippy 不适用

日志 /tmp/s378/（本地：sweep_ie.log、sweep_netapi.log、sweep_uievents.log、lib_engine.log、build_prestash.log、f378.txt join 集、shim_concat*.js）。
