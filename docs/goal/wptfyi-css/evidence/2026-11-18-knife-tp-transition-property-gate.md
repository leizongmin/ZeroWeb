# 刀 TP：transition-property 写面文法门（白名单 65 + 单分派臂）

日期：2026-11-18。认领：CLAIM-knife-tprop-invalid-gate.md（21:3x 预研
落盘）；执行窗：R2 落库 `2fffd1d5f` 后开启（22:1x）；执行流：knife-r2
CLAIM 位 fork（R2 收口让位后同流接续）。

## 面与机制

`transition-property` 白名单接入（VALIDATED_PROPERTIES 64→65）+ 单分派臂
（transition-timing-function 臂后、shape-margin 臂前）：

- 文法：`none | <single-transition-property>#`（css-transitions-1
  §transition-property-property）；单项 = `all | <custom-ident>`，逗号
  分层列表，括号感知切分（`split_top_level_commas`）。
- 拒收集：css-wide keyword（initial/inherit/unset/revert/revert-layer）
  与 `default` 作列表成员（整值形由上游 :2572-2575 Cascading-4 §7.3
  截获放行，成员混入不可达该单值匹配）、`none` 混列表、单项空白分隔多
  token（`one two three`）、数字开头形（`1, 2, 3`）＝invalid 页 15 形
  逐形覆盖。
- `<custom-ident>` 识别文法近似：`transition_property_ident_valid`
  （起始 [a-zA-Z_]/非 ASCII，余 [a-zA-Z0-9_-]/非 ASCII；转义序列 `\`
  形放行保底——counter-increment 臂同口径，FIXME 全谱挂账）。
- 接受集：none 整值单置 / all / custom-ident 任意大小写保形
  （`ALL, INVALID, SYNTAX, SRC`）/ 逗号 idents 列表 / `--custom-prop`。
- 替换函数形（var()/attr()）放行 store 原文（R2 四臂同口径）。
- shorthand `transition` 不在白名单（级联层展开），零涉。

## 语料 DOA 预核（v2 电池 b0=HEAD-R2 态逐值实证，22:1x）

union v2 电池 `results-knife-r-union-v2/css-transitions.b0.json`（R2
落库后、TP 前，门态精确）：

- transition-property-invalid：**0P 15F**——15 形与拒收集逐形吻合
  （one two three / 1,2,3 / none,one / css-wide 与 default 双序 ×12）。
- transition-property-valid：**7P 0F** 全在接受集；computed：2P 0F；
  transition-property-001：5P 0F（inline CSS 面，不经 CSSOM 门）；
  002：6P 1F（`parse 'inherit'` 属 parse API 面非门车道，保持 F）。
- 零 DOA、零 P2F；账面口径 **+15F**。

## 调用面（P2F 风险域核查）

`validate_decl_value` 生产调用三站：typed-om 桥（callbacks.rs）、
CSSStyleDeclaration setProperty/cssText（css_style_declaration.rs）、
WAAPI keyframes（js_anim_registry.rs）——均接受集内既有值形态，零新增
拒收交叉。inline style 属性解析不经门。

## 门禁（22:2x）

- `cargo fmt -p zero-style-system -- --check` 绿（白名单条目 + 臂 +
  helper + 钉测四笔）。
- `test-guard` 包裹 `cargo clippy -p zero-style-system --all-targets`
  绿（5.29s，E0308 白名单计数 64→65 修正一笔后复跑）。
- 钉测 `knife_tp_write_gate_verdicts` **首执即绿**（1 passed 0 failed，
  2569 filtered）——DOA 预核兑现（钉测未执过=首执 DOA 预防闭环第二例，
  R2 后连续）。

## 电池（22:2x 发射，账后回填）

systemd unit `zw-knife-tp-battery-2227b`；脚本
knife-tp-battery.sh（沿 knife-r-union 配方：b0=HEAD apply.rs / b1=HEAD+
TP face 双建 + M2 md5 异同性证 + ride-along s1/s2 双侧相消法证——在飞
他流件 parse_basic_shape/parse_extended_border（calc 修笔）/
es_module/part115（s419/s423）两侧同态）。腿：css-transitions（主靶
transition-property 页）× b0/b1 + css-properties-values-api（哨兵：
typed-om/CSSOM setter 面跨回归）× b0/b1。

首射 `zw-knife-tp-battery-2227` rc=126（脚本缺执行位），chmod 后重射
`zw-knife-tp-battery-2227b`：双建 22:27–22:29（M2 双 bin md5 必异，b1
`e10e991f2909c1a5816a80e2a59bfd68`；ride-along s1/s2 双侧同态），四腿
22:29:57→22:47:00 GATE-DONE，BATTERY-RC=0 零 LEG-FAIL。

**A/B 纯度窗法证**：双 bin 建成于 22:29 前；巡检笔 `0a3d1e4d7`
（22:35:24 落地，唯一源码件 webview.rs +8）与 animation WIP（clip lerp，
mtime 22:30–22:31）均在构建窗后，未入任一 bin——b0/b1 仅差 apply.rs
TP face。

## 电池账（22:47 收账）

| 语料 | F2P | P2F | flip |
|---|---|---|---|
| css-transitions | +16（原始） | 0 | 561 |
| css-properties-values-api（哨兵） | 0 | 0 | 0 |

分账与归因：

- **主靶 transition-property-invalid：+15 −0 flip0，认领口径精确兑现**
  （DOA 预核 15 形逐形对齐，零 P2F、零 DOA）。
- transitioncancel-003 +1 F2P＝flake 不入刀账：页源零
  `transition-property` 引用，全程 `transition` 简写（inline style +
  CSSOM 简写赋值，不在白名单、级联层展开零涉门车道）；5000ms
  duration/delay 时序敏感页，归既有异步漂移家族。
- properties-value-inherit-002 flip561＝b1 侧页级蒸发向漂移：b0 561
  子测（560F+1 Timeout）→ b1 仅 1 子测（560 Fail→None 蒸发 +1
  Timeout→Fail），err 双侧全空零崩溃签名；双侧 0 Pass，ΔP 中性，不入
  F2P/P2F 账。该页测 css-wide 继承语义，非 setter 拒收面。
- pva 哨兵四腿 0/0/0：typed-om/CSSOM setter 面跨回归零位移。

**定谳：刀 TP 账 ＝ css-transitions +15F 零 P2F + 哨兵全净**（corpus
原始 F2P +16 内含 1 flake，不计入刀账）。
