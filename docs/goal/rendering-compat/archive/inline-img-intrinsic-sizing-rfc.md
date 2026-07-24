# Spec/RFC：inline>inline-IMG 固有尺寸打通（IFC measure path 缺 img intrinsic ratio 访问）

**版本**：v1.0
**日期**：2026-07-17
**作者**：rally agent
**状态**：已确认（rally 续跑，承接 R1577 forward）
**谱系**：R1576（inline-box-model 递归 LANDED）→ R1577（深诊断）→ 本文（实施）→ R109/inline-box-model 高风险主线

---

## 0. 执行摘要

- **一句话目标**：让 IFC measure path 能访问 `<img>` 的固有尺寸（intrinsic size），使「inline 元素（`<a>`/`<span>`）包裹 auto-width img」不再塌缩为高度 0（wintertc footer `<p><a><img class="h-6 inline-block" src="logo.svg"></a></p>` 真实产品 bug）。
- **本期范围**：仅打通 **measure path**（`measure_text_content` 驱动的 IFC）对 `img_intrinsic_sizes` 的访问 + `collect_inline_items` img 分支按固有宽高比推导缺失维度。第一切片默认关闭、env-gated `ZW_IFC_IMG_INTRINSIC`，全量 13 dir A/B + product-smoke 守 net≥0。
- **明确排除**：(1) paint/final path（`compute_final_inline_layouts`/postprocess）同步——作第二切片，仅当 A/B 揭示 measure-vs-final 发散时才做（R227 教训）；(2) `apply_replaced_element_sizing`（tree.rs，final-layout path 已正确处理 both-abs + 显式高 + auto 宽，经 R1577 核查）；(3) vertical writing-mode（R109-blocked，gate `!self.vertical` 沿用 R1576）。
- **核心约束**：① 默认关闭 kill-switch + 全量 A/B；② 仅当 img 恰有一侧已知维度（w>0⊕h=0 或 w=0⊕h>0）且在 `img_intrinsic_sizes` 中时才推导；③ 零回归（13 dir + product-smoke welcome<20%）；④ R1492/R1494 inline-ownership reverted 先例 → 须 load-bearing 单测。
- **推荐方案**：IFC struct 增 `img_intrinsic_sizes` 字段（镜像 `inline_block_sizes` 模式，默认空 map 零回归）+ measure path 3 闭包点捕获 `&img_intrinsic_sizes` 传入 + img 分支 guarded 推导。
- **首个落地步骤**：先加 IFC `img_intrinsic_sizes` 字段 + `with_img_intrinsic_sizes` builder + img 分支 guarded 推导（env-gated，default-off），编译 + 单测验证推导逻辑；再 plumbing 进 measure_text_content 3 闭包点。

---

## 1. 背景与目标

### 1.1 背景（R1577 诊断复述）

R1576 LANDED 了 `collect_inline_items` 对普通 inline 元素（`<a>`/`<span>`）的递归，使 IFC 能进入 inline 元素收集嵌套 atomic inline 后代。但**剩余**：当 inline 元素包裹一个 **auto-width** `<img>`（仅显式 height，如 `class="h-6"` = `height:1.5rem`），容器仍塌缩 h=0。

**根因（R1577 5-case 矩阵 A-E 隔离 + 代码核查确认）**：

| case | 结构 | `<p>` height | 说明 |
|------|------|-------------|------|
| A | `<p><a><img h-only inline-block></a></p>` | **0 塌缩** | inline 包裹 auto-w img |
| B | `<p><img h-only></p>` | 24 OK | img 直接子（block taffy 节点，`<p>` 保 taffy 高） |
| C | `<p><a><img w56 h24></a></p>` | 24 OK | 显式宽 → IFC img 分支收集成功 |
| D | `<p><a>text</a></p>` | 18.6 OK | text IFC item |
| E | `<p><span><img h-only></span></p>` | **0 塌缩** | 同 A |

塌缩**专属**于「inline 元素包裹 auto-width img」。机制：

