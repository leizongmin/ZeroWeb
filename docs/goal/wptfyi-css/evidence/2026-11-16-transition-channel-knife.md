# Transition 通道 gCS 同步读可见性刀——consult-only 两世界注册表（Phase 1）

日期：2026-11-16（二十轮，wptfyi-css）。十八轮 §7 定谳的 T 通道全死根因
（corpus interpolation-testcommon T 测试同一脚本轮次 setProperty(from)→gCS
flush→写 transition-*+setProperty(to)→gCS 断言读，无泵帧介入→start_transitions
挂泵首帧，断言读恒先发生→gCS 恒落 cascaded 终值 echo）的本刀清偿：Phase 1
consult-only 架构（零 webview/pipeline 改动，完全镜像 jsanim 十轮刀已证模式）。

## 改动面（4 文件，es_module.rs s419 peer WIP 恒不入按惯例排除）

- **engine/animation.rs**：`interpolate_property_value_strict`（~180 行）——
  宽松插值器 `interpolate_property_value`（返回 String、端点 0 数值化）的严格
  门镜像：双端解析任一失败即 None（非插值对判定）。臂域 opacity/颜色族/
  长度族/translate/rotate/scale/offset-distance·rotate·anchor·position/
  offset-path/filter/backdrop-filter/transform/clip-path/font-weight/
  font-size-adjust/text-indent/min-max-sizing；无臂属性默认 None。
  helper：`parse_angle_deg_strict`（仅 Ndeg 形）、`parse_color_strict`
  （rgb()/rgba() 逗号形严格，gCS 颜色序列化同源）。
- **engine/js_dom_bridge/transition_registry.rs**（新建 ~330 行）：
  `World::{Gen,View}` 两域 + `TransitionEntry`（端点对 gCS 序列化形/duration/
  delay/timing/start_s）+ thread_local ENTRIES + MONOTONIC_START 墙钟锚。
  登记 `register_from_style_change`：transition_property 非空门→逐属性
  none/empty/all 跳过（FIXME(all) Phase 3）→duration>0 门→端点序列化非空
  且不等门→**严格插值门**（`interpolate_property_value_strict` None 即跳过
  ——css-transitions §starting-of-transitions 非 animatable 对不启动过渡，
  echo 池结构性不可触碰）→同 (world,identity,property) retain+push 替换。
  咨询 `query_override_for_sel/handle`：活跃区间 [delay, delay+duration]
  （css-transitions-1 §animations）→progress 钳 [0,1]→`apply_timing_function`
  →宽松插值→canonicalize（to 值恒等返 None 回落 echo——终态无须摘除条目）。
  6 单测钉：harness 中点（duration=100s+delay=-50s→progress=0.5，linear
  中点 100px）、at 编码进 timing（createEasing 对称 bezier c=(8at−1)/6，
  eased(0.5)=at 代数精确——at=0.3→80px）、非插值对不登记、替换+共存、
  世界隔离、marker provenance。
- **engine/js_dom_bridge/callbacks.rs**（Gen 通道三站）：
  `__zw_get_computed_style` need_reset 判定后清空 map 前**前值捕获**（html 门
  ——异快照跨文档不登记）；miss 闭包内登记（find_by_selector 取 nid）；
  cache-hit 与 miss 两 consult 位（jsanim consult 位镜像后置）；register_dom
  _callbacks 清表位（jsanim clear_all 同点）。
- **engine/js_dom_bridge/computed_style_cache.rs**（View 通道三站）：
  `computed_style_by_handle` 槽重建前旧槽前值捕获（html 门；同键命中 prev
  ==new 由登记期 from==to 门拒绝——零扰动）；styles.map 内登记先于本读
  consult（同一脚本轮次 setProperty(to)→读：登记与咨询同读完成）；
  jsanim consult 后 `.or_else` transition consult；rect 通道零行为变化
  （consult 仅 handle gCS 消费）。
