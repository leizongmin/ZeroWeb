# R5175 perspective used value clamp + perspective() 平面投影折叠

日期：2026-10-18 ｜ 前轮：R5174（父元素 perspective 投影合成） ｜ 基线 corpus：1,085（R5174 ON 终态）

## 域核与定谳

目标族：R5174 轮内遗留 perspective 残案——`perspective-zero` / `perspective-zero-point-five`（各 2.08%）、`perspective-zero-2`（6.25%），及 R5174 tail 观察到的函数式案 `transform3d-perspective-005/006`。案形：

- `perspective-zero`：单盒 `transform: perspective(0) translateZ(0.5px)`（origin top left），绿块应 ×2 放大盖红块（`#cover-me` position:relative 负 margin 叠放）。
- `perspective-zero-point-five`：同形 `perspective(0.5px)`。
- `perspective-zero-3`：父 `perspective: 0px` 属性 + 子 `translateZ(0.5px)`（R5174 ctx 合成路径，属性值 0 未 clamp 前不建上下文）。
- `transform3d-perspective-005/006`：`perspective(100px) translatez(50px)`（005 inline + 父属性域 / 006 style block），s = 100/(100−50) = 2。

规范依据：css-transforms-2 §perspective-property + §funcdef-perspective；csswg-drafts#413（属性 `perspective: 0` used value clamp 到 1px，仍建立透视上下文 ≠ none）、#6320（`perspective()` 函数参数 clamp ≥1px；0 是合法声明、负值语法无效 per `<length [0,∞)>`）。

**三层根因**：

1. **属性 used value 无 clamp**（style-system）：`perspective: 0px` 直存 Px(0.0)，与 none 哨兵同值 → R5174 painter `d > 0.0` 门判「无上下文」，zero-3 不翻。
2. **`perspective()` 函数参数负值拒绝过宽 + 投影零消费**（css-parser + paint）：`val <= 0.0` 拒收 0（#6320 语义错误）；R5174 的折叠只覆盖父属性 ctx，函数式 perspective() 的恒定 z 平面投影未消费。
3. **R5174 解析失效过宽误伤**：translateZ × 非平面函数整表失效把 `perspective(100px) translatez(50px)` 一并丢弃——perspective() 是可折叠形（单源 + 平面后缀），应豁免。

## 实现

四处（函数折叠/存储 clamp/解析放开不受 kill-switch 控制，R5174 解析归一判例延续）：

- **存储 clamp**（`crates/style-system/src/property/apply_advanced.rs` "perspective" 显式 length 臂）：`Px(px) if px < 1.0 → Px(1.0)`（#413 used value clamp；`perspective_length_is_valid` 保持负值声明无效）。none 保持 `Px(0.0)` 哨兵：绘制侧以 `d > 0.0` 门区分「无上下文」与「d=1px 上下文」。FIXME 挂账：非 px 单位/Calc 绘制侧 resolve 后 <1px 未 clamp（罕见路径）。
- **函数折叠**（`crates/engine/src/paint/transform.rs`）：单 `Perspective(d)` 源（count==1）+ 后缀全平面函数（`flat_plane_list_translate_z`）时，恒定 z 平面投影 = 绕 transform-origin 均匀缩放 s = d/(d−τ)。**交换性论证**：列表局部坐标（origin 已被外层 T(o)·M·T(−o) 包裹）下均匀缩放是标量 s·I，与所有函数交换 → 整表 ≈ 既有合成仿射 × s（`a..d *= s; trans = s·trans + o·(1−s)`）。bail：`w = 1−τ/d ≤ 1e-6`（quad 触达/越过视点，投影非仿射）、τ=0（s=1 零作用）、双 perspective 非单源。`d_used = (*pd as f32).max(1.0)` 绘制侧 clamp（#6320，保留 CSSOM 0 值）。
- **解析放开**（`crates/css-parser/src/values/parse_transform.rs`）：perspective() 臂 `val <= 0.0` 拒绝 → `(value >= 0.0).then_some(value)`（#6320：0 合法、负值语法无效）；R5174 translateZ 失效触发器豁免 `TransformFunction::Perspective(_)`（可折叠形），translateZ + rotateX 等逐点 z 变函数仍整表失效（equivalence 拆对域保留）。
- **painter s>0 守卫**（`crates/engine/src/paint/painter/mod.rs`）：父 ctx 折叠消费点追加 `s > 0.0` 门——#413 clamp 使 d=1px 后 `w = 1−τ/d ≤ 0` 域可达，负/零缩放不得发射。

## 迭代定谳（三次 corpus）

| 迭代 | 结果 | 归因 |
| --- | --- | --- |
| ON v1 | 函数案 005/006 未翻 | REFTEST_DUMP PNG：006 渲染未放大 50×50 绿块——R5174 失效触发器把 `perspective(100px) translatez(50px)` 整表丢弃，解析层先于折叠生效 |
| ON v2 | 006 翻绿；zero 系仍红 | `perspective(0)` 被 `val <= 0.0` 拒收，`>= 0.0` 放开后 zero-3/005 翻绿（属性 clamp + ctx 合成已在位） |
| ON v3（终态） | **1,082 翻绿恰 3 / 翻红 0** | `perspective-zero` 红块形态完整未动（无 TP 清白发生）——painter 全链 probe 单测（Perspective(0)+Translate3d → TP s=2 发射）绿，证明变换数学已对；最终卡点是**变换元素绘制顺序**（见挂账），非本轮修改面 |

## kill-switch OFF 轮语义

