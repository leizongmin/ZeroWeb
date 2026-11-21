# 刀 CALC-SIZE-S2B（calc-size 非 definite 值承载＋布局期 basis 解析）证据

日期：2026-11-21。执行 fork：d2452915（face 作者，预注册
NOTES-knife-s2-calcsize-basis-claim.md）；电池执行＝本 clone 1849 接手
（d245 重发链 18:1x 死锁仲裁停后车道孤儿化，face 在树 strip-check OK
＋md5 绑定吻合，launcher zw-knife-s2b-v3-relaunch-1849 恢复队位）。上游：
css-values-5 §8 `calc-size()`。

## face（5 文件，patch f3a625740f875d6264ebc6ce339754fb）

- `crates/style-system/src/property/apply.rs`：calc_size_slot_capture
  （definite-sum None 落点 raw 捕获收窄 width/height/flex-basis 三属性，
  v3 口径）＋clear_calc_size_slot（后声明胜出清位，物理尺寸槽 ×10 臂）＋
  calc_size_parts/calc_size_substitute_size pub 导出（layout 单源消费）＋
  S2B marker。
- `crates/style-system/src/property/computed_style.rs`：CalcSizeSlots 七槽
  结构＋`calc_size_slots` 字段（anchor_size_slots 同型口径，None 零成本）。
- `crates/style-system/src/property/default_impl.rs`：None 初始化。
- `crates/layout-engine/src/engine.rs`：calc_size mod 挂载＋sizing pass 组
  wiring。
- `crates/layout-engine/src/engine/calc_size.rs`（新）：布局期 basis→px
  解析 pass（nested/any/stretch/fit-content/min/max-content/auto/
  length-percentage basis 表＋`size` 词元代换＋parse_calc 求值＋taffy
  尺寸回写＋统一重跑）。

## 电池（knife-s2-calcsize-basis-battery-v3.sh）

A/B 双臂：b0r＝树面剥本刀 face（git apply -R patch，md5 f3a62574 绑定）；
b1r＝树面＋face。门：PATCH-MD5／reverse-apply check／M2 双臂 md5 异／
FACE-MARKER 'CALCSIZE-S2 basis-resolve' b0=0 b1≥1；ride 面（树内 35 文件
在飞 peer WIP）双臂同载相消由 patch 域拆保证（TREE-MODIFIED 清单备案）。
腿：css-values（主靶）＋css-flexbox/css-sizing/css-variables（哨）×双臂。
判读：knife-s2-calcsize-basis-judge.py（页级 P 门＋靶族 P2F 门＋哨兵
SET-IDENTICAL＋全量 F2P/P2F 清单）。

## v3 电池实绩（zw-knife-s2b-v3-relaunch-1849，GATE PASS 19:09:00）

- 门证据：FM 'CALCSIZE-S2 basis-resolve' b0r=0 / b1r=2；M2 异 md5
  （b0r 248e39ec1511e2786e6cc108fac19cc0 / b1r
  8c8f411a8e1075fea3f320654ccc9666）；patch md5 f3a62574 绑定过；
  TREE-MODIFIED 35 文件；b0r/b1r build 绿（18:53:50 / 18:54:33 起）。
- 判读：**GATE PASS——GRAND F2P 16 / P2F 0**，无页回归；靶族 calc-size
  16/16（全 16 枚 F2P 均在 calc-size 家族）：
  - flex-basis-on-column +2（stretch size*2／auto size*1.6）；
  - flex-basis-on-row +4（stretch size/2、auto size*2、auto size*1.6+、
    auto and width calc-size(max…)）；
  - calc-size-height +1（definite height 容器内 calc-size(31%, size)）；
  - calc-size-width +9（fit-content ×3、max-content、min-content、auto、
    stretch size/2−10%、nested calc-size(calc-size(min-content,size),size)）。
  v2 回归的两枚 auto basis 臂（on-row/on-column）在 v3 修复分支下翻转
  ＝auto 三分支修复实证。
- 总量：css-values P 6078→6094（+16）／F 3786→3770（−16）；
  css-flexbox P 1423→1423、css-sizing P 3006→3006、css-variables
  P 302→302 全零位移；css-variables 哨兵 SET-IDENTICAL（58 页）。
- 预注册对账：F2P 16＝预期主数精确兑现（「+2」为乐观上界余量，auto 臂
  已含在 16 内）；P2F 0＝红线零触。

## v2 前轮 GATE FAIL 与修复（同提交前置证据）

v2 电池（patch 9061b93c，15:41-15:57）GATE FAIL：F2P 16 / P2F 5。P2F 逐案
归因＝真实解算 bug：①`auto` basis 无 definite 声明落 max-content 宽度测量
（width 页期望 500px 得 600px；height 页 203px）；②高度轴 intrinsic 用宽度
测量（size 常数 ≈123.3px：×1.5=185/×1.2=148/÷2=62 全吻合）；③box-sizing
未建模（19px/7px 同得 203px）。预注册「该域 b0 全 F 无 P2F 面」误判
（nested/auto 高度形存在 b0 级联基巧合通过）。v3 修复：auto 三分支
（flex-basis 槽保持 automatic main size 近似；width 槽 block=stretch-fit＝
父 content 宽（css-sizing-3 §5.3）、其余盒型 shrink-to-fit；height 槽
fail-closed None）；高度轴 min/max/fit/content 全 fail-closed None（回落
b0 行为零漂移，FIXME: block 高度 intrinsic 测量挂账随高度 intrinsic 刀）；
box-sizing 挂账同刀。

## 面外留置

- `apply.rs` 的 URLSMOD hunk（validate_decl_value url-modifier 消费臂）在
  patch 外＝本提交不含；提交后 apply.rs diff 仅剩该 hunk，URLSMOD 电池按
  解锁序 stock 重发。
- min/max-width/height 槽捕获 v3 起停用（interpolate-size 期望串 222 形
  P2F 面，claim note v3 口径），随高度 intrinsic 刀扩。
