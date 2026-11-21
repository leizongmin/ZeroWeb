# 刀 MOTION-2 v4 · offset-path 失配车道拆分订正（2026-11-21）

## 缘起：v3b 20 P2F 回归与 001 残 18F

MOTION-2 v3（face `30de4389` stash，+584 已入账）的结构失配处理是
**臂内 canonical 离散**（lerp None 时 t<0.5 取 from canonical / else
to canonical，`Some` 发射）。v3b 复查发现两处破损：

1. **008 页 20 P2F**：path↔shape 混合对走动画车道翻转时，v3 的单侧
   canonical 化把 `path("M10 10 h 5")` 侧发射成
   `path('M 10 10 H 15')`，而语料期望**声明原文 verbatim**（双引号）。
2. **001 页 18F 残**：臂内 canonical 离散发 `Some`，骗过 transitions
   启动门（css-transitions §starting-of-transitions：结构失配对非
   animatable，过渡不得启动＝静态跳终值）——门判可插值 → 过渡启动 →
   t<0.5 发 from ≠ 语料期望终值。

## v4 机制：能力信号与发射风格分权

face（2 文件，clip_interp.rs +56 行 ±、animation.rs +29 行 ±）：

1. `interpolate_offset_path` 结构失配改发 **None**（纯能力信号，不再
   臂内离散）；
2. 新增 `discrete_offset_path_canonical`（clip_interp.rs）只承担**动画
   车道**（@keyframes/Web Animations 求值 `None` 回落，css-values-4
   §8.5.6）翻转发射：双侧端点均为无 fill-rule path() → 绝对化 canonical
   后单引号发射（纯 path() 失配对的车道端点是 computed 双引号形，原文
   翻转引号风格不符语料）；任一侧非 path() → 双侧声明原文 verbatim
   翻转（008 形）；
3. `interpolate_property_value` offset-path 臂 None 回落改调上件
   （animation.rs:1432）；transitions 臂严格 `None` 不登记 → 启动门
   正确拒启。

钉测 `test_interpolate_offset_path_m2_path_canonical` 扩臂（PIN-OK
先执后落库）：结构失配 strict None 断言＋双引号 computed 端点单引号
canonical 发射＋path↔shape 混合对双侧 verbatim 翻转（0.3 from /
0.6 to）。

## 电池（unit zw-motion2-v4-battery-0609）

06:10:15 发射，06:16:28 MOTION2-V4-DONE rc=0（wall 6m13s，CPU
10m40s，peak 4.1G）。flock /tmp/knife-motion2-v4.lock；test-guard
全包裹；PIN 1 passed 0 failed；b1 现树构建（release，Finished 1m35s），
md5 `90f8bfd15da7e20acd61d642948fa1fa`；b0 复用 v3b 产物
（`b0-v3b-runner`，FM0=0，JSON 拷入自包含）。

产物：`.acceptance/wptfyi-css/results-knife-motion2-v4/`（bin/b0/b1
+ pin-test.log；发射日志 knife-motion2-v4-battery-0609.log）。

## 结果（css-motion b0→b1，case 键级全对账）

| 页 | b0 | b1 | Δ |
|---|---|---|---|
| interpolation-001 | 134/288 | 276/288 | +142 |
| interpolation-002 | 6/144 | 122/144 | +116 |
| interpolation-003 | 8/192 | 184/192 | +176 |
| interpolation-004 | 12/192 | 180/192 | +168 |
| interpolation-008 | 410/614 | 410/614 | ±0（v3b 20 P2F 清偿） |

净 **+602（F2P 602 / P2F 0）**，四页恰集翻转、008 回基线全 Pass；
missing=0 extra=0（gained 页断言数双臂对称零幻影）。哨兵
css-transforms/css-typed-om **b0/b1 JSON md5 全同**（零位移，
deterministic runner）。

入账口径：累计已含 MOTION-2 v3 +584，本笔订正增量 **+18**
（gross 602 − 584；001 启动门臂 18F 兑现 + 008 20 P2F 回归清偿）。

## 代际与 ride 法证

- face 文件 mtime 06:08:35/06:08:56 < b1 build 06:11:25–06:13:00，
  无后半程编辑窗；收账时（08:17）全机无 cargo/runner 进程、无活跃
  unit，产物无覆写。
- b1 树载对端 CALC-SIZE face（style-system apply.rs +483，编译清单
  含 zero-style-system）——b0（v3b 二进制）不含该 face，ride 非双臂
  同载；但翻转恰集为 motion 4 页、哨兵两语料字节恒同 → CALC-SIZE
  face 在本电池三语料零行为位移，+602 归因干净（CALC-SIZE 自账另收）。
- FACE-MARKER needle `MOTION2 offset-path-path-absolute-canonical-
  discrete-single-quote` 已在 HEAD（v3 提交烙入），b1 FM=1 不区分
  v3/v4——v4 载入由行为签名独立证实（P2F 0 + 001 +18）。

## 残面（MOTION-3 候选池，维持）

005 多子路径 canonical 深水区 308F＋composition ray() 组合异机制
84F＋001 段数失配采样形残（v4 后剩 ~12F）＋composition add 41F。
