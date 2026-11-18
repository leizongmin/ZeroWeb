# 刀 K：css-multicol-2 columns 斜杠 column-height + 四通道插值（2026-11-17）

## 面与根因

css-multicol corpus（B 账 `results-cssa-j1-full`）F 密集三页：

- `columns-interpolation.html` 282F——columns 简写插值 + 斜杠 column-height 组件；
- `column-width-interpolation.html` 148F；
- `column-count-interpolation.html` 120F。

根因链（三件套缺一）：

1. **简写文法**（css-multicol-2 §4.5 ED 2026-02）：`columns: [ <'column-width'> ||
   <'column-count'> ] [ / <'column-height'> ]?`——斜杠段 = column-height（auto |
   `<length [0,∞]>`，by computed value type 动画）。expand_columns 旧文法无斜杠
   分支：斜杠声明整条无效，且 slash tokenization 需处理 `10/100px` 粘连形
   （CSS tokenization 中 `/` 自成 token、空白可选）。columns 有效声明同时复位
   column-wrap 初值（同节明文）。
2. **gCS canonical 序列化**（columns-computed.html 27 行表，font-size 40px 锚定）：
   width/count auto 槽各自省略、全 auto→`auto`；height 非 auto 时 slash 恒显
   （`auto / 100px`、`10px 2 / 100px`、`2 / 100px`）、height auto 时整体省略。
3. **插值通道**（css-values-4 §8.5.6 by computed value type）：column-width/
   column-height = px 域 lerp 钳 0（`<length [0,∞]>` 域外负外推落 0px）；column-
   count = lerp 钳 1 取整（`<integer [1,∞]>`）；columns = 组件槽逐项 lerp——
   存在集恒等且各槽值存在性恒等才可插值（auto↔值槽形变落离散翻转 t≥0.5→to）。

## 码面（六文件 + 共文件 hunk + 收账接管抢救扩面）

1. `crates/style-system/src/shorthand/mod.rs`【本刀独占，含早前 css-gaps 面遗留
   rustfmt 折行 hunk 同文件】：expand_columns 斜杠 tokenization（括号外 `/` 补拆、
   残缺段整条无效、单斜杠上限）+ 文法必选前组（空 pre 拒绝）+ column-height
   复用 is_valid_column_width 门 + column-wrap 复位 + css-wide 关键字四 longhand
   落盘 + pending_shorthand_longhands 映射加 column-height。
2. `crates/engine/src/js_dom_bridge/computed_style.rs`【本刀独占】：
   column_height_to_css helper + columns_shorthand_to_css 加 height 参（slash
   canonical 规则）+ 串化表 column-height/columns 臂。
3. `crates/engine/src/animation.rs`【与对端刀 K1（kf-inherit）流共文件，hunk 级
   拆分提交（filter-k-anim-mine-hunks.py，map-first + 内容互证，对端
   filter-k1-anim-hunks.py 集合互补）】：松臂（column-width/height 钳 0 lerp、
   column-count 钳 1 取整、columns 槽 compatible 门 + 逐槽 lerp/离散翻转）+
   严格臂（parse_px_strict/parse_f64 端点，槽失配 None）+ apply 臂（端点写槽、
   缺槽 Auto、column-wrap 复位）+ ColumnsEndpoint/parse_columns_endpoint/
   serialize_columns_components helpers + 8 钉测。
4. `crates/engine/src/js_dom_bridge/transition_registry.rs`【本刀独占】：
   TRANSITIONABLE_PROPERTIES 56→60（column-count/column-height/columns/column-width
   入清单——transition:all 通配展开探测面）。
5. `crates/engine/src/js_dom_bridge_tests/part04.rs`【本刀独占】：gCS 钉测
   #cbh/#cbha/#cbhn/#cbhwn/#cbhs/#ch 六形（slash 四 canonical + height auto
   省略 + column-height 直读）。
6. `crates/style-system/src/tests/shorthand_coverage.rs`【本刀独占】：5 钉测
   （slash_column_height/slash_glued_token/slash_invalid_height/slash_auto_height/
   column_height_longhand）。
7.【收账接管抢救扩面】`crates/style-system/src/computed.rs`：column-width/
   column-height 枚举包裹 Length 字段 computed 就地 em→px 解析 + 负值 clamp 0
   （R907 同模式，css-multicol-2 §3.1/§4.2 值域 [0,∞]）——编排器 RC=20（style-system
   10 红）归因三缺陷之一；paint/动画/gCS 消费端仅匹配 Length(Px)，em 不解析则静默丢失。
8.【收账接管抢救扩面】`crates/style-system/src/shorthand/tests/error_cases.rs` +
   `columns_border_image.rs`：14 个过时 arity 测试 len 2→4（columns 简写恒四
   longhand：width/count/height/wrap）+ `[2]=("column-height","auto")`、
   `[3]=("column-wrap","auto")` 断言。测试文件不进 runner bin（A′ 手术集外），
   随本刀提交。
9.【收账接管抢救】`crates/engine/src/animation.rs` test_columns_component_interpolation
   期望修正：fork 手写期望与其自身输入算术矛盾（width 10px→20px 才得 15px），
   判实现正确、期望错，改 `"150.00px 15"`。

范围外（记账挂起）：allow-discrete transition-behavior（css-transitions-2，
columns 族 ~84F + 集群 ~140F）未实现——另行开刀；继承面 inherit-from 端点
（Animations/WAAPI 通道）F 保持——对端刀 K1 kf-inherit 码面（本刀 A′ bin 对称
在树，其效果双侧消去）。