- **engine/js_dom_bridge.rs**：模块声明（jsanim 同款注释惯例）。
- **js_anim_registry.rs**：`canonicalize` pub→pub(super)（本刀复用，序列化
  恒等返 None 回落语义共享）。

## 机制要点

- harness 语义：duration=100s + delay=-50s 钉死 progress=0.5；
  `at` 编码进 timing 函数——createEasing(at) 对称控制点 c=(8at−1)/6 的
  cubic-bezier 在 x=0.5 处 y(0.5)=0.75c+0.125=at 代数精确（at∈[0,1] 全域，
  外推形 engine apply_timing_function bezier y 不钳天然支持）。
- **前值捕获必须按 identity 键（首轮 A/B 零差定谳的根因修）**：v1 前值取
  文档级槽/per-selector map 换代边界 diff——corpus interpolation harness
  两阶段结构（`for targets { interpolate() } for targets { measure() }`，
  全部 target 先写后读）下，handle 视图槽是文档级全局，A 的 measure 前槽
  已被其他 target 的写+读重建到终态 → prev 恒落 to 态 → 登记期 from==to
  全拒（v1 B1 七腿 39,383 子测零差，探针 402 enter/0 OK 定谳）。修 = 注册表
  内 `LAST_STYLE: HashMap<(World, identity), ComputedStyle>` per-identity
  最近观测快照——只在本元素读点 take/register/store，穿插写零干扰；harness
  flush 读存 from 态、measure 读 register(from→to)——恰为 css-transitions
  §2 before-change style 在 gCS 观测点的正确近似。
- corpus T 元素主通道实测定谳（stderr 插桩探针，提交前撤）：harness target
  = `querySelector('.target') || 动态容器` → handle 身份 → **View 世界为
  主战场**（探针 register 全 `world=View`）；Gen 世界（静态 selector 元素）
  同机制并存。
- 严格插值门三重保护 echo 池（1,752P 全保护红线）：非插值对登记期即拒；
  eased=1（at=1）canonicalize(to)==级联基值返 None 回落；表空 O(1) None。
- canonicalize 尾门（jsanim 共享，`pub(super)` 提级复用）：raw 施于基样式
  clone 过 `AnimationClock::apply_to_computed_style` + gCS 序列化器，applier
  不支持的形恒等返 None 回落——**shape() 串（clip-path 新文法）动画 applier
  不解析 → clip-path shape() 面本刀零增益**（raw 插值全对、canon=None 实证
  ——独立残面挂账下刀队列：applier clip-path 臂 shape() 解析支持）。
- 与渲染管线 TransitionClock 零交互：独立 thread_local 实例（jsanim 同款），
  导航/新注册清表；事件派发（transitionrun/transitionend）不在 Phase 1 面。

## A/B 权威账

- B0=bin-shapeout-b1v4（md5 70f90c73 复验一致，08:36 终态树≡HEAD a7c81024d
  码态采认不重跑；es_module s419 WIP 两侧同载零贡献在案）；css-masking/
  css-shapes/motion/css-typed-om 四腿 JSON 复用 results-shapeout-b1v4（runner
  确定性 g36 先例），filter-effects/css-gaps/css-multicol 三腿 B0 二进制补跑
  （EXIT=0 ×3，results-trans-b0）。
- B1 v1=bin-trans-b1 md5 d0cf6f8d（零差轮，LAST_STYLE 根因修前）；
  **B1 v2=bin-trans-b1 md5 1ff4c7b1**（LAST_STYLE 修后权威批）全七腿新跑；
  transcript /tmp/zw-trans-{b0,b1}-\<corpus\>.txt；JSON 分目录
  results-trans-{b0,b1}；test-guard 包裹 --per-proc-mem 4 --total-mem 8
  --time-limit 3600。
