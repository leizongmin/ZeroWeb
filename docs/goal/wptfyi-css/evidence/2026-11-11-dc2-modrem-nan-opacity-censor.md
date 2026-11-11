# DC-2 mod/rem 除零 + opacity censor 批（g9）

**日期**：2026-11-11 深夜
**基线**：results-g8-round 2,450P（values）/ css-color 10,970P（cqw 基座刀后）
**结果**：values **2,450 → 2,464 净 +14 逐页零回归**（256 页仅 round 族两页漂移，
余 254 页逐位一致）；css-color **10,970P 零漂移**；css-parser lib **3107P/0F**；
make test **3879P/9F**（9F 逐名同形在册组：7F 既有欠账 + 2F f415be99c
execCommand；3879 = 3877 + 远端 14b6ca724 embed 批 2 桥测，零新增失败）。

## 改动面

1. **css-parser types.rs**：mod/rem 除零 None（无效）臂 → IEEE NaN——
   css-values-4 §10.5 mod()/rem() 同段："If B is 0, the result is NaN."；
   `av - bv*(av/bv).floor()` 与 `av % bv` IEEE 路径 B=0 自然产出 NaN，直接
   撤 None 分支。序列化/censor 走既有 NaN 管线（round 族 g8 同道）。
2. **engine computed_style.rs**：opacity computed NaN censor-to-zero（§10.12，
   r2320 individual transform 同规约先例）。**语料 inert 定谳**：values 语料
   computed 读走 shim 侧路径（该侧 NaN censor 臂缺位，serialize 页 6 NaN 腿
   "got NaN" 即此），宿主 face 今日亦不可达——parse_opacity 拒 calc 族
   （=下批 validity 放行刀的根因桶）。保留为 validity 放行刀的负载臂：
   parse 放行后 `opacity: calc(round(1,0))` 折叠 NaN 需此臂按 §10.12 出 '0'。
3. **basic.rs**：mod(1,0)/rem(1,0) None 钉改 NaN 钉（rustfmt 收敛）。

## 逐页归属

- round-mod-rem-computed **+12**（181→193）：mod/rem B=0 computed 等价类全数兑现。
- round-mod-rem-serialize **+2**（8→10）：`calc(mod(1,0))`/`calc(rem(1,0))` 与
  `scale(...)` 变体 specified 面 `calc(NaN)` 形翻绿（round 在 g8 已通，mod/rem
  随本批并入同管线）。

## provenance

- **22:04 初步跑 = 混合树**：g9 构建 22:01:17 起跑（42.06s 完成），而
  computed_style.rs 22:02:10 才编辑完——censor 臂未入该二进制；树另含 ba
  js hunk（21:59:54，语料中性：本跑 css-color 已零漂移）。**g9b 干净复跑**
  （23:13 systemd unit zw-g9b-modrem-2313，HEAD 14b6ca724 + g9 三文件，
  ba js 已 stash 隔离）values JSON 与初步跑 **byte-identical** → censor 与
  ba hunks 对 values 语料双双 inert，+14 归因清白全落 types.rs。
- unit zw-g9b-modrem-2313（run-g9b-modrem.sh：build + values + color +
  make test）；CSS_CORPUS_TRANSCRIPT 唯一路径 ×2；css-parser lib 定向快检
  test-guard --compile-first 3107P/0F。
- ba 流在途件：stash@{0}「ba-flow part11 js hunks」（part11-color.js
  `__zwContainerInlineSize` 出口 + part11-computed-color.js 通用通道 cq/em/px
  基座与 sign/abs）——21:59:54 编辑，含入口探明接线，css-color 对照零漂移
  旁证其语料中性；待 ba 认领或按孤儿件仲裁采纳。

## 残余分桶（下批输入）

- **serialize 页 14F**：opacity validity 3（round/mod/rem specified 于 opacity
  写面拒）+ scale-matrix 5（transform computed 未 matrix 化，在册桶）+
  computed-NaN 6（3 opacity + 3 scale，shim 侧 censor 臂缺位）。
- **computed 页 50F**：opacity-validity 32 + transition-delay used-value 15
  （数学函数形写面拒，parse 臂拒 calc 族）+ nan-serialize 3。
- **下批候选 = 数学函数 validity 放行**（parse_opacity/transition parse 臂放
  calc 族 + shim computed NaN censor 臂）：两页合计 ~47F 直接面，css-parser
  侧折叠已备（g8/g9 NaN 语义落位）。