1. `<a>`（display:Inline 但有 Element 子 `<img>`）→ `build_subtree` all_inline 检查 `no_elem_child=false` → `<a>` 作 taffy 节点（`new_with_children`），CSS display:Inline。
2. taffy 把 `<a>` 作 `<p>` 的 inline 内容，经 **measure callback**（`measure_text_content`，`inline_finalization.rs:947`）测高度。
3. measure path IFC（`measure_text_content` 内 `InlineFormattingContext::new`，`:1098`）的 `inline_block_sizes`（`ib_sizes`，`:1059`）**只收集直接 InlineBlock 子**（`:1068` gate `DisplayValue::InlineBlock`），且从 CSS computed 解析（非 LayoutBox）。
4. R1576 递归让 IFC 进入 `<a>` 收集嵌套 `<img>`，但 img 分支（`inline/mod.rs:929-988`）依次回退：HTML attr（无）→ CSS computed（h=24, w=auto→0）→ CSS %（无）→ `inline_block_sizes`（measure path 无此 img 条目）→ **w=0**。
5. img 分支要求 `w>0 && h>0`（`:989`）才产 item → **不收集** → IFC 0 item → `<a>` measured h=0 → `<p>` h=0。

**关键**：`<p>` h=0 来自 **taffy measure path**（非 `compute_final_inline_layouts` postprocess——后者 530-945 全函数无 `.height=` 写）。而 final-layout 阶段 img 经 `apply_replaced_element_sizing`（`tree.rs:413-465` both-abs 分支）正确设 `aspect_ratio` + 从显式 height 推导 width（`width = ch × eff_ratio`），获 taffy 节点正确尺寸——**measure-vs-final 发散**：measure 给 `<a>`/`<p>` h=0，final 给 img 正确尺寸。

### 1.2 目标

- **产品目标**：wintertc footer `<p>` 不再塌缩（h≈24px，img w≈56px），matrix logo 可见。`check_collapsed_containers` 可入 product-smoke gate。
- **reftest 目标**：css-position（nested-inline-abspos-child-with-siblings / position-absolute-dynamic-relayout-004）+ css-tables（display-contents / fixup-anonymous-inline-table 4 案）塌缩类改善/flip。
- **架构目标**：打通 IFC measure path 对 img intrinsic 的访问，为后续 paint-sync（第二切片）奠基。

### 1.3 范围边界

- **在范围内**：IFC `img_intrinsic_sizes` 字段 + builder；`collect_inline_items` img 分支 guarded 推导（env-gated）；`measure_text_content` + 3 闭包点 plumbing；load-bearing 单测。
- **不在范围内**：paint/final path 同步（第二切片，仅发散时做）；`apply_replaced_element_sizing` 改动（已正确）；vertical writing-mode（gate `!vertical`）；inline 元素 padding/border bleed per-fragment（独立 RFC `per-fragment-inline-border-padding-bleed-rfc.md`）。

---

## 3. 功能需求

### FR-001：IFC 持有 img_intrinsic_sizes 字段（dormant infra，零回归）
- **描述**：`InlineFormattingContext` struct 增 `pub img_intrinsic_sizes: HashMap<NodeId, (f32, f32)>` 字段（镜像 `inline_block_sizes`），`new()` 初始化空 map，`with_img_intrinsic_sizes(sizes)` builder。
- **优先级**：必须
- **实现来源**：仓内自实现，`crates/layout-engine/src/inline/mod.rs`（复用既有 `inline_block_sizes` 模式 `:100`/`:233`/`:290`）。
- **验收场景**：
  - 场景(dormant)：假设 env `ZW_IFC_IMG_INTRINSIC` 未设（default-off），当 渲染任一既有页面，那么 行为字节不变（img 分支 guarded 推导不触发），验证：全量 reftest 13 dir 与 baseline 字节一致。
  - 场景(builder)：假设 构造 IFC 并 `.with_img_intrinsic_sizes(map)`，当 读取 `ctx.img_intrinsic_sizes`，那么 返回传入 map，验证：单测 `test_ifc_img_intrinsic_sizes_builder_roundtrip`。

