# 刀 FAM-D1 solo 探针判定笔（2026-11-22 06:0x，fork-d）

## 探针档案

- 页：`wpt-data/css/css-values/zz-famd1-intake-probe.html`（zz- 探针域不入
  tally；预注册六臂 P1–P6，判读矩阵 PREP-famd1-intake-probe.md）。
- bin：F2 电池 b0-runner（md5 `3f5a0bdb6c9eb48a0459908e1a83ef00`＝
  131086735+s419−F2face；animation/keyframe 路径与 HEAD 恒等，探针为机制
  观察非 A/B 记账）。serialize 门过（05:57 census 后 M4-a 05:58 才发射，
  探针 06:02 单页 60s 墙钟与其 build 窗并行但零资源冲突——serialize 纪律
  偏差如实记：census 时窗已闭，发射撞窗属巧合非纪律违犯）。
- 产出：`results-famd1-probe/`（b0 侧 19 subtest＝2P/17F，rc=0，非空洞）。

## 预注册矩阵全命中（06:03 实测）

| 臂 | 预注册 | 实测 | 判读 |
|---|---|---|---|
| P1 CS ∞ | 恒 i64MAXpx | i64MAX at -0.25..0.75、P at 1/1.25 | pinned 形确证 |
| P2 CS auto-from | 恒 90px | 90px at 0/0.5、P at 1 | pinned 形确证 |
| P3 Anim calc(10px+5px) | 15px↔auto 分叉 | auto 全 t | calc-length intake 全面拒收 |
| P4 Anim ∞ | 恒 auto | auto 全 t | intake 拒收确证 |
| P5 WAAPI ∞ | 恒 auto | auto 全 t | 同上 |
| P6 CS %-mix | raw to echo | `calc ( 100 % - 10px )` | 端点未求值确证 |

P3 对照定谳＝**auto**：Anim/WAAPI keyframe apply 面对纯 px calc（连
`calc(10px + 5px)` 都拒）与 ∞ 形一律拒收——非 ∞ 特异。

## 机制收敛（证据驱动修正三笔）

1. **CS 车道真机制＝transition 未触发**：P1 恒 i64MAX 含 t<0.5（离散 echo
   应给 from '0px'）；触发门对非 px 端点不放行 → 动画不创建 → observed＝
   to 侧静态 computed（∞calc setter 面已求值→i64MAXpx；%-mix 保持 calc 串
   →raw echo；显式 px to→90px）。P6 raw echo 无需独立 echo 机制。
2. **Anim/WAAPI 真机制＝keyframe apply 面值解析拒收**：block_to_points
   （animation.rs:4145）properties 保 raw string；拒收点在 apply_single_property
   （:4317）——`_ =>` 静默丢（left/top/right/bottom **无臂**）＋width 系
   `parse_px` 0 数值化 calc。lane 级 b0 数据同证：left Anim/WAAPI 0P/14F＋
   0P/14F，width Anim/WAAPI 35P/216F＋43P/208F（plain px 过、calc 0 化）。
3. **分槽**：D1-a＝CS 触发门/捕获归一（横切 152 interpolate-size＝FAM-B 域
   ＋CS ∞＋P2 形）；D1-b＝keyframe apply 面长度族 calc 归一＋position 臂补缺
   （本刀，face 立项）；D1-c＝%-mix 语境求值（CB/字语境，后置）。

## 悬案与偏差记档

- 探针页 06:02–06:11 窗被外部删除（fork-d 无 rm；M4-a battery.sh grep 无
  wpt-data 操作）；b0 数据已采全不受影响。同窗 fork-d 树上 face 后续被剥
  （06:13-06:14 窗，NOTES-arb-0525 更正笔）——两事件或同源（他流清场），
  待认领；教训＝face 必须立即落 patch 副本不裸驻（F2 patch-only 协议本旨）。
- D1-b face 首写 06:08-06:13 被剥；内容在案（PREP 下步节＋claim NOTES），
  M4-a EXIT 后重写＋patch＋md5 落盘再驻树。
