# offset-position 属性刀——Normal/Auto/position 三形五站 + 4 值 edge-offset 共享扩展

日期：2026-11-15（十四轮，wptfyi-css；承接十三轮 offset 簇刀同日第二刀）
改动面（8 文件，均 Rust）：
`crates/style-system/src/property/{types,registry,inherit,default_impl,apply_advanced,computed_style}.rs`
+ `crates/engine/src/{animation.rs,js_dom_bridge/computed_style.rs}`。

## 机制要点

**目标面**：十三轮残面桶「offset-position 属性缺面 ~386」（interpolation 312
+ composition 56 + computed 15 + parsing-invalid 3 + shorthand 3 +
inheritance 2 + supports-calc 1）。刀前 offset-position 无模型：apply 拒、
gCS 无臂、白名单缺、WAAPI 链无臂。

**五站**（css-motion-1 §offset-position，非继承，初始 normal）：

1. **类型/注册**（types.rs/registry.rs）：`OffsetPositionValue`
   （Normal/Auto/Position(LengthValue, LengthValue)——LengthValue 非 Copy 故
   无 Copy derive，同 anchor Option 对先例）+ 白名单第五串 + 借
   Transform(None) initial 位（真实重置在 apply_initial_value）。
2. **apply**（apply_advanced.rs）：`parse_offset_position_value`
   normal|auto|`<position>`；<position> 复用 parse_origin_xy——none/角度/
   三词形自然拒收（parsing-invalid 三形零专用码）。
3. **初始/继承**（inherit.rs/default_impl.rs）：spec 初始 normal。
4. **gCS 计算值通道**（engine computed_style.rs）：序列化臂（normal/auto
   直串；position 对经 length_to_css，calc 形 canonical 化同通道）。
5. **WAAPI 插值链**（engine animation.rs）：underlying 臂（声明缺席落
   spec 初始 normal）+ 插值臂（position 对同单位线性；normal/auto 端点或
   跨形离散 ≥0.5 → to，WPT 关键字端点组期望离散翻转实测一致）+
   apply_single_property 写回臂（unsupported 形保持现值，anchor 同语义）。

**共享扩展（4 值 edge-offset）**：parse_origin_xy 新增 4 词分支
`<edge> <offset> <edge> <offset>`（css-shapes-3 §position）——首词定轴序，
offset 必 length-percentage，`right/bottom o` 语义 100% − o 以 Calc 形存储
（CalcExpr::BinaryOp(Percentage(100), Subtract, o)）；center 作 edge 拒、
同轴双 edge 拒。transform-origin/offset-anchor 全部同构消费方连带获得 4
词形（此前 anchor-interp 混合对的 `right 10px top 20%` 形同卡）。

**触面**：白名单 +1 串为纯增量（十三轮 typed-om/cssom 护栏同款复核）；
parse_origin_xy 4 词分支为纯新增分支（2 词路径不动）。

**本刀不做（残面后置）**：composition add/accumulate（56F，需 KeyframeEffect
composite 语义共享刀）、calc 混合形插值（`left 480px` → `right -140%`
端点组 48F，需 calc 代数插值）、offset 简写（3F）、supports-calc（1F，calc
形 gCS 序列化依赖）。

**css-motion-1 参考**：<https://drafts.fxtf.org/motion-1/#offset-position-property>；
<position> 文法 <https://drafts.csswg.org/css-shapes-3/#supported-basic-shapes>
（§position）；离散边界 css-values-4 §8.5.6。

## A/B 双面账（干净二进制重放，十三轮 stale-B0 法已入 learnings）

- B0 = `bin-posp-b0/zero-wpt-runner`（md5 62ae854f）：stash 八刀文件后于
  HEAD(6b84507fe) 重建（树载 peer es_module s419 WIP 同基座）。B1 =
  `bin-posp-b1/zero-wpt-runner`：WIP 树重建。