- corpus 腿：css-masking + motion（主面）+ css-shapes + filter-effects +
  css-gaps + css-multicol（全 T 面）+ css-typed-om（护栏）。css-transitions
  corpus 本体不入腿——其测试多为帧驱动异步形（真 TransitionClock 管线面），
  非本刀同步读机制覆盖。
- DOWNS 红线：echo 池 1,752P 全保护；任何净 DOWNS>0 回滚分相。
- 单案机制实证（v2 探针轮）：motion offset-distance-interpolation 36 登记
  全 OK / 36 咨询 canon=Some（raw=-110px/-30px/-20px eased=-1/0/0.125 精确）；
  masking clip-path-interpolation-shape 30 登记 OK / 30 咨询 canon=None
  （shape() applier 缺口，见残面）。

## 结果（批后填）

### v2 中间轮发现（1ff4c7b1，未入账）：at(1) 组 REG 定谳与修

v2 批 REG 全部集中于 `at (1)` 子测（backdrop-filter/filter/offset-anchor/
offset-distance 等页 3-14 例/页），探针取证 got=**from 值**（eased=0）。
根因：`steps(1, start)`（createEasing(1) 形）解析为
`TimingFunctionValue::Steps(1, StepPosition::Start)`，而引擎共享
`apply_timing_function` 的 Steps 臂**未吃 position 参数**（恒 jump-end
floor）→ t=0.5 误落 0。修 = 注册表咨询位本地 `entry_eased`（css-easing-1
§step-easing-functions：Start/Both ceil 系、None round 系、End floor 系），
**共享函数不动**（全局行为面 jsanim 同载，独立刀验证）。v3=bin-trans-b1
md5 7b378e941707。

### v3 权威账（7b378e941707，七腿 LEG-EXIT=0 ×7）

**TOTAL: gained=288 lost=8 regressed=26，pages_gain=15 pages_loss=3**
（B0 vs B1v3 全七套件逐子测对账；比较器消费两侧 39,383 子测、页集合相等）。

增益面：filter-effects/backdrop-filter 五页 +3~+38（backdrop-filter-001
70→108）、motion 八页 +4~+25（offset-distance 32→57、offset-path-008
225→249）。lost 8 例全部为 offset-path-interpolation-{001,002,003} 的
`at (1)` 端点形（eased=1 本地 path 插值序列化形 ≠ echo 形）。

残留 26 REG 两类机制定谳：

1. **端点形类（at(1)，eased==1.0）**：hue-rotate 多单位 2 例
   （`100grad`→`90deg`、`1cm`→`10mm` echo 归一形 vs 本地插值形不一致）+
   offset-path 8 lost 同机制。css-transitions §animations：eased==1 值恒等
   `to` == 级联基值 → **回落 echo 恒正确**（附带规避形差）。
2. **量域钳制类（at(1.5)，eased=1.5 overshoot bezier）**：filter 量
   `grayscale/sepia/invert/opacity` 域 [0,1]（css-filters-1 §filter-functions
   量值界）——外推量越界（sepia(1.5) 等），WPT/Chrome 期望恒为钳后形（=
   echo）。16 例全此类。

**整体 `eased≥1→None` 方案否决**（全账扫描实证）：B1v3 本刀通道
B0-Fail→B1-Pass @at>1 真外推增益 **63 例**（offset-distance/anchor/position/
rotate `at(2)`、brightness/contrast/saturate/hue-rotate/blur/drop-shadow/
shape() `at(1.5)`）——域无界族合法外推，None 化全部翻回；另 echo 池已过
@at>1 242 例恒等。修法收敛两臂（注册表本地，共享插值器不动）：

- `eased == 1.0 → None`（严格相等：steps 越点精确 1.0；bezier 外推
  eased>1 不落此门）；
- filter/backdrop-filter raw 量钳制 `clamp_filter_unit_interval_amounts`
  （[0,1] 四族逐调用钳制；brightness/contrast/saturate/blur/hue-rotate/
  drop-shadow 不钳——63 例零冲突实证）。
