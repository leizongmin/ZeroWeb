# 刀 M4-a hyphens 初始值 v1 判定笔（2026-11-22 06:3x，fork-d witness 接管收账）

## 判定 TOTAL：gained=0 lost=0（四处全谱零位移）＋主判据不命中（observed 未变）＋钉测 DOA → **v1 face 目标错位，不入库**；真实初始值源定谳于 `default_impl.rs`，v2 face 重跑（r2 在飞）

## 电池档案

- unit `zw-knife-m4a-hyphens-initial`（甲 fork 05:58 发射；甲 session EXIT 后
  零收账迹象——EXIT 时点后 `.acceptance` 零新增文件、无在飞进程——06:31 起
  witness 接管判读），log `knife-m4a-hyphens-initial-battery-0558.log`，
  EXIT rc=0 06:31:03。
- face v1：`knife-m4a-hyphens-initial-face-draft.patch` md5
  `0564ed63647c8d8964929309dc12dc69`，双文件（registry.rs initial_value 臂
  None→Manual＋spec 注释；apply_coverage.rs 钉测 test_hyphens_initial_manual）。
- bins：b0 `4ebcfe8ee221699e42acac7fbcdca363` / b1 `556ee2fca88471886997484c5d8a8a98`
  （M2 门过＝md5 异，face 已烙入）；RIDE 1 文件＝s419（es_module.rs），
  S1/S3 md5 恒等零扰动。
- 语料腿 4×b0/b1 全 rc=0：values 3m47s/2m46s、css-text 2m33s/2m32s、
  animations 3m00s/3m11s、transitions 6m56s/6m59s。
- 快照：`.acceptance/wptfyi-css/snap-knife-m4a-0631/`（JSON/bin/log 全套 pristine）。

## RECONCILE 逐页（判读三头）

1. **主判据页零效应**：`css/css-text/animations/hyphen-no-interpolation.html`
   P 28→28、126 案 status+message 逐案 diff **全零差分**——hyphens 面
   CSSA/WAAPI t≥0.5 observed 未由 `"none "` 变 `"manual "`。按 claim 预注册
   分支：**不变＝另有路径**，M4 探针三头异常根因 a（registry 默认臂错注册）
   证伪——registry `initial_value("hyphens")` 臂非 observed 回声源。
2. **全谱零位移**：TOTAL gained=0 lost=0（values/text/animations/transitions
   四处逐页 P 计数恒等）；P2F=0 红线绿（无 fail-stop 面）。
3. **ASYM 3 行挂起零折算**（截断疑云，非本刀判据域）：
   css-animations event-order.tentative（b0=4/b1=1）、css-transitions
   animate-with-color-mix（b0=5/b1=4）、event-dispatch.tentative（b0=18/b1=11）。

## 钉测 DOA 防线（claim 收账路径必经）揭出真实机制

- 首执 DOA：`test_hyphens_initial_manual` panic——
  `ComputedStyle::default().hyphens` = None ≠ Manual（apply_coverage.rs:1263）。
- 溯源：初始值真实源为 `property/default_impl.rs:426` 手工 `impl Default for
  ComputedStyle`（`hyphens: HyphensComputedValue::None`）；registry
  `initial_value` 臂仅服务 `hyphens: initial` 声明解析（`apply_initial_value`），
  与 Default impl 是两个独立源。v1 只改前者 ⇒ gCS 无声明读数（inherit/clone
  链上游）恒 None ⇒ observed 回声不变 ⇒ 电池零位移。两判读同源互证。
- 连带发现：v1 face 破坏既有域内测试 `test_hyphens_initial_value`
  （extended.rs:1702，断言 initial→None 旧错注册）——claim「make test 不入本刀
  窗」条款不可豁免域内直接破坏，v2 随刀更新该测期望。

## 双 witness 判读对账（06:5x 补节）

姊妹 witness 实例同窗独立判读（`2026-11-22-knife-m4a-hyphens-abandon-verdict.md`，
snap-knife-m4a-hyphens-0632）结论同为 v1 不入库，但两处依据经本实例键级重放勘误
（仲裁 NOTES 勘误二笔 06:5x 节）：

- 其「P2F=13」实为 ASYM 截断分解：event-order/event-dispatch/
  animate-with-color-mix/display-none-dont-cancel 四页 b1 缺失案在 b0
  **全为 fail 态**（零 Pass 丢失）——intersection 全案 status diff
  P2F=0／F2P=0，与 claim「ASYM 行挂起零折算」一致；「fail-stop（P2F>0）」
  不成立，四分类记 flake/截断候选。弃置结论不受影响（主判据不命中独立成立）。
- 其「registry None→Manual 零效应→转 M4-c 刀」侦查线：本实例钉测 DOA 已
  定位真实源＝default_impl.rs Default impl（非「另有路径」的新机制），
  v2 face 直修正；r2 主判据兑现则 M4-c 前提消解，不兑现则合流转 M4-c。
- 时序勘误：registry.rs＋apply_coverage.rs 的剥除系姊妹 witness 06:36 前
  执行（非本实例 06:39 git apply -R——该调用原子拒绝＝无 face 可剥）。

## v2 face（r2）与窗序

- v2 patch `knife-m4a-hyphens-initial-face-v2.patch` md5
  `4a877c80e04308f2a8ae0f48c13981c5`，5 文件：registry.rs（承 v1）＋
  **default_impl.rs None→Manual（真实回声源主修）**＋types.rs 枚举 doc 初始值
  事实校正＋extended.rs test_hyphens_initial_value 期望 Manual＋apply_coverage.rs
  钉测（承 v1）。dev 测 `test_hyphens*` 5 passed（钉测＋initial_value＋inherit
  全绿，DOA 解消）。
- r2 电池 unit `zw-knife-m4a-hyphens-initial-r2` 06:42 发射（serialize 门
  06:42:37 过，ride 4 文件追踪）；判定承袭 claim 预注册（主判据 observed
  翻转＋TOTAL lost=0 红线＋ASYM 挂起），详见 claim NOTES 与 r2 battery
  脚本头注。
- v1 face 形不入库（零增益＋错位目标）；本笔为 v1 判定存档，tally 不动。