## 验证：钉测（2026-11-17 15:0x 收账）

- style-system lib 全量 2555/0F（含 shorthand_coverage 5 钉 + error_cases/
  columns_border_image arity 14 测更新 + computed.rs 解析臂）
- engine lib 全量 4148/0F（含 animation test_column* 8 钉 + part04 gCS 钉测）
- css-parser lib 全量 3299/0F（旁证腿——本刀不动 css-parser，零扰动）
- filter 精读：style-system test_column* 33/0F（shorthand_coverage 模块 39/0F）、
  engine test_column* 16/0F、part04 gCS 钉测 1/0F

## 验证：reftest 探针差（15:36/15:41 双族实测回填）

靶面：css-multicol 布局消费面——column-height-001..009（早前 column-height
longhand 刀已开文法，本刀简写斜杠面补消费）+ multicol-columns-002/003/005/006
（slash 形 reftest 页，账外池）。双 bin（bin-j23KA/bin-j23B）实测：

- `reftest` 全套（725 本地 fixture，reftest-j23-{B,KA}.txt）：multicol/column-height
  lens diff **零差**——本地 fixture 无 slash 形消费页，预期内。
- reftest-upstream（wpt-data，reftest-up-j23-{KA,B}-multicol.txt）：multicol lens
  **零页级翻转**（column-height-021/022 双侧 ✓ 但 0.02↔0.03% 噪声漂移、
  column-height-023/multicol-width-ch-001 双侧 ✗）——**+176 全落 testharness
  断言面**（简写文法/串化/插值为 CSSOM 语义；布局消费面 column-height-001..009
  已被早前 longhand 刀过），零布局回归同窗实证。

## 权威账（B↔A′ 对账，链后填实）

树态：HEAD 63b0d6b73 + peer WIP 对称在树（对端刀 K1 kf-inherit 五文件、刀 J2
shape 流 css-parser/border_shape、typed-om 刀H es_module、examples 删集）。
A′ bin 现建（build-j23-bins.sh 三刀联合窗：本刀六文件反向补丁 + computed.rs
RC20 抢救臂反向 + animation.rs 本刀 hunk 过滤反向补丁，对端 hunks 原位保留）
——刀 J 先例：peer WIP 漂移不可廉价证，现建保对称消去无假设。B↔A′ 差量 =
纯本刀（对端面双侧同态抵消）。

- A′ bin：`17f80e5d0db498a09716539b17238ed4`（bin-j23KA）
- B bin：`78abca7310474e57a3896bcfbe40bc65`（bin-j23B）

### 五 corpus 对账（reconcile-five-corpus.py，results-j23r2-j23{B,KA}，15:2x 收账）

| corpus | B | KA | delta | 判据 |
|---|---|---|---|---|
| css-gaps | 3838 | 3838 | +0 | SET-IDENTICAL ✓ |
| css-multicol | 943 | 767 | **+176** | 列页恢复 ✓ |
| css-typed-om | 9947 | 9947 | +0 | SET-IDENTICAL ✓ |
| css-shapes | 5123 | 5123 | +0 | SET-IDENTICAL ✓ |
| css-masking | 2321 | 2321 | +0 | SET-IDENTICAL ✓ |

页集合全等（75/91/359/145/61 双侧全等）、NEW/GONE 零。multicol 页级：
column-width-interpolation +86（74→160）、column-count-interpolation +57
（78→135）、columns-interpolation +32（102→134）、column-width-computed +1。
跨链确定性互证：独立 r2 链双跑 j23B（results-j23-j23B vs results-j23r2-j23B）
gaps 3838=3838、multicol 943=943 零位移。

### 残面与本刀回归

- 本刀 16 回归（F2P 16）：全部 `columns from [inherit] to [auto]` +
  transition-behavior:allow-discrete 形——插值臂认领该族后端点离散回退语义与
  harness underlying 语义分歧（A′ 面该族不动画落静态继承偶然 Pass）。
  FIXME(k-discrete-inherit) 挂账随 allow-discrete 刀清偿。
- 范围外在册：allow-discrete transition-behavior（css-transitions-2）；
  `CSS.supports('columns', '<w> / <h>')` 支持面 false（supports 通道 slash
  文法未接线——columns-interpolation 页 222F 族，B/KA 双侧同态，非本刀回归，
  下一刀候选池）。

## 尾注

- 共享树多流仲裁：对端刀 K1 流 11:40 CLAIM（css-gaps inherit 100F）与本刀
  multicol columns 集合不相交；animation.rs 双侧 hunk 拆分（对端证据稿
  2026-11-17-knife-k1-inherit-gcs-channel.md 同款协议，互为补集过滤器）。
  构建窗互斥由编排器静默门保障（无 target/release 直用腿 + 无他流手术脚本 +
  cargo 空闲 + animation.rs mtime ≥180s）。
- 提交：4ff161f72（九文件，animation.rs 本刀 6 hunks——filter-k-anim-mine-
  hunks.py map-first 集含过时行号 5353 误收对端钉测 hunk，收账轮按内容定谳
  剔除（两 test_keyframe_inherit_* 纯 K1 面），终局拆分 K=6/K1=15 并集 21=
  全量 hunk 集，字节级闭环（工作树反向 apply→forward 复原逐字节全等）；
  bin 账不受影响——lib 测试码不入 runner bin）。快照编译证明：K-face
  animation.rs × HEAD 态 K1 独占五文件 cargo check --tests 绿。
