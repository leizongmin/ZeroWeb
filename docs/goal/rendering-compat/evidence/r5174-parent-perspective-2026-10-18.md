# R5174 父元素 perspective 属性 + 恒定 z 平面投影合成

日期：2026-10-18 ｜ 前轮：R5173（rotateX 轴向直绘） ｜ 基线 corpus：1,099（R5173 ON 终态）

## 域核与定谳

目标族：`perspective-origin-001..006`（各 1.56%，0.5-2% 带内最大同构族，R5173 点名独立切片）。案形：父 `perspective:2px` + `perspective-origin` 关键词变体，子 `translateZ(0px)`（红块）/`translateZ(1px)`（绿块，应 ×2 放大盖红块）。

REFTEST_DUMP PNG 取证：绿块未放大、红块露出。根因双层：

1. **`translateZ` 无解析臂**（css-transforms-1 §16：translateZ(t) ≡ translate3d(0,0,t)）——含 translateZ token 的整个 transform 列表按语法失效丢弃（含 sorting 族的 `translatez(10px) translatey(100px)` 纯平面列表，一并丢失）。
2. **父元素 `perspective` 属性零消费**（css-transforms-2 §18-21：perspective ≠ none 为直子建立 3D 透视上下文，恒定 z 平面子元素投影 = 绕 perspective-origin 等比缩放 s = d/(d−τ)，τ = 子元素 z 平移累计）。

## 实现（kill-switch `ZW_PARENT_PERSPECTIVE=0`）

三处：

- **解析层**（`crates/css-parser/src/values/parse_transform.rs`）：`translatez` 臂归一 `Translate3d(0, 0, tz)`；token 层记录 translatez 出处；新增 `is_flat_plane_function` 单源判据（2D 仿射 + translate/translate3d；rotateX/Y/3d、perspective()、scale3d/scaleZ 等逐点 z 变函数不入）。**translateZ × 逐点 z 变函数混用列表整表按语法失效**（= 基线「translateZ 不可解析」精确语义；translate3d 混合列表不受影响——基线本就可解析且双侧近似同相）。
- **分类器/矩阵**（`crates/engine/src/paint/transform.rs`）：`flat_plane_translate_z`（style 级）/`flat_plane_list_translate_z`（list 级，复用解析层判据）提取累计 τ；`compose_perspective_scale`（折叠 s 进既有仿射，p' = s·M(p) + o·(1−s)，union rect 按 R3915 同款重算）；`perspective_scale_tp`（自身矩阵 None 时合成纯缩放 TP）。
- **painter**（`crates/engine/src/paint/painter/mod.rs`）：`ParentPerspective{node, origin_x, origin_y, d}` ctx 栈（paint_node 入栈/出栈）；变换站点沿 DOM 祖先链走查消费——直子直接承接；preserve-3d 中间代 τ 累加透传（spec §3d-rendering-contexts）、flat 中间代截断；逐点 z 变函数断链；**链长 > 1 限定叶实心群体**（R5085 差集判据去变换形态/祖先条件）。直绘/TP 双路同折叠（改路不拆对，R5173 教训）。

### 迭代定谳（三次 corpus + 单案取证）

| 迭代 | 结果 | 归因 |
| --- | --- | --- |
| 直子门 v1 | ON 1,086：翻绿 15 / 翻红 2 | `perspective-transforms-equivalence`、`3d-scene-with-iframe-001`（均基线绿）拆对 |
| +compute 混合丢弃 + 链承接 v2 | ON 1,087：equivalence/iframe 修复，但 translate3d-001 / split-by-zero-w 新红 + flattening-001 回红 | compute 层按函数形状丢弃不可行——**translateZ 与 translate3d 出处不可辨**：translate3d-001/split-by-zero-w 基线即可解析且双侧近似同相；flattening-001 ref 侧 `translate(50,100) perspective(1000px) rotateX(30deg) translateY(50px) translateZ(100px)` 属基线丢弃域 |
| **+解析层失效 + 链叶实心门 v3（终态）** | **ON 1,085 翻红 0** | 丢弃域收窄到 translateZ 出处（解析层 token 可辨）；equivalence 混合失效消除 R3833 退化兜底不对称（rotY 归零 / rotZ 幸存 → 同页 90° 旋转差）；iframe-001 链承接使 test 侧绿块 τ=200 → s=2 与 ref 对齐（transform2 空 subtree 整体跳过、零重放） |

- `preserve3d-and-flattening-001` 回落基线红：前轮「绿」= ref 侧混合列表近似投影与 test 侧 rotateX 近似巧合对齐（有损近似巧合对齐类，非稳定修复），解析层失效后 ref 回基线丢弃态。真 3D 管线域挂账。
- **链承接叶实心门取证**：iframe-001 的 `.transform2`（包 iframe，非叶）若合成 TP，其 union rect 重放会清白先绘的已缩放绿块（区域型 `apply_transform_post` 无子树隔离概念，R5085 同谱）——PNG 形态取证后加门。

## kill-switch OFF 轮语义（与基线差异集归因）

OFF（`ZW_PARENT_PERSPECTIVE=0`）**1,097** ≠ 基线 1,099，差异集逐案归因：

