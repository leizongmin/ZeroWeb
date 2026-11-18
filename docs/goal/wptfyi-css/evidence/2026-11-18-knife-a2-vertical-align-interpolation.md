# 刀 A2：vertical-align 插值三通道接入（2026-11-18）

## 靶面与根因

- 主靶 `css/css-transitions/animations/vertical-align-interpolation.html`：A1d 残面池候选清单 #4，solo 基线（刀 Z 态 bin）182 forms 136F/46P。
- 根因＝vertical-align 全通道零插值覆盖：`TRANSITIONABLE_PROPERTIES` 无条目（transitions 不启动）+ animation.rs lenient/strict/applier 三臂全缺——全部回落离散 echo。
- 7 个 test 块构成：5 个可插值 LP 块（neutral=underlying 10px、initial/inherit/unset→LP 即 baseline≡0 参与语料钉面）+ 40px↔40% calc 混合块 + `test_no_interpolation`（super→40%）块。

## 语义定谳（实测修正一处设计假设）

- lenient 臂（css-inline-3 §vertical-align-propdef + css-transitions-1 §animations by computed value type）：LP 对线性；**baseline 端点按 0 长度参与**（语料 initial/unset→length at(0) 期望 '0px'，Chrome parity）；关键字对/关键字↔LP 50% 离散翻转（css-values-4 §8.5.6，CSSA/WAAPI 通道语义）；px↔% 跨单位发射 **px 系数在前** 的语料 canonical 形 `calc(Xpx ± P%)`（与 blend_rule_inset_lp 的 % 前缀形相反——rule-inset 槽序差异，专属 blend_va_lp）；px 系数 0 折叠纯 %（at(1) 期望 '40%'）。
- strict 门=**真实可插值性**（两端点 LP/baseline 解析成功才 Some）：首版恒 Some 委托被 solo 实测推翻——语料确有 `test_no_interpolation`（super→40%），恒 Some 会把非插值对登记成真过渡致 transitions 通道按 50% 翻转（-6F）。修正后非插值对恒 None：transition 注册拒绝→**css-transitions-1 §starting-of-transitions 无过渡立即跳终值**＝harness transitions 通道 `expectFlip(from, to, -Infinity)` 恒 to 形；CSSA/WAAPI 通道不经本门，lenient 0.5 翻转（`expectFlip 0.5` 形）不变。harness 四通道 nonInterpolationExpectations 实读：cssAnimations/WAAPI=0.5、cssTransitions/transition:all=-Infinity。
- applier 臂 `parse_vertical_align` 落 `style.vertical_align` 槽；gCS 串化臂既有（vertical_align_to_css）。

## 码面（2 文件）

- `crates/engine/src/js_dom_bridge/transition_registry.rs`：清单 80→81 加 `vertical-align`。
- `crates/engine/src/animation.rs`：lenient 臂 + `parse_va_endpoint`/`lerp_va_length`/`blend_va_lp` helpers + strict 臂真实可插值门（guard arm + None 臂）+ applier 臂 + 钉测 `test_vertical_align_lerp` 15 断言（engine lib test 断编译挂账对端 part116 include! 悬空，补跑随树清窗）。

## 残面池（FIXME(va-calc-slot)）

- calc 混合对 9F（40px↔40% at(-0.5/0/0.3/1.5) ×transitions 双通道 + WAAPI at(1) echo 形）：`LengthValue` 无 Calc 变体（css-parser/types.rs 纯单位枚举）→ calc 混合存储不可能，跨切面改动不动（对端 fv1 活跃 crate 避让）。
- CSS2/linebox 与 flexbox 垂直对齐静态布局面（A1d 池 +40F 记账）＝layout 域，不入本插值刀。

## 八腿 A/B（battery `knife-a2-r1-084503`，08:45:03→09:08:20 GATE-DONE，EXIT=0，8 腿全 rc=1 零 LEG-FAIL）

b0=bin `6541c6d3` 存档副本；b1=bin `e625eff5`（电池前后 md5 双证未漂移）。四语料 × 两 bin：

| 语料 | b0 | b1 | Δ |
|---|---|---|---|
| css-transitions | 118 页 667P/841F/15T | 673P/835F/15T | **+6P/−6F** |
| css-animations | 123 页 491P/804F/7T | 491P/805F/7T | +1F |
| css-backgrounds | 124 页 3488P/2758F | 3489P/2757F | +1P/−1F |
| css-masking | 62 页 4703P/1652F | 4703P/1652F | 0 |