### FR-002：collect_inline_items img 分支按固有宽高比推导缺失维度（env-gated）
- **描述**：img 分支（`inline/mod.rs:929-988`）在所有现有回退（HTML attr / CSS computed / CSS % / `inline_block_sizes`）后，当 env `ZW_IFC_IMG_INTRINSIC` 开启且恰有一侧维度已知（`w>0 && h<=0` 或 `w<=0 && h>0`）且 img 在 `img_intrinsic_sizes` 中时，按固有宽高比推导缺失侧：`h<=0 → h = iw × w / iw... `（精确见 RFC §8.4）。
- **优先级**：必须
- **实现来源**：仓内自实现，`crates/layout-engine/src/inline/mod.rs` img 分支。
- **验收场景**：
  - 场景(显式高推导宽)：假设 img intrinsic=(75,32)、CSS height=24、width=auto、env ON，当 collect_inline_items 处理该 img，那么 w=56.25 (=75×24/32)、h=24 → 产 InlineBlock item，验证：单测 `test_img_branch_derives_width_from_intrinsic_ratio`。
  - 场景(显式宽推导高)：假设 img intrinsic=(75,32)、CSS width=40、height=auto、env ON，当 处理该 img，那么 w=40、h=17.07 (=32×40/75)，验证：同单测对称分支。
  - 场景(env OFF 不推导)：假设 env `ZW_IFC_IMG_INTRINSIC=0`，当 处理 auto-w img，那么 w 仍=0 → 不产 item（塌缩行为不变），验证：单测 env=0 时 `must grow` assertion FAIL。
  - 场景(两侧都未知不推导)：假设 img intrinsic 存在但 CSS w/h 都 auto 且无 attr，env ON，当 处理该 img，那么 不推导（避免与 `apply_replaced_element_sizing` 的 default-object-size 300×150 冲突），交由 final path，验证：单测两 auto 时不改 w/h。
  - 场景(img 不在 intrinsic_sizes)：假设 img 无解码尺寸（缺失资源），env ON，当 处理，那么 不推导（无 ratio），验证：单测 missing-intrinsic 时行为不变。

### FR-003：measure path IFC 注入 img_intrinsic_sizes
- **描述**：`measure_text_content`（`inline_finalization.rs:947`）增 `img_intrinsic_sizes: &HashMap<NodeId,(f32,f32)>` 参数；其内 IFC 构造（`:1098`）链 `.with_img_intrinsic_sizes(img_intrinsic_sizes.clone())`。3 个 measure 闭包点（`engine.rs:175`/`:232`/`:616`）捕获 `&img_intrinsic_sizes` 并传入。
- **优先级**：必须
- **实现来源**：仓内自实现。`img_intrinsic_sizes` 已存在于 `compute_with_img_intrinsic`（`engine.rs:140`）作用域，且 `:146` 已 clone 为 `intrinsic_for_r695`（r695 用）——plumbing 复用该 clone 或新增 clone 传入闭包。
- **验收场景**：
  - 场景(measure 注入)：假设 wintertc footer `<p><a><img h-6 inline-block></a></p>`、env ON，当 taffy measure `<a>` 调 measure_text_content，那么 IFC img 分支用 img_intrinsic_sizes[matrix.svg]=(75,32) 推导 w=56 → `<a>` measured h≈24，验证：单测 `test_measure_text_content_uses_img_intrinsic_for_nested_img` + product-smoke wintertc footer `<p>` h>0。
  - 场景(参数传递不变行为)：假设 env OFF，当 3 闭包点传 `&img_intrinsic_sizes`，那么 img 分支仍不触发（FR-002 env gate），行为字节不变，验证：13 dir reftest baseline 一致。

### FR-004：vertical writing-mode 排除（gate !self.vertical）
- **描述**：img 分支 guarded 推导仅当 `!self.vertical` 时触发（沿用 R1576 gate，保护 vertical-rl/lr R109-blocked territory）。
- **优先级**：必须
- **验收场景**：假设 vertical-rl/lr 容器内 inline>img，env ON，当 处理，那么 不推导（gate 拦截），writing-modes dir 零回归，验证：writing-modes 135=135 baseline。

### FR-005：load-bearing 单测
- **描述**：新增 ≥1 单测，env ON 时 PASS、env OFF（`ZW_IFC_IMG_INTRINSIC=0`）时 FAIL，证 fix 真 load-bearing（沿用 R1576 `test_collapsed_containers_detects_inline_wrapping_inline_block` 模式）。
- **优先级**：必须
- **验收场景**：单测 ON PASS / OFF FAIL。

