# easing 感知离散翻转采纳刀（flip adoption）——机制、A/B 双面账与门禁

日期：2026-11-15（十二轮，wptfyi-css）
改动面：`crates/engine/src/js_dom_bridge/js_anim_registry.rs`（引擎 registry 侧）+
`crates/engine/src/js_dom_shim/part03.js`（仅 provenance 标记注释，零行为变化）。

## 机制要点

**断点**：jsanim v2 fallback 检测器（十轮）把「开区间 tick 输出 ≡ 端点原文」一概
判为引擎未真插值的 echo（关键字/calc-size/url 离散 fallback，canonicalize 还会
变异裸串）而回落 B0——但 spec 离散/步进行为的合法输出**本来就该**恒等端点：
css-values-4 §8.5.6 不可插值对在 transformed progress 0.5 处翻转（< 0.5 → from、
≥ 0.5 → to），step-start/end 的 eased 钳 0/1 同形。十轮账实测该全折折损
**离散面 ~50 真增益**（v1 586G 内含、v2/v3 清零），控制面挂账 ~88G（含未发现的
grid-template 等离散面）。

**解法**：检测器放宽为 **eased 一致性采纳**——仅 `flip_ok` 形态（恰 2 帧、端点
offset 0/1（选择器域判定，数组序不信任）、direction normal）对端点恒等输出重算
`eased = apply_timing_function(prog, entry.timing)`：输出 ≡ to 且 eased ≥ 0.5 →
采纳；≡ from 且 eased < 0.5 → 采纳；侧别失配/NaN eased → 维持回落（零回归方向）。
eased 与引擎段内映射同源（合成规则帧不携带 animation-timing-function——
`_zwWaapiSpecKeyframes` 丢帧级 easing 键——引擎段内恒用全局 timing；span 1 归一化
恒等，整体进度 ≡ 段进度）。

**形态门**：多帧（段选址扭曲整体进度）、reverse 族（方向反演）、端点缺 0%/100%
帧的条目不进门，保持 v2 拒绝行为。恒值对（from ≡ to）任何进度期望恒同，无条件
采纳。端点对按帧选择器归侧（首帧 0% → from、次帧 100% → to），同帧重复声明
首值胜（与 endpoints 去重一致）。

## A/B 双面账

- B0 = `bin-cadd-b1`（round-11 B1 快照 = 本刀前 HEAD 行为；五电池侧直接复用
  `results-cadd-b1` 存量）。B1 = `bin-flip-b1`（本刀）。
  **修正（收口复核）**：上句「B0 = 本刀前 HEAD 行为」不严格成立——法证发现
  bin-cadd-b1 内嵌非 HEAD 内容（tick 钳制位无发射瞬态 + 缺 cadd animation.rs
  臂），对间净差 ≠ 本刀，账目三桶拆分见下。
- **provenance**：marker `zw-easing-flip-adopt-2026-11-15`（part03.js 注释经
  include_str! 入二进制——Rust 侧 const/static 不被引用不落二进制，实测 0 命中
  后改走 shim 注释先例）：strings 抽验 B1=1 / B0=0。
- 新面八任务双腿（results-flip-b0/b1）：easing、display、lists、borders、ui、
  text-decor 全量 + grid(grid-template)、animations(interpolat)、
  transitions(interpolat) 定向。
- 护栏面五电池 B1 侧（B0=results-cadd-b1 存量）：css-transforms、motion、
  css-backgrounds、css-values、css-fonts。