**b0 污染定谳（subtest 指纹闭合）**：b0 主靶页 167P/15F 恰合刀 A2 **v1（恒 Some strict）** solo 形——15F 指纹 = 8 calc 混合（transitions 双通道 at(-0.5/0/0.3/1.5)）+ 6 `super→40%` 恒 Some 误注册（transitions 双通道）+ 1 WAAPI at(1)；b1 9F = v2 真实可插值门形（8 calc + 1 WAAPI 残面）。b0 打包窗（08:39:38）树内已含 A2 v1 hunks → **b0 非真基线，电池实量 v1→v2 修正段**。印证 [[ab-b0-bin-must-probe-against-ledger]]（打包前未跑靶面探针比对收账数的未执行项）。

**账本定谳（净 +144）**：刀 A2 全貌对 **刀 Z 落库态**（bin `41cfb19f`）css-transitions **全语料** = 529P/980F/15T → 673P/835F/15T，**净 +144P**（results-knife-z/css-transitions.json 08:14 × battery b1 逐页 diff，零域外页）。分解：主靶页 **+127**（46P→173P，三端点锚链：solo 基线 136F 刀 Z 态全离散 echo 形 → v1 15F → v2 9F × 电池两端点指纹复现）＋搭载面 **+17**——vertical-align-composition.html **+16**（Compositing CSS/WA 通道随 lenient 臂搭载；battery b0 已含=v1 态得，A2 lenient 为该页唯一 vertical-align 插值机制变量，display 臂/serialize 域与此页无涉）＋ all-interpolates-same-as-explicit-property.html **+1**（transition:all≡显式属性等价断言转真；registry 无条目时 `all` 通配不探测 vertical-align 的清单差为候选机制，fv1 串化窗扰不能全排除，采信 A2 窗归因）。电池 v1→v2 段 +6P/−6F 全部落主靶页（css-transitions dP=+6 与主靶页翻转数恰等），其余三语料零 A2 归因翻转。

**归其系漂移（窗口法证）**：css-animations +1F = display-none-dont-cancel.tentative.html 新增子测试（'Animating from display:none to display:none with an intermediate…'，s560 display apply 臂 4d82f45e4 窗内编辑先于 b1 构建 08:44:36——其 87cbe3e5b 自述 display 臂 P2F 观察哨同域，归其系）；css-backgrounds +1P = background-position-x computed calc 列表串化翻转（fv1 serialize.rs 域）；css-masking 零位移。

## 门禁

- **行为验证链**：solo 136F→15F→9F 三端点 + 电池 b0/b1 指纹复现（harness 行为级，847+182×2 子测试面）＋刀 Z bin 全语料对（+144 逐页分解，results-knife-z × results-knife-a2 JSON）。
- **钉测补跑 PASS（本窗兑现）**：part116 工作树删除已恢复（HEAD 双含 part116+part117——上游迁移已收口，本地 `D` 为被超越残留），engine lib test 编译解；`test_vertical_align_lerp` **PASS**（1 passed / 4176 filtered，test-guard `--compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900` 定向腿）。A1/B1/Z 引擎钉测补跑同窗解锁。
- fmt：`cargo fmt --all -- --check` 14 处漂移（animation.rs 12 + registry 2）**全部先在于 HEAD**——git show HEAD 版 + config-path rustfmt 复核行位一一对应，全落本刀 hunks 区间外，上游窗口遗留不代修。
- clippy：`cargo clippy -p zero-engine --lib` 零新增；余 3 处全既有提交态（`too_many_arguments` 8/7 对端挂账 + animation.rs:2345-6 f64 cast，HEAD:2260-1 同在）。
- perf-gate：循刀 Z/刀 A1e 先例豁免——registry +1 条目 + match 臂新增，无热循环改动。
- **行为重绑要求**：本账绑 bin `e625eff5`（源态＝applier 臂在 engine animation.rs）。电池收官后 A2 码面收敛（registry 注释 v1→v2、applier 臂迁 `style-system/src/property/apply.rs:1081` 同构臂、断言 rustfmt 化）——迁层后提交前以重建 bin solo 探针主靶页，期望 9F/173P 与 v2 终态同数（差>±1＝迁层行为分歧，回退或补账）。协议见 `.acceptance/wptfyi-css/NOTES-knife-a2-accounting.txt`。