- 批（zw-posp-ab.service 23:42–23:49 + b1 侧补跑批 23:51–23:54）：motion
  + typed-om/cssom 白名单触面护栏 × {b0,b1}，test-guard 包裹，transcript
  唯一名，JSON 分目录 `results-posp-{b0,b1}`。

**首轮（B1 md5 4883a45d）**：+27/0/0 三页——anchor-interpolation +24、
anchor-computed +2、inheritance +1，**offset-position 四页零变化**（312/
56/15/3 恒同 B0）。法证：anchor 4 词形翻绿证 parse_origin_xy 扩展已入
B1，offset-position 面零动 = **白名单漏串**——registry.rs 有 initial 值
表与 `known_properties()` 白名单两处，本刀只改了 initial 表段
（`"offset-position"` 漏入白名单数组），CSS.supports decl_supported 门与
harness supports 检查全被卡在第一站。补串后重建 B1 重跑 B1 侧三腿（B0
侧快照不受影响，树态两处均无该串）。

**终轮（B1 md5 e2fe2a8d 重跑 b1 侧，23:51–23:54 批 zw-posp-b1-batch）**：

| corpus | gained | lost | regressed | pages_gain |
|---|---|---|---|---|
| motion | +108 | 0 | 0 | 5 |
| css-typed-om | 0 | 0 | 0 | 0 |
| cssom | 0 | 0 | 0 | 0 |
| **合计** | **+108** | **0** | **0** | **5** |

- offset-position-interpolation 0→68（312→244：纯 px 对/% 对 CSS
  Transitions 主通道线性 + auto 端点离散部分）；offset-position-computed
  0→13（15→2，supports 门全过含 calc 形——parse_origin_length_percentage
  Calc 臂连带；4 词形同过）；offset-anchor-interpolation 44→68 +
  offset-anchor-computed 10→12（4 词 edge-offset 共享扩展连带）；
  inheritance 4→5（initial normal + 不继承）。
- **白名单触面护栏零差**：typed-om/cssom 逐页零位移（known_properties
  +1 串纯增量十三轮结论复证）。

**残面桶（motion 页余 244F，下一刀候选）**：

- harness 全局关键字端点缺口 108（initial/unset/inherit 端点组
  'from' value should be supported——**跨 corpus 全局在册共享面**（此前
  定谳见十三轮 evidence），ZeroWeb-3 流 part05 在飞，本刀不触）。
- CSS Animations / Web Animations / transition:all 三通道组 ~90（纯 px/%
  对与 auto 组各通道整组 F，仅 CSS Transitions 主通道过）——**动画值进
  gCS 同步读链主矛盾面**（控制面地图校正③同源，上限远超本刀面）。
- calc 代数插值 46：`left 480px top 400px → right -140% bottom -60%` 22 +
  `left top → left 8px bottom 20%` 24——期望 calc 混合形序列化
  （`calc(420px + 30%)` 形），需混合单位 calc LengthValue 插值共享能力
  （anchor 混合对 268 / distance %/混合 166 同此前置）。
- edge-offset calc 折叠/简化 2（offset-position-computed 剩 2F：
  `right 10%` 期望折叠 `90%` 非保留 `calc(100% - 10%)`、`right -20%`
  期望 `120%`、`bottom 5em` 期望 em 绝对化 `calc(100% - 200px)`——
  canonical 化通道缺同侧 % 折叠与 font 绝对化）。
- offset-position-composition 56（add/accumulate，需 KeyframeEffect
  composite 语义共享刀）。
- offset-position-parsing-invalid 3（`none`/`30deg`/三词形 setProperty
  拒收——apply parse 已拒，残因在 harness shim 写路径 decl 存储层，texp
  域共享面）。
- offset-position-parsing-valid 12P 持平；offset 简写 3（需 offset 简写
  parse）；supports-calc 1（calc 形 gCS 序列化）。

**终轮（B1 = e2fe2a8d，白名单补串，zw-posp-b1r.service 23:51–23:54）**：