- **二进制内容 provenance（mtime 法证）**：bin-flip-b1 构建 18:35:24，装载面 =
  HEAD(e42482bfd) + 本刀（js_anim_registry.rs 18:33:53 / part03.js 标记
  18:34:43）+ peer script-sandbox es_module.rs s419 WIP（18:16:31 在树）——
  es_module 为 SW 模块导入绑定面，CSS corpus 页纯 script 无模块路径，零行为
  重叠面（附注备查）。**不含** offset-\* 刀 WIP（style-system 18:46:42+ /
  engine computed_style.rs 18:49:03 / animation.rs 18:53:04 均晚于构建）——
  本刀 A/B 对 offset 刀与 peer WIP 三向隔离，净差即本刀。
  **修正（收口复核）**：「零行为重叠面」「净差即本刀」两项不成立——es_module
  s419 在对间内容差实锤（nm 符号集 76 vs 460），且 bin-cadd-b1（17:43 构建）
  早于 cadd 提交（18:16），animation.rs 臂域 + 疑似 tick 边界瞬态构成额外
  内容差；拆分归账见「账目三桶拆分与二进制对内容差法证」段。

## A/B 数字（批跑收口回填）

### 新面八任务双腿（B0=bin-cadd-b1 / B1=bin-flip-b1，results-flip-b0/b1）

**全部零位移**：easing / display / lists / borders / ui / text-decor 全量 +
grid(grid-template) / animations(interpolat) / transitions(interpolat) 定向——
双腿 transcript 逐字节一致（cmp 实证 easing/display/lists/gridtmpl 四对
IDENTICAL），gained/lost/regressed 全 0。

**目标选择事后剖析（重要）**：八任务系上轮会话按「离散翻转面在哪」的猜测选的，
未核 corpus 库存——实测 css-easing/display/lists/borders/ui 六组 **corpus 无 WAAPI
动画面**（无 animations/ 目录页；css-lists/css-lists-no-interpolation.html 与
css-text-decor/animations/discrete-no-interpolation.html 存在但字节零位移，系引擎
keyframe 值臂对 list-style/text-decoration 属性族无 echo 输出——fallback 检测器
与 flip 采纳整链 inert）；css-grid grid-template 定向与 animations/transitions
interpolat 定向同零（grid-template WAAPI 面引擎无 echo、transform 插值族不在
interpolat 子串页集内已由护栏腿覆盖）。真增益面 = 引擎 keyframe 值臂**已有 echo
输出**的属性域——transform 插值族实证（下；后经账目三桶拆分，transform 族
+42 中仅 3 例为本刀直接，主增益面实为 fonts/values/transforms 三域离散对
interior 采样共 21 例，见下拆分段）。round-10 v1 账「fonts 离散面 ~50」
复查：fonts v1 增益 180 实为 font-size-adjust-composition 122（cadd 刀已收 +31）
+ font-variation-settings-composition 8 + 端点钳制/插值形余量（v3 已保留），
离散 no-interpolation 页 v1 实为 64→63 **净 −1**（7 REG）——上轮「离散面 ~50」
归因过宽，本刀实测修正。

### 护栏面五电池（B0=results-cadd-b1 存量 / B1=results-flip-b1）

| 组 | gained | lost | regressed | 备注 |
|---|---:|---:|---:|---|
| css-transforms | **+42** | 0 | 0 | 7 页全双边转移（001 +12/002 +7/003 +3/004 +3/005 +10/007 +6/verify-reftests +1）——三桶拆分：flip 离散直接 3（007 matrix3d↔matrix 离散对 interior 端点期望）+ **外来钳制面 39 不记本刀**，见下账目拆分段 |
| motion | 0 | 0 | 0 | offset-\* supports 门未开（归 offset-\* 刀），flip 无触碰面 |
| css-backgrounds | 0 | 0 | 0 | discrete-no-interpolation.html 背景族（background-* 6 属性 ~119 候选之一）未现增益——WAAPI 页面集引擎无该族 echo 输出，flip 链 inert（同八任务剖析结论）；零回归护栏 ✓ |
| css-values | +6 | 0 | 0 | 2 页（max-width +3 / max-height +3）——三桶拆分：flip 离散直接 4（`none↔20px` 离散对 at 0.5/0.6 interior）+ **外来钳制面 2**（at 1.5 越界，不记本刀） |
| css-fonts | **+70** | 0 | 0 | 4 页——三桶拆分：flip 离散直接 14（metric 混排离散对 interior）+ cadd 臂迟达 5（add 组合中点真插值）+ **外来钳制面 51 不记本刀**，见下账目拆分段 |

