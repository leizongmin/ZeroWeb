# 刀 Z：z-index 整数插值三通道接入（2026-11-18）

## 靶面与根因

- 主靶 `css/css-transitions/animations/z-index-interpolation.html`：刀 A1d 收账基线 166F/250（b0=results-knife-a1d-v2 css-transitions 365P/1159F）。
- 根因＝z-index 全通道零覆盖：`TRANSITIONABLE_PROPERTIES` 无条目（transitions 不启动）、animation.rs lenient/strict/applier 三臂全缺——整数插值回落 to-echo（样例 from[-2]→to[5] at(0) got "5" 含 t0 伪 through，B0 伪通过池同形）。
- 另 `z-index.html` 4F（`Can set 'z-index' to a number` → `Invalid unit for property`）＝part10-typed-om.js:1768 CSSStyleValue set 门 `<integer>` 缺表——刀 P 残面池（P2 池）追加，不入本刀。

## 码面（2 文件净 +61/−2）

- `crates/engine/src/js_dom_bridge/transition_registry.rs`：`TRANSITIONABLE_PROPERTIES` 79→80 加 `z-index`（strict 门 auto/小数端点 None 不登记，静态跳终值）。
- `crates/engine/src/animation.rs`：
  - lenient 臂：两端 `<integer>`（`parse::<i64>` 严格，接受 +/- 号与 css-syntax 符号法一致）→ lerp → **round half towards +∞**（css-values-4 §8.1.2；f64 `.round()` 是 half-away-from-zero，-0.5→-1 不符，用 `(v+0.5).floor()`）；auto/小数端点离散回落（css-values-4 §8.5.6）。
  - strict 臂：整数同型门（css-transitions-1 §animations z-index as `<integer>`）。
  - applier 臂：离散 echo 串落 `ZIndexValue` 槽，溢出 i32 保旧值（css-position-3 §z-index）。
  - 钉测 `test_z_index_lerp` 10 断言随码（engine lib test 断编译挂账对端 part116 include! 悬空，补跑随树清窗）。

## 四腿 A/B（b0=results-knife-a1d-v2；b1=bin-knife-z md5 `41cfb19f`，同树同 provenance——fv1 在飞 WIP 两侧同扫）

| 腿 | b0 | b1 | delta |
|---|---|---|---|
| css-transitions | 365P/1159F | 529P/995F | **+164/−164 零 lost 零 REG** |
| css-masking | 4701P/1654F | 4703P/1652F | +2/−2 零 lost |

- **本刀净账 +158**：`z-index-interpolation.html` 84→242P（166F→8F）。
- 混窗增益窗口法证归 fv1 已落库码（b0 bin 早于其落库）：transitions 腿 `transition-timing-function-valid` +4（刀 U timing canonical）、`transition-property-valid` +1（刀 U property 归一小写）、`all-interpolates-same-as-explicit-property` +1；masking 腿 `clip-path-shape-parsing` +2（刀 T arc small/rotate 缺省补刀）。刀 Z diff 对上述页面机制零触及（grep 证）。
- **残 8F 全 WAAPI 通道**（残面池挂账）：`from [auto] to [10]` 离散翻转 7F + `from neutral to [5] at (1)` 1F——WAAPI js_anim 通道 z-index 端点处理（同 A1d WAAPI bg-position 列表通道挂账模式）。

## 门禁

- `cargo check -p zero-engine --release` 过（5.77s）；本刀 hunks fmt 清（animation.rs 余 12 处 fmt diff 全刀 B1 已落库区，不代修）。
- runner release bin 不受 engine lib test 断编译影响（part116 include! 悬空属 fv1 part117 迁移半程）。

详账见 [master](../master.md) 刀 Z 段。
