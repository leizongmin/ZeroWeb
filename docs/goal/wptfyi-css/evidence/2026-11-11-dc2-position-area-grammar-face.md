# DC-2 组F：css-anchor-position position-area 语法面（单实现双通道 + shim 薄委托）

日期：2026-11-11。基座：13535525b（并行流 Rust 侧，本批 shim 接线为其后置提交）。

## 基线（anchorbase）

css-anchor-position 全语料 253 案：**7460P / 5356F**
（results-anchorbase/，唯一 transcript 重录；08:42 双轮同 transcript 撞车双零
产出后取证，见 .acceptance/wptfyi-css/notes-anchor-collision-forensics.md——
shim/transcript 单写者教训，后续探针一律 CSS_CORPUS_TRANSCRIPT 唯一路径）。

position-area 直接桶：position-area-parsing 1800F（decl 校验 + canonical
序列化缺面）、position-area-computed 633F（计算值解析缺面）。

## 撞车仲裁（同批双流）

本流先落 JS 解析器（part12 自包含 IIFE + 三钩子），并行流 zeroweb-2-2f 同步
落 Rust 文法模块（dom_bindings/position_area.rs，grid_cssom 单实现双通道架构）。
双方通读对方实现 + 期望矩阵交叉验证后定案**提案一（grid G1 模式）**：

- 文法/序列化矩阵单点在 Rust `position_area::canonicalize_decl` /
  `computed_value`（单测 5 组以 position-area-parsing/computed 语料为 oracle）；
- shim 薄委托：part06 decl 臂经 `__zw_position_area_normalize_decl`（'null'
  哨兵=拒）、part01c 计算臂经 `__zw_position_area_computed`；
- JS 解析器（part12）删除，JS supports 前置钩不落（宿主 decl_supported 臂
  已裁，双钩冗余）。

期望矩阵双方独立推导一致（九关键词类 X/Y/B/I/SB/SI/S/SS + FLEX center/
span-all；canonical 主槽序 X/B/SB 前置、Y/I/SI 配 center 时 center 前置、
S/SS 保参序同词折叠；span-all+轴关键词 parse 期折叠但 S/SS 保形存槽位；
computed 逻辑 long→short 同形塌缩、span-all 槽位消歧 `start span-all` →
block-start / `span-all start` → inline-start）。本流 JS 版曾以 node 24 锚
独立验证后并入仲裁。

## 实现（本批 shim 接线，5 文件形）

- part06 `_zwNormalizeDeclValue` position-area 臂：薄委托 grid 臂同形；
- part01c `__zwPlainComputed` position-area 臂：非继承只读 inline；css-wide
  keyword 计算期语义（Cascading 4 §6-§7）——initial/unset/revert/revert-layer
  → 'none'、inherit 强制取父链首非 css-wide 声明计算值（§7.3 对非继承属性
  亦生效）；FIXME 作者样式表臂未接；
- part01 gCS direct-access 白名单 + position-area 宿主回声覆盖两臂
  （host 对 registry 外未设值属性回 'initial'/'inherit' 字面量兜底，非空
  短路 plain 臂——plain 优先覆盖，color 抢答同型）。

## 终验（anchorbase2 全量轮，results-anchorbase2/）

css-anchor-position 253 案：**9893P / 2923F**，对基线**净 +2433F**，逐页
零回归（improved 2 页、regressions 0）：

| 页 | 基线 | 终验 |
|---|---|---|
| position-area-parsing.html | 1800F | **0F（页清零）** |
| position-area-computed.html | 633F | **0F（页清零）** |

paprobe 定向轮（results-paprobe/）：parsing 2125/2125、computed 633/633、
position-area-value 7/7、try-tactic-position-area 131/131、
position-area-fixed 7/7。残余 position-area 布局行为页（scrolling/insets/
wm-dir/in-position-try/align-justify 等 329F）需真实定位布局实现，不属语法
面，挂账 DC-3 后续批。

## 门禁

- fmt EXIT=0；workspace clippy -D warnings EXIT=0（组合态）；
- make test（systemd zwgf-test 唯一日志 zwgf-make-test.log）：**3846P/7F**，
  7F = main 既有欠账组逐位一致（img settle ×2 / fetch / iframe SW /
  readable stream / websocket ×2，HEAD 判定见 master.md 碰撞记录），零新增。

## 挂账

- position-area 布局行为桶 ~329F（scrolling/insets/wm-dir/align-justify/
  in-position-try）——需 anchor 定位布局实现；
- position-area 作者样式表声明臂（shim 计算链 FIXME）；
- @position-try CSSOM（CSSPositionTryRule，at-position-try-cssom 265F）。