**五电池合计 +118 / 0 lost / 0 regressed**——按属性分组定谳（zw-flip-gr-by-prop.py）：
REGRESSED 节空表（无任何属性域回归，白名单收紧预案未触发）；GAINED 四域
font-size-adjust 67 / transform 42 / max-{width,height} 6 / font-weight 3。
背景族 discrete-no-interpolation 预期 ~119 候选未兑现（引擎无该族 echo 输出，
采纳链 inert）——控制面 ~88G 挂账实际落在 transforms/values/fonts 三域。

### 账目三桶拆分与二进制对内容差法证（重要修正）

118 增益按子测采样位置 × 期望值形态程序化分类（`at (t)` 正则 + 名内
`should be [X]` 对 from/to 端点字面量），三桶：

| 桶 | 数 | 判据 | 归属 |
|---|---:|---|---|
| 外来钳制面 | **92**（transforms 39 / fonts 51 / values 2） | 采样 t≤0 或 t≥1（at(0)×36、at(-0.3)/at(-1)/at(-0.5)/at(1.5) 等越界/边界） | **不记本刀**——B0 tick 钳制位无发射形态，见下法证 4 |
| flip 离散直接 | **21**（transforms 3 / values 4 / fonts 14） | interior 采样且期望值 ≡ from/to 端点（离散对 `none↔20px`、metric 混排、matrix3d↔matrix） | **本刀直接效应**——eased 一致采纳设计目标面 |
| cadd 臂迟达 | **5** | interior 采样且期望值 ∉ 端点（`from [none] to add [2.0] at (0.5) should be [1.5]` 加性中点真插值） | **cadd 刀 animation.rs 臂效应**——flip 采纳仅端点恒等输出，中点真插值机制上不可能出自本刀 |

二进制对（bin-cadd-b1 17:43:41 / bin-flip-b1 18:35:24）内容差法证链：

1. **flip 自身排除（钳制位无罪）**：本刀 WIP 对 HEAD 仅 3 删行（import 行 +
   v2 eval fallback 块）＝注册流纯增量零触碰；eval 在 prog≤0/≥1
   （open_interval=false）新旧双侧恒等（fallback=false → 端点采纳不变）——
   钳制位翻转非本刀代码可致。
2. **shim 排除（逐字节）**：bin-cadd-b1 含 cadd 提交版 part03.js **完整
   verbatim blob**（2,004,418 字节 bytes 精确匹配 1 命中；python bytes.find
   全串比对）——对间 shim 差＝本刀 +2 注释行零行为。
3. **中点判据（fonts 5 例归 cadd 臂）**：flip 采纳门要求输出恒等端点；
   add 组合页中点期望 1.5 ∉ {0.5, 2.5}，需引擎 font-size-adjust 数值插值臂
   （cadd 三臂）产出——bin-cadd-b1 构建早于臂落树实锤（strings 判定失败：
   `{m}` 字面量 rustc 优化合并不落二进制双向 0 命中，以行为判据定谳）。
   cadd 轮账 +272 系 shim-only 效应，臂效应 +5 迟达于此对现形。