---

## 6. 约束与假设

### 6.1 必须约束（Must）
- 全量 13 dir reftest-oracle A/B（同二进制 env 切换）NET ≥ 0，无 pass→fail flip。
- product-smoke welcome < 20%（DC-13 gate）+ wintertc/morning-work struct-check 全 PASS。
- env-gated `ZW_IFC_IMG_INTRINSIC`（default-off），kill-switch 可瞬时回退。
- 每一改动可追溯到 FR；load-bearing 单测覆盖 FR-002 推导逻辑。

### 6.2 禁止约束（Must Not）
- 不改 `apply_replaced_element_sizing`（tree.rs，final path 已正确）。
- 不改 paint/final path（`compute_final_inline_layouts`/postprocess）——第二切片才做，仅 A/B 揭示发散时。
- 不对 vertical writing-mode 启用推导（gate `!self.vertical`）。
- 不在两侧维度都未知时推导（避免与 300×150 default-object-size 冲突）。

### 6.3 已定决策
- IFC 字段模式镜像 `inline_block_sizes`（`:100`/`:290`），不引入新抽象。
- 推导用 CSS aspect-ratio 优先（`computed.aspect_ratio`），否则固有 w/h 比（与 `apply_replaced_element_sizing:436` `eff_ratio` 一致）。
- env-gated default-off（高风险，R1492/R1494 inline-ownership reverted 先例）。

### 6.4 技术约束
- `measure_text_content` 是 free function（taffy measure callback），不能持状态 → 经参数 + 闭包捕获注入。
- `img_intrinsic_sizes` 在 `compute_with_img_intrinsic` 被 move 进 `build_layout_tree_with_r109`（`:153`）前已 clone `intrinsic_for_r695`（`:146`）→ 闭包捕获该 clone（或新增 clone）。
- 3 个闭包点（`engine.rs:175`/`:232`/`:616`）须同步改，避免 measure 不一致。

### 6.5 假设
- 假设1（已验证，R1577）：`<p>` h=0 来自 measure path，非 compute_final postprocess。
- 假设2（已验证，本文 §1.1）：`apply_replaced_element_sizing` both-abs 分支正确处理 final path img sizing。
- 假设3（待 A/B 验证）：measure path 单独打通即可解塌缩（final path 已正确，无需 paint-sync）。若 A/B 揭示 final path 仍发散 → 第二切片补 `compute_final_inline_layouts`/postprocess 的 img_intrinsic_sizes 注入。

### 6.6 代码变更边界
- **允许修改**：`crates/layout-engine/src/inline/mod.rs`（IFC struct + builder + img 分支）；`crates/layout-engine/src/inline_finalization.rs`（measure_text_content 签名 + IFC 构造链）；`crates/layout-engine/src/engine.rs`（3 闭包点）；`crates/layout-engine/src/engine/tests/tests_9.rs`（measure_text_content 3 测试调用签名对齐）；新增/改单测文件。
- **禁止修改**：`crates/layout-engine/src/tree.rs::apply_replaced_element_sizing`（已正确）；paint path（`compute_final_inline_layouts`/postprocess）——第二切片；vertical 相关推导。

### 6.7 执行技能提示
| 范围 | Skill | 模式 | 原因 |
|------|-------|------|------|
| 渲染/布局变更验收 | `make reftest`/`make product-smoke` | required | run-rules 强制 test-guard 包裹 + DC-13 gate |

---

## 7. 实施交接（Implementation Handoff）

### 文件/模块清单

| 路径/模块 | 动作 | 目的 | 风险 |
|----------|------|------|------|
| `crates/layout-engine/src/inline/mod.rs` | 改 | IFC `img_intrinsic_sizes` 字段 + `with_img_intrinsic_sizes` builder + img 分支 guarded 推导 + `!self.vertical` gate | 中（img 分支推导逻辑） |
| `crates/layout-engine/src/inline_finalization.rs` | 改 | `measure_text_content` +`img_intrinsic_sizes` 参数；IFC 构造 `:1098` 链 builder | 低（机械 plumbing） |
| `crates/layout-engine/src/engine.rs` | 改 | 3 measure 闭包点（`:175`/`:232`/`:616`）捕获 + 传参 | 低 |
| `crates/layout-engine/src/engine/tests/tests_9.rs` | 改 | 3 处 `measure_text_content` 测试调用签名对齐（`:502`/`:549`/`:565`）+ `&HashMap::new()` | 低 |
| `crates/layout-engine/src/inline/...tests` 或新测 | 新增 | FR-002/FR-005 load-bearing 单测 | — |