- `text-perspective-001` 翻红（0.00%，mismatch 型 3/3 稳定）：`translateZ(-9px) scale(10)` + 父 `perspective:1px`——基线绿靠含 translateZ 列表整表丢弃（scale(10) 不渲染）；OFF 保留解析归一（kill-switch 只门控合成）→ scale(10) 失去 s = 1/(1+9) = 0.1 补偿破版。ON 下折叠恰偿 → 绿。
- `transform3d-sorting-002/003/004` 保持绿：`translatez(10px) translatey(100px)` 纯平面列表解析即修复（parse-only 效应，与合成无关）。

kill-switch 语义：回退父 perspective 折叠合成主体；解析层归一与混合失效不受开关控制（解析语义非合成路径，无出处回退手段——compute 层按 z≠0 丢弃的替代方案经 corpus 实证翻红 3 撤销）。

## 验证

- unit +10（`crates/engine/src/paint/tests/r5174_parent_perspective.rs`）：translateZ 解析归一 + 合成 TP（a=d=2、ty=−50、union rect (0,0)-(100,100)）；自身仿射折叠（tx = s·own_tx + ox(1−s) = 20）；隔代不承接（默认 flat）；rotateX 子不折叠（R5173 直绘门 quad y∈[37.5,62.5] 未外扩证明）；translateZ(0) 零合成；百分比 origin 解析（tx=−100、origin=(100,50)）；混合列表解析失效（rotateY+tZ 整表 None / translate3d+rotX 照常解析 / tZ+rotZ 平面照常解析）+ individual rotate 不受牵连；preserve-3d 链承接（s=2）；逐点 z 变中间代断链（恰 1 TP 且 d=0.5）；非叶实心断链（2 fill 差集）。
- corpus 16,590 A/B 同 binary：**ON 1,085**（vs 基线 1,099 翻绿恰 14：perspective-origin ×6 + perspective-origin-x/xy + perspective-translateZ-0/negative/positive + transform3d-sorting-002/003/004；翻红 0；equivalence / iframe-001 基线绿维持）；**OFF 1,097**（差异集见上）。
- **tip 复核（rebase b6c7038db 后终态复跑，stash 归零态基线同窗）**：基线 **1,099** 恰等 R5173 ON 终态；ON **1,085** fail 清单 comm 双向 diff 与上轮逐案一致（翻绿恰 14 / 翻红 0）；OFF **1,097** 差异集同上（sorting×3 翻绿 + text-perspective-001 翻红）。kill-switch 判别在终态复跑下成立。

## 门禁

- fmt/clippy（`-p zero-engine -p zero-css-parser --all-targets -D warnings`）清。
- zero-engine lib **3,136 绿**（+10）；zero-css-parser 全绿（2,983）。
- make reftest **712/712**。
- make test 腿（workspace / quickjs clippy / renderer --bin 0P+--lib 272P / browser --bin xvfb 前缀 658P+1I / browser --tests cdp 三件 6P+4P+9P / gpu 96P+1P / quickjs crates 768P+59I 等）绿；**预存环境红逐案归因（stash 归零态基线同测复跑）**：`bookmark_context_menu_open_in_new_tab_loads_url`（共享档案书签栏几何敏感）与 quickjs renderer --lib `run_page_scripts_reports_uncaught_error_r2940`（错误消息保真族，另一流域 D1 诊断中）**基线同红，本轮零归因**；browser --tests 腿 bin artifact 裸跑 2 环境红（剪贴板 + `window_surface_present_smoke` 无 display，R385 记录在案）——xvfb 前缀复跑 window_surface 绿、剪贴板首轮绿，recipe 缺 `ZW_GUARD_RUNNER_PREFIX` 既有挂账延续。
- product-smoke：**EXIT=0**，welcome oracle diff **15.48%**（oracle 重录后基线，ad27f7143）+ 全 struct PASS（desktop 800 / 320 article / 375 各 fixture）。
- bench-gate 定向 `ZERO_WEB_BENCH_CRATES=zero-css-parser,zero-engine`：ON **2 FAIL**（page/medium、page/morning `first_paint_wall_ms` 墙钟 324.96/184.44 vs 预算 306.94/174.37）；**kill-switch OFF 同窗复跑同 2 指标同族超预算（312.62/182.08，量级相当）**——环境负载噪声族不 relax，OFF ≡ ON 失败集修改面零 bench 归因（R5172 判例延续）；33 指标其余全 PASS 含 paint_ms 族、NEW=0。

## 挂账（新增/延续）

- preserve-3d 完整域：嵌套 perspective、非叶合成重放、3D sorting（z 序）——须真 3D 场景图，`preserve3d-and-flattening-001`（3.90%）等域内红案维持基线态。
- `paint_node_in_rect` 脏矩形自递归路径不入 ctx 栈（FIXME 代码挂账）。
- rotateY 域 reveal 缺陷（R5173 延续）；apply_transform_post 露出区域下层恢复根修（独立轮评估）；perspective-origin-scroll 2.08%（恒值簇）；R5172 挂账组延续。
