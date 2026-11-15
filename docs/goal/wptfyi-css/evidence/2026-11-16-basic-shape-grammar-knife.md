# 基本形状文法族刀（shape()/path()/rect()/xywh()）——css-parser 文法 + 四站接线

**日期**：2026-11-16（十八轮；设计稿
[next-knife-shape-family-design](../../../.acceptance/wptfyi-css/next-knife-shape-family-design.md)
十五轮出稿，上轮实施 + rate-limit 中断，本轮续跑收口）
**状态**：已收口（权威 A/B **B1v3 净 +939**/UPS 951/12D 真实残面/页 0 移，护栏 typed-om 0/0/0；验证链见 §5）

## 1. 刀体

一块共享文法吃三属性（clip-path / shape-outside / offset-path）的
css-shapes-2 基本形状族：`shape()`（命令式 move/line/hline/vline/curve/
smooth/arc/close，to|by 双向，`with` 控制点，`of <radii> [cw|ccw]
[large|small] [rotate <angle>]` 修饰序不敏感解析）、`path()`（SVG path
data 双引号承载）、`rect()`（空白分隔 4 值 + round）、`xywh()`（+ slash
圆角）。arc 修饰 canonical 序 cw → large → rotate；缺省省略（cw/small、
ccw/large、rotate 0deg、缺省方向）；裸数字补 px；1 词位置补 2 分量。

五站：
1. **css-parser**：`parse_basic_shape.rs`（新，~1.6k 行）+ `types.rs`
   `ClipPathValue` 四变体（Box 载荷控制枚举尺寸）+
   `parse_extended_border.rs::parse_clip_path` 派发四臂 + `serialize_math.rs`
   `fold_decl_specified` clip-path 声明级 canonical 臂。单测
   `parse_basic_shape/tests.rs` 25 钉（语料 42 形矩阵）。
2. **style-system apply**：offset-path 首片文法门放宽（ray() 全形 +
   基本形状全族 + coord-box 后缀 + 纯 coord-box 关键词；`parse_ray_decl` +
   `split_coord_box_suffix`；文法裁决单点 parse_clip_path）。
3. **engine gCS**：`computed_style.rs::clip_path_to_css` 四臂（shape() 走
   computed 级 `shape_to_css_computed`——关键字折 %、rotate 归 deg；
   path/rect/xywh 声明级 verbatim）。
4. **engine transition/animation**：transition 通道 clip-path get 臂
   （无臂两侧同空串恒不启动）；WAAPI underlying clip-path 臂 +
   `clip_interp.rs`（新，~613 行）clip-path 同构逐分量插值 +
   offset-path ray/shape/path 逐命令插值（异构离散 ≥0.5 翻转）。
5. **paint**：border_shape 四变体 FIXME 空顶点（渲染面无语料消费方）。

## 2. probe A/B（11-16 03:54–，zw-shape-ab.service，B0=88b6ab519 重建
md5 6f551c72… / B1=WIP 重建 md5 bd4cf74e…，es_module s419 WIP 双侧随行零贡献）

### css-shapes（已收）

- 页面 145→145（GONE 0 / NEW 0）
- 子测 **1,840 → 1,884 净 +44**（UPS 45 / DOWNS 1）
- UPS：shape-function-computed +26（0P/35F → 26P/9F）、
  shape-function-valid +17（31P/20F → 48P/4F）、rect-function-valid +2
- DOWNS 1：`curve to 20em 20pt with 10vw 30vh`——b0 经 shim 宽收 verbatim
  回显假过，b1 走 canonical 后 `20pt`→`26.666667px` 失配（共享解析器
  types.rs:3224 把 pt 折 px，LengthValue 无 Pt 变体，声明级 canonical
  无法保原单位形）

### 串化双通道修（本轮补刀）

- **fold 臂物理单位守卫**：`has_physical_length_unit`（pt/pc/cm/mm/in/Q
  数字后紧随判定）→ 命中跳过 canonical 回落原文通道（b0 行为恢复）。
- **gCS computed 级串化**：`shape_to_css_computed`（= shape_to_css_mode
  参数化 + `shape_axis_comp_resolved` 接线）——`from center`→`50% 50%`、
  `hline to left`→`0%`、edge right/bottom→`calc(100% − o)`、rotate 归 deg；
  声明级通道保关键字/rad 不变（valid 页语料钉面）。computed 页余 9F 全属
  此缺口（26 UPS 后残余），修后预期 F→P。
- 钉测 3 个：物理单位判定矩阵 / fold 守卫 / computed 关键字折算矩阵
  （22 → 25 钉）。

### A/B v2 批（zw-shape2-ab.service 04:11–04:45 七腿全 EXIT=0；**被 §4 B1v3 权威账取代**——v2 二进制 04:11:37 构建早于 twin 04:17 clip_interp/parse_basic_shape 修改，motion 腿少 89 UPS）