### 推荐修改顺序

1. **IFC infra（dormant，先零回归基线）**：`inline/mod.rs` 加字段 + builder（默认空 map）+ img 分支 guarded 推导（env-gated，default-off）。→ 验证：`cargo test -p zero-layout-engine` 全绿（dormant 不触发）。
2. **img 分支单测**：写 FR-002 推导单测（ON PASS / OFF FAIL），`cargo test` 验证推导逻辑正确。
3. **measure path plumbing**：`measure_text_content` 加参数 + IFC 构造链 builder；`engine.rs` 3 闭包点捕获 `&intrinsic_for_r695`（或新增 clone）传参；`tests_9.rs` 3 调用签名对齐。→ 验证：`cargo build --workspace` + `cargo test -p zero-layout-engine` 全绿。
4. **product-smoke 单案**：`make product-smoke` wintertc footer（env ON vs OFF）确认 `<p>` h>0 flip。
5. **全量 A/B**：`make reftest`（env ON vs OFF 同二进制）13 dir，记录 NET + flip。
6. **裁决**：NET ≥ 0 + 零 pass→fail + welcome <20% → default-on land；否则 revert + 记 evidence。

### 首批提交建议

| 提交 | 范围 | 预期结果 | 验证 |
|------|------|----------|------|
| Commit 1 | FR-001~005 全量（env-gated default-off）+ 单测 + measure plumbing | wintertc footer `<p>` 不塌缩 / 13 dir NET≥0 | `make test` + `make reftest` A/B + `make product-smoke` |
| (条件) Commit 2 | default-on 翻（移除 default-off，kill-switch 保留） | NET≥0 确认后翻 default | 全量复测 |

---

## 8. 技术设计（RFC）

### 8.1 现状分析
- IFC struct（`inline/mod.rs:100`）有 `inline_block_sizes`，无 `img_intrinsic_sizes`。
- img 分支（`:929-988`）回退链止于 `inline_block_sizes`，要求 `w>0 && h>0`（`:989`）才产 item。
- measure path `measure_text_content`（`inline_finalization.rs:947`）的 `ib_sizes`（`:1059`）只收直接 InlineBlock 子，不递归、不含 img intrinsic。
- `img_intrinsic_sizes` 已存在于 engine（`engine.rs:140`）+ tree.rs BuildContext（`:283`），`apply_replaced_element_sizing` final path 已正确用之，但 **IFC measure path 无访问**。

### 8.2 目标状态
IFC measure path 经 `img_intrinsic_sizes` 访问 img 固有尺寸，img 分支对「恰一侧已知」的 auto-w/h img 按固有比推导，使嵌套 img 参与 line box 高度计算 → 父容器不塌缩。

### 8.4 详细设计

**img 分支 guarded 推导伪代码**（插入 `inline/mod.rs:988` `if w>0 && h>0` 之前）：

```text
# 在所有现有回退（HTML attr / CSS / % / inline_block_sizes）之后
if env ZW_IFC_IMG_INTRINSIC on
   && !self.vertical                         # FR-004 vertical gate
   && (w > 0.0) != (h > 0.0)                 # 恰一侧已知（FR-002 异常路径）
   && let Some(&(iw, ih)) = self.img_intrinsic_sizes.get(&child_id)
   && iw > 0.0 && ih > 0.0:
    let eff_ratio = styles.get(&child_id)
        .and_then(|s| s.aspect_ratio)        # CSS aspect-ratio 优先（与 tree.rs:436 一致）
        .unwrap_or(iw / ih);
    if w > 0.0 && h <= 0.0:                  # 显式宽推导高
        h = w / eff_ratio;
    else if h > 0.0 && w <= 0.0:             # 显式高推导宽（wintertc：w=75×24/32=56.25）
        w = h * eff_ratio;
# 之后照常 if w>0 && h>0 → 产 InlineBlock item
```