OFF（`ZW_PARENT_PERSPECTIVE=0`）**1,096** = R5174-OFF 1,097 − transform3d-perspective-006（函数折叠 ungated，OFF 下新翻绿）。OFF 新红 vs ON 14 案 = ctx 合成依赖集：R5174 域 11 案（perspective-origin ×6 + origin-x/xy + translateZ-0/negative/positive）+ perspective-zero-3 + transform3d-perspective-005（父属性路径）+ text-perspective-001（R5174-OFF 特有红维持）。开关语义：只回退父元素 perspective ctx 合成；存储 clamp、函数折叠、解析放开不受开关控制（无出处回退手段，解析语义非合成路径）。

## 验证

- unit +11（`crates/engine/src/paint/tests/r5175_perspective_used_value.rs`）：属性 clamp 建上下文（d=1px、τ=0.5 → s=2 TP、origin (0,0) 零平移）；τ≥d bail（w<0 无折叠）；none 哨兵不建上下文；painter 全链 probe（单盒 [Perspective(0), Translate3d(0,0,0.5)] → TP a=2 发射）；解析豁免 e2e（`perspective(100px) translatez(50px)` 可解析并端到端折叠 s=2、tx=ty=−50；`translateZ(100px) rotateX(30deg)` 仍整表 None）；006 折叠（rect (25,25,50,50) → union (0,0)-(100,100)）；函数 0 clamp（d=1 → s=2）；后缀 rotateX 不折叠（维持 scaleY 近似，a=1）；behind-viewer（τ≥d → None）；双 perspective → None；τ=0 noop（纯平移不缩放）。style-system `test_perspective_apply` 扩展："0px"→Px(1.0)、"0.5px"→Px(1.0)。
- 解析钉 8 文件翻转（`perspective(0)` none→some、负值维持 None）：coverage_round4/coverage_round7/tests_3/tests_12/transform_coverage/values basic/values coverage/transform_coverage_final——语义注释标 R5175/#6320。
- corpus 16,590 A/B 同 binary：**基线（stash 归零态）1,085** 恰等 R5174 ON 终态；**ON 1,082**（fail 清单 comm 双向 diff：翻绿恰 3——perspective-zero-3 / transform3d-perspective-005 / transform3d-perspective-006，翻红 0）；**OFF 1,096**（差异集见上）。
- 残案定谳（ON 终态红维持）：perspective-zero / perspective-zero-point-five 各 2.08%（10,000px 精确红块全露）；perspective-zero-2 6.25%（同 R5174 tip）。

## 门禁

- fmt/clippy（`-p zero-css-parser -p zero-engine -p zero-style-system --all-targets -D warnings`）清。
- zero-css-parser **2,983 绿**；zero-engine lib **3,151 绿**（+11）；zero-style-system **2,419 绿**。
- make reftest **712/712** EXIT=0。
- make test 全腿：workspace / quickjs clippy / renderer --bin+--lib / browser --bin（xvfb 658P+1I）/ cdp 三件 6P+4P+9P / gpu probe→**gpu 96P+1P** / quickjs crates（768P+59I 等 10 套）/ quickjs renderer --bin 0P / quickjs renderer --lib **269P + 1 预存环境红**；browser --tests 腿 656P + 2 已知环境红（剪贴板前置 + `window_surface_present_smoke` 无 display，R385 在案）xvfb 前缀复跑 2P 绿。quickjs renderer --lib `run_page_scripts_reports_uncaught_error_r2940`：R5174 已 stash 归零态基线同红（错误消息保真族，另一流域 D1 诊断中），HEAD 未变（b0a398271）归因直接沿用，本轮零归因。
- product-smoke：**EXIT=0** 全 struct PASS（desktop 800 / 320 article / 375 各 fixture）。
- bench-gate 定向 `ZERO_WEB_BENCH_CRATES=zero-css-parser,zero-engine`：ON **2 FAIL**（page/medium、page/morning `first_paint_wall_ms` 墙钟 319.56-320.79/176.95-180.63 vs 预算 306.94/174.37，连续两轮同值非瞬时）；**stash 归零态基线同窗复跑同 2 指标同族超限（317.53/176.97）**——环境负载噪声族不 relax，基线 ≡ ON 失败集修改面零 bench 归因（R5172/R5174 判例延续）；33 指标其余全 PASS 含 paint_ms 族、NEW=0。

## 挂账（新增/延续）

- **变换元素 stacking-context 绘制顺序**（新域，perspective-zero/zero-point-five 残案根因）：css-transforms-1 Appendix E step 8——浏览器把 transformed 元素与 positioned 元素同层按 DOM 序绘制；ZW painter 在 static 相位绘制变换元素 → position:relative 的 `#cover-me` 红块后绘盖住已正确缩放的绿块。painter probe 单测 + 006 全链翻绿证明变换数学/解析/折叠全链正确，纯绘制序问题。
- **body-background 传播缺陷**（新域）：ref green.html 依赖 body 背景传播整页，ZW 渲染为 100×100 块——reftest ref 侧保真问题，影响以 body bg 为比对面的一族案。
- perspective-zero-2 preserve-3d 函数式 perspective 继承（真 3D 管线域，R5174 挂账延续）。
- 非 px 单位/Calc perspective 绘制侧 resolve 后 <1px 未 clamp（FIXME 代码挂账）。
- 延续：preserve-3d 完整域、rotateY 域 reveal 缺陷、apply_transform_post 露出区域下层恢复根修、perspective-origin-scroll 2.08% 恒值簇、R5172/R5171 挂账组。