- **B1 provenance v2**：bin-shape-b1 md5 `d3145336…`（04:11:37 重建 = HEAD
  88b6ab519 + 刀体 + 串化双通道修；批全程 md5 未换装）；B0 复用
  `6f551c72…`（88b6ab519 重建，css-shapes b0 03:58 账直接采认）。
  es_module.rs s419 WIP 双侧随行零贡献（提交排除）。
- **css-shapes**：145→145 页（GONE 0/NEW 0），1,840→1,894 **净 +54**/
  UPS 54/DOWNS 0——computed +35（probe 余 9F 全翻正）、valid +17、
  rect-valid +2；probe 期唯一 DOWNS（`20pt` 物理单位假过翻红）**消除**
  （fold 守卫生效）。
- **css-masking**：61→61 页，668→995 **净 +327**/UPS 327/DOWNS 0——
  clip-path 插值三页 +320（shape +154/control-points +126/xywh-rect +40）、
  parsing/valid +7。
- **motion**：45→45 页，576→1,045 **净 +469**/UPS 481/**DOWNS 12**——
  插值页 +456（offset-path-interpolation-005/006/007/008/009 +424、
  coord-box +28）、computed/parsing +21；12 DOWNS 全落
  offset-path-interpolation-007 neutral↔none 端点对（见 §3 残面 5）。
- **css-typed-om（护栏）**：359→359 页，9,872→9,872 **0/0/0**
  （same_pass 9,872 持位）——白名单/apply 改动零外溢实证。
- **合计：净 +850 / UPS 862 / DOWNS 12 / GONE 0 / NEW 0**。

## 3. 残面（下刀队列）

1. **shape-outside 兄弟页**（~2,184 子测）：registry 白名单缺
   shape-outside/shape-margin/shape-image-threshold 三串（本刀
   registry.rs 零改动——supports 门 `CSS.supports('shape-outside',…)` 恒假，
   插值页 780+546+478+140+120+120 全灭实证）。需白名单 + ComputedStyle
   存储槽 + apply + gCS + WAAPI 臂五站。
2. **rect/xywh computed 折算后置**（round 圆角 computed 串形挂账）。
3. **path() 段级 canonical**（绝对化 + 空格归一，computed 面后置）。
4. shape() 命令级兼容插值 / calc 代数插值（offset 刀残面同型）。
5. **neutral↔none 离散端点对**（motion 007 页 12 DOWNS，**v3 实测定谳
   真实残面**）：CSS Transitions 通道 t<0.5（expect from=underlying
   circle(10px)）与 CSS Animations/WAAPI 通道 t≥0.5（expect to=none）
   全 Fail——插值函数级（`interpolate_offset_path`/get 臂
   `unwrap_or_else("none")`/flip 逻辑）读码与探针均正确产出端点串，
   页级端到端未触达该路径；根因在 shim/engine 对 neutral keyframe 页
   构造（transitions:all 声明注入、动画值应用点）的未建模环节，
   活体插桩定位挂下轮；007 页净 +13 已如实计入总账。
6. **<position> 3 词混合形**（`right 30% top`，规范合法）：实现
   `parse_shape_position` 3 词一律拒收（parse_basic_shape.rs `3 => None`），
   4 词 [edge offset][edge offset] 支持、3 词 [edge offset][edge] 缺；
   钉测按现状钉负例（423 行），语料若现此形需补 3 词臂。

## 4. 权威 A/B 终账（B0=6f551c72… HEAD 88b6ab519 / **B1v3=9c06ccb6…** HEAD+全 WIP
终态含 04:17 clip_interp/parse_basic_shape 修复，release 重建 34.05s
（首发 zw-shape-v3.service 04:47 PATH 未注入 cargo 127 即修），es_module
s419 WIP 随行零贡献两方在案；committed animation.rs 04:59 mtime 晚于构建
的差额仅测试钉（release 不含 #[cfg(test)]），行为臂经 corpus 结果反证
全在 b1v3 内（masking +327 证 clip-path 臂、motion +558 证 offset-path
臂+fallback）；zw-shape-b1v3-ab.service 04:51:41–04:58:15
四腿全 EXIT=0，journal scope 记账 CPU 6m28s，transcripts zw-b1v3-*.txt 在盘）

### css-shapes（b0 03:58 复用 / b1v3 04:53）

- 页面 145→145（GONE 0 / NEW 0）
- 子测 **1,840 → 1,894 净 +54**（UPS 54 / **DOWNS 0** / same_pass 1,840）
- shape-function-computed **+35**（0P/35F → 35P/0F）、
  shape-function-valid +17（→48P/4F）、rect-function-valid +2（→满）

### css-masking（b0 04:18 复用 / b1v3 04:55）

- 页面 61→61（GONE 0 / NEW 0）
- 子测 **668 → 995 净 +327**（UPS 327 / **DOWNS 0** / same_pass 668）
- clip-path-interpolation-shape **+154**、…-shape-control-points **+126**
  （clip_interp.rs shape() 逐命令插值兑现）、
  clip-path-interpolation-xywh-rect **+40**、clip-path-valid +4、
  clip-path-shape-parsing +3

### motion（b0 04:33 复用 / b1v3 04:56）

- 页面 45→45（GONE 0 / NEW 0）
- 子测 **576 → 1,134 净 +558**（UPS 570 / **DOWNS 12**）
- UPS 明细：offset-path-interpolation-006 **+203**、008 **+173**、
  005 +104、coord-box +34、007 +25、computed +18、009 +10、
  shape-computed +3
- **12 DOWNS 定谳（v3 伪影假说证伪）**：v2 批 12 DOWNS 曾按 mtime 法证
  归因「旧二进制伪影」（04:11:37 构建早于 04:17 none 端点修复），预期
  B1v3 重跑消除。**实测 v3（含全部修复）DOWNS 恒 12**，全落
  offset-path-interpolation-007 neutral↔none 端点对——伪影假说证伪，
  **定谳真实残面**：插值函数级探针 ✓ 但页级端到端未触达该路径，
  shim/engine 对 neutral keyframe 页构造（transitions:all 声明注入、
  动画值应用点）仍有未建模环节（§3 残面 5 挂账；007 页净 +13 已如实
  计入总账）。按「收口行转抄红分类前独立核机制」：伪红标签不得触发
  跳修，本例即 v2 收口行转抄伪影归因被 v3 实测推翻的活例。

### css-typed-om（b0 04:41 复用 / b1v3 04:58）

- 页面 359→359（GONE 0 / NEW 0）
- 子测 **9,872 → 9,872 净 +0**（UPS 0 / DOWNS 0 / same_pass 9,872）
- **护栏零位移**：解析域（typed value 串形）A/B 全等——刀体不外溢。

### 合计

**净 +939 / UPS 951 / DOWNS 12（真实残面）/ GONE 0 / NEW 0**——
较 v2 批净 +850 再 +89（motion 修复兑现：006/008/coord-box/007 UPS 增量）。

## 5. 验证（fmt/clippy 已绿；make test 拆腿账见下；bench-gate 定向串行后置）

### make test 拆腿（zw-shape-mkt.service 04:53:35 起，systemd-run + test-guard 逐腿）

| 腿 | 结果 | FAIL 归因 |
|---|---|---|
| companion-build | EXIT=0（冷建 31s） | — |
| workspace-test（162 bins） | 10,923P/**2F** | image_batch_e2e + same_origin_image_burst——image_batch 两形态预存红 |
| renderer-lib | 371P/**1F** | js_worker fetch_without_handler——worker-fetch 预存红 |
| browser-bin | 741P/**2F** | tab_js_worker fetch + default_actions——预存红 |
| browser-tests（12 bins） | 772P/**2F** | 同 browser-bin 两个 |
| gpu-adapter / gpu-consistency | 96P/0F、1P/0F | — |
| quickjs-workspace（17 bins） | 825P/**1F** | image_batch 形（quickjs 面现形，预存） |
| quickjs-renderer-bin | EXIT=0 | — |
| quickjs-renderer-lib | 369P/**1F** | js_worker fetch_without_handler——worker-fetch 三形预存红（05:11 收尾腿） |
| quickjs-clippy（14 crate `-D warnings`） | EXIT=**101** | `es_module.rs:2217 too_many_arguments (8/7)`——script-sandbox 流 s419 WIP 在途态（该文件 04:57 仍活跃编辑；非本刀面，提交排除，peer 收口自清） |

全部 FAIL 对号预存红总账（worker-fetch 三形 + default_actions +
image_batch 两形态；bookmark 本轮未现形），**零刀致新红**；与十七轮
mkt3 账逐腿同形（741P/2F ×2、775P/1F、369P/1F 逐数吻合）。

### bench-gate（定向，target 被磁盘守卫清空后冷建）

- **过程**：首发 zw-shape-bench（05:11:37）撞 target-disk-guard——173GB
  超阈值触发分级清理，删 debug/incremental 32GB 后 rm 撞并发写者中止
  （target 残 4.1MB，磁盘回 872G 空闲；mkt 门禁 05:11:19 先收口无碍，
  A/B 二进制在 .acceptance/ 未受损）。二发 zw-shape-bench2（05:12:58
  冷建 ~3min）EXIT=0。
- **判定**：css_bench 微基准 **1/1 passed**；form-input-perf-gate
  **PASS**（绝对预算）；perf-gate WARN = 报告硬件（AMD 9700X）与基线
  （i5-13500H）不匹配，相对门禁不启用——绝对预算通过在案
  （benchmark_20261116_051259.json）。三 crate 扩展腿
  （zw-shape-bench3：css-parser+engine+style-system，05:27:20–05:32:04
  暖建 4.7min）**EXIT=0，微基准 3/3 passed**，form-input-perf-gate
  PASS（绝对预算），perf-gate WARN 同上（硬件基线不匹配相对门禁不启用，
  benchmark_20261116_052720.json）——**定向 bench 门禁全绿**。

### 单元测试 solo 腿（test-guard 包裹，04:3x 批链在跑期发射——轻量不占
cargo 构建锁，与已编译 runner 进程无冲突）

- **zero-css-parser --lib：3262 passed / 0 failed**——25 钉全绿（物理单位
  判定矩阵 / fold 守卫 / computed 折算矩阵）。发射首跑 101 包名错
  （css-parser → zero-css-parser），非编译红。
- **zero-style-system --lib：2546 passed / 0 failed**（修复后；首跑
  2544 passed + 2 红，红转绿）。
- **复跑双钉腿**（zw-shape-pins.service 04:52，批收口后）：css-parser
  `--lib parse_basic_shape` 25 passed/0 failed（filter 3,237 非空真绿）；
  engine `--lib test_interpolate` **23 passed/0 failed**（本刀 5 钉在列）。

### fmt / clippy

- **fmt**：`--config-path rustfmt.toml` 十五文件（全刀改动面含 twin
  registry/inherit 两文件）EXIT=0（tests.rs 两钉漂移已修——首版检查
  管道吞码假绿 + 未带 config，带 config 复验揪出）。
- **clippy**：css-parser/style-system/engine 三 crate `--all-targets`
  EXIT=0 净（zw-shape-pins.service 04:52；es_module s419 WIP 不在扫描面）。

### 存量红三修（非本刀改动面，定谳后顺手清偿）

- **定谳**：core.rs `test_apply_initial_value_all_properties` +
  `test_initial_value_completeness` 两红——HEAD 88b6ab519 白名单已含
  "column-rule"/"backdrop-filter"（grep 计数 =2 实证）而 initial_value
  无臂，HEAD 上必然红 = 存量真实红（四分类），非本刀回归。column-rule
  = 十七轮刀解锁白名单的验收债；backdrop-filter 更早解锁同型债。
  apply_initial_value 侧 backdrop-filter 已有臂（inherit.rs:1529），仅
  column-rule 简写缺。
- **修**：registry.rs initial_value 表补 `"column-rule"` 简写臂（无聚合
  PropertyValue 变体，返回首子属性形，gap 简写臂先例）+
  `"backdrop-filter"` 臂（`Filter(Vec::new())`，与 default_impl
  `backdrop_filter: Vec::new()` 一致，filter 臂同形）；inherit.rs
  apply_initial_value 补 `"column-rule"` 简写臂（三子属性槽回落）。

## 6. 过程记

- **resume-swarm 对称收敛实例**：本 session rate-limit 中断多重 spawn
  （在册形态），收口期间 twin 实例对 computed_style.rs Shape 臂做了与
  本实例同向接线（措辞异、语义同——`shape_to_css_computed` 消费），
  按「同 session N×resume 对称收敛取磁盘终态」协议采认合并；
  三新符号唯一性扫描 + 门禁编译验证兜底。
- probe 期批链占用（zw-shape-ab systemd scope）本实例不发射重型入口，
  门禁全部压后到批收口之后。
- **接替批（04:09:06）**：旧 probe 批单元被 twin 停机（journal 显式
  Stop，CPU 15m26s 后收摊）；twin 以 `zw-shape2-ab.service` 续批——
  B1 重编为 knife v2（md5 d3145336…，含 fold 物理单位守卫 +
  computed 串化器两修），队列 masking/motion/typed-om 六腿 +
  b1 css-shapes 重跑，旧单元 b0 css-masking 殿后作废重跑。本实例按
  对称收敛协议不竞争发射，承接只读归因（css-shapes 权威对账上表）
  并改随新日志监视。
- **B1v3 权威化（04:47–04:58）**：twin 先发 zw-shape-v3.service 重建
  （首发 127 PATH 未注入 cargo 即修，二发 release 34.05s 快照
  bin-shape-b1v3 md5 9c06ccb6…），再发 zw-shape-b1v3-ab.service 四腿
  批（EXIT=0 ×4）。本实例挂 zw-shape3-watch.sh 看门（三条件：二进制
  换装/BATCH-DONE/twin 消失），采认不重跑——一账两用。
- **伪影假说证伪闭环**：v2 批 12 DOWNS 的 mtime 法证归因被 B1v3 实测
  推翻（§4 motion 段），真实残面定谳挂 §3 残面 5——收口行红分类
  转抄前独立核机制条款的直接兑现。