| corpus | gained | lost | regressed | pages_gain |
|---|---|---|---|---|
| motion | +108 | 0 | 0 | 5 |
| css-typed-om | 0 | 0 | 0 | 0 |
| cssom | 0 | 0 | 0 | 0 |
| **合计** | **+108** | **0** | **0** | **5** |

- offset-position-interpolation 0→68（312→244）：CSS Transitions 通道值
  插值可拿面全翻（px 对 7 + % 对 7 + `right 10px top 20%`→auto 离散 22 +
  auto→px 离散 22 + calc 端点原样 2 + neutral 8）。残 244 分桶：initial/
  unset/inherit 端点组 108（harness 全局关键字缺口，十三轮定谳跨 corpus
  形——属性模型解不动）、CSS Animations/Web Animations/transition:all
  通道组 ~90（动画值进 gCS 同步读链主矛盾）、calc 代数插值 46（`left
  480px`→`right -140%` 端点 calc 形）。
- offset-position-computed 0→13（15→2）：残 2 = calc 形 supports 检查
  （`calc(5em + 20%) center` / `calc(10px - 0.5em) calc(10px + 0.5em)`）——
  set 成功但 supports 读回门未过，随 calc 形 gCS 序列化面后置。
- offset-anchor-interpolation +24 / anchor-computed +2：4 词 edge-offset
  共享扩展连带（anchor 页同形卡点同解）。
- inheritance +1（offset-position initial normal）。
- 白名单触面护栏 typed-om/cssom 双轮（漏串轮 + 补串轮）逐字节零差。

**门禁**：make test 拆腿（zw-posp-mkt.service）分腿账见下节。

## 单测

- style-system `offset_position_forms`：normal/auto/px 对/px% 对/4 词
  edge-offset Calc 形/calc 形/invalid 三形拒收。
- engine `test_offset_position_interp_and_discrete`（同单位线性 + auto 端点
  离散）+ `test_offset_position_apply_roundtrip_underlying`（写回往返 +
  normal 形）。
- `cargo test -p zero-style-system --lib offset_props` 5 通过；
  `cargo test -p zero-engine --lib test_offset` 7 通过。FMT-SCOPE 八文件
  --check ✓（--edition 2024 + config-path）；clippy 两 crate 净（唯一
  warning = peer es_module too_many_arguments 在册连坐）。

## 门禁（make test 拆腿形态，zw-posp-mkt.service，23:55–00:14+）

拆腿脚本镜像十三轮 zw-offset-mkt3 形态（全局 fmt 步按 composite-add/flip
先例拆出，peer es_module s419 WIP fmt 漂移阻断；本刀八文件 FMT-SCOPE
--check ✓）。companion-build 温缓存 28s；分腿账：

| 腿 | rc | 红项 |
|---|---|---|
| companion-build | 0 | |
| workspace-test | 101 | es_module 6 WIP 红（peer s419 在途名单与十三轮逐名恒同），441P/6F |
| quickjs-clippy | 101 | `too_many_arguments(8/7)` es_module.rs:2217（peer WIP 连坐，在册） |
| renderer-bin | 0 | |
| renderer-lib | 101 | worker-fetch renderer 形 1F（在册 c6dcbce0a 镜像），371P/1F |
| browser-bin | 101 | tab worker-fetch + default_actions 双红（在册），741P/2F |
| browser-tests | 101 | 同对（在册），741P/2F |
| gpu-adapter / gpu-consistency | 0/0 | |
| quickjs-workspace | 101 | image_batch_e2e 负载形（在册伪红，775P/1F 20.00s） |
| quickjs-renderer-bin | 0 | |
| quickjs-renderer-lib | 101 | worker-fetch quickjs 形（在册） |

**本刀域零红**：workspace 腿 441P/6F 的 6F 全为 peer es_module 名单；
本刀 5+7 钉在 dev profile 定向腿全绿（A/B 前置已验）；红腿名单与十三轮
zw-offset-mkt3 账逐名恒同（零新增红、零红项位移）。
