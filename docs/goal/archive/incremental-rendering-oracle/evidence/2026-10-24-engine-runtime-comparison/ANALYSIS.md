# Engine 面运行时对照收口：多次导航 + 命中 + 像素容差运行时应用（DC-3/DC-4 运行时半边）

**运行身份**：审计/测试基线 HEAD `ddcd6efde`（占用标记轮）。**切片性质：纯测试切片**——新增 `crates/engine/src/tests/incremental_oracle.rs`（365 行，3 测试）+ `tests/mod.rs` 一行注册；**零生产代码变更**。进场依据：html-semantics-compat s57 收口后控制面宣告切片间隙（本轮复查仍开放）+ rendering-compat 活跃面（paint/painter/effects.rs、text.rs、layout-engine 生产文件）与本流新增测试文件零交叠。

## 三件事与测试映射

控制面「下一步」要求解封后完成三项运行时对照，均落 `incremental_oracle.rs`：

| 控制面余项 | 测试 | 断言面 |
|---|---|---|
| 多次导航运行时对照（A→B→A′） | `inc04_navigation_a_b_aprime_matches_cold_reference` | 同管线 A→B→A′：A′ 帧/A′ 命中网格/A′ `document_height` 与首轮 A **逐位/逐点/f32 精确**一致，且与全新管线冷启动参考逐位一致；B 帧像素、B 命中网格、B 溢出高三向与 A 可区分（对照非同态自比）；A′ 链接命中回到 A 的 href（跨导航换血，无 B 残留） |
| 命中对照 + 必须失效/必须保留门（INC-03） | `inc03_hit_test_warm_matches_cold_with_invalidation_gates` | warm 变更序列（SetAttr href / SetStyle margin-top / SetStyle background-color / SetText）每步与 cold 全量重解析的命中网格（320×400 @16px 步进 500 点，selector+盒几何投影）逐点一致；步 2 显式门：旧点位 (60,100) 脱靶 `#m`、新点位 (60,260) 命中；步 3 显式门：paint-only 换色后网格逐点不动 |
| 像素容差契约运行时应用（INC-04） | `inc04_pixel_contract_warm_frames_bitwise_equal_cold` | 同一 warm 管线四步变更帧与等价终态文档 cold 全量重解析帧，共用 FontLoader/GlyphCache/ImageCache 经 `render_full_scene` 单线程确定性路径光栅化，**逐位相等、不设容差**（2026-10-23 定案契约）；步 3 为半透明图元（rgba 0.5），与 m3-c rasterize 层 partial==full 先例同族 |

路径事实（读源码核正，非文档叙述）：`render_with_dom_mutations_persistent` 的结构变更臂（SetAttr/SetStyle 布局相关/SetText）经 HTML 快照**全量重建**（pipeline/mod.rs:2119 附近，`render_html(&snapshot, css)` + handle 重锚）；paint-only 臂（白名单含 `background*`）走**真增量** `paint_only_incremental`——`compute_styles_incremental` 只重算变更节点子树 + 复用 `cached_layout` + 重绘（pipeline/mod.rs:2371 起）。故本文件同时覆盖：变更应用/快照往返/handle 重锚/缓存换代（结构臂）与增量样式重算+布局复用（paint-only 臂）两条 warm 通路对 cold 参考的输出等价。测试用空 `FontLoader`（render-foundation cpu/tests.rs 先例，字形不进对照面——字形等价已由 render-foundation/compositor 常驻测试覆盖），对照面为 fill 几何/半透明混合/溢出/命中。

## fail-first 注入演练（绿灯有效性证据）

三处临时注入均不入库（`git restore` 后 `git status` 复核，暂存面仅测试文件）：

| 注入 | 位置 | 结果 |
|---|---|---|
| A：跳过 `cached_layout` 换代（旧文档布局保留） | `render_html` 尾（pipeline/mod.rs:1722） | **3/3 全红**——导航测试、命中门、像素契约全检出 |
| B：跳过变更 helper 的 `cached_layout` 更新 | pipeline/mod.rs:1787 helper | **全绿（注入未达路径）**——追踪发现该 helper 不在 `render_with_dom_mutations_persistent` 调用链上（结构臂走 `render_html` 全量重建）；按实际调用链改打 C |
| C：跳过 `compute_styles_incremental`（旧样式重绘） | `paint_only_incremental`（pipeline/mod.rs:2400） | **像素契约步 3 红且签名精确**：`步3 半透明换色: 首像素差异 (0,300): [127,191,127,255] != [255,127,127,255]`——(0,300) 恰为移动后 `#p` 条顶边，warm 侧为旧绿色 0.5 混白、cold 侧为新红色 0.5 混白；步 1/2 先行通过证明逐步检出而非任意步红。命中门保持绿属正确行为（paint-only 变更不动几何，该故障类由像素契约独占） |

B 的一轮落空本身入账：印证 guideline #18「code-trace 须以实际输出验证」——首选拦截点经注入实证为死路径后重瞄准，避免把无效注入误记为「断言通过演练」。

## 验证记录（test-guard 包裹，HEAD `ddcd6efde`）

- 新 oracle 3/3（`cargo test -p zero-engine incremental_oracle`；注入撤除后复绿）。
- zero-engine lib 全量 **3452P/0F**（3449 + 新 3，勾稽一致；clippy 修复后复跑确认）。
- 本流双 oracle 在 HEAD 刷新（上次刷新 c409ca548 后 R5250–R5253/s57 触及 layout-engine/engine paint 生产面）：layout-engine **10/10**（2123 filtered）、style-system **11/11**（2438 filtered）。
- `cargo fmt -p zero-engine --check` PASS；`cargo clippy -p zero-engine --all-targets -- -D warnings` PASS（24G systemd scope，中途两轮修复：`chunks_exact`→`as_chunks::<4>`、迭代器形态）。
- 工具面备忘：`rasterize_full_scene` 默认分发线程臂（`ZW_RENDER_THREAD` enabled_unless_zero），常驻对照走 `render_full_scene` 单线程确定性入口（cpu/mod.rs:761 弃用注记同向）。
- push 前 rebase 拉入 ec4ebf484（R5254 engine paint 圆角弧带）+ d1aaff275（s58 engine shim tabular-data）：组合树复跑 zero-engine lib **3453P/0F**（+1 为 s58 新测试，勾稽一致），本流 3 测试含内全绿。

## 结论与余项

**DC-3/DC-4 的 engine 运行时半边收口**：多次导航缓存换代（m3-b 静态审计的运行时对应半边）、命中必须失效/保留双门、像素容差契约（自对照逐位）全部有常驻测试与 fail-first 演练证据。至此五项 DC 全部 DONE、M1–M4 全部 DONE，本 goal 收口。

余项移交/保留（非本 DC 阻塞）：
- browser 条带 chrome 半透明 AA 观察项——zero-web 面协调（m3-c 已定性，维持观察）。
- 反例 A（style-system 增量 API 契约）——生产不可达，m1-style 固化为 Divergent 检出，保持文档开放。
- 退场：engine 面占用标记清除（本切片完成后），html-semantics-compat 可按其挂账序恢复 engine shim 面切片。