**measure path plumbing**（`measure_text_content` 签名 + 闭包）：

```text
# inline_finalization.rs:947 签名增参
fn measure_text_content(
    doc, styles, dom_id, known_dimensions, available_space,
    img_intrinsic_sizes: &HashMap<NodeId, (f32, f32)>,   # 新增
) -> Size<f32> { ...
    # :1098 IFC 构造链
    let mut inline_ctx = InlineFormattingContext::new(width)
        .with_vertical(is_vertical)...
        .with_inline_block_sizes(ib_sizes)
        .with_img_intrinsic_sizes(img_intrinsic_sizes.clone());  # 新增
}

# engine.rs 3 闭包点（:175/:232/:616）
|known_dimensions, available_space, _node_id, context, _style| {
    let dom_id = ...;
    measure_text_content(doc, styles, dom_id, known_dimensions, available_space,
                         &intrinsic_for_r695)   # 捕获 engine scope 的 clone
}
```

**注**：`intrinsic_for_r695`（`engine.rs:146` 的 clone）在 `img_intrinsic_sizes` 被 move（`:153`）后仍存活，闭包可直接捕获其引用。若 borrow 冲突，新增独立 clone。

### 8.5 风险与缓解
- **风险1（measure-vs-final 发散）**：measure 给 img 推导 w=56，final path 是否一致？final path 经 `apply_replaced_element_sizing` 已正确推导（R1577 核查），故应一致。**缓解**：A/B 观察 wintertc footer img 几何；若发散 → 第二切片补 final path。
- **风险2（广泛影响所有 auto-w + 显式-h inline img）**：推导改变所有此类 img 的 line box 高度。**缓解**：env-gated + 全量 13 dir A/B + product-smoke 守；13 dir NET≥0 即留。
- **风险3（R1492/R1494 inline-ownership reverted 先例）**：inline 元素子树所有权改动高回归。**缓解**：本切片不改所有权（R1576 已改递归），仅补 img intrinsic 访问；scope 最小。
- **风险4（`tests_9.rs` 3 调用签名）**：`measure_text_content` 加参破 3 测试。**缓解**：传 `&HashMap::new()`（无 intrinsic → 推导不触发 → 测试行为不变）。

### 8.8 测试策略
- **单元测试**：FR-002 推导逻辑（显式宽/高对称、两侧 auto 不推导、missing-intrinsic 不推导、env OFF 不推导、vertical 不推导）；FR-005 load-bearing（ON PASS/OFF FAIL）。
- **集成测试**：`measure_text_content` 嵌套 img 测量返回非零高度。
- **A/B 回归**：`make reftest` 13 dir env ON vs OFF（同二进制）+ `make product-smoke`（welcome/wintertc/morning-work）。

### 8.9 回滚计划
env `ZW_IFC_IMG_INTRINSIC=0`（kill-switch）瞬时回退（default-off 则天然回退）。若已翻 default-on，回 commit 移除 default 翻转，保留代码 + kill-switch。

---

## 9. Spec Lint 报告（自检）

### 结构完整性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要 | ✅ Pass | §0 含目标/范围/排除/约束/方案/首步 |
| 场景存在性 | ✅ Pass | FR-001~005 均有 ≥1 验收场景 |
| 异常路径覆盖 | ✅ Pass | FR-002 含 env-OFF/两侧-auto/missing-intrinsic/vertical 4 异常场景 ≥ 正常 |
| 测试绑定 | ✅ Pass | 每场景绑单测名/命令 |
| TBD 清零 | ✅ Pass | 假设3 待 A/B 但非阻塞（第二切片条件触发） |

### 一致性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 范围冲突 | ✅ Pass | §1.3 排除（paint path/vertical）与 FR 无交集 |
| 代码边界完备 | ✅ Pass | §6.6 允许/禁止修改声明 |
| 实现来源闭合 | ✅ Pass | FR-001~003 均标注仓内自实现 + 模块路径 |