- 负向投影：[0,1] 族 @at<0 双挂例 22 例（v4 钳制潜在新增益，批后核实）。
- 单测 8/8：+`test_eased_one_falls_back_to_echo`、
  `+test_filter_amount_clamp`（含多族混排/空量形）。

v4=bin-trans-b1v4 md5 639f7bd14539183a3edf76a6439582bc，七腿全量重跑
（results-trans-b1v4，transcript /tmp/zw-trans-b1v4-*.log/txt；发射前活跃
单元核查零碰撞——异流 ihe1 在 ZeroWeb-3 独立 clone，仅 sleep 尾等待）。

### v4 权威账（639f7bd1，本刀定稿账）

**TOTAL: gained=322 lost=0 regressed=0，pages_gain=15 pages_loss=0**
（B0 vs B1v4 全七套件；七腿 LEG-EXIT=0 ×7，比较器消费两侧全子测、页集合
相等断言过）。

- DOWNS 红线：lost=0 ✓（v3 的 8 lost 全回收——offset-path 三页回到 B0
  水位且中段增益保持）；echo 池 1,752P 全保护 ✓（regressed=0）。
- 26 REG 全清：at(1) 端点形 10 例回落 echo（hue-rotate 多单位 2 +
  offset-path 8）；at(1.5) 钳制类 16 例量钳后转绿。
- 增益分布：filter-effects 八页 +17~+40（backdrop-filter-001 70→110、
  filter-003 107→143）、motion 七页 +4~+25；较 v3 净增 +42 = 26 REG 转绿
  + 钳制臂负向侧新增益。
- 63 例外推真增益（v3 扫描定位的无界族 @at>1）全保（B0 视角零回归直接
  蕴含——v4 lost=regressed=0）。

## 门禁

- 单测：`cargo test -p zero-engine --lib transition_registry` **8/8 绿**
  （6 枚基线 + eased==1.0 回落钉 + filter 量钳制钉，多族混排/空量形）。
- 定向全量腿（test-guard `--compile-first --per-proc-mem 4 --total-mem 8`
  包裹，Makefile make test 同口径）：engine lib **4,068P/0F**；webview lib
  **852P/2F**——2F 逐名对号在册环境形红：`image_decoder::
  proxy_mode_decodes_png_correctly`（spawn 缺件形，s168c/s256/s336 三账在
  案，补建 `cargo build -p zero-image-decoder` 后单跑复绿 1P）+
  `csp_gate::csp_connect_gate_blocks_fetch_sh1_m2s5`（s348 在册环境红，
  HEAD 基线 2026-11-10 两账逐名同挂）——**零新增红**。
- clippy：`cargo clippy -p zero-engine --all-targets` 净（唯一告警
  script-sandbox 系 peer s419 WIP 同载，非本刀面）；fmt：engine 列
  `cargo fmt -p zero-engine` 已净。
- 全量 make test 未跑的定谳与挂账：fmt 门被**HEAD 既有入库漂移**挡死
  （`crates/dom/src/tests/tests_9_query_coverage.rs:455,462`、
  `tests/wpt-runner/src/testharness.rs:3755,3762`——工作树干净反证漂移在
  HEAD，非本刀引入）+ peer s419 WIP `es_module.rs:2040,2164` fmt 漂移
  （在途不可触碰）。三处皆挂账；本刀按质量门禁「按实际影响执行检查」
  以定向腿覆盖（改动面 = crates/engine 单 crate + webview 消费面）。
- A/B 权威账：v4 = 639f7bd14539183a3edf76a6439582bc（七腿 LEG-EXIT=0 ×7），
  B0 = 70f90c739ffa99a973b18e19fff1eb93（shapeout-b1v4 码态采认 + 三腿补跑
  EXIT=0 ×3）；DOWNS 红线 lost=0 达成。