4. **外来钳制面归因（92 例，主候选 animation.rs 构建瞬态）**：registry v3
   端点钳制采纳两二进制相同，其生效前提是 tick 先产出端点输出——B0
   钳制位输出 "none" ＝ tick 在 prog≤0/≥1 无发射。矛盾排除法：注册配置同源
   （shim verbatim 同 + 注册流零触碰）⇒ 同输入必同 tick 输出，而观测异
   ⇒ **tick 域代码内容必异** ⇒ 主候选 animation.rs 17:43 树态含未提交
   tick 边界修改、提交（18:16）前回退——commit diff 对「未提交后回退」
   不可见；次候选 es_module s419（nm 符号集 76 vs 460 内容差实锤，但
   diff 全部输出锁 module 编译分支 `with (__zwImpH$)`/holder Proxy/deps
   fn，探针页 transform-interpolation-001.html 纯 classic script
   + testharness.js 无 module 路径）。92 例挂待归因账，由后继干净 A/B 对
   （cadd 线基线重建后）收口。
5. **护栏有效性不依赖归因**：0 lost / 0 regressed 全电池成立——无论外来差
   为何，B1 相对 B0 无任何子测劣化，flip 未砸面。

## 过程记录

- `systemctl is-active` 查 user 级 transient unit 漏 `--user` → 查到 system
  systemd 无此 unit 返 exit 4，误判「批中服务死亡」；实际 active 且腿账推进中。
  user unit 全部 systemctl 查询须带 `--user`。
- clippy `-p zero-engine` 被 peer script-sandbox 在途 WIP 的
  `clippy::too_many_arguments` 阻断（engine 依赖 script-sandbox，-D warnings 连坐）；
  改跑无 -D 形态过滤 engine 自域诊断（0 警告）取证，peer 域 lint 归 peer 流。
- `KeyframeBlock`（css-parser AST）无 offset 字段（offset 在选择器域
  `KeyframeSelector::Percentage/From/To`）——首版 `rule.keyframes[0].offset` 编译
  失败两处，改选择器域判定。顺带修正端点对归侧：不信任数组序，按选择器 0%/100%
  归 from/to。
- provenance 标记三试：const 不引用不落二进制 → static+allow(dead_code) 仍 0 命中
  （rustc 不 emit 未引用 static）→ part03.js 注释（include_str! 全文入二进制）
  1 命中 ✓（round-11 shim 标记先例同型）。
- shim 拼接校验正则 `part\d+\.js` 漏 part01b/01c/01d/01e/01f/01g/01h/02a/08a-c/
  08d/09b 后缀件（22 件真集）→ 拼接 blob 缺件 node --check 假红；修正为
  `part\w+\.js` 后 22 件 OK。
- 旁证流并发只读：批跑期间另有会话以 bin-cadd-b1 跑 css-fonts 腿（相对路径，
  独立 results/transcript 域），只读共享快照零冲突。

## 验证

- rustfmt ✓；clippy -p zero-engine --all-targets 自域 0 警告（无 -D 形态，peer
  WIP lint 连坐另账）；`cargo test -p zero-engine --lib anim` 123/123 ✓（含
  flip_eased_adopts 六新测：linear 侧别/0.5 边界归 to/step 钳/ease-in 中点/
  恒值无条件/异值防御）。
- corpus A/B：见上表与账目三桶拆分段（数字回填 + 分类器：`at (t)` 采样位置 ×
  名内 `should be [X]` 端点形态，composition 命名 `from replace [x]`/`to add [x]`
  前缀兼容，状态值 `Pass/Fail` 归一）。
- make test：见 master.md 门禁段（zw-flip-mkt2 拆腿形态，fmt 全局步因 peer WIP
  四文件漂移以本刀文件单核替代——js_anim_registry.rs rustfmt --check 单独绿）。
  19:50:11 收全 11 步：fmt 本刀文件 ✓ / companion ✓ / GPU adapter+consistency ✓ /
  renderer bin ✓ / quickjs renderer bin ✓；红腿 8 处全对号在册/peer 账（本刀域零红），
  唯一新面孔 gpu_compositor_path 双会话合计 solo ×11=8 绿 3 红（红侧=compositor
  通道关闭后 signal 11 断帧，门禁收后低载仍现）——纯负载相位解释存疑，分类归
  compositor 域后续查，不入本刀账（详见 master.md 门禁段）。
