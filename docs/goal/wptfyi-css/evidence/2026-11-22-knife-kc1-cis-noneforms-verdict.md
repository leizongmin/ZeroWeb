# 刀 KC-1：contain-intrinsic-size 分轴 none 混形受理 — 判据与收账（2026-11-22 12:2x，ea04bbfb 系续开 fork）

## 认领与侦查（CLAIM-kc1-cis-animation-arm.md）

- 主判据页 `css/css-sizing/contain-intrinsic-size/animation/contain-intrinsic-size-interpolation.html`
  84F 全 `'from' value should be supported`。
- 侦查定谳（rlib 直链探针 probe/probe2，bin=HEAD rlib）：84F 全部死于 harness
  precheck `CSS.supports(property, from)`（interpolation-testcommon.js:396）——
  `'none 15px'`/`'15px none'`/`'none none'` 全 false（分轴 none 混合形被
  shorthand apply 臂拒绝）；`'20px 10px'`/`'auto 0px 0px'`/`'none'` 全 true
  （style-system 门健康）；非法形 `'none 15px 20px'`/`'min-content'`/`'auto'`
  正确 false。根因单一化＝per-axis none 缺臂。
- face＝apply_advanced.rs 单文件单 hunk 组：token 循环 `lens:Vec<Length>` →
  `axes:Vec<Option<Length>>`，'none' token 推 None 分轴；1 token 双轴同赋／
  2 token w×h／其余 false。auto 全局跳过语义不动；gCS 不动（shorthand 无臂
  ＝双侧空串共模，离散块比较吸收）。
- 语义：css-sizing-4 §contain-intrinsic-size（`none` 分轴＝该轴记忆尺寸无，
  混形离散插值块 precheck 解封后按 discrete 断言翻面）。

## 电池（unit `zw-knife-kc1-cis-noneforms-battery`，log `knife-kc1-cis-noneforms-battery-1210.log`）

- 窗序兑现：bordershape3 GATE-DONE 12:08:50（TRAP-NOOP 树净）→ serialize 门
  首轮 BUSY（对端收尾残影）二轮 12:10:47 放行 → 发射 12:10:17。
- 门全绿：OTHER-FACE 残留门（bordershape face 6 目标文件 b0 前零 worktree
  diff）；ride 形态门（es_module.rs ' M'＋animation.rs staged placebo
  cached-diff 指纹 s1/s2/s3 恒同）；M2 bin md5 异门 b0 `f51e6dbe` ≠ b1
  `15be337a`（face 无新增二进制可见字符串 → 无 SYMBOL-FM，M2＋树 grep 双
  标记代之）。
- legs 六腿全 rc=0（12:12:45→12:25:20，corpus-script 路径）：
  css-sizing b0/b1、css-contain b0/b1、css-values b0/b1。

## RECONCILE（键级，本 witness 独立复算与电池一致）

| 语料 | gained | lost | 判读 |
|---|---|---|---|
| css-sizing | 89 | 0 | 预注册恰集 84 全翻＋同机制域外溢 +5 |
| css-contain | 0 | 0 | 哨兵 SET 零漂移 |
| css-values | 1 | 0 | random-computed flake 键（OM-18 排除域）不入账 |

- css-sizing 细分：interp 页 84＝discrete 六 lane × test_no_interpolation 全翻
  （precheck 解封后离散块两侧同期望）；域外溢 `contain-intrinsic-size-009.html`
  ×3＋`-031.html` ×2（claim 预注「同机制域外溢以电池实测为准不预扩」兑现；
  009/031 均为 none 混形行为页，机制同源）。
- css-values +1＝`random-computed.tentative.html` `Shared globally:
  random(--identifier, a, b)` P 119→120——OM-18 排除域 flake（本日第四刀同键
  显形：FAM-F r3 −1/r4 −1、bordershape v2 +1、KC-1 +1，双向翻转史在案），
  不入账。
- P2F 红线（css-sizing 全谱键级零 Pass 丢失）零触发；contain/values 双哨零
  漂移线兑现（values 仅 flake 键）。

## 判读

- **COLLECT 净 +89**（tally +13528→+13617；基 +13528＝origin +13526＋fork-d
  FAM-F r4 勘误 +2，勘误笔 11:48 工作树件随本批代呈）。
- GAINED 案名抽样：CSS Transitions 与 Web Animations 双车道
  `from [none 15px]` 系全翻——Shorthand apply 臂修复经 transition_registry/
  animation registry 双通道浮出，零 registry 增补需求（离散块由
  transition_registry 既有管辖）。
- 残面：`contain-intrinsic-size-030.html` 21F／`auto-006.html` 18F 等行为页
  缺口＝布局语义域（auto 记忆尺寸存储模型），**KC-1b 另立不入本刀**（claim
  预划）。parsing/valid 3F 疑同因未在本腿显形（parsing 页不在三语料腿内，
  待 KC-1b 电池圈定）。

## 落库

- face：`crates/style-system/src/property/apply_advanced.rs`（patch md5
  `feebaf08a914e3e4d191458966c0314e`；EXIT 后树态 reverse-apply 恒同证）。
- 钉测：claim 预划「dev 实跑后随面」——本刀判定 WPT interp 页 84 案即常驻
  回归证据（上游语料每电池复跑），Rust 侧单 hunk 无独立解析态，暂缓钉测
  （pin-tests-doa 纪律：无实跑断言不落库）。
- 落库解锁 FAM-E 窗（fork-d 12:1x 认领 composition 残池，窗开前置＝kc1 落库
  ＋树净）。