**门禁判定**：Fail = 0 → 允许实施（rally 续跑协议，不阻塞用户确认）。

---

## 12. 实测结论（R1578 首切片验证后，2026-07-17）

**首切片（measure path）已实施并验证机制正确，但实测揭示 R1577 路径误诊——wintertc footer 塌缩不在 measure path。**

### 已验证（机制正确）
- ✅ IFC `img_intrinsic_sizes` 字段 + `with_img_intrinsic_sizes` builder + img 分支 guarded 推导（env `ZW_IFC_IMG_INTRINSIC=1` default-off）实施 + 编译 + clippy clean。
- ✅ load-bearing 单测 `test_r1578_measure_uses_img_intrinsic_for_auto_width_img`：env ON 时 `<a>` measured h≈24（img 推导 w 后驱动行盒），env OFF 时塌缩（h_on > h_off + 4px）——**推导机制经 measure path 验证工作**。
- ✅ `measure_text_content` + engine.rs 3 measure 闭包点 plumbing：实测 measure_text_content 被调用时 `img_intrinsic_sizes` 含 10 entry（matrix.svg 在内，build_img_intrinsic_sizes 正确返回）。
- ✅ reftest A/B（css-position 59→59 / css-tables 79→79 / normal-flow 616→616）：**NET 0 零回归**。

### 实测纠正（R1577 路径误诊）
- ❌ **measure_text_content 未被调用于 wintertc footer 的 `<a>`**（img NodeId(88v1) 的父 `<a>`）。探针证 `measure_text_content` 调用列表无该 `<a>`。即 R1577「`<a>` 作 taffy measured node 经 measure callback 测高」假设对 wintertc footer **不成立**。
- 实测 img 88v1 被 `collect_inline_items` 处理时 `img_intrinsic_sizes.len()=0`（两次，w=0 h=24 + w=0 h=0）——即处理 footer img 的 IFC 是 **非 measure path**（compute_final_inline_layouts / remeasure_inline_only_containers 等 post-layout IFC，`:823`/`:1421` 等），这些 IFC 构造点**未 plumbing img_intrinsic_sizes**（默认空 map）。
- matrix.svg（`<svg width="75" height="32">` BothAbs）经 `build_img_intrinsic_sizes` **正确进入 image_sizes**（in_sizes=true，10 entry 之一）——R1577「img intrinsic-ratio 未应用」的 secondary 疑虑**排除**（build 端正确），缺口在 IFC 消费端（post-layout path 未 plumbing）。

### 修正后 slice 划分
- **Slice 1（本轮 LANDED，dormant env-gated default-off）**：IFC `img_intrinsic_sizes` 字段 + builder + img 分支推导 + measure_text_content plumbing + 单测。机制正确但 measure 非 wintertc 驱动路径 → 当前 NET 0 / 不解 wintertc。作 enabling infra 保留（slice 2 复用字段+推导）。
- **Slice 2（next，激活 wintertc）**：plumb `img_intrinsic_sizes` 进 post-layout IFC 构造点——`compute_final_inline_layouts`（inline_finalization.rs:823）+ `remeasure_text_with_float_exclusions`（:1276）+ `remeasure_inline_only_containers`（:1421/:1440）。这些函数增 `img_intrinsic_sizes: &HashMap<NodeId,(f32,f32)>` 参数，从 engine.rs caller（`compute_final_inline_layouts(&mut root_box, doc, styles, &[])` → 加 `&intrinsic_for_r695`）+ postprocess caller 传入；内部 IFC 构造链 `.with_img_intrinsic_sizes(...)`。**此切片才真正解 wintertc footer 塌缩**。
- Slice 2 风险：remeasure_inline_only_containers 的「grow height」逻辑（:1456 `if content_height > box_node.content_height`）现在因 IFC 算出正确 img 高度会 grow 父容器——须全量 A/B 守回归（可能影响所有含 inline img 的容器高度）。

### 决策
首切片作 dormant enabling infra 保留（env-gated default-off，零风险，单测证机制）。**不翻 default-on**（NET 0）。下轮起 slice 2（post-layout IFC plumbing），A/B 验证 wintertc footer `<p>` h>0 + 全量零回归后才考虑 default-on。
