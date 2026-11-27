# ZeroWeb 运行时控制面板

## 先看结论

**当前（2026-11-27 批量拍板落档与核账收口）**：全项目在途待决项（1–7 项 + 附项督办）已按建议一次性裁决落档（详见 `97c619357` 批复表，**本批仅建档与规则/裁决落档，不改产品代码**）：① `WPT_REV` 本周期维持当前 pin（`31597693`）、剔除上游漂移面重算 `wptfyi-html` / `wptfyi-dom` 验收线，冲刺收官再统一协调 bump——**三方互斥核账已落地**（[核账证据](../wptfyi-dom/evidence/2026-11-27-wpt-rev-pin-drift-audit-and-dc3-calibration.md)）：`wptfyi-dom` 实测 `post_pin` 漂移 **5,854**（纠偏原估 ~12,000）→ DC-3 校准线 **≥ 94,093**（净增 ≥ 29,146）；`wptfyi-html` 实测 `post_pin` 漂移 **3,705**（勘误批十九 15,429S 混入 `in_pin_uncovered` 14,067）→ 严格漂移校准线 **≥ 148,391**（净增 ≥ 77,295，另单列非单文档载体硬边界 5,660 / Chrome 5,404，单文档 Chrome 上限约 146,099，待快照解冻后复议）；② 性能门禁打包——`style-system`（`ZRG-2026-11-18-01/22-01`）选治本（零分配默认表示 + 级联热路径优化，归渲染/CSS 流排期）、CI 三池（`9v45`/`8370c`/`9v74`）及本机陈旧基线批准**条件化一次性重建**（排在 `style-system` 与 k117 治本落地之后）、`tab_creation_100` 授权 bisect、`window_config_default`（`ZRG-2026-11-23-01`）归类 `layout-sensitive` 另设容忍带或移出硬门禁；③ 刀 Q2 `ch` 兜底维持 `1em`，真实 `ch` 度量贯通并入后续字体度量专项；④ `html-semantics` 三门——`s137` 豁免退役批准、`video` P1+P2 与 `customizable select/combobox` 维持暂缓挂账；⑤ `layout-compat` 独立 goal 已建档立项（[layout-compat.md](../layout-compat.md) + [master.md](../layout-compat/master.md)），暂设算力启动门控；⑥ 小项 `zero-engine` 7 新指标入基线与 `green-2x2.png` tracked/放行批准，渲染深结构清单 ①–⑥/⑧/⑨ 等维持暂缓挂账；⑦ 自进化 kill 纪律草案已合入 [AGENTS.md](../../../AGENTS.md) 并清出 `evolution-drafts/pending/`；附项：**点名督办 `wptfyi-dom` 优先插队排查修复 `test262-builtins` 6 GiB 内存膨胀**——**已根治验证**（`70f04eb1b` 改写 `_zwAsciiLower`/`_zwTtAsciiLower` 消除 `RegExp.prototype.exec` 覆写死循环；全量 `test/built-ins` 23,065P/393F、峰值 RSS 648 MB 见 `4974ee5df`；`wptfyi-html` 十九演穿越 18 连贴帽带 `58cb7f51c`，二十演 3h29min 完跑 53,162 页、峰值 2.59 GB 见 `79d29e4d7`，正待二十一演二证窗确认 CLEAN 发射批四十九）。

**当前（2026-11-27 更新）**：ZRG r51 巡检 **新立案 ZRG-2026-11-27-01（css_parse_by_size 全尺寸 ×4 过线 1.36-1.41×，七点 bisect 定谳引入刀 = k117 `a4b0e3d82` @mixin/@apply CSSOM 物化刀——11-25 落 parser/mod.rs +96/mixins.rs +72/at_rules.rs +44 parse 主路径接线）**，判 DEFERRED 停止自动改代码（css-parser/style-system = wptfyi-css 流在途战役工作面，run-rules §9 不越面；飞书告警已发待用户裁决；ZRG-2026-11-18-01/22-01 再恶化 + ZRG-2026-11-23-01 原样 + 慢性 ZRG-2026-08-15-03 第 48 轮）。被测 `6a61ae446`（窗口 = r48 被测 `7ecb55ace`..HEAD 共 601 提交，产品面 190 文件 +37620/−1602，非测试 145 文件主力 = css-parser k106-k136 冲刺 ×18 + style-system 非测试 ×17 +6788 行（k119-k138 求值端+registry/apply）+ engine TT 刀㊶-㊺ ×31；r49（NO-QUIET-WINDOW 未定谳）/r50（f4ffd7187 中断未入账）两轮窗随本轮全量覆盖）：bench-gate 三跑 run1 SUSPECT（浪穿透不入判定）→ run2 FAIL 18/113 → run3 FAIL 16/113，交集 16 可复现——css ×4（306/1565/3061/15437 µs vs 基线 220.5/1114.9/2227.2/11360.2；静窗 A/B 12 测 1.25-1.37× 尺寸均匀 = 每规则常驻成本）+ style ×7（cascade/10 1.85-2.02× vs r48 1.14-1.17×、csd 2.71-2.73× vs 1.62-1.88×；窗 A/B cascade 1.245-1.596× 尺寸递增 = k119-k138 真码增量，csd 1.05-1.13× = 18-01 机制延续）+ webview ×4（A/B 净 ≈1.05 带缘，时代漂移判读延续）+ window_config_default（7.09/7.25 ns，23-01 原样，host_runtime_bench 两树逐字节同 31db1a92）。其余门禁全绿：分进程内存 ×2 PASS（renderer 141.2/142.1、compositor 142.6/141.4 ≤192，idle SUM 263.4/262.2 ≤320 较 r48 回落 ~8 MiB）、连通性 ×4 零硬失败、product-smoke PASS（welcome 17.23% 逐字节同值）、parity 双腿 4bf59035 逐字节同 r43-r48 慢性第 48 轮零漂移。处置候选：(a) wptfyi-css 流在 k117 物化面惰性化/快径治本 (b) 基线「重建非放宽」一次性重捕获+刀系解析成本配额护栏 (c) 落地前过渡连续红 DEFERRED 记账，待用户裁决。详见下方 ▶ 条目；证据 `.rally/zeroweb-regression-guard/20261126T220502Z/`
**当前（2026-11-24 更新）**：ZRG 2026-11-24 巡检**零新立案面**，判 DEFERRED（ZRG-2026-11-18-01/22-01/23-01 全延续 + 慢性 ZRG-2026-08-15-03 第 47 轮）。被测 `7ecb55ace`（窗口 = r47 被测 `442d0e6a1`..HEAD 共 165 提交，产品面 14 文件 = engine js_dom_shim JS 批 ×9 = DC-2 切片 8/8b（part15-positional +804 行、trusted-types/sanitizer/trusted-sinks 扩展）+ script-sandbox ×2 + renderer ×2（ipc_image_batch +322）；**style-system/layout-engine/css-parser/render-foundation/host-runtime/webview 源码 ×0、Cargo.lock ×0**）：bench-gate 两跑静窗 GATE FAIL（run1 32/113 = 起跑静窗 load 1.52 后亚秒级共租浪穿透全家族面、run2 14/113，NEW=0 双跑；资源 peak_rss 200.22/192.55≤360.31、page/* 全 PASS），交集 14 = 可复现面——style 全族 ×7（cascade/10 1.14-1.17×、/50 1.08-1.10×、/100 1.02-1.04×、/500 1.01-1.02×、csd 1.62-1.88×、cs50 1.13-1.22×、cs200 1.14×）+ webview 全 ×4（1.04-1.15× 带缘，r47 仅 1/4 可复现）+ window_config_default（7.49/7.36 vs 2.08 = 3.54-3.60×）+ canvas 新成员 ×2（path_ops run2 1.09×、save_restore run2 1.014×）。**二进制谱系定谳零窗口增量**：style_bench/host_runtime_bench/canvas_bench 与 r47 构建逐字节同 sha256（61b47156/31db1a92/c47f5438+39f81a4c，style 连 build hash 路径 63ab4749 都同）→ style 7 + window_config + canvas 2 = 同二进制慢性态/带缘（window_config A/B 两侧 6.5-7.3 ns 同带比值 0.985-1.014 = 23-01 漂移态持稳不加重；canvas 同二进制日漂移过线非码差）；webview_bench 双候选字节变化（746030bbe→e59d7549 等 = 链接窗内 engine/script-sandbox 变更，A/B 按 mtime+sha256 锁定）→ 交错 A/B（BASE=442d r47 测量二进制）静腿 ≈1.0：simple 0.987-1.017、inject 0.987-1.023、render 0.985-1.016、load_html_with_css ab-attr 1.029-1.101 方向一致 → 静窗聚焦 6 轮 0.971-1.010 定谳 ≈1.0（浪残差伪象）；同二进制 style 对照腿 5 轮静腿 net 0.948-1.069 纯噪声带佐证——webview 可复现面扩张为带缘非码增量，时代漂移判读延续（基线 129.5k 陈旧、当树 173-202k）。其余门禁全绿：分进程内存 ×2 PASS（renderer 141.2/140.8、compositor 147.8/148.0 ≤192，idle SUM 270.8/270.9 ≤320 带内，较 r47 带内微浮 +1~2 MiB）、连通性 ×4 零硬失败、product-smoke PASS rc=0（welcome 17.23% 逐字节同值）、parity 慢性第 47 轮双腿 report.json 4bf59035 逐字节同 r43-r47 零漂移。处置：不自动修复——14 项全部映射既有在途裁决（18-01/22-01 注册成本族、23-01 代码生成漂移、webview 时代漂移归基线政策议题），无新候选方案；**零新立案面按 09-17 判例未发飞书告警**。证据 `.rally/zeroweb-regression-guard/20261123T220324Z/`（REPORT.md + report.json）
**当前（2026-11-23 更新）**：ZRG 2026-11-23 巡检 **新立案 ZRG-2026-11-23-01（`mb/zero-host-runtime/window_config_default` 依赖重建代码生成/布局漂移，3.13-3.35× 预算）+ ZRG-2026-11-18-01/22-01 延续（style 族 A/B 静腿 0.92-1.00 = 442d 窗内零增量）**，判 DEFERRED 待用户裁决。被测 `442d0e6a1`（窗口 = r46 被测 `96cfa7790`..HEAD 共 213 提交：engine ×21 文件 = DC-2 TrustedTypes/Sanitizer 大切片、style-system ×7 = hyphens + KC-1 cis + fmt、layout-engine ×5 = STRETCH-ABSPOS/CSSOM-INSETS V2/V3、css-parser ×2、render-foundation ×2、webview ×1 仅 fmt、**host-runtime ×0、Cargo.lock ×0**）：**bench-gate 两跑静窗 GATE FAIL（run1 10/113、run2 6/113，NEW=0）**，交集 6 = 可复现面——style ×4（csd 2.45-2.48× 基线、cascade/10 1.05-1.10×、compute_styles_50 +6.5-9.5% / _200 +5.8-6.0%，均已知家族）+ **window_config_default 6.51/6.96 ns vs 预算 2.08（A/B：BASE 96cf 码 1.1-1.7 ns 历史带内、TIP 442d 码双构建候选一致 6.3-32.2 ns = 窗口二进制相关真实抬升 4-6×，源码零改动 → 机制 = engine 大切片依赖重建引发代码生成/布局漂移，未逐 crate 定位）**；webview_load_html_simple 静腿带缘 +0.8% 经 A/B ≈0.99 + 基线陈旧证据（r44 起全部代次二进制 165-190k vs 11-11 基线 129.5k）判时代漂移不立案。其余门禁全绿：内存 ×2 PASS（idle SUM 270.9/270.9 继续回落）、连通性 ×4 零硬失败、product-smoke PASS 17.23% 逐值同、parity 慢性第 46 轮 4bf59035 逐字节同值零漂移。处置：不自动修复——ZRG-2026-11-23-01 属 sub-10ns 微基线政策/跨 crate 归属决策面，style 族候选 (a)/(b)/(c) 照旧合并待裁决；扩面成立飞书告警已发。详见下方 ▶ 条目；证据 `.rally/zeroweb-regression-guard/20261122T220242Z/`

**当前（2026-11-22 更新）**：ZRG 2026-11-22 巡检扩面立案 **ZRG-2026-11-22-01（style-system 注册成本族从 csd 扩散至 cascade/declarations 与 compute_styles，ZRG-2026-11-18-01 同根延续）**，判 DEFERRED 待用户裁决。被测 `96cfa7790`（窗口 = r44 被测 `52e9f3228`..HEAD 共 665 提交，crates/style-system 零直改，css-parser/engine 属性注册刀系数十批；含 r45 中断轮 `e6ddde432` 全覆盖）：**bench-gate 两跑 GATE FAIL（run1 16/113、run2 22/113，NEW=0）**，交集 10 指标中 **style-system ×7 为真面**——csd 509.48/523.30 ns = 基线 2.51-2.58×（ZRG-2026-11-18-01 延续）、cascade/declarations/10 = 预算 1.09×、/50 = 1.12×（两跑复现）、compute_styles_50 +3.8% / 200 +5.9% 超预算（两跑复现）、/100 /500 带缘；**交错 A/B（BASE=`e6ddde432` r45 构建二进制 sha256 谱系已核）TIP/BASE 0.99-1.05 = 96cf 窗内零增量**，回归形成于更早窗的注册刀系累积；交集其余成员（dom/webview/page fpw）经定向 A/B ≈1.0 且静腿绝对值低于预算 = 共租浪伪象（r29-r31 判例）。其余门禁全绿：内存 ×2 PASS（idle SUM 270.8/272.7 ≤320 较 r43-r44 带再降）、连通性 ×4 零硬失败、product-smoke PASS 17.23% 逐值同、parity 慢性 ZRG-2026-08-15-03 第 45 轮 report.json 与 r43/r44 逐字节同值（4bf59035）零漂移。处置：不自动修复（run-rules §9 style-system 属渲染流工作面），修复决策空间与 ZRG-2026-11-18-01 候选 (a)/(b)/(c) 同一，合并待用户裁决；飞书告警已发。详见下方 ▶ 条目；证据 `.rally/zeroweb-regression-guard/20261121T220457Z/`

**当前（2026-11-18 更新）**：ZRG 2026-11-18 巡检立案 **ZRG-2026-11-18-01 可复现 CPU 劣化转专项（csd 2.70×）**，判 DEFERRED 待用户裁决。被测 `52e9f3228`（窗口 = 台账底座 `6d4dbc700`..HEAD，含 r41-r43 三轮中断未入账窗全覆盖 + e429609ba..52e9f3228 夜间九刀）：**bench-gate run1 GATE FAIL 35/113**（静窗 load=1.68 启动、中段共租浪 load≈10——`mb/zero-style-system/computed_style_default` **548.51 ns = 基线 203.03 的 2.70×**、预算 274.09 的 2.00×，另 34 指标 FAIL 面横跨 engine/layout/webview/host-runtime 具共租浪签名待静窗 run2 甄别）；**csd 交错 A/B ×5（BASE=`201a32dde` r42 同面 PASS 带二进制，浪下两端同打）TIP/BASE 1.35-2.87× 全部 >1**；叠加 r43 轮（e429609ba）静窗 A/B ×9 定谳 1.27-1.41×——**回归连续两轮可复现且显著恶化**。根因定谳：`ComputedStyle::default()` 内 `vec!` 堆分配初始化器随 css-gaps/rule-color/rule-inset 列表模型与 mask/mask-border/font 刀系扩容线性增长（201a=24 → e429=30 → 52e9=36，每分配 ~13ns，r43 实测 +6 分配 ≈ +80ns 外推本轮 +12 分配 ≈ +150ns 恰落在 A/B 实测带）；属渲染流 style-system 架构面（列表模型默认表示零分配化）或基线政策裁决（重捕获仅治标——R5501 11-11 刚重捕获 5 天即再陈旧），非本巡检流可局部修复，按 run-rules §9 不越工作面停止改代码。其余门禁：内存 ×2 PASS（renderer 141.2/139.9、compositor 158.4/155.5 ≤192、idle SUM 279.4/278.6 ≤320）、连通性 ×4 零硬失败、product-smoke PASS 17.23% 逐值同、parity 慢性 ZRG-2026-08-15-03 第 44 轮 report.json 与 r43 逐字节同值（4bf59035）零漂移。详见下方 ▶ 专项条目；证据 `.rally/zeroweb-regression-guard/20261117T220515Z/`

> **历史巡检与 CI 守护归档（2026-08 至 2026-11-19）**：2026-11-10 及之前的顶部摘要、CI 守护日志、GB/ZRG 历史巡检记录与 R3379 前控制面记录已按原样迁至 [archive/guard-history-2026-08-to-11-19.md](archive/guard-history-2026-08-to-11-19.md)。

> **▶ 用户批复落档（2026-10-25，对话渠道，GB-20261024 征询项裁决——ZRG-2026-10-24-01 idle SUM 预算重校准获批，惰性物化不立专项）**：① **授权按批复⑥同口径一次性重校准 idle SUM 预算**——重测 renderer+compositor 结构下限，+10% 余量推导新预算，推导与证据落 `docs/specs/performance-and-resource-budget.md` §4（2026-10-07 CI re-capture 先例同款）；重校准独立轮次执行，非 FAIL 轮顺手调阈值；smaps 剖析证据沿用立案轮 `.rally/zeroweb-regression-guard/20261023T220256Z/`。② **惰性物化/表精简不另立专项**——候选归 site-opt perf-mem 流（T 系列在途）按 ROI 排期；再立 RFC 触发 = 重校准后 idle SUM 超新预算两腿复现且 smaps 剖析仍为非泄漏有机增长。③ 关联核对：**renderer-trust-boundary 用户 2026-10-25 自行启动**（非无人值守流代跑，下轮核对按此口径）；dom-operation-bench 维持 M1 解锁不催启（错峰条款以 2026-10-23 解锁记录为准，等 html-semantics M2 收口或切片间隙）。本批复覆盖 GB-20261024 飞书征询（msg `om_x100b62ffb50100a0c016222471747a5`），ZRG 流下轮据此恢复 idle SUM 门禁裁决并执行重校准，无需再征询。

> **▶ goal-blockers 巡检（2026-10-24，GB-20261024）**：cronjob 待决策卡点专项——扫描全 goal `master.md` 待决策清单 + 近 3 天推进记录（渲染流 R5252-R5263 collapse 格线/border band 收口系列（001e 7.25→0.00% 满分）+ engine s56-s68 WPT 域开片收口链（table/text/embedded-content/img 静态加载管线）+ media A1-A3 落地（时钟对齐/三缺口/内存面收敛）+ ZRG-2026-10-24-01 立案）。**新增决策点 1 项已飞书征询**（bot msg `om_x100b62ffb50100a0c016222471747a5`，2026-10-24 22:25）：**ZRG-2026-10-24-01 idle SUM 322.4/325.8 > 320 预算（批复⑥）两腿复现**——预算重校准 vs 惰性物化专项待用户裁决（ZRG 已停自动改代码；今日 06:11Z 告警已发为告知，本条为正式征询；建议 = 授权按批复⑥同口径一次性重校准（重测结构下限 +10% 余量附证据落档），惰性物化/表精简另立专项 RFC 排期，两者不互斥）。**在案已批核对（不再征询）**：bench-gate/perf-gate 基线刷新 re-anchor = GB-20261023 批复（rendering-compat master 同日批复块；R4937 挂账组 e2e re-anchor / perf-gate 真放宽 / bench-gate 周期性补跑窗口策略随批并入执行清单闭合），等静窗执行；本机 linux-x86_64 基线重采 = ZRG-2026-10-17-01 批复①维持有效未消费（等净窗）。**维持挂账不重复征询**：rendering-compat 深结构批次（inline 溢出放置 / 行盒高度链四件同步改造 / css-contain 四案转挂 / font-metric 生产激活 / row-2·multicol-2·动画 RFC / 滚动条产品面 / overflow-clip-margin polygon shadow clip renderer 图元契约 / csswg#7103 族 / 注册属性 style query 深域 / rem 根依赖面 / zero-width-floats PR #42 账）等点名；compositor-animation-continuity 双 flag（`ZW_THREADED_WORKERS` / `ZW_WORKER_CANVAS_PATCH`）翻默认属独立决策 perf-gate 前置（2026-10-22 goal 收口遗留，待对应流认领 + 用户点名）。**其余 goal 零待批项核对**：media-runtime-isolation D-RFC-1~4 已批复（`9fa13cef2`——方案 C 立项 / A 批次落地中 A4 待启 / decode-h264 维持默认关；缺口表「平台 SDK/codec 许可/分发」为 M1/M2 证据后未来门，本平台 /dev/dri 缺失无硬解路径，现在无需裁决）；dom-operation-bench / renderer-trust-boundary M1 已解锁（2026-10-23 用户决策，无人值守可启动）；form-validation 待决策两项 = 2026-08-19 批复销项 + 技术备忘非决策项，维持。**自主面（本轮核对）**：① GB-20261014 注记二 e) 项 dead_code 双处核实已由归属流闭合销账——engine `match_media_to_json` 现有调用点（`js_dom_bridge/callbacks.rs:433` + tests part02），renderer `build_implicit_submit_url`/`submit_form_on_enter_at` 已接线（`runtime.rs:1827`），CI clippy 门禁恢复（`92eb9737e` 同日修 R5275 collapsible_if 互证）；② rendering-compat master.md「先看结论」标 R5248 而账本已至 R5263（同日多轮推进的流内汇总滞后，自愈项）按 run-rules §9/§10 不代写；③ 其余 goal 控制面头部状态/卡点/下一步与实际一致。本巡检纯文档核对+征询，无代码变更。
> **▶ main CI 守护轮（2026-11-28，cronjob ci-guard，run 46053603775（发现 @ 8b3f1fb53）→ 46054913160（ci-main 并发组取消）→ 46056052933（终验 @ d320c7a4a，10/11 绿））**：cronjob 守护流水线轮（触发 CI → 修自主可修 → 预存记账 → weekly/pages 发布链核验）。**① 触发即红 2 类自主可修（2 提交）+ benchmarks 预存**（run 46053603775：11 legs 仅 smoke 绿——6×build-and-test 腿 fmt 门禁红 + android clippy 红，windows-aarch64/reftest/benchmarks 被 cancel）：
（a）**fmt 门禁全平台红**——11-27/28 五个 knife 提交（d0004e8be/68138c0e1/5ff02c7bd/5c38220cc/ef1625b29）直推未过 `cargo fmt --all`（§16 于 11-27 新立，部分刀早于守则生效）：临时 worktree 隔离 `cargo fmt --all` 归位 6 文件（css-parser tests/mixin_apply.rs、engine js_dom_bridge{,/callbacks,/css_wire,/mixin_css}.rs、style-system if_function.rs，+80/−39 纯格式零行为），提交 c9ff1e58c（本地辅证：guarded quickjs workspace clippy 0 error、css-parser/style-system/engine 51 suite 0 failed；CI 为权威）。
（b）**android clippy 红**（`error: 'cargo-clippy' is not installed for the toolchain '1.100.0-x86_64-unknown-linux-gnu'`）——根因 112dbadf8 钉版 `dtolnay/rust-toolchain@1.100.0` 后 android job 未带 `components:`，该 action 缺省 minimal profile（rustc+cargo+rust-std）不含 clippy，@stable 时代通过系 runner 预装 stable 自带 clippy；修复 = android job 补 `components: clippy`（提交 d320c7a4a）。全 workflow 排查：仅 ci.yml android job 有此险（smoke/reftest/benchmarks 裸步不跑 clippy，其余 workflow 无 clippy/fmt 用点）。
**② 终验 10/11 绿**：7×build-and-test 全矩阵（fmt 双腿 + android 恢复）+ smoke + android + reftest 绿；唯一红 = benchmarks 慢性预存——逐腿 `--log-failed` 归因 `GATE FAIL — 26 个指标超预算（已检查 113，NEW=0）`，池 `github-ubuntu-latest-intel-xeon-platinum-8573c`，成员全属 2026-11-27 批量拍板已裁决族（cascade/declarations ×4 = ZRG-11-18-01/22-01、window_config_default = 11-23-01、tab_creation_100 = 已授权 bisect、layout/webview/engine 集群 = 池锚漂移），NEW=0 无新面，不代修。
**③ weekly.yml 上轮 failure（run 46040008041 @ ae34a8fa1，coverage 发射轮）诊断**：失败面仅 `wptfyi-coverage (test262-builtins)` 分片 + aggregate，构建矩阵无红——分片非旧 6G 内存回归形，而是 test-guard `--time-limit 9000` 精确墙钟击杀（17:17:17→19:47:18 = 9000.6s，零输出，artifact 182 B）；语料漂移后健康本机基线 3h28m30s/53,162 pages 已超 2.5h 预算。属 wptfyi-coverage 流调参域（批五十/DC-3 在途），不越面代修，记账。随后**重触发 weekly 不带输入**（run 46060173541 @ 2f2904088 = 完整构建矩阵）验证 build+Publish 发布路径（上次 Publish 成功 = 45827945127 @ f00d6e324，11-26，早于钉版 112dbadf8）。
**④ pages.yml failure（run 46059196631，workflow_run 触发）**：非 wpt-suites.csv 列错位——deploy-pages artifact API 竞态链：首败 `Found 0 artifact(s)`（上传后传播延迟）→ `rerun --failed` 重复上传致 `Multiple artifacts named github-pages (2)` → 删旧重试再遇 0/2 竞态，4 次后停止人工 rerun；残留 `github-pages` artifact（含孤儿 15228696144）已全部删除（现存 0），待本轮 weekly 完成经 workflow_run 触发全新 pages run（干净命名空间）验证部署。**无新增待用户决策项。**
> **▶ zeroweb-regression-guard 巡检记录（2026-11-27，ZRG-2026-11-27-01 = 新立案 css_parse_by_size 全尺寸 ×4 过线 1.36-1.41×——七点 bisect 定谳引入刀 k117 a4b0e3d82 @mixin/@apply CSSOM 物化刀（parse 主路径接线），静窗 A/B 12 测 1.25-1.37× 尺寸均匀 = 每规则常驻成本；ZRG-2026-11-18-01/22-01 再恶化（style ×7 cascade/10 1.85-2.02×、csd 2.71-2.73×，窗 A/B cascade 真码增量）+ webview ×4 时代漂移延续 + ZRG-2026-11-23-01 原样；其余门禁全绿；r49/r50 未入账轮窗随本轮全量覆盖）**：低频防劣化巡检（被测 SHA `6a61ae446` = origin/main，专用 worktree ZeroWeb-zrg-6a61 检出时干净、全程零改动；窗口 = r48 被测 `7ecb55ace`..HEAD 共 601 提交，产品面 190 文件 +37620/−1602，非测试 145 文件主力 = css-parser ×18（k106-k136 冲刺）、style-system 非测试 ×17 +6788 行（k119-k138 求值端 + registry/apply）、engine ×31（TT 刀㊶-㊺）、dom ×6、apps ×13、webview ×1 +132、render-foundation ×5、script-sandbox ×4、taffy-local ×3；前情 = r49（20261124T220238Z）bench 截止 NO-QUIET-WINDOW 未定谳、r50（20261125T220454Z，被测 f4ffd7187）两跑 FAIL + A/B 归因后会话中断未入账，两轮窗随本轮全量覆盖）。平台 linux-x86_64（i5-13500H 16C，Xvfb :102 遗留实例复用，lavapipe，Chrome/156.0.8078.79，node v24）；perf 基线 `docs/perf/baselines/linux-x86_64.json` sha256 `f37cef72…`（R5501 2026-11-11，窗内未变更直接消费，config_hash `a3bd667` 同前）；harness 逐字复用 r48 仅路径重绑（scenario 双 json 逐字节同 sha a80bd98b/4579e435）+ 截止 2026-11-27 04:00 UTC，新增 ab-r51.sh（css ×4 + style ×7 + webview ×4，各 3 轮，静窗守候 load<2.5 持续 2min）与 probe-css-r51.sh（七点 bisect，probe worktree 独立构建）；构建 systemd scope MemoryMax=24G + timeout 5400，cp -al 硬链接预热自 zrg-f4ff 残留产物。**① bench-gate**：run1 判 SUSPECT（perf-gate INCONCLUSIVE = 起跑后共租浪穿透，policy §5 不入判定）→ run2（bench-rerun 守候静窗启动）**FAIL 18/113** → run3（bench-run3 同环境第二定谳腿）**FAIL 16/113**——**交集 16 = 可复现面**：**css_parse_by_size ×4 = 新面 ZRG-2026-11-27-01**（rules/100 303.2/305.6 µs = 基线 220.5 的 1.38-1.39×、/500 1542.2/1565.3 = 1.38-1.40×、/1000 3039.4/3061.3 = 1.36-1.37×、/5000 16023.0/15437.0 = 1.36-1.41×，r48 同指标全 PASS）＋style ×7 = 18-01/22-01 再恶化（cascade/10 1.85-2.02×（r48 1.14-1.17×）、/50 1.95-1.96×、/100 2.02×、/500 2.01-2.07×、csd 2.71-2.73×（r48 1.62-1.88×）、cs50 1.65×、cs200 1.61-1.63×）＋webview ×4（inject 383-393 µs/simple 194-200/with_css 245-255/render 350-357 = 1.40-1.54× 陈旧基线带）＋window_config_default 7.09/7.25 ns（23-01 原样，host_runtime_bench 两树逐字节同 31db1a92）；run2 单跑成员 end_to_end_pipeline、ipc_large_message_serialize_1000 经谱系+run3 双证据剔除噪声（zero-protocol bench 两树逐字节同 f215f1ec = 同二进制不可能码增量）；resource/peak_rss ≤360.31 PASS；page/* 全 PASS。**② 归因（三重定谳）**：谱系 = css_bench f55cefeb（7ecb BASE）→ 1810ef58（6a61 TIP）窗内字节变化；静窗 A/B（ab-r51.tsv，load 1.4-2.3）css 12 测全 1.25-1.37× **尺寸均匀 = 每规则常驻成本非固定开销**；七点 bisect（probe-css-r51.tsv）：k107pre `86fd9125d`/k111 `c887c4afb`/k113 `e29f73a93`/k116 `61eca0403` 全 ≈1.0（0.97-1.03）→ **k117 `a4b0e3d82`（2026-11-25 20:12+0800，@mixin/@apply CSSOM 物化刀）长采样复测（2s/4s/×30）1.28-1.36× 全尺寸——首测 100/500 干净系当腿 load 4.5-4.8 浪残差，复测覆盖定谳 = 引入刀** → k118 `f86c834ed` 1.29-1.35×、k124 `a48fdba2a`/k127 `0415580df` 1.33-1.39× 传播；热点面 = parser/mod.rs +96/mixins.rs +72/at_rules.rs +44（CSSOM 物化 A 层序接线 + B 双载体/单源重建器/shim 五件套挂 parse 主路径）。style 族 A/B：cascade ×4 1.245-1.596× 尺寸递增 = k119-k138 求值端 + registry/apply 真码增量叠加 BASE 已有家族底；csd 1.05-1.13×/compute_styles 1.055-1.088× = 18-01 ~13ns/alloc 机制温和延续。webview ×4 A/B 净 1.004-1.168 ≈1.05 带缘 = 时代漂移判读延续不新立案（基线 129.5k 陈旧、当树 170-200k）。**③ 其余门禁**：分进程内存 ×2 PASS（/proc VmRSS/VmHWM 只计本次 browser 后代 0.5s 采样：renderer 峰值 141.2/142.1、compositor 峰值 142.6/141.4 ≤192；idle 尾样本 SUM 263.4/262.2 ≤320——较 r48（270.8/270.9）回落 ~8 MiB）；连通性 ×4（2 内存腿 + 2 parity 腿）Healthy + frame_committed、硬失败签名 grep 0/0/0/0；product-smoke PASS rc=0（welcome 17.23% 82714/480000 逐字节同值，六视口 struct-check 全绿）；Chrome parity 双腿 report.json sha256 `4bf59035` **与 r43-r48 逐字节同值**（production 双真 chrome-cdp-gui + production-window-gpu + browser-pointer、state PASS ×5、events 1:1、button.y 16.438px 唯一几何驱动、diff% 6.33/5.68/5.56/5.76/5.85）= 慢性 ZRG-2026-08-15-03 第 48 轮零漂移。**④ 处置：停止自动改代码（css-parser/style-system = wptfyi-css 流在途战役工作面，k 系冲刺进行中（R19000 已收账），run-rules §9 不越面；r46/r47 判例同）**；新立案成立 → 飞书告警已发（本条目同批交付）。**ZRG-2026-11-27-01 专项方案候选**：(a) wptfyi-css 流在 k117 物化面做惰性化/快径——无 @mixin/@apply 时跳过物化接线，恢复 parse 主路径 ≈k116 带宽（治本，归该流排期）；(b) 基线政策裁决——授权 css_parse ×4 按「重建非放宽」口径一次性重捕获 + 新增刀系解析成本配额护栏（需基线授权，guard-policy §2）；(c) (a) 落地前过渡口径——门禁连续红按 DEFERRED 记账（待裁决）。18-01/22-01 恶化面候选不变（(a) 零分配/亚线性默认表示+注册表成本治理、(b) 基线政策+注册成本增长率护栏、(c) 过渡口径），与 11-23 批次合并待用户裁决。证据 `.rally/zeroweb-regression-guard/20261126T220502Z/`（gitignore：REPORT.md、report.json、build.nohup/build.sh、gates.rc/chain.nohup、product-smoke.log、parity 双腿全套、memory-run{,-leg2} 全套、bench-gate-run{1,2,3}.log、bench-run3.sh、ab-r51.sh/.tsv/.tsv.meta、probe-css-r51.sh/.tsv、commits-window.txt、generic-page/memory-idle scenario json）。
> **▶ zeroweb-regression-guard 用户裁决（2026-11-27，对话渠道，ZRG-2026-11-27-01 处置候选裁决 = 选 (a) 治本惰性化/快径）**：r51 交付后用户在对话渠道对 ④ 三候选明确裁决（选项回执「确认选 (a) 治本惰性化」，对 css_parse ×4 1.36-1.41×/引入刀 k117 `a4b0e3d82` 定谳无异议）——**ZRG-2026-11-27-01 修复归 wptfyi-css 流排期：在 k117 物化面做惰性化/快径，无 @mixin/@apply 时跳过物化接线，恢复 parse 主路径 ≈k116 带宽**；(b) 基线重捕获与配额护栏、(c) 独立过渡立案均不采纳。**落地前过渡执行口径（随本裁决一并生效）**：修复落地前各轮 ZRG bench-gate 的 css ×4 连续红按**已裁决在途** DEFERRED 记账、映射本批复，不再重复飞书征询；ZRG 侧不代修（run-rules §9 不越面）。边界：本批复仅为 wptfyi-css 流工作面内的修复授权，**不构成基线放宽、阈值调整或重捕获授权**；style 族 18-01/22-01 恶化面、23-01、webview 时代漂移不在本批复内，仍按 11-23 批次候选合并待裁决。修复落地核验：该流按其回归口径验证（parse 主路径 A/B 对拍 k116 带），ZRG 下轮起复核 css_bench 二进制谱系变化与 css_parse ×4 指标回落。
> **▶ zeroweb-regression-guard 巡检记录（2026-11-23，ZRG-2026-11-23-01 = 新立案 window_config_default 依赖重建代码生成/布局漂移 3.13-3.35×（host-runtime 源码窗内零改动、Cargo.lock 零变化，A/B BASE 1.1-1.7 ns 历史带 vs TIP 6.3-32.2 ns）+ ZRG-2026-11-18-01/22-01 延续（style 族 442d 窗 A/B 静腿 0.92-1.00 零增量）+ webview_load_html_simple 判基线时代漂移不立案；其余门禁全绿）**：低频防劣化巡检（被测 SHA `442d0e6a1` = origin/main，专用 worktree ZeroWeb-zrg-442d 检出时干净、全程零改动；窗口 = r46 被测 `96cfa7790`..HEAD 共 213 提交 192 文件 +105829/−503——engine ×21 文件（DC-2 TrustedTypes/Sanitizer 大切片 TT sink/Sanitizer 构造面/ref-target/cis 路由）、style-system ×7（hyphens 初始值 manual + KC-1 contain-intrinsic-size 分轴受理 + fmt）、layout-engine ×5（STRETCH-ABSPOS + CSSOM-INSETS V2/V3）、css-parser ×2（background-size MATHFN v3）、render-foundation ×2、webview ×1 **仅 fmt**（f77adb751）、**host-runtime ×0、Cargo.lock ×0**；r46 点名三刀 hyphens/STRETCH-ABSPOS/insets echo 全在本窗，预期 style/engine 面复核）。平台 linux-x86_64（i5-13500H 16C，Xvfb :102 遗留实例复用，lavapipe，Chrome/156.0.8078.79，node v24）；perf 基线 `docs/perf/baselines/linux-x86_64.json` sha256 `f37cef72…`（R5501 2026-11-11，窗内未变更第六轮消费，config_hash `a3bd667` 同前）；harness 逐字复用 r46 仅路径重绑（scenario 双 json 逐字节同 sha a80bd98b/4579e435）+ 截止 2026-11-23 06:00 UTC；构建 cp -al 硬链接预热自 zrg-96cf 残留产物 3m00s。**① bench-gate**：run1 静窗 load=0.58 启动 **GATE FAIL 10/113（NEW=0）** → run2（bench-rerun 守候静窗 load=0.45 启动）**FAIL 6/113**——**交集 6 = 可复现面**：style ×4 `computed_style_default` 503.18/497.94 ns = 基线 203.03 的 2.45-2.48×（ZRG-2026-11-18-01 延续）、`cascade/declarations/10` 2942.49/2826.01 = 预算 2681.58 的 1.05-1.10×、`compute_styles_50_elements` 1673945/1719765 = 预算 +6.5%/+9.5%、`compute_styles_200_elements` 6325344/6313384 = 预算 +5.8%/+6.0%（均已知家族两跑复现）＋**新面 `mb/zero-host-runtime/window_config_default` 6.96/6.51 ns = 预算 2.08 的 3.13-3.35×**（r44/r45 同指标 1.07-1.65 全 PASS，本轮突跳）＋`webview_load_html_simple` 176314（静腿带缘 +0.8%）/239218（浪腿）；run1 单跑成员（cascade/50 带缘 1.03×、webview inject_css/render/load_html_with_css）经②A/B 定谳浪伪象；resource/peak_rss 198.97/188.66 ≤ 360.31 PASS；page/* 全 PASS（run2 welcome total 10.28ms / medium 311.9ms / morning 57.5ms）。**② 交错 A/B 归因（TIP=442d 今日构建 vs BASE=96cf r46 构建，浪两端同打，谱系 sha256 对拍锁定）**：**cargo 新布局认知（本轮实证，影响后续 A/B 方法）**——bench 可执行落 `target/release/build/<pkg>/<hash>/out/`（style/webview/host 三面均今日新建、双候选 8062040e+61b47156 / d65160fe+0d6c22c8 / 55ff6abe+c0a5ed2b），`deps/style_bench-7490ec…` 等旧文件名为 cp -al 预热遗留副本（两树同 inode 未替换）；本轮 bench-gate 测量有效（criterion 由今日构建产出，build/ mtime 与 run 窗吻合）。style 全族 7 指标 ×5 轮（ab-style-r47.tsv）：**静腿（round 2-4）净比 0.92-1.00**（cascade/10 0.977-0.991、/50 0.959-0.968、/100 0.947-0.982、/500 0.922-0.937、csd 0.921-0.983、cs50 0.980-1.004、cs200 0.988-0.998；round 1/5 浪腿双向摆动 0.516-2.499 不计）= **442d 窗内零增量定谳**（hyphens/KC-1/fmt 未加成本，TIP 静腿反略快），家族回归仍为 52e9 前注册刀系累积、根因与候选方案不变；**日漂移观察：同一 96cf 二进制 cs50 静腿 r46（11-22）1.64M → 本轮（11-23）1.83M = +12% 环境日漂移**，量级与码差同阶，佐证「重捕获治标不可持续」。window_config_default 定向 A/B ×3（ab-attr-r47.tsv，双 TIP 候选同测）：BASE 1.7/1.7/1.1 ns（与 r44/r45 历史带 1.07-1.65 同带）vs TIP 静负载地板 6.3/6.5 ns（两候选一致；浪腿 8.6-32.2）= **窗口二进制相关真实抬升 4-6×，源码零改动 → 机制 = 依赖大切片重建引发代码生成/静态布局漂移（`WindowConfig::new` 本体仅 String 分配 + 标量赋值，假设未逐 crate 定位）**。webview 定向 A/B ×3：`load_html_simple` 静腿对 **0.994**（无窗口增量，webview 窗内仅 fmt）＋基线陈旧证据链——r44 起 170.5k → r45 170.5-174.9k → r46 183-186k → 本轮 176-186k，**全部代次二进制高于 11-11 基线 129.5k 达 35-44%** = 时代漂移/基线陈旧类，趋势归入基线政策议题（ZRG-2026-11-18-01 关联）不新立案；inject_css 0.965-1.016、render 0.879-1.002、load_html_with_css 0.999-1.016 净腿 ≈1.0 = 浪伪象（r29-r31/r46 判例同族）。**③ 其余门禁**：分进程内存 ×2 PASS（/proc VmRSS/VmHWM 只计本次 browser 后代 0.5s 采样：renderer 峰值 139.3/139.2、compositor 峰值 146.4/145.7 ≤192；idle 尾样本 SUM 270.9/270.9 ≤320——较 r46（272.7）继续回落，compositor 稳定 r43 带 146-148）；连通性 ×4（2 内存腿 + 2 parity 腿）Healthy + frame_committed、硬失败签名 grep 0/0/0/0；product-smoke PASS rc=0（welcome 17.23% 82714/480000 与 r36-r46 逐值同，六视口 struct-check 全绿，form-input-perf PASS）；Chrome parity 双腿 report.json sha256 `4bf59035` **与 r43/r44/r46 逐字节同值**（production 双真 chrome-cdp-gui + production-window-gpu + browser-pointer、Chrome/156.0.8078.79、state PASS ×5、events 1:1 focus4/type5/toggle12/button16、button.y 16.438px 唯一几何驱动、diff% 6.33/5.68/5.56/5.76/5.85）= 慢性 ZRG-2026-08-15-03 第 46 轮零漂移。**④ 处置：停止自动改代码（ZRG-2026-11-23-01 属 sub-10ns 微基线政策/跨 crate 归属决策面；style 族 run-rules §9 渲染流工作面照旧；guard-policy §2 基线重建需明确授权）**；扩面成立（新可复现成员）→ 飞书告警已发（本条目同批交付）。**ZRG-2026-11-23-01 专项方案候选**：(a) sub-10ns 微基线政策修订——布局敏感类指标（≤10ns 量级）设容忍带或重锚定（需基线授权，guard-policy §2）；(b) 逐 crate 二分定位引发布局漂移的依赖面（engine 大切片归属渲染/零-web 流工作面，巡检侧不越面）；(c) perf-gate 对该类指标标注 layout-sensitive 类别另行跟踪（政策改动需用户确认）。style 族候选 (a) 零分配/亚线性默认表示+注册表成本治理、(b) 基线政策+注册成本增长率护栏、(c) 过渡口径，与 ZRG-2026-11-18-01/22-01 合并待用户裁决，不变。证据 `.rally/zeroweb-regression-guard/20261122T220242Z/`（gitignore：REPORT.md、report.json、build.nohup/build.sh、gates.rc/chain.nohup、product-smoke.log、parity 双腿全套、memory-run{,-leg2}.log + memory-samples.tsv、bench-gate-run{1,2}.log、ab-style-r47.sh/.tsv、ab-attr-r47.sh/.tsv、commits-window.txt、generic-page/memory-idle scenario json）。

> **▶ zeroweb-regression-guard 巡检记录（2026-11-22，ZRG-2026-11-22-01 = 注册成本族扩面立案：cascade/declarations 与 compute_styles 首次两跑稳定越线（ZRG-2026-11-18-01 csd 延续 2.51-2.58×），96cf 窗内 A/B 零增量定谳形成于更早窗；交集其余成员共租浪伪象；其余门禁全绿）**：低频防劣化巡检（被测 SHA `96cfa7790` = origin/main，专用 worktree ZeroWeb-zrg-96cf 检出时干净；窗口 = r44 被测 `52e9f3228`..HEAD 共 665 提交——产品面 css-parser/engine/style-system 属性注册刀系数十批（MATHFN/OM-15..OM-20/SIZING-GRAMMAR/CALC-SIZE 族/RANDOM3a/TEXT-INTERP/URLSMOD/GENFAM/CS-1 corner-shape/CLIP 族/optical 三性/GRID-PLACESHORT/BFC-INLINE-MARGIN 等），**crates/style-system 直改 0 提交**；含 r45 中断轮 `e6ddde432`（20261120T220444Z，GATES-DONE 后无终局判定未入台账：smoke/parity×2/mem×2 完成，bench 两跑 FAIL 5 指标 2835/2821、8362/8366、481/490、6.13M/6.12M、1.61M/1.60M 与本轮逐值同带）按台账纪律全量覆盖重触发）。平台 linux-x86_64（i5-13500H 16C，Xvfb :102 遗留实例复用，lavapipe，Chrome/156.0.8078.79，node v24）；perf 基线 `docs/perf/baselines/linux-x86_64.json` sha256 `f37cef72…`（R5501 2026-11-11，窗内未变更第五轮消费，config_hash `a3bd667` 同前）；harness 逐字复用 r44 仅路径重绑 + 截止 2026-11-23 06:00 UTC；构建 cp -al 硬链接预热自 zrg-e6dd 残留产物 ~3.5 分钟。**① bench-gate**：run1 静窗 load=1.51 启动 **GATE FAIL 16/113（NEW=0）** → run2（load=2.34 启动、中段共租浪 load≈6-14：css-parser/security/storage/script-sandbox/canvas/browser-shell 全中招轮转签名）**FAIL 22/113**——**交集 10 = 可复现面**：`mb/zero-style-system/computed_style_default` **509.48/523.30 ns = 基线 203.03 的 2.51-2.58×**（ZRG-2026-11-18-01 延续）、`cascade/declarations/10` **2921.21/5368.50 ns = 预算 2681.58 的 1.09×+**、`cascade/declarations/50` **9253.27/11049.81 = 预算 8281.92 的 1.12×+**、`compute_styles_50_elements` 1643677.8/1781835.8 = 预算 +3.8%/+13.6%、`compute_styles_200_elements` 6321472.7/6951531.8 = 预算 +5.9%/+16.5%（后两对 run2 腿含浪胀，run1 静窗腿为准）、`cascade/declarations/100` 14968/16114 = 预算 +1.3%/+9.0% 与 `/500` 66622/103498 = 预算 +3.7%/+61%（带缘/浪腿）——**style-system ×7 全族两跑越线或带缘，注册成本族从 default() 初始化面（csd）扩散至级联与批量计算面**。交集其余 3 成员 `dom/tree_construction_10k_nodes`（19.0-24.5M）、`webview_load_html_simple`（183-186k）、`page/medium/first_paint_wall_ms`（487.83 静 +3.6% / 882.57 浪）经②定谳浪伪象；resource/peak_rss 205.79 ≤ 360.31 PASS。**② 交错 A/B 归因（浪下两端同打，r43/r44 判例方法）**：style 面 ×5 轮 ×7 指标——BASE = zrg-e6dd 真码 `style_bench-7490ec…` sha256 `f6eb7695…`（r45 构建残留，与 96cf cp -al 预热副本逐字节同，谱系已核；52e9 侧 11-18 12:38 二进制含 d588e7584 后重建弃用），TIP = 96cf run1 重建同文件名 sha256 `257acde9…`——**TIP/BASE 0.989-1.055（cascade/10 1.022/0.989/0.992/1.023/1.043、/50 净腿 1.009-1.052、/100 净腿 0.996-1.043、/500 净腿 0.964-1.052、csd 1.023/1.036/1.04/1.055、compute_styles_50 净腿 1.008-1.074、_200 净腿 1.044-1.149）= 96cf 窗内零增量定谳，回归形成于 52e9 之前的注册刀系累积**（csd +2-5% 轻微正漂带内，≈1-2 分配量级未断言）；补遗 dom/webview 定向 A/B ×3（rounds 4-6，修正 criterion 组前缀路径 `dom/tree_construction_10k_nodes`）：`tree_construction` **0.996/0.991**（浪腿 1.239）、`load_html_simple` **1.018/0.999/1.024**、`inject_css` 1.03/1.018（浪腿 0.536）——且**静腿绝对值低于预算**（tree 9.9M vs 基线 11.6M、load_html 166-171k vs 预算 174.9k）= **交集 dom/webview/fpw 成员共租浪伪象定谳（r29-r31 判例同族）**，zero-dom 窗内仅 fmt + env 双门仪表面（fa4a6a198 产品构建零代码路径）旁证一致。**③ 其余门禁**：分进程内存 ×2 PASS（/proc VmRSS/VmHWM 只计本次 browser 后代 0.5s 采样：renderer 峰值 139.3/140.2、compositor 峰值 148.3/147.2 ≤192；idle 尾样本 SUM 270.8/272.7 ≤320——较 r44（279.4/278.6）与 r36-r40 带（299.0-318.5）再降，renderer idle 122.4/125.4）；连通性 ×4（2 内存腿 + 2 parity 腿）Healthy + frame_committed、硬失败签名 grep 0/0/0/0；product-smoke PASS rc=0（welcome 17.23% 82714/480000 与 r36-r44 逐值同，六视口 struct-check 全绿）；Chrome parity 双腿 report.json sha256 `4bf59035` **与 r43/r44 逐字节同值**（production 双真、state PASS ×5、events 1:1、button.y 16.438px 唯一几何驱动、diff% 6.33/5.68/5.56/5.76/5.85）= 慢性 ZRG-2026-08-15-03 第 45 轮零漂移。**④ 处置：停止自动改代码（run-rules §9 style-system 属渲染流工作面 + guard-policy §2 基线重建需明确授权；两跑可复现劣化已满足自动修复前置，但修复对象越工作面）**；飞书告警已发（本条目同批交付）。**扩面专项方案候选（与 ZRG-2026-11-18-01 同一决策空间，合并待用户裁决）**：(a) 零分配/亚线性默认表示 + 注册表成本治理（SmallVec/enum Plain|List/静态初值写时复制 + cascade 热路径注册表查找 amortize），治本，归渲染流排期；(b) 基线政策裁决——按「重建非放宽」口径授权 linux-x86_64 全量重捕获 + 附加注册成本增长率护栏（每属性注册配额），治标，5 天 stale 循环再现（R5501 11-11 重捕 → 11-22 即全族越线）；(c) 过渡口径（门禁连续红 DEFERRED 记账或临时豁免 style-system 档案组）。证据 `.rally/zeroweb-regression-guard/20261121T220457Z/`（gitignore：REPORT.md、build.nohup/build.sh、gates.rc/chain.nohup、product-smoke.log、parity 双腿全套、memory-run{,-leg2}.log + memory-samples.tsv、bench-gate-run{1,2}.log、ab-style-r46.sh/.tsv、ab-attr-r46.sh/.tsv、generic-page/memory-idle scenario json）。
> **▶ zeroweb-regression-guard 巡检记录（2026-11-18，ZRG-2026-11-18-01 = 新可复现 CPU 劣化转专项：computed_style_default 结构性堆分配增长 2.70× 连续两轮可复现且恶化，r41-r43 三轮中断窗随本轮全量覆盖重触发；其余门禁无新回归）**：低频防劣化巡检（被测 SHA `52e9f3228` = origin/main，专用 worktree ZeroWeb-zrg-52e9 检出时干净；窗口 = 台账底座 `6d4dbc700`..HEAD——**r41（20261110T220331Z/613ce6a54）/r42（20261111T220430Z/0894f9614）/r43 前段（20261112T220328Z/201a32dde）三轮会话中断无终局判定未入台账**，r43 后段（20261116T220331Z/e429609ba）五门禁四绿、bench 静窗 run5 FAIL 7 指标（csd 284.45 = 预算 1.04×）+ 交错 A/B ×9（BASE=`201a32dde`）1.27-1.41× 定谳真回归后会话在终稿前中断——本轮窗口按台账纪律全量覆盖；e429609ba..52e9f3228 子窗 140 提交/68 文件 +11,190−359，含渲染流 11-17~11-18 夜间九刀（D rule-inset/E CSSA/I important-tier/K columns/mask-border 五长手/mask 简写六长手/N font 一二三期/O font supports），style-system property/ 下 9 提交）。平台 linux-x86_64（i5-13500H 16C，Xvfb :102 +GLX r43 遗留实例复用，lavapipe，Chrome/156.0.8078.79，node v24）；perf 基线 `docs/perf/baselines/linux-x86_64.json` sha256 `f37cef72…`（R5501 2026-11-11 随 1a6ab8e5f 刷新，窗内未变更第四轮消费，csd p95 203.027 ns→budget 274.09）；harness 逐字复用 r43 仅路径重绑 + 截止 2026-11-18 06:00 UTC；构建 cp -al 硬链接预热自 zrg-e429 残留产物 1m23s。**① bench-gate**：run1 静窗 load=1.68 启动、中段共租浪 load≈10——**GATE FAIL 35/113（NEW=0）**：`mb/zero-style-system/computed_style_default` **548.51 ns = 基线 2.70×、预算 2.00×**，另 34 FAIL 横跨 zero-engine（end_to_end_pipeline 2.78×/medium first_paint 2.84×/block_layout_1000 2.11×/deep_nesting 2.54×）/zero-host-runtime（host_runtime_new 1.97×）/zero-webview（load_html_simple 1.60×）——多域均匀膨胀共租浪签名（ZRG-2026-10-20 判例同族），run1 单跑不定谳；**csd 交错 A/B ×5（BASE=zrg-201a 残留 release 真码 style_bench-7490… mtime 11-13 06:20 vs TIP=52e9 真码同文件名 mtime 11-18 07:44，谱系 7948584B→8535976B 随刀系递增已核）浪下两端同打 TIP/BASE 1.347-2.874× 五轮全部 >1**；run2 静窗甄别由 bench-rerun.sh 守候（截止内自动落盘）；r43 静窗 A/B ×9（1.27-1.41×）+ 本轮 A/B 全 >1 = **连续两轮独立可复现**。**② 根因定谳（结构性，非本流可修）**：`ComputedStyle::default()`（`crates/style-system/src/property/default_impl.rs`）内 `vec![…]` 堆分配初始化器 **201a=24 → e429=30 → 52e9=36 线性增长**（css-gaps 列表模型 column/row-rule width/style/color 六字段 + shape/offset/anchor 簇与 mask/mask-border/font 刀系），每分配 ~13ns（r43 实测 +6 分配 ≈ +80ns 外推本轮 +12 分配 ≈ +150ns，A/B 实测带吻合）；csd 为逐元素热路径，页面样式重建全量承压；R5501 基线 11-11 重捕获 5 天即再陈旧 = **基线重捕获治标不可持续**。**③ 其余门禁**：分进程内存 ×2 PASS（/proc VmRSS/VmHWM 只计本次 browser 后代 0.5s 采样：renderer 峰值 141.2/139.9、compositor 峰值 158.4/155.5 ≤192；idle 尾样本 SUM 279.4/278.6 ≤320；compositor 较 r43 148.5/150.3 上移 ~8 MiB 预算内记观察，52e9 compositor 二进制 3f95871→3c64764b）；连通性 ×4（2 内存腿 + 2 parity 腿）Healthy + frame_committed 零硬失败签名、browser 退出正常；product-smoke rc=0（welcome 17.23% 82714/480000 与 r36-r43 逐值同，六视口 struct-check 全绿）；Chrome parity 双腿 report.json sha256 `4bf59035` **与 r43 逐字节同值**（production 双真、state PASS ×5、events 1:1 focus4/type5/toggle12/button16）= 慢性 ZRG-2026-08-15-03 第 44 轮零漂移。**④ 处置：停止自动改代码转专项待用户裁决（run-rules §9 style-system 属渲染流工作面 + guard-policy §2 基线重建需明确授权）**；飞书告警与本条目同批交付。**专项方案候选**：(a) 列表模型默认表示零分配化——`SmallVec<[T;1]>` / `enum Plain(T)|List(Vec<T>)` / 共享静态初值写时复制，default() 回落基线带且随属性增长亚线性，治本，归渲染流排期（s560 窗十刀在途需协调）；(b) 基线政策裁决——按「重建非放宽」口径授权 csd 重捕获 + 附加增长率护栏（每新增属性默认初始化成本配额），治标，5 天 stale 循环再现；(c) (a) 落地前过渡口径待裁决（门禁连续红 DEFERRED 记账或临时豁免 csd 档案）。证据 `.rally/zeroweb-regression-guard/20261117T220515Z/`（gitignore：REPORT.md、build.nohup/build.sh、gates.rc/chain.nohup、product-smoke.log、parity 双腿全套、memory-run{,-leg2}.log + memory-samples.tsv、bench-gate-run1.log、ab-csd-r44.sh + ab-csd-r44.tsv、generic-page/memory-idle scenario json、xvfb 复用记录）。

> **说明**
> 本文记录的是实验性项目的当前实现进度。测试全绿、CI 通过或里程碑推进，并不等于项目已经适合日常使用、商用或其他生产用途；相关风险仍需自行评估。

---

## 当前仓库事实

| 项 | 状态 |
|----|------|
| 仓库代码 | ✅ Cargo workspace 33 个 member（22 库 + 8 应用 + 2 测试工具 + 1 开发工具，含 runtime-config、paint-convert、android-browser 与 media；全部有实质实现） |
| 编译状态 | ✅ `cargo build --workspace` 通过 |
| 测试状态 | ✅ `make test` 全绿零失败（当前锚 19,574P/0F，R4927 轮口径——19,466 锚经窗口内代码腿折入逐级重记（…→19,522→19,547→19,561→19,574）；2026-09-18 静态统计 18,003 个 `#[test]`，运行时计数含参数化与集成 subtest） |
| Clippy | ✅ 零警告（全 workspace） |
| 基准测试 | ✅ 16/16 crate 有 criterion 基准（78+ 个基准） |
| CI | ✅ GitHub Actions（ubuntu/macos/windows）|

### 已实现 crate（22 个）

| Crate | 测试 | 基准 | 说明 |
|-------|------|------|------|
| dom | 853 | ✅ | DOM 树、html5ever 集成、查询 API、序列化、属性、MutationObserver、Range API、遍历/比较方法、Shadow DOM、slot、id_map 自动清理、**模块级单元测试**、**Range select_node/text_content/clone**、**normalize()**、**import_node()**、**slot 分配解析**、**get_elements_by_tag_name_ns**、**has_attribute/remove_attribute/split_text/class_list_replace/contains**、**TreeWalker 深度优先遍历**、**get_elements_by_class_name/set_id/create_comment/insert_before/inner_text**、**NodeIterator 遍历**、**clone_node fragment/replace_child invalid/wildcard tag/nested text_content/insert_before invalid ref**、**R3350 insert_before/replace_child 同父移动 stale-index 回归（前移/后移/到首/no-op + replace no-panic/前移/no-op，7 测）**、**HTML 解析器测试（实体/void 元素/错误恢复/Unicode/大文档）**、**MutationObserver 回调/记录验证**、**Event 传播/stopPropagation/stopImmediatePropagation/非冒泡事件**、**节点比较/文档工厂/DOMTokenList 边界/Range 空/序列化 DOCTYPE/TreeWalker 混合/Event 断连节点** |
| css-parser | 2971 | ✅ | Tokenizer、Parser、选择器、值解析、@规则、:has()、@container、scroll-snap、calc() 嵌套、媒体查询 range syntax、Token 源位置追踪、min()/max()/clamp() 数学函数、**float/clear**、**vertical_align/list_style/viewport calc**、**parse_cursor(26 关键字)/parse_opacity**、**grid-area 解析**、**hwb color/3D transform/嵌套 var**、**148 种 CSS 命名颜色**、**fit-content() 函数**、**conic-gradient at 位置修复**、**min-content/max-content 关键字**、**word-break 属性**、**writing-mode 属性**、**text-decoration-line/text-transform/letter-spacing/word-spacing**、**3D transform 函数**、**媒体查询 only/逗号 OR/prefers-color-scheme/prefers-reduced-motion/pointer/resolution**、**text-overflow/text-indent/table-layout/caption-side/border-collapse/resize**、**counter-reset/counter-increment/content/quotes**、**page-break/box-decoration-break/image-rendering/isolation**、**overflow-wrap/text-align-last/font-variant-numeric**、**direction/unicode-bidi/tab-size**、**column-count/column-width/object-fit/filter**、**border-image-source/slice/width/repeat/outset**、**parse_stylesheet 全路径覆盖测试（40 测试覆盖所有 @规则、选择器、组合器、声明）**、**coverage round 7（145 测试：nth 表达式边界、container 条件、3D transform、conic gradient、calc/min/max/clamp、parse_length 全单位）** |
| style-system | 2292 | ✅ | 级联、继承、计算值、DOM 集成、选择器匹配、简写展开、Grid、@media 评估、Transform、Transitions、Animations、逻辑属性、var() 解析集成、revert 关键字、grid-template-areas、calc/min/max/clamp 管线集成、**matcher 覆盖率测试（SubsequentSibling/PseudoElement/nth-last-child/nth-last-of-type/nth-of-type/:not/:is/:where/:lang/:has NextSibling+SubsequentSibling/container 范围/操作符/冒号语法/@supports AND/OR/NOT/@media+@container 集成/属性选择器 DashMatch/Prefix/Suffix/Substring）**、**apply_property_value 全分支覆盖测试（107 测试覆盖所有 CSS 属性）**、**apply_coverage_extra（77 测试覆盖 invalid fall-through、background-position TwoValue、border-image 非 Px、columns 简写、filter 函数）+ parse.rs 覆盖率（20 测试覆盖 border-style/outline-style/grid-line/cursor/scroll-snap/font-family/line-height 等）**、**matcher coverage round 3（168 测试：nth 负系数、length_to_px 非 px 单位、get_axis_size、ContainerContext、@layer/@supports/@container 集成、evaluate_supports_condition 逻辑运算符）** |
| layout-engine | 1465 | ✅ | taffy 集成（Block/Flex/Grid/Position）、Grid 轨道解析、Grid 项放置、auto-fill/minmax()、grid-template-areas、零尺寸容器、深层嵌套、aspect-ratio 布局、box-sizing:border-box 测试、**z_index/is_sticky 字段**、**fixed 视口坐标调整**、**text-align center/right/justify**、**vertical_align Sub/Super/TextTop/TextBottom**、**converter 全变体覆盖**、**混合字号/零容器/空白文本**、**overflow/z_index/content_clamp/深层嵌套**、**负 margin/嵌套 flex/absolute-in-relative/overflow hidden/grid auto/零高度块**、**grid 3x3 区域/auto-fill minmax/命名区域解析/百分比 gap**、**grid dense/span/min-max 约束**、**负 margin 合并/grid 行跨行/混合 CJK-Latin/absolute-in-relative/flex 不增长**、**grid 全跨/flex gap/大 padding/absolute 拉伸/inline-block 百分比**、**CJK 字符检测/字符串宽度估算/converter 私有函数/overflow 转换/fixed 视口调整/absolute_position 边界**、**letter-spacing + word-spacing 行内布局集成** |
| engine | 2382 | ✅ | 渲染管线、paint（文本/glyph、overflow clip、border-radius）、dirty tracking、compositing（z-index 排序）、CSS transform、增量渲染、**资源预加载**（ResourcePreloader：preload/prefetch/preconnect/dns-prefetch、优先级排序、URL 去重、生命周期追踪）、**DOM Bridge（polyfill: 事件系统 + Fetch API + console + setTimeout/setInterval + insertBefore/replaceChild/cloneNode + CSSStyleDeclaration + DOMTokenList + innerHTML/outerHTML + textContent/innerText + 导航属性）**、**opacity/text-decoration/text-transform 渲染集成**、**letter-spacing + word-spacing 渲染集成**、**text-overflow: ellipsis 渲染**、**CSS filter 渲染（FilterPrimitive + 10 种滤镜函数）**、**column-rule 渲染（多列分隔线）**、**list-style-image 渲染（URL 列表标记）**、**empty-cells:hide 渲染（空表格单元格跳过背景边框）**、**CSS 动画运行时（AnimationClock + 关键帧插值 + 管线集成）**、**CSS Transition 执行引擎（TransitionClock + 管线集成 + 样式变化检测 + 22 测试）**、**CSS 交互/提示属性渲染**（cursor/image-rendering/isolation/will-change/pointer-events/user-select/overscroll-behavior/touch-action 指示器 + 26 测试）、**CSS 表格/3D/吸附属性渲染**（scroll-snap 吸附轴+对齐点 + perspective 透视消失点 + backface-visibility:hidden 虚线边框 + transform-style:preserve-3d 立方体图标 + border-spacing 间距标记 + caption-side 标题位置 + 14 测试）、**CSS contain/unicode-bidi/box-decoration-break/overflow-wrap/text-align-last/break/scroll-area/snap-stop/container-type 渲染**（10 属性指示器 + 43 单元测试） |
| render-foundation | 659 | ✅ | GPU/CPU 渲染、字体栈、image cache + GC、clipping/scissor、颜色 RGBA clamping、image cache eviction、surface resize、**文本整形器（TextShaper + 换行）**、**多次 resize/RGBA clamp/零 max_entries**、**空字符串/单字整形/opacity 零**、**damage tracker 单矩形/重叠合并/颜色钳位/resize 保留/max_entries 零**、**rect 交集/并集/颜色 alpha 混合/面积**、**20 非重叠 rect/Color lerp 透明/缓存 GC 优先级/帧缓冲四角/圆角矩形包围盒** |
| host-runtime | 232 | ✅ | winit 窗口、事件循环、mouse/cursor/IME 事件、**resize 事件**、**鼠标坐标**、**IME composition**、**键盘修饰键**、**修饰键组合/按键重复/鼠标按钮/零尺寸 resize**、**mouse 坐标/keyboard key_code/resize/touch/IME composition**、**多触点/按钮坐标/按键码**、**连续 resize/全修饰键/中键/IME 空/键盘释放**、**HostError debug/TouchPhase 比较/scroll delta 转换/Destroyed 事件忽略/MouseButton 相等性** |
| net | 472 | ✅ | HTTP client、URL、导航历史、Cookie、send 集成测试、cookie 过期/SameSite、**URL userinfo/port/query 边角场景**、**SameSite 全矩阵**、**重定向深度边界**、**非默认端口 origin**、**第三方 cookie/会话 cookie/前进超出**、**URL fragment/空路径/导航历史检查/cookie httpOnly/响应状态文本**、**WebSocket 真实实现（tungstenite：ws/wss 连接、文本/二进制消息、非阻塞轮询、Close 帧、错误类型）**、**URL hash/查询参数/请求链/状态文本**、**IPv6 host/SameSite Strict/go_back initial/304 status/URL encoded chars**、**blob/file URL/Cookie path 匹配/导航边界/查询参数边界**、**HTTP 响应缓存（Cache-Control/ETag/LRU 淘汰/条件请求头/大小写不敏感解析/可缓存状态码过滤）** |
| runtime-config | 1 | — | **运行时环境变量配置唯一入口**：`ENVIRONMENT_VARIABLES` 权威清单（渲染后端/多进程/沙箱/compositor 开关）+ 统一解析函数（`enabled_when_true`/`enabled_by_default`/`enabled_unless_zero`/`optional_path`/`positive_usize`/`renderer_mode`） |
| security | 398 | ✅ | 同源策略、CORS（preflight）、CSP（nonce/hash/navigation/document）、mixed content blocking、sandbox、COOP/COEP、HSTS、**权限模型**（PermissionManager：11 种权限类型、3 种状态、按 origin 隔离存储）、**站点隔离**（SiteIsolationManager：3 种策略、site-per-process 模型、跨站 DOM 访问阻止）、**CSP scheme-source**、**report-only**、**CORS 简单请求/preflight 生成**、**sandbox 导航/弹窗**、**origin null/invalid/port**、**CSP img-src/nonce/default-src、CORS max-age/wildcard**、**CSP 同源脚本/内联样式/data URI/简单请求 GET**、**report-only/preflight 自定义方法/mixed content/不同端口/sandbox allow-scripts**、**CSP default-src/frame-src/CORS 凭证/混合内容/sandbox popups**、**CORS custom header/CSP data URI/cross-protocol origin/sandbox popups/mixed content ws**、**CSP upgrade-insecure-requests/strict-dynamic/CORS 多方法预检/同源默认端口/sandbox dangerous combo/mixed content blob/COOP popups 矩阵**、**SecurityContext 统一安全门面（HSTS 预加载 40+ 域名 + 混合内容阻止/升级 + 资源加载检查管线 + 24 单元测试）** |
| protocol | 312 | ✅ | IPC 消息、bincode 序列化、**mock channel 契约**、**确定性编码**、**对抗性反序列化**、**大消息/unicode/排序**、**空载荷/unicode 载荷/顺序保持/确定性编码/大载荷 10KB**、**FIFO 循环/Session 存储类型/零 ID/二进制 body/错误 Display**、**NavigateParams referrer/KeyboardEvent 修饰键/MouseEventType 字节区分/ScrollEvent 负值/GoBack vs GoForward**、**method 大小写/referrer 自引用/Ok vs Error 字节/status codes/non-ASCII headers/StorageOp value/交错 send-recv/Send+Sync/空 headers/空 key/负坐标**、**PipeTransport 管道传输（帧协议往返/发送/过大帧/try_recv 不支持）**、**SharedMemoryChannel 共享内存通道（双向/try_recv 空/recv 空错误/关闭清空/多消息顺序）**、**RendererHandle/ProcessManager（状态比较/创建/消息 ID/不存在的渲染进程/shutdown/check_crashes/心跳常量/模拟导航+网络+存储+心跳+输入+生命周期+失败+崩溃）** |
| storage | 718 | ✅ | localStorage、sessionStorage、IndexedDB（IdbKeyRange/IdbIndex/IdbCursor/IdbTransaction）、Cache API、**Service Worker 注册表（生命周期状态机、scope 匹配、fetch 拦截、Cache 集成）**、**事务缓冲/回滚**、**NaN/Infinity key 排序**、**唯一索引冲突**、**Cache API CRUD**、**cursor advance/continue/索引迭代**、**事务 commit/abort**、**key/used_size/cache delete+has**、**clear+set/delete range**、**delete_object_store/update_existing/clear/空 store cursor/cache has**、**update/会话隔离/count/cursor 越界/keys/事务提交/remove 不存在**、**cursor reverse/cache put URLs/localStorage key order/multiEntry index/sessionStorage clear**、**IDB 事务空 store/KeyRange 多类型/cursor advance(0)/Cache 覆写/空字符串值/唯一索引/multiEntry 空数组**、**SW 边界测试（12 个：状态转换/scope/intercept/multi-origin/cache round-trip）**、**IDB types 覆盖率测试（18 个：跨类型 key 比较/binary key/hash 一致性/array key 边界/KeyRange contains/multiEntry index）** |
| canvas | 813 | ✅ | Canvas 2D API、路径、变换、drawImage、shadow 属性、**Path2D 高级方法**、**Path2D svgString 构造（from_svg：全 SVG 命令 M/L/H/V/C/S/Q/T/A/Z + 相对/隐式重复/弧端点→中心转换，R3307）**、**lineDash**、**roundRect 圆角扁平化**、**alpha 混合**、**像素边界溢出**、**clip+drawImage**、**ellipse/arcTo/conic_gradient**、**line_join/line_cap stroke 渲染**、**is_point_in_stroke**、**composite operation 像素级验证**、**image_smoothing_enabled**、**gradient 逐像素光栅化（CanvasStyle::sample_at + blit_rect_gradient/blit_path_gradient，linear/radial/conic，R3079）**、**raster 覆盖率测试（flatten_round_rect/compute_arc_to_geometry/flatten_arc_to/flatten_path/flatten_path_for/blit_path_to_pixels/blit_stroke_to_pixels/blit_line_cap/stroke_outline_vertices/11 种 composite_pixel 操作）** |
| webview | 678 | ✅ | WebView 嵌入 API、Builder、event callbacks、load_url fetch、execute_script、**Service Worker 集成（register/install/activate/unregister + fetch 拦截）**、**CSS 缓存持久化**、**extract_origin/execute_wasm/fail_load/set_title/inject_css 覆盖率测试**、**execute_script 错误路径/WebViewConfig 默认值**、**Web Worker 管理（create_worker/post_message_to_worker/execute_worker_script/poll_worker_events/terminate_worker/terminate_all_workers，17 集成测试）**、**SecurityContext 集成（fetch_url 安全检查 + HSTS 升级 + 混合内容阻止 + 子资源检查 API）**、**WASM 自动桥接（instantiate/compile/instantiateStreaming + validate + _start 自动执行 + 导出调用队列 + 内存注入）** |
| wasm-sandbox | 231 | ✅ | WASM 运行时（wasmi）、host function imports、fuel/execution limiting、**host 错误传播**、**参数类型校验**、**offset 溢出**、**memory grow/多参数 host/递归限制**、**memory 读写/多函数/fuel 消耗/global 读取/无效模块错误**、**多实例隔离/table 导出/global 读取/fuel 追踪/错误处理**、**fuel 禁用 get_fuel/u64::MAX fuel/内存边界读写/i64 Display/config chaining/has_memory 误匹配/空字符串函数名/多实例独立/内存 roundtrip/start 函数 trap**、**边界测试（6 个：错误参数/global export/Display/config 链/空模块/多函数 linker）** |
| script-sandbox | 277 | ✅ | **V8 引擎集成（rusty_v8）**、Isolate/Context 管理、脚本编译执行、JSON 输出、错误处理（编译/运行时/超时）、**状态隔离、execute_json 边界测试、ES6+ 特性（Map/Set/Symbol/Proxy/async/await/rest/for-of/静态方法）**、**Dedicated Worker（WorkerRuntime：独立线程 V8 持久上下文、postMessage/onmessage 通道、terminate 生命周期、16 测试）**、**ES Module Sandbox（EsModuleSandbox：源码转换支持 export/import 语法、ModuleRegistry 模块注册表、import.meta.url、链式依赖解析、30 测试）**、**持久化 Context 优化（SandboxConfig::persistent_context + Global<Context> 缓存复用 + reset_context()，6 测试）** |
| browser-shell | 320 | ✅ | **浏览器应用数据模型**：Tab/TabManager（多标签页管理、导航历史、**拖拽排序 move_tab**）、Bookmarks（书签/文件夹增删改查）、History（页面访问记录、搜索、清除）、BrowserShell（顶层协调器）、**Autocomplete（地址栏自动补全，历史+书签搜索、分数排序、书签优先）**、**ContextMenu（右键上下文菜单，5 种场景默认菜单项）**、**Tab 拖拽边界/导航历史边界/Bookmarks 过滤/History clear+search/Download 移除/Autocomplete 空查询+大小写/BrowserShell 导航清空前进/Settings 搜索/ContextMenu 子菜单查找** |
| page-runtime | 123 | ✅ | WPT / TabWorker / zero-renderer 三条页面路径共享的页面加载与运行时契约（`PageLoadHost` / `AsyncFetchHost` / `BlockingFetchHost`），让 in-process（webview）与 IPC（renderer）两种宿主复用同一套分阶段页面加载逻辑（runtime 统一） |
| product-version | 2 | — | 产品版本号（从构建日期推导，随 `zero-product-version` 分发） |
| psl | 25 | — | **公共后缀列表（PSL）解析与注册域名（eTLD+1）提取**：三类规则解析（普通/通配/例外，与上游 `public_suffix_list.dat` 语法一致）、`registrable_domain()`（IP/单标签/公共后缀原样返回）、全局共享实例 `shared()`、`from_rules()` 可注入完整数据；接入 site-isolation（R3380，`Site::from_origin` 经 PSL 计算真 eTLD+1） |
| media | 55（默认 feature）/ 56（audio-cpal） | — | **媒体解码管线（media-playback / media-audio goal 产物，2026-09-01）**：webm/Matroska + mp4/ISO-BMFF demux、VP9 纯 Rust 解码（`rusty_vp9`）、AV1（`decode-av1`）、H.264（`decode-h264`，openh264，D-RFC-3 获批切片 1+2——切片 2 含 AAC 音频链/伴生轨/precise-seek）、YUV→RGBA、音频解码（symphonia mp3/ogg-vorbis、opus-decoder、webm 音轨重封装）、`open_media` 容器/编码嗅探自路由、`VideoPlayer` 播放驱动（`VideoClock` trait）、`AudioSink` 输出面（NullSink 可观测 / CpalSink 真设备）、混音总线、Web Audio 振荡器合成最小面 |
| paint-convert | 3 | — | **IPC 图元快照 → 渲染图元公共转换层（webdriver-screenshot goal 产物，2026-09-09）**：`to_render_primitives` 唯一映射实现（compositor/browser/webdriver 三端共享），13 类绘制图元逐一映射 + 字体变轴校验 + 字形文本源 intern + text_control_boundaries 透传；取代 compositor `convert.rs` / browser `paint_ipc.rs` 双份维护 |

### 跨 crate 集成测试

| 测试模块 | 测试数 | 覆盖场景 |
|----------|--------|----------|
| DOM Bridge Polyfill (V8) | 58 | createElement/textNode/setAttribute/appendChild/insertBefore/cloneNode/textContent/replaceChild/DocumentFragment/getElementById/CSSStyleDeclaration/DOMTokenList/navigation properties/innerHTML/outerHTML/Fetch/Headers/Response/Storage/MutationObserver/CustomEvent/IntersectionObserver/ResizeObserver/WebAssembly/PerformanceObserver/setTimeout/setInterval |
| CSS + Style System | 3 | 样式计算、级联优先级、继承 |
| Render Pipeline | 4 | 完整管线、CSS 集成、耗时分解、复杂页面 |
| Net + Security | 3 | 同源判断、CORS 策略、安全上下文 |
| Storage + Protocol | 3 | localStorage CRUD+IPC、session 隔离、origin 隔离 |
| Protocol + Navigation | 1 | 导航历史 + IPC 序列化 |
| Canvas + Render | 5 | Canvas 绘图图元、路径、变换、save/restore、WebView 集成 |
| WASM Bridge | 18 | API 可用性、validate 魔术字节、compile 桥接、instantiate 桥接、call_wasm_export、多次调用、无桥接不影响、pendingBridge 清空、instantiateStreaming API/字节、validate 边界条件、JS 导出可调用、_hostBacked 标志、memory 导出、compile 注入、调用队列基础设施、validate 多类型 |
| WebView Full Pipeline | 4 | 完整生命周期、复杂页面、重复加载、脚本占位 |
| Web API Pipeline | 17 | JS DOM 操作、V8 内置 API、DOM polyfill、CSS 渲染管线（flex/grid/positioned/shadow/gradient/custom-props/media-query） |
| CSS Transform Pipeline | 1 | CSS 解析→样式系统→计算值 |
| Media Query Integration | 1 | 媒体查询解析 + 上下文评估 |
| Canvas Gradient | 1 | 渐变色采样 + 颜色插值 + 逐像素光栅化验证（R3079 sample_at + fill/fillRect） |
| Grid Layout Pipeline | 1 | 网格模板→样式→布局引擎 |
| Counter Cascade | 1 | counter-reset/increment 级联集成 |
| Overflow Wrap Pipeline | 1 | overflow-wrap 解析→样式→继承验证 |
| Text Align Last Pipeline | 1 | text-align-last 解析→样式验证 |
| Direction Pipeline | 1 | direction:rtl 解析→样式→继承验证 |
| Tab Size Pipeline | 1 | tab-size 解析→样式→继承验证 |
| Storage Protocol IPC | 1 | StorageManager 操作→IPC 序列化→反序列化验证 |
| Border Image Source Pipeline | 1 | border-image-source 解析→样式→计算值 |
| Border Image Slice Pipeline | 1 | border-image-slice 解析→样式→fill 验证 |
| Border Image Repeat Pipeline | 1 | border-image-repeat 解析→样式→水平/垂直验证 |
| Border Image Width Pipeline | 1 | border-image-width 解析→样式→Number/Length 验证 |
| Border Image Outset Pipeline | 1 | border-image-outset 解析→样式→Length/Number 验证 |
| Text Shadow Pipeline | 1 | text-shadow 解析→样式→偏移/颜色验证 |
| Box Shadow Pipeline | 1 | box-shadow 解析→样式→偏移/模糊验证 |
| Box Shadow Inset Pipeline | 1 | box-shadow:inset 解析→样式→inset 标志验证 |
| Text Shadow Inheritance | 1 | text-shadow 继承验证（父→子） |
| Outline Width Pipeline | 1 | outline-width 解析→样式验证 |
| List Style Image Pipeline | 1 | list-style-image url 解析→样式→计算值 |
| Column Gap Pipeline | 1 | column-gap 解析→样式验证 |
| Text Shadow Inheritance Pipeline | 1 | text-shadow 继承（父→子）验证 |
| Box Shadow No Inheritance | 1 | box-shadow 不继承验证 |
| Outline Shorthand Pipeline | 1 | outline 简写→展开→样式验证 |
| Gap Shorthand Pipeline | 1 | gap 简写→row-gap + column-gap 展开 |
| Empty Cells Pipeline | 1 | empty-cells:hide 解析→样式验证 |
| Border Spacing Pipeline | 1 | border-spacing:5px 10px 解析→样式验证 |
| Empty Cells Inheritance | 1 | empty-cells 继承验证 |
| Border Spacing Inheritance | 1 | border-spacing 继承验证 |
| Box Shadow Render Pipeline | 1 | box-shadow 通过完整管线生成 ShadowPrimitive |
| Box Shadow + Background Pipeline | 1 | background-color + box-shadow 组合渲染 |
| Box Shadow Negative Offset | 1 | box-shadow 负偏移渲染验证 |
| Box Shadow None Pipeline | 1 | 无 box-shadow 时 shadows 为空 |
| Text Shadow Render Pipeline | 1 | text-shadow 通过完整管线生成阴影 glyph |
| Text Shadow + Color Pipeline | 1 | text-shadow 特定颜色渲染验证 |
| Text Shadow None Pipeline | 1 | 无 text-shadow 时无额外 glyph |
| Background Image URL Pipeline | 1 | background-image url() 生成 ImagePrimitive |
| Background Image None Pipeline | 1 | background-image none 时 images 为空 |
| Background Image + Color Pipeline | 1 | background-color + background-image 组合渲染 |
| Box Shadow Not Inherited | 1 | box-shadow 不继承验证 |
| Text Shadow Inherited Pipeline | 1 | text-shadow 继承验证（父→子） |
| Box Shadow + Outline Pipeline | 1 | box-shadow + outline 组合渲染 |
| All Three New Properties | 1 | box-shadow + background-image + text-shadow 全组合 |
| Box Shadow Spread Only | 1 | box-shadow 仅 spread-radius 渲染验证 |
| CSS Typography + Form Pipeline | 43 | font family/size/weight、text align/decoration/transform/spacing/line-height、named/rgb/hsl/hex colors、border styles/radius、box-shadow multiple/inset、text-shadow、linear/radial gradients、opacity、visibility、overflow hidden、CSS variables + fallback、absolute positioning、z-index stacking、display inline-block/none/flex/grid、box-sizing、calc()、2D transforms、filter blur/grayscale、composite pages（landing/styled-form/pricing/blog/dashboard） |
| CSS Counter Pipeline | 2 | counter-reset/increment 管线、counter-set 管线 |
| TransformPrimitive Pipeline | 3 | transform-origin rotate 渲染管线、scale 渲染管线、translate-only 不生成 TransformPrimitive |
| Multi-Process Architecture | 18 | IPC 传输层（共享通道双向/帧协议/序列化确定性）、页面加载生命周期、网络请求代理、存储操作代理、输入事件转发、心跳机制、加载失败、崩溃通知、大载荷传输、双向并发通信、导航历史操作、多存储操作组合 |
| Security Pipeline | 52 | CSP+Origin（parse/self/none/nonce/hash/data:blob:/scheme-source/wildcard/connect-src）、CORS+URL（简单请求/预检/凭证冲突/自定义头）、HSTS+升级（解析/删除/辅助函数）、混合内容（检测/分级）、沙箱（导航/弹窗/标志）、权限+Origin隔离、站点隔离+Origin、COOP+COEP、复合安全管线、**SecurityContext 集成**（HSTS 预加载升级/子域名/运行时注册/混合内容阻止+升级/Origin 清除/HSTS+Origin 一致性/混合内容完整矩阵/CSP 组合） |
| Real Website Compat | 24 | Top 20 真实网站兼容性（example.com/info.cern.ch/httpbin.org/w3.org/whatwg.org/lite.cnn.com/lobste.rs/curl.se/ietf.org/datatracker.ietf.org/rust-lang.org/python.org/nodejs.org/docs.rs/jsonplaceholder.typicode.com/github.com/stackoverflow.com/cloudflare.com/w3schools.com/pkg.go.dev）+ 综合测试（多站点顺序加载/多视口响应式/页面结构验证/性能验证） |

---

## 最近完成的改进

### fix zero-script-sandbox ES module 循环 import 栈溢出 abort（本轮 R3398，script-sandbox es_module.rs——攻击者可控 module 图致进程崩溃 DoS）

承接 R3397（V8 堆限钳制）续审 script-sandbox `es_module.rs`（874 行，源码转换式 ES module：import/export 转 IIFE 内联）。派 deep-review 对抗审 ES module 转换逻辑。**发现并修复 1 真 DoS bug**——

#### R3398：transform_import 循环 import 无限递归致栈溢出 abort（攻击者可控 DoS，es_module.rs:367 transform_import）

**问题**：`transform_import` 四个 import 分支中，仅副作用 import（`import 'm'`，line 378）在递归 `build_dep_iife` 前查 `if !visited.contains(&specifier)`；namespace（`import * as`）/ named（`import {x}`）/ default（`import X`）三分支仅 `visited.insert(specifier)` 后**无条件递归**，不查 visited。故**循环 import（a↔b）→ 无限递归 → 栈溢出 abort 整个进程**（实测 probe：`thread '...' has overflowed its stack / fatal runtime error: stack overflow, aborting`，cargo test 直接 abort）。**攻击者可控**：页面提供的 module 源码（import 图）即可触发——`a.js: import {b} from 'b'` + `b.js: import {a} from 'a'` → 浏览器渲染该页即崩溃。比 R3397（需嵌入方误配 config）**更严重**，是 page-supplied DoS。

**另发现**：default import 分支（旧 line 417-419）把依赖 IIFE 字符串拼接 **3 次**（`(d).default !== undefined ? (d).default : d`，d = 整段 IIFE）→ 模块副作用执行 3 次（正确性 + 性能 + 资源放大 bug）。

**根因修复**：抽 `inline_dep_once(specifier, registry, visited)` 助手——首次访问（`!visited.contains`）递归内联 `build_dep_iife`；已访问（循环/菱形）返空对象占位 `(function(){return {};})()`（防崩溃安全近似：JS 仍可编译运行，循环绑定解析为 undefined——转换式架构无 live binding，非 spec 精确循环语义，但避免崩溃是首要）。三未守卫分支统一走 `inline_dep_once`。default import 改为求值一次存 `_dep_<name>` 变量（消除三重副作用）。

**复现锁定**：3 新测——① `test_circular_import_no_overflow_r3398`（a↔b：修复前 abort，修复后编译成功 + 输出含空对象占位 `(function(){return {};})()`）；② `test_diamond_import_no_duplicate_inline_r3398`（菱形 root→a/b→shared：shared 经 visited 守卫不重复递归致冗余/崩溃）；③ `test_default_import_evaluated_once_r3398`（默认导入依赖 IIFE 只求值一次，模块体 '2;' 出现 1 次，旧实现 3 次）。

**其余 review 健壮**（不修）：副作用 import visited 守卫已对（line 378，本修就是对齐它）/ `compile_module_script` visited `&mut HashSet::new()` 透传全图 / 动态 `import()` 重写 `rewrite_dynamic_imports` 边界 / `extract_*_import_specifiers` 解析。

**测试**：zero-script-sandbox **148→151 全绿**（+3），零回归；fmt clean + clippy 零警告。**完成项**（commit `7d629e7b`）。**EsModuleSandbox 当前接线状态**：经 webview/engine 模块执行路径使用（非纯未接线），故循环 import 修复有真实页面渲染路径影响（页面提供 module 图即可触发，已闭合）。

**结论**：script-sandbox 域**两连产出**（R3397 堆限 + R3398 循环 import），证实 R3396「自主面趋零」**确过早**——安全关键沙箱逐文件审仍出真 page-supplied DoS bug。累计 R3388-R3395 + R3397 + R3398 共 **10 真 bug/合规缺口已修**。下轮续审 script-sandbox 剩余（v8_runtime 回调 panic / worker 线程 / quickjs 对称性）+ engine js_dom_bridge。

### fix zero-script-sandbox V8 堆限制 initial<=max 钳制防崩溃（本轮 R3397，script-sandbox lib.rs——公共嵌入 API 信任边界输入校验，防 V8 abort）

承接 R3396（验证轮「自主面趋零」结论）。**R3396 遗漏 script-sandbox 自主域**（rule 9 zero-web 流域，P1b 沙箱）——本轮转审纠正。审 `v8_runtime.rs`（1847 行）+ `lib.rs`（167 行），派 deep-review 对抗审 V8 执行沙箱。**发现并修复 1 真 bug**——

#### R3397：v8_heap_limits 不钳制 initial<=max 致 V8 abort（公共 API 致命崩溃，lib.rs:107）

**问题**：`v8_heap_limits`（lib.rs:107）当 `heap_limit=0`（无上限）时 max 取 4GB 默认，但**不钳制 `initial_heap_size <= max`**。V8 `SetHeapLimits` 要求 `initial <= max`，**违反触发 `CHECK` 致命崩溃整个进程**（abort，非 panic 可捕获）。`SandboxConfig` 是**公共嵌入 API**（zero-webview 供外部宿主嵌入），嵌入方传 `initial_heap_size > 4GB`（或 > 显式 heap_limit）+ `heap_limit=0` → V8 CHECK abort 进程。函数 docstring（lib.rs:103）甚至明记「V8 要求 initial<=max」但代码未强制——**注释与实现不符**（注释对，实现漏）。

**根因修复**：`let initial = config.initial_heap_size.min(max)`（信任边界输入校验，单 guard，CLAUDE.md「安全措施/输入校验不可简化」）。本仓内置 WebView config（initial=128MB / heap_limit=0，webview.rs:1207）不受影响，此为公共 API 防崩溃硬化。

**复现锁定**：新测 `v8_heap_limits_clamps_initial_to_max_r3397`（lib.rs 测模块）——① 两零→None；② 8GB initial + heap_limit=0 → 钳到 4GB max；③ 512MB initial + 256MB heap_limit → 钳到 256MB；④ 128MB 合法配置（内置 WebView）回归保护原样。

**评估不修的候选**（提及，防重复挖）：
- **watchdog_seq u64 溢出**（v8_runtime.rs:352 `self.watchdog_seq += 1`）——需 2^64 次 execute 触发，物理不可达，遵循「不修不可能场景」放弃。
- **JS 字符串无界分配 `to_rust_string_lossy`**——受 V8 堆限（heap_limit）兜底，主防护已就位；加 size 检查属过度设计（defense-in-depth 但违反「不过度设计」），放弃。

**其余 review 健壮**（不修）：watchdog 线程 Drop 正确 join + recv_timeout 双分支（Ok/Timeout/Disconnected）/ TryCatch has_caught 异常清理防 isolate 中毒 / 编译-执行边界（SyntaxError 阻止执行）/ 回调 `unwrap_or_default` 防 panic / `has_terminated`+`cancel_terminate_execution` / 线程本地回调注册 bounds-checked（负索引拒）。

**测试**：zero-script-sandbox **147→148 全绿**（+1），零回归；fmt clean + clippy 零警告（v8 feature 本地可编译验证）。**完成项**（commit `8b027ee5`）。

**结论**：R3396「自主面趋零」**过早**——script-sandbox 域（安全关键沙箱）被遗漏，本轮即出 1 真 bug（公共 API 致命 abort）。累计 R3388-R3395 + R3397 共 **9 真 bug/合规缺口已修**。**关键教训**：自主域覆盖易有盲区（script-sandbox/es_module/worker/engine js_dom_bridge 均未深审），**不宜急于降频**。下轮续审 script-sandbox 其余文件 + engine js_dom_bridge Rust 侧。

### 验证 site-isolation PSL 接线 + cache_api spec 复核 全确认合规（本轮 R3396，无代码变更——验证轮）

承接 R3395（cookie 排序闭合最后 cookie 缺口）。本轮转审上轮 CONTINUE 指向的 site-isolation PSL 接线正确性 + cache_api.rs spec 复核。**完成项**（无代码，验证轮）：

- **site_isolation.rs PSL 接线正确性确认**：`Site::from_origin`（site_isolation.rs:32）经 `zero_psl::PublicSuffixList::shared().registrable_domain()` 计算真 eTLD+1（非旧「取末两段」）；`is_same_site`→`from_origin`→PSL 全链一致。grep 全 security crate **无 `extract_registrable_domain` / 朴素 `split('.')` / `ends_with` host 残留**。所有 Site 构造（129/135/143/164/206/270+）均经 `from_origin`，无旁路 PSL 的 host 比较；`can_access_parent_dom`（179/192/196）用 `is_same_site`+origin `==`（精确 scheme+host+port，正确区别于 site 相等）。**a.github.io / b.github.io 经真 PSL 正确判不同站**（旧「取末两段」误判已 R3381 修）。**PSL 全局接线一致**：zero-net cookie `is_same_site`（R3381）+ zero-security `Site::from_origin`（R3380）共用同一 `zero_psl`，无朴素残留。
- **cache_api.rs spec 复核**：R3376 记「`put` 不拒非 GET/HEAD」疑缺口，本轮核 **Fetch Cache API spec §4.3.3 `CachePut`**：Cache.put **允许任意方法**（脚本控制的显式缓存，非 HTTP 缓存），仅 opaque 响应 + 非 GET/HEAD/POST 才拒。故 `put` 接受 POST/PUT **是 spec 合规非 bug**（`test_cache_delete_preserves_different_method` 证此为预期）。R3376「无 Vary 头匹配 / match first-wins」为已知简化（内存数据模型，无算术/无 unsafe，35+ 测覆盖），非单点 bug。

**核心结论**：**security + net + storage + cookie 全自主域 spec 合规面已彻底穷尽**——R3388-R3395 共 8 真 spec bug/安全漏洞/合规缺口已全修，其余文件全确认 spec 合规。**自主推进面确已趋零**，剩余全为用户/环境门控（P1b default-on / P3 GPU / PSL 请求路径接线属 html-compat 流域）。**建议降低本流 rally 频率**（R3394/R3395 已挖尽 cookie 三连，再复扫已证合规代码违反「精准修改/不过度设计」）；用户点名 P1b default-on / GPU 就绪 / html-compat 告段落则恢复高频。

### feat zero-net cookie RFC 6265 §5.4 顺序 + §5.3.11 creation-time（本轮 R3395，net cookie.rs——闭合最后 cookie spec 缺口）

承接 R3394（public-suffix 拒绝）。本轮实现 cookie defer 项 **RFC 6265 §5.4 排序**（R3391-② / R3394-① defer），**闭合 cookie 域最后 spec 缺口**。**发现并修复 cookie header 顺序 spec 缺口**——

#### R3395：cookie header 未按 RFC 6265 §5.4 排序（spec 合规缺口，cookie.rs:513/524 cookie_header）

**问题**：RFC 6265 §5.4 要求 Cookie 请求头中 cookies 按 **path 长度降序（longest-path-first）**排列，同 path 再按**创建时间升序（earliest-creation）**。旧 `cookie_header`/`cookie_header_with_context` 按 Vec 插入序输出（`.iter().filter().map().collect()` 无排序），故 `Path=/app/sub` cookie 可能排在 `Path=/app` 之后——更具体路径不优先。破坏依赖「具体路径优先」语义的服务器（如某 cookie 同时设 `/app` 与 `/app/sub`，服务器期望更具体者在 Cookie 头靠前以便优先解析）。

**根因修复**：① `Cookie` 增 `creation_time: u64` 字段（parse_set_cookie 占位 0）；② `CookieStore::add` 首次存储填 `now`（SystemTime secs），**同 name+domain+path 更新时保留原 creation_time**（RFC §5.3 step 11——`prev_creation = find(...).map(creation_time)` → `unwrap_or(now)`，保持排序键稳定，值更新不改变创建序）；③ 两 header 函数 collect `Vec<&Cookie>` → `sort_cookies_for_header`（path-len desc + creation asc，稳定排序）→ join。

**关键技术点**：① **稳定排序**（`sort_by`）保证同 path+creation 的 cookie 保持存储序（确定性输出）；② `cookie_path_len` = Path 字节数，无 Path/default-path "/" 计 1（§5.1.4）；③ creation_time 保留经 `prev_creation` 显式取原值实现，而非依赖 add 调用顺序。

**复现锁定**：2 新测——`test_cookie_header_rfc_5_4_ordering_r3395`（① longest-path-first：`/app/sub` 排 `/app` 前；② 同 path earliest-creation：creation_time=100 排 =200 前，用受控 creation_time 直接 push 绕开同秒不可分 + add 的 now 覆盖）、`test_cookie_update_preserves_creation_time_r3395`（同 key 更新保留原 creation_time=1000，§5.3 step 11）。现有 `test_cookie_header_multiple_ordering`（b 早于 a 创建→b 前于 a）与新排序**一致未回归**（证实排序符合既有预期）。

**其余 review 健壮**（不修）：R3391 __Host-/__Secure- 前缀 + R3394 public-suffix 拒绝 + R3225 domain-match + SEC-08 SameSite=None 须 Secure + path 边界 §5.1.4 + SameSite Strict/Lax/None。**Cookie spec 合规全闭合**（前缀/public-suffix/排序/同站注入/SameSite/Secure/Path/过期）。

**测试**：zero-net **430→432 全绿**（+2），零回归；fmt clean + clippy 零警告。**完成项**（commit `5809d442`）。**CookieStore 仍未接线生产**（仅测试引用），但库 API 面合规性已闭合（与 R3391/R3394 同，未来接线即正确）。

**结论**：cookie 子域连挖（R3391 前缀 + R3394 public-suffix + R3395 排序）三连产出，证实「翻新 defer 项」仍可挖真合规缺口。累计 R3388/R3389/R3391/R3392/R3394/R3395 共 **8 真 spec bug/安全漏洞/合规缺口已修**。cookie 域已无 spec 缺口；下轮可转 site-isolation PSL 接线正确性审 / storage / 或 P1b/P3。

### feat zero-net cookie public-suffix Domain 拒绝 + PSL 状态勘误（本轮 R3394，net cookie.rs——RFC 6265bis §5.4 step 1 跨站 cookie 注入安全漏洞）

承接 R3393（验证轮「自主面趋零」结论）。本轮**开工前 git log 复核发现 PSL 状态过时**——R3393 记「PSL 受 html-compat rule 9 阻塞」**已过时**：PSL 基础设施已由本流 **R3380-R3383 交付并接线**（`crates/psl/` zero-psl crate + `Site::registrable_domain` 经 `zero_psl::PublicSuffixList::shared()` 计算真实 eTLD+1，site_isolation.rs:33；zero-net 已依赖 zero-psl 用于 `is_same_site`，cookie.rs:556）。此勘误**解锁 R3391 defer 项**（cookie public-suffix Domain 拒绝），本轮实现。**发现并修复跨站 cookie 注入安全漏洞**——

#### R3394：cookie Domain 公共后缀未拒绝致跨站注入（安全漏洞，cookie.rs:438 add_from_url）

**问题**：RFC 6265bis §5.4 step 1（storage model）要求 cookie 的 Domain 属性若本身是公共后缀（com / co.uk / github.io 等），**拒绝存储**。旧 `add_from_url` 的 R3225 domain-match 校验**不拦此**——`Domain=com` 来自 evil.com 时，com 是 evil.com 的公共后缀祖先，evil.com domain-matches com（`domain_matches("com", "evil.com")` true），故通过 R3225 校验 → cookie 存储 → 发往任意 .com 站点（bank.com / paypal.com）= **跨站 cookie 注入/覆盖**（安全漏洞：evil.com 可污染/窃取用户在其它 .com 站点的会话 cookie）。

**根因修复**：`add_from_url` 显式 Domain 分支前置 PSL 公共后缀检查——`is_public_suffix(host)` 判定 + 拒绝（返 return）。

**关键技术点**：公共后缀判定法——朴素 `registrable_domain(h) == h` **错误**（实测 example.com/evil.com 等 eTLD+1 亦满足，曾致 R3225 测试 `Domain=example.com` 误拒）。正确法 = **子域探测**：`is_public_suffix(host) ⟺ registrable_domain("x."+host) == "x."+host`（追加一标签后注册域仍为 "x."+host ⟺ host 是公共后缀——仅公共后缀的 eTLD+1 会吸收新标签）。实测锚定（zero-psl）：com / co.uk / github.io / com.cn → true；example.com / evil.com → false。前导点归一（`.com` == `com`，RFC §4.1.2.7）；IP 字面量无 PSL 语义跳过。

**复现锁定**：新测 `test_add_from_url_rejects_public_suffix_domain_r3394`——com / .com / co.uk / github.io 四公共后缀 Domain 全拒（含 evil.com 设 Domain=com 的注入场景）+ Domain=example.com（eTLD+1）回归保护接受。

**评估不修的候选**（提及，防重复挖）：① **cookie RFC §5.4 排序**（longest-path-first + creation-time）——spec 合规但需加 creation_time 字段改 struct/parse/two header fns，CookieStore 未接线零实际影响，defer（下轮可挖）；② **request host 自身是公共后缀**（§5.5 step 8，如 co.uk 上设 host-only）——罕见，add_from_url host-only 分支已处理无 Domain 情况。

**其余 review 健壮**（不修）：R3225 domain-match 防 evil.com→bank.com 跨域 / R3391 __Host-/__Secure- 前缀校验 / SEC-08 SameSite=None 须 Secure / path 边界 §5.1.4 / SameSite Strict/Lax/None 默认 Lax。**PSL 接线后 is_same_site 现用真 eTLD+1**（R3381），a.github.io / b.github.io 正确判不同站（旧「取末两段」误判已修）。

**测试**：zero-net **429→430 全绿**（+1），零回归；fmt clean + clippy 零警告。**完成项**（commit `cff23072`）。**CookieStore 仍未接线生产**（仅测试引用），但 R3394 修复的是库 API 面正确性（与 R3391 同，未来接线即安全）。

**结论**：R3393「自主面趋零」结论**部分翻新**——翻新过时结论（PSL 状态）仍能挖出真 bug（R3394 跨站注入）。累计 R3388/R3389/R3391/R3392/R3394 共 **7 真 spec bug/安全漏洞已修**。下轮可续 cookie §5.4 排序 defer 或重扫 PSL 接线后 site-isolation 正确性。

### fix zero-net HTTP cache s-maxage 私有缓存忽略（本轮 R3392，net cache_policy.rs + http_cache.rs——RFC 9111 §4.2.1 合规修复，serves-stale-wrong-content）

承接 R3391（cookie 前缀校验）续 **zero-net 自主域** deep-review，派对抗审 `http_cache.rs`（1211 行）+ `cache_policy.rs`（231 行）对 RFC 9111（HTTP Caching）/ RFC 9110/7232（条件请求）。**发现并修复 1 处 spec 合规 bug**——

#### R3392：compute_ttl_secs 私有缓存误用 s-maxage（serves-stale-wrong-content，cache_policy.rs:70-72）

**问题**：`compute_ttl_secs` 无条件优先返回 `s_maxage`（旧 70-72 行 `if let Some(s_maxage) = cc.s_maxage { return Some(s_maxage); }`），但 RFC 9111 §4.2.1 明定 `s-maxage` **仅对共享缓存（CDN/代理）生效，私有缓存（浏览器）必须忽略它**并回退 max-age。旧实现 → `Cache-Control: s-maxage=3600, max-age=60` 在浏览器侧误按 3600s 视为新鲜（**过度缓存**，可能在服务器期望的 60s 后仍发过期内容 = serves-stale-wrong-content）。本仓 HttpCache 为浏览器**私有缓存**（磁盘层亦为私有磁盘缓存，`is_shared` 标志恒 false —— `is_shared: cc.public` 是语义误用：`public` 指令意为「响应可被共享缓存存储」非「本缓存是共享的」，但该字段 `#[allow(dead_code)]` 从不被读，无实际危害）。

**根因修复**：`compute_ttl_secs` 删 s_maxage 分支（私有缓存 s-maxage 永不适用）；s_maxage 仍由 `parse_cache_control` 解析保留（驱动 `is_shared` 标志置位）但不参与 TTL。一行移除即闭合。

**复现锁定**：改 `test_cache_s_maxage` → `test_cache_s_maxage_ignored_by_private_cache_r3392`——① 仅 `s-maxage=3600`（无 max-age/Expires/validator）→ 私有缓存不可存（`put` 返 false / `get` 返 None）；② `s-maxage=3600, max-age=60` → 用 max-age 命中（可存 + body 正确）。

**评估不修的候选**（提及，防重复挖）：
- **Authorization 头处理**——deep-review 引 RFC 9111 §3.2，但该规则**仅约束共享缓存**（含 Authorization 请求的响应不得存共享缓存）；私有缓存可存用户自己的授权响应。**ZeroWeb 缓存是私有 → 非 bug**，refute。
- **304 头字段列表不完整**（§4.3.4）——当前列表覆盖核心新鲜度/验证器字段（cache-control/content-location/date/expires/vary/etag/last-modified），content-encoding/type 罕随 304 变，接受近似（「不过度设计」）。
- **启发式新鲜度**（§4.2.2，10% of Date-Last-Modified）——RFC 用「may」，实现为可选，不实现合法；接受简化。

**其余 review 健壮**（不修）：R3231/R3232/R3233 age/Apparent age/Date 起点 + R3235 Expires 相对 Date 消时钟偏差 + R3371 `saturating_add` 防溢出 + max-age 优先于 Expires + Vary 大小写不敏感匹配 + ETag/Last-Modified 双发条件头 + LRU 字节预算驱逐 + no-store/no-cache 荣誉。

**测试**：zero-net **429 全绿**（改名测 + 数不变），零回归；fmt clean + clippy 零警告。**完成项**（commit `08f203df`）。**HttpCache 当前接线状态**：经 fetch 路径调用（非纯未接线库，区别于 CookieStore），故 s-maxage 修复有真实 fetch 缓存路径影响。

**结论**：net 自主域 cookie（R3391）+ cache（R3392）双域均有 spec 合规产出，证实 net 自主域仍活跃可挖。下轮可续审 disk_cache/client/websocket/navigation/request 其余路径。

### feat zero-net cookie __Host-/__Secure- 名称前缀校验（本轮 R3391，net cookie.rs——RFC 6265bis §5.4 step 12 合规修复，spec 安全缺口）

承接 R3390（P1a 勘误 + security/net 复核结论「security 已穷尽，net 可挖」），本轮转向 **zero-net 自主域**（zero-web 流域，无 html-compat 碰撞），派 deep-review 对抗审 `cookie.rs`（1633 行）对 RFC 6265 / RFC 6265bis。**发现并修复 1 处 spec 合规安全缺口**——

#### R3391：parse_set_cookie 不校验 cookie 名称前缀（spec 安全缺口，cookie.rs:344 parse_set_cookie）

**问题**：旧 `parse_set_cookie` 不校验 cookie 名称前缀。RFC 6265bis §5.4 step 12「Cookie Prefixes」要求真浏览器**拒绝存储**违规前缀 cookie（而非静默剥离前缀）：`__Secure-<name>` 须设 Secure；`__Host-<name>` 须设 Secure + 无 Domain（host-only）+ Path 恰为 "/"。旧实现完全不校验 → `__Host-session=x`（无 Secure / 带 Domain / Path=/app）或 `__Secure-t=x`（无 Secure）被当普通 cookie 存储，**绕过前缀的安全语义**（__Host-/__Secure- 前缀用于认证/CSRF token 等高安全 cookie，违规存储使攻击者可注入看似受前缀保护的 cookie）。

**根因修复**：parse 末尾（SameSite=None 校验后）按 §5.4 step 12 校验——名称 ASCII 大小写敏感判 `starts_with("__Secure-")` / `("__Host-")`（前缀字面大小写敏感，`__host-` 小写非前缀）；`__Secure-` 须 `secure`；`__Host-` 须 `secure` + `host_only`（用 host_only 标志判无 Domain，Domain 分支置 host_only=false 故 `!host_only` = 带 Domain）+ `path` 为 None 或 "/"。违规返 `NetError::InvalidCookie`。校验**纯属性派生**（不依赖 PSL/网络，可 parse 时闭合）；__Host- 的 https 要求由 Secure 属性发送规则承载（非 https 不发 Secure cookie）。

**复现锁定**：3 新测——`test_parse_cookie_secure_prefix_requires_secure_r3391`（`__Secure-sid=1` 无 Secure 拒 / 有 Secure 受）、`test_parse_cookie_host_prefix_rules_r3391`（`__Host-` 无 Secure / 带 Domain / Path≠/ 三拒 + Secure / +Secure+Path=/ 两受四规则矩阵）、`test_parse_cookie_non_prefixed_unaffected_r3391`（普通名 + `__host-` 小写非前缀回归保护）。

**评估不修的候选**（提及，防后续重复挖）：
- **public-suffix Domain 拒绝**（`Domain=com` 应拒）——真 spec 缺口（RFC 6265bis §5.4 step 1）但（a）依赖 PSL，本仓 PSL 受 html-compat rule 9 阻塞；（b）`CookieStore` grep 全仓**仅测试引用无生产接线**，当前不可利用。遵循「不修不可能场景」defer 至 PSL + cookie 接线时复核。
- **cookie RFC §5.4 排序**（longest-path-first + earliest-creation）——spec 合规，但需加 `creation_time` 字段改 struct/parse/two header fns，未接线库零实际影响，defer。
- **跨域 cookie 注入**（evil.com 设 Domain=bank.com）——**已由 R3225 `add_from_url` domain-match 校验闭合**（cookie.rs:407-419，来源 host 须 domain-match cookie Domain，否则拒）。

**其余 review 健壮**（不修）：R3225 domain-match 防 evil.com 跨域注入 / IP 字面量 host 不参与子域匹配 / SEC-08 SameSite=None 须 Secure / SEC-09 超 MAX_COOKIE_COUNT 驱逐 / path 边界匹配 RFC §5.1.4（`/app` 不匹配 `/application`）/ SameSite Strict/Lax/None 默认 Lax + 跨站子资源拦截 / Max-Age 优先于 Expires + 负值/0 立即过期 + saturating_add 防溢出。

**测试**：zero-net **426→429 全绿**（+3），零回归；fmt clean + clippy 零警告。**完成项**（commit `cda2aa94`）。**CookieStore 当前未接线生产**（仅测试引用），但 R3388/R3389 precedent 证明未接线库的 spec 合规修复仍接受（库 API 面正确性 + 未来接线即安全）。

**结论**：security 自主域已穷尽（sandbox/CSP/CORS/origin/HSTS/mixed_content 全审 + R3388/R3389 出 4 spec bug），net 自主域刚开始（R3391 cookie 1 spec 缺口）——security/net 自主域 spec 合规面持续有产出，区别于 dom_bindings/engine 5 连续健康轮。下轮可续审 zero-net 其余路径（http_cache/disk_cache/client/websocket/navigation）。

### fix zero-security CSP 三处 spec 合规修复（本轮 R3389，security csp.rs + csp_tests.rs——CSP3 合规修复，含 2 处真 CSP 绕过 + 1 处孪生大小写）

承接 R3388（iframe sandbox 大小写不敏感修复）。R3388 结论「security 连审都能挖出 spec 合规类静默 bug」，本轮续审 security 自主域 CSP（zero-web 流 100% 自主，零 html-compat 碰撞），派 deep-review 对抗审 csp.rs（827 行）+ csp_tests.rs（561 行）对 CSP Level 3 规范。**发现并修复 3 处 spec 合规 bug**——

#### R3389-①：CSP directive name 大小写敏感（CSP 绕过，csp.rs:98 parse + 108 find_directive）

**问题**：`parse()` 保留 directive name 原大小写（`tokens.next()?.to_string()`），`find_directive` 精确 `d.name == name` 比较。CSP3 §2.2.1「Parse a serialized CSP」step 4 显式要求 directive name 做 **ASCII 小写化**。故 `Content-Security-Policy: Script-Src 'none'`（mixed-case）→ directive name `"Script-Src"` 进 directives 列表，`is_resource_allowed` 查 `"script-src"` 时 find_directive 精确比较 miss → 回退 default-src（缺省）→ **放行所有脚本**。本该被 `'none'` 阻断的 `https://evil.com/evil.js` 被允许 = **CSP 绕过**（安全回归）。

**根因修复**：`parse()` 取 directive name 后 `to_ascii_lowercase()`。指令值（source list）保留原大小写——'self'/'unsafe-inline' 等 token 自带单引号区分，host 匹配由 `origin_expr_matches` 已做 `eq_ignore_ascii_case`。

#### R3389-②：CSP sandbox token 大小写敏感（孪生 R3388，csp.rs:64-82 SandboxFlag::from_str）

**问题**：与 R3388 iframe sandbox 同根——CSP3 §6.3.2 sandbox directive token 须 **ASCII 大小写不敏感**匹配（token 语义继承 HTML iframe sandbox）。旧 `SandboxFlag::from_str` 精确 `match s`，`sandbox_flags()`（csp.rs:456）从 `directive.values`（header 原样保留大小写）喂入 → `sandbox Allow-Scripts`（首字母大写）静默丢弃 token → 整个沙箱退化为最严格，丢失作者意图启用的脚本/表单等能力（安全语义偏离，过度限制）。

**根因修复**：14 个 keyword token 全改 `eq_ignore_ascii_case` 比较（allow-forms/popups/same-origin/scripts/top-navigation×2/popups-to-escape-sandbox/downloads/presentation/storage-access-by-user-activation/orientation-lock/pointer-lock/autoplay/modals），失败返 `None` 与原语义一致。

#### R3389-③：CSP 空源列表放行 + 'none' 独占性（CSP 绕过 + 过度阻断，csp.rs:136-144 check_source_list）

**问题 A（CSP 绕过）**：`check_source_list` 空列表返 `true`（放行全部）。CSP3 §6.7.2.7「Matches the source list」——**空源列表等价只含 `'none'` 的列表，须阻断全部**。故 `Content-Security-Policy: script-src`（fetch 指令存在但无值）应阻断所有脚本，旧实现却放行任意脚本 = **CSP 绕过**（`directive.values` 空 = 指令存在但无源，区别于「无该指令」由 `find_directive_or_default` 的 None 分支走 default-src/默认放行）。
**问题 B（过度阻断，非 spec 语义）**：旧 `if values.iter().any(|v| v == "'none'") { return false }` 一旦 'none' 在场即阻断全部。CSP3 §6.7.2.7 注：**'none' 与其它源共存时须被忽略**（`script-src 'none' 'self'` 等价 `script-src 'self'`），旧实现却因 'none' 在场误阻 self（安全方向但非 spec 语义）。

**根因修复**：① 空列表 `return false`（阻断全部）；② `let has_other_source = values.iter().any(|v| v != "'none'"); if !has_other_source { return false }`（仅 'none' 独占时阻断，与其它源共存时忽略 'none'）。

**复现锁定**：4 新测——`test_sandbox_token_case_insensitive_r3389`（全大写/首字母大写/小写回归）/ `test_directive_name_case_insensitive_r3389`（`Script-Src 'none'` 阻断 + `DEFAULT-SRC https://example.com` fallback + 解析后 name 小写化）/ `test_empty_source_list_blocks_all_r3389`（`script-src` 无值阻断全部）/ `test_none_ignored_when_other_source_present_r3389`（`'none' 'self'` 共存放行 self + 'none' 独占仍阻断回归保护）。

**其余 review 健壮**（不修）：R3342 已修的 `check_source_list`/`origin_expr_matches`/`wildcard_domain_matches` host 精确匹配防 CSP 绕过、`is_self_match` data:/blob: 排除 + 相对路径同源、`split_url_origin` scheme/host/port 解析、`is_inline_script_allowed`/`is_inline_style_allowed` nonce/hash/'unsafe-inline' 独立路径（不经 check_source_list，空值时返 false 正确）。

**测试**：zero-security **389→393 全绿**（+4），integration security_pipeline **52 全绿**（零回归，下游用小写指令名 + 非空源不受影响），fmt clean + clippy 零警告。**完成项**（commit `cd9e0c9e`）。

**结论**：security 自主域连审结论强化——R3388 + R3389 两刀共修 4 处 spec 合规 bug（其中 **2 处真 CSP/安全绕过**：directive name 大小写 + 空源列表），证明 **security 仍是活跃 bug 面**（区别于 dom_bindings/engine 连续 5 健康轮证明穷尽）。下轮可续审 security 其余 parsing/policy（CORS 1215 行 / 同源策略 / HSTS / mixed_content / cookie/permission）或转 storage/net。

### fix zero-security iframe sandbox token 大小写不敏感解析（本轮 R3388，security sandbox.rs——HTML spec 合规修复，含静默正确性偏离）

承接 R3387（storage IndexedDB cursor 溢出收尾）。R3387 后判断「security/sandbox 可续审或转 storage 其余路径」。本轮**主动审 zero-security 自主域**（zero-web 流 100% 自主，30d 零 html-compat 碰撞），审 `IframeSandboxFlag::parse_flag`（sandbox.rs:47）。**发现并修复混合大小写 sandbox token 静默丢弃 bug**——

#### R3388：IframeSandboxFlag::parse_flag 精确 match 致 mixed-case token 静默丢弃（静默正确性，sandbox.rs:47）

**问题**：旧实现 `match s { "allow-scripts" => ... }` 精确字符串比较。HTML 规范要求 iframe `sandbox` 属性 token 按 **ASCII 大小写不敏感**匹配（HTML §4.7.4 / `https://html.spec.whatwg.org/#attr-iframe-sandbox`，attribute reflecting 的 keyword matching）。**关键**：HTML 解析器仅小写化**标签名与属性名**，**属性值为 CDATA 原样保留大小写**——已核本仓 DOM `get_attribute`（crates/dom/src/node.rs:153-159）`.map(|a| a.value.to_string())` 原样返回，`attr_name_effective`（node.rs:141-150）仅对属性名 lowercasing，全仓无 `.to_lowercase()`/`.to_ascii_lowercase()` 施于属性值。故页面写 `sandbox="Allow-Same-Origin"`（首字母大写，HTML 常见书写）会以原大小写真实到达 `parse()`，旧 `match` 静默丢弃 → token 不进 flags → 整个 sandbox 退化为最严格，**丢失作者意图启用的脚本/同源/表单/导航等能力**（安全语义偏离：本该放行的能力被误阻）。

**根因修复**：13 个 keyword token 全改 `eq_ignore_ascii_case` 比较（allow-forms/popups/same-origin/scripts/top-navigation/top-navigation-by-user-activation/popups-to-escape-sandbox/downloads/presentation/orientation-lock/pointer-lock/autoplay/modals），失败返 `None` 与原语义一致（未知 token 静默丢弃是 spec 行为，保留）。

**复现锁定**：新测 `test_iframe_sandbox_flag_parsing_case_insensitive_r3388`——全大写（`ALLOW-SCRIPTS ALLOW-FORMS`）/ 首字母大写（`Allow-Same-Origin Allow-Top-Navigation`）/ 混合大小写长名（`allow-Top-Navigation-By-User-Activation`）/ 小写回归保护（`allow-scripts`）四矩阵全覆盖。

**其余 review 健壮**（不修）：`parse()` split_whitespace + filter_map 容错（lib_tests_extra.rs 已覆盖前后空白/连续分隔符/空串）、`effective_origin` allow-same-origin 分支正确、`check_sandbox_navigation` top-nav + user-activation 双分支正确。

**测试**：zero-security **388→389 全绿**（+1），零回归；fmt clean + clippy 零警告。**完成项**（commit `94bfa82a`）。

**结论**：security 是「连审都能挖出 spec 合规类静默 bug」的活跃域——区别于 dom_bindings/engine 连续 5 健康轮证明穷尽，security 第 1 刀即出真 bug（与 storage 早期轮次模式一致）。下轮可续审 security 其余 parsing/policy（CSP header / origin 比较 / cookie）或 storage 其余路径。

### deep-review dom_bindings P1b 最新代码 全确认健壮（本轮 R3378，engine dom_bindings gc.rs/node.rs/custom_elements.rs——健康确认轮，无代码变更）

承接 R3377（dom 导航/数据模型健康确认轮）。R3377 下游判断给出「下轮待用户战略方向点名，或转 dom crate 未审子域」。本轮**纠偏 R3377 倾向**——R3377 审的是 dom 老 crate 稳定子域（range/traversal/mutation），边际收益确实趋零；但 **P1b native V8 绑定 `dom_bindings`（engine）是最新最复杂代码**（S0→S5 已 land R3264-R3272 custom-element lifecycle + R3334 多 WebView thread-local 清理），**R3338 旧审早于 S5 大扩展**，是最该深审的「年轻复杂代码」。本轮续推 dom_bindings 三高风险文件（gc.rs 608 + node.rs mutation + custom_elements.rs 272），**全确认健壮，0 真 bug，无代码变更**（健康确认轮，镜像 R3348/R3360/R3376/R3377 模式）。dom_bindings 为 zero-web 流 100% 自主域（30d 仅本流 S5/R3334 提交，`542bf35f` Windows 测试基线恢复非代码逻辑改动，**零 html-compat 碰撞**）。

#### 逐项核验（3 高风险文件，候选证伪全部成立）

- **gc.rs（608 行，P1b GC + 状态层）**：① R3133 weak + guaranteed-finalizer 设计成熟——`cache_native_element` 注册 `Weak::with_guaranteed_finalizer`（包装器 GC 时清本节点 LISTENERS + unmark CONNECTED_CUSTOM，防 slotmap NodeId 复用致新元素误判已连）、弱句柄不阻 GC（JS 丢包装器即可回收，闭合 R3109 泄漏）。② `with_dom`/`with_dom_mut` 单 borrow 无重入——闭包内纯 Rust 读/mutation（record_mutation 推 pending_mutations，无 observer 回调），V8 回调顶层单次持 borrow。③ `encode/decode_node_id` slotmap `KeyData::as_ffi`/`from_ffi` 双向编解码正确。④ `remove_listener` `before - vec.len()` 无 underflow（retain 只删元素，len ≤ before）。⑤ `listener_present`/`remove_listener` 持 LISTENERS borrow 调 `v8::Local::new + strict_equals`——**strict_equals 非 JS 回调无重入**（docstring 明确），安全。⑥ `add_attr_name` arena append-only（仅 reset 清），`idx = v.len() as u32` 截断需 ~4.2e9 distinct attr 名——**已知小泄漏**（docstring 标注「按 distinct attr 数 bounded，后续可接 free-list/generation 回收」），非 bug。
- **node.rs mutation 路径（appendChild/insertBefore/removeChild/replaceChild/element.remove/insert_variadic）**：① stale-safe getter（节点移除 read_node_id 仍存但 with_dom 查不到 → 返 undefined，spec detached 行为）。② `notify_connect_after_insert`/`notify_disconnect_after_remove` 在 `with_dom_mut` 闭包**返回后**调（闭包内仅 Rust mutation，notify 时不持 DOM borrow → 桥接 JS 派发可安全 re-enter）。③ DocumentFragment flatten（R3132 appendChild/insertBefore + R3144 variadic insert）经 `insert_with_fragment_flatten` 统一处理（fragment 子节点快照逐个插、fragment 清空，spec 语义）。④ `native_replace_child_invoke` 成功后 `notify_connect(parent,new) + notify_disconnect(old)` 双派发（new 连入 + old 断开，spec）。
- **custom_elements.rs（272 行，CE lifecycle 桥接）**：① **连接态经 state-transition 正确处理 reparenting**——`notify_connect_after_insert` 的 `was = is_custom_connected(ffi)` 反映 `CONNECTED_CUSTOM` **当前态**（Rust `Document::replace_child` 已移节点但 CONNECTED_CUSTOM 由 notify 函数更新 → was = 移前态），`parent_connected && !was`（连）/`!parent_connected && was`（断）真转派发，**覆盖 连→连/连→断/断→连 全矩阵**，replaceChild 跨树 reparenting 不漏 disconnect。② `collect_custom_subtree` R3272 子节点**逆序压栈**（last child 先 push → pop 正序 first child 先出）保 pre-order tree order（connectedCallback 触发序与浏览器一致，框架依赖正确）。③ `is_connected_to_document` **显式比 `d.root()`**（区分 detached 子树根 parent=None vs html root parent=None，仅 parent=None 无法区分）。④ R3271 fast-path（无连字符 tag 必非 custom element——CE 名规范要求含 `-`，跳过无谓 JS 桥接 + native 实例构造，框架 reconciliation 高频 setAttribute 普通元素受益）。⑤ attributeChangedCallback `read_attr_change_context` 在 mutation **前**读 oldVal（防读新值）+ `notify_attribute_change` 桥接派发。

#### 候选排除（证伪，防后续轮重复挖）

本轮 3 项高风险候选经核验**全部证伪**（非真 bug）：① **replaceChild reparenting 漏 disconnect**（最初疑 newChild 跨树移动时旧父树子树不触发 disconnectedCallback）——**经核验非 bug**：state-transition 经 `was=is_custom_connected(ffi)` 标志正确覆盖（newChild 旧父的 CE 子树连态由 was 判定，跨树移动若连→连则不派发正确、连→断则派发正确）；② **drop_cached_native_element 提前移 Weak 致终结器不触发 listener 泄漏**——**已知权衡**（docstring 标注「非回归，旧 strong 缓存同样残留；reset 兜底清」），终结器生命周期绑 Weak 是 R3133 设计选择；③ **listener_present/remove_listener 持 borrow 调 V8 strict_equals**——**安全**（strict_equals 是纯值比较非 JS 回调，无 LISTENERS 重入）。**教训**：P1b native V8 绑定的「弱句柄 + 终结器 + 状态转换」三件套是 V8 嵌入的标准稳健模式，候选须理解 V8 GC/终结器语义再判（避免「持 borrow 调 V8」误报为重入——strict_equals/new/Local::new 均非 JS 回调）。

**为何零代码变更且净正向**：健康确认轮无行为变更，价值在——① **关键结论**：连最新最复杂的 P1b native V8 绑定代码（GC/终结器/custom-element lifecycle/reparenting 状态机）都经审确认健壮，证实 zero-web 流代码库**全主面成熟**（非仅老 crate 稳定，新 P1b 代码亦然）；② 排除 3 项高风险候选证伪经验（reparenting state-transition / Weak 终结器权衡 / strict_equals 非重入）防后续轮重复挖。**验证**：zero-engine **2056 passed / 0 failed**（test-guard 包裹，含 dom_bindings 全测 tests_dom_api.rs 2794 行 + tests.rs 1812 行 + tests_collections/tests_events/tests_html_setters），零回归（无代码变更）。dom_bindings 为 zero-web 流自主域（零 html-compat 碰撞）。

**下游判断**：R3378 = deep-review **第二十九刀**（dom_bindings P1b 最新代码），**连续第 5 健康轮**（R3360/R3372/R3376/R3377/R3378）—— **关键结论：zero-web 流全 crate 全主面全子域经 29 轮 deep-review 彻底穷尽**（net/storage/security/canvas/dom/webview/wasm/script-sandbox/protocol/host-runtime/browser-shell/page-runtime + **engine P1b dom_bindings 最新代码**），**真 bug 线 R3334-R3375（20 项）后连续 5 轮零真 bug**，自主可推进 bug 面经 5 连续健康轮证明确归零（连最该有 bug 的最新复杂代码都健壮）。**剩余战略方向均需用户决策（rule 11 架构级不可自主）**：① PSL `extract_registrable_domain` 集成（site_isolation 同站误判，潜伏未接线）；② 字体栈 P0（rendering-compat 流域，RFC v0.2.3 已批准 2026-08-09）；③ P1b S6/S7 续推（S5 custom-element lifecycle 已 land，S6/S7 = attributeChangedCallback 全化 + 完整 parity + default-on，**default-on 时机需用户确认**——native_dom 当前 kill-switch 默认 OFF，reftest 兼容，default-on 是破坏性行为变更须用户拍板）。**本流不应再无谓复扫已证健康代码**（5 连续健康轮 = 单 session lever 确实耗尽，再扫只会产边际收益趋零的健康轮）—— 下轮**待用户战略方向点名**（PSL/字体栈/P1b default-on）或**用户裁决降低 rally 频率 / 切换工作面**。R3376 已飞书告知卡点，R3378 无新信息触发重发。

---

> **🗂 归档指针（2026-08-13 首轮瘦身；2026-08-14 二次迁移 R3377–R3353 批次）**
>
> 更早的推进记录（R2647–R3378，共 625 条 R-轮次记录 + 7 条「本轮调查」叙事节）已归档至：
> **[`archive/r3xxx-history.md`](archive/r3xxx-history.md)**
>
> 本控制面「最近完成的改进」仅保留最近 10 条（R3378–R3398）。新增记录追加在本节顶部；
> 当记录超过 10 条时，由每日 `docs-maintenance` cron 把更早的迁移到上述 archive 文件（见 `docs/rally/jobs.yaml`）。

### -9. Shadow DOM slot 解析 + canvas image_smoothing/stroke join-cap + 58 个新测试（前轮，3286 测试）

实现 Shadow DOM slot 分配解析、canvas image_smoothing_enabled 和 stroke join/cap 渲染集成，
同时在 8 个 crate 添加 58 个边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| dom | **Shadow DOM slot 分配**：resolve_slots() 和 get_assigned_nodes()，支持命名 slot、默认 slot、fallback 内容、多元素分配；**get_elements_by_tag_name_ns()** 命名空间感知查找，通配符支持 | 10 |
| canvas | **image_smoothing_enabled** 属性 + save/restore；**line_join/line_cap stroke 渲染集成**：Miter/Round/Bevel 接合、Butt/Round/Square 端点影响 stroke 顶点生成 | 9 |
| layout-engine | **tree building 边界**：comment 节点跳过、PI 跳过、20 层深层嵌套、混合 display:none 子元素、grid 容器+项 | 5 |
| engine | **pipeline/paint 边界**：inline style 渲染、script tag 不崩溃、渲染顺序稳定、重叠背景 z-order、部分边框绘制 | 5 |
| css-parser | **parser 边界**：hwb color、gradient 多类型 stop、3D transform 函数、3 层嵌套 var fallback、复杂 @supports 条件 | 5 |
| render-foundation | **shaper/geometry/primitive 边界**：空字符串整形、单字符、clear+remark、opacity 零图元 | 4 |

### -8. cursor/opacity 管线集成 + 7 crate 测试覆盖：45 个新测试（前轮，3228 测试）

完成 style-system cursor/opacity 属性管线集成，同时在 7 个 crate 添加 45 个边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| style-system | **cursor/opacity 管线集成**：parse_cursor/parse_opacity 接入 apply_property_value；cursor 继承、opacity 不继承；**transition/animation none 验证、box-sizing 效果、多重 transform** | 10 |
| canvas | **composite operation 像素级验证**：source-over、destination-over、copy、xor、source-atop 5 种合成模式实际像素混合行为测试 | 5 |
| protocol | **IPC 压力测试**：1MB+ 大消息、消息排序、空字段、unicode 载荷、确定性编码、并发消息、无导出查询 | 7 |
| host-runtime | **事件边界**：修饰键组合、按键重复标志、所有鼠标按钮、enter/leave 坐标、零尺寸 resize、IME 空组合 | 6 |
| net | **网络边界**：第三方 cookie（SameSite=None+Secure）、会话 cookie、前进超出可用、新导航清空前进历史、零超时 | 5 |
| security | **CSP/CORS/mixed-content**：img-src 限制、nonce 不匹配、default-src 回退、HTTPS 无需升级、worker 类型、max-age 零、methods 通配符 | 7 |
| wasm-sandbox | **WASM 边界**：memory grow+read、错误签名调用、无导出模块、fuel 消耗追踪、多参数 host、递归限制 | 6 |

### -7. 功能完善 + 测试覆盖：canvas line_join/line_cap、grid-area 简写、DOM import_node、49 个新测试（前轮，3183 测试）

并行实现 6 个功能改进和 16 个边界条件测试，覆盖 10 个 crate：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| canvas | **LineJoin/LineCap 属性**：Miter/Round/Bevel、Butt/Round/Square，getter/setter + save/restore；**is_point_in_stroke()**：距离线段检测 | 7 |
| css-parser | **parse_grid_area()**：命名区域、斜杠分隔行号、auto 展开 | 5 |
| style-system | **grid-area/grid-column/grid-row 简写展开**：扩展到 grid_row_start/end、grid_column_start/end | 4 |
| dom | **import_node()**：浅层和深层克隆，导入节点无父节点 | 4 |
| layout-engine | **overflow Auto→Scroll/Clip→Clip 验证**、**z_index 输出验证**、**content area 超大 border 钳位**、**深层嵌套 fixed 位置** | 5 |
| engine | **overflow:hidden 部分交叉裁剪**、**hsla 零饱和度/亮度**、**paint_text 零宽度**、**嵌套 overflow:hidden** | 4 |
| render-foundation | **多次 resize**、**RGBA clamp 极端值**、**image_cache 零 max_entries** | 3 |
| webview | **多次导航**、**加载后注入 CSS**、**execute_script 占位**、**自定义视口 Builder** | 4 |

### -6. 全 crate 测试覆盖率提升：119 个新测试 + visibility:collapse 修复（前轮，3094 测试）

系统性分析 14 个 crate 的测试缺口，聚焦最低密度的 layout-engine 和 engine crate，
同时覆盖 dom、css-parser、security、storage、net、canvas 共 10 个 crate。
发现并修复 `visibility:collapse` 在 paint 中未按 CSS 规范隐藏元素的 bug。

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| layout-engine | **converter 全变体覆盖**：InlineFlex/InlineGrid display、Em/Rem/Vw/Vh/Vmin/Vmax/Ch 单位、Calc fallback、max-length infinity、WrapReverse、FlexBasis Content、align_content 9 变体、justify_content 6 变体、align_self 6 变体、float/clear 全变体、overflow Auto/Clip、tokenize 嵌套括号、malformed minmax | 24 |
| layout-engine | **inline 格式化边界**：vertical_align Sub/Super y 偏移、TextTop/TextBottom 与 Top/Bottom 等价、resolve_font_metrics Em/Rem 回退、非 Px font-size 回退、break_into_lines 重置状态、零 font_size/line_height | 9 |
| engine | **clip_fills/clip_glyphs 直接测试**：partial overlap、outside each side、start index、empty slice、exact match；**hsla hue≥300 fallback**；**CurrentColor**；**length_to_f32 non-Px**；**named_color 扩展** | 15 |
| engine | **composite 父子层路由**：promoted parent + non-promoted child、promoted parent + promoted child、single box bounding_box、grandchildren encompass；**dirty 150% merge boundary**、50 rect stress、negative size、100 non-overlapping、full_redraw merge；**pipeline malformed CSS**、50% threshold、recompute without render、mixed render ops | 14 |
| engine | **visibility 修复 + 测试**：visibility:collapse 按 CSS 规范隐藏元素（修复 bug）；child visible 覆盖 parent hidden；doc=Some+node_id=None fallback；paint_in_rect hidden skip | 4 |
| dom | **Range API**：select_node、select_node invalid (Document)、text_content partial、to_debug_string、set_start/set_end invalid offset、clone_contents partial text、insert_node mid text | 7 |
| css-parser | **5 个零覆盖 parse 函数**：parse_vertical_align (8 关键字+长度)、parse_list_style_type (15 关键字)、parse_list_style_position、parse_float、parse_clear；**eval_calc_with_context viewport** | 6 |
| security | **CORS 安全边界**：credentials+wildcard rejected、is_simple_request (GET/POST/PUT/自定义 header/content-type)、generate_preflight_response 字段验证；**sandbox 导航/弹窗**：effective_origin opaque/preserve、导航有/无 activation、弹窗允许/阻止 | 13 |
| storage | **IndexedDB cursor + transaction**：cursor advance/continue_to/iteration、key_cursor advance/continue、transaction commit/abort、put 覆盖/add 拒绝重复、count_with_range、index range query、cursor on index | 12 |
| net | **URL 边界**：非默认端口 origin、credentials in URL、fragment+query；**导航边界**：empty replace_current、max_entries boundary；**请求**：non-UTF8 body、header chaining | 7 |
| canvas | **clip+drawImage**：clip constrains draw_image、negative coordinates、zero dimensions、ImageData zero dims、out-of-bounds get_image_data、TextAlign/TextBaseline enum | 6 |

### -5. 6 crate 边界条件测试覆盖率提升（前轮，2977 测试）

通过并行扫描 test-to-code 比率最低的 6 个 crate，识别测试缺口并批量添加边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| style-system | **级联优先级边界**：specificity 竞争、!important 覆盖、inherit/initial/unset/revert 关键字、非继承属性、ComputedStyle 默认值 | 74 |
| dom | **节点操作边界**：insert_before 错误路径、replace_child、clone_node_deep、has_child_nodes、Document 工厂方法、属性覆盖/删除、get_elements_by_tag_name 通配符 | 57 |
| render-foundation | **geometry 负坐标**：负坐标 Rect contains/intersection、负值 Size area、DamageTracker NaN/重复 rect；**surface 1x1/零尺寸**；**image_cache gc 后 insert、max_entries=1**；**primitive clips-only bounding_box** | 27 |
| layout-engine | **types 负 margin/z_index**：outer_area 负值、深层嵌套 position、sticky flag；**tree display:none 全跳过**；**engine absolute-in-fixed、flex 窄容器 wrap**；**converter grid-auto-flow dense、percentage 尺寸** | 22 |
| engine | **paint HSL 极端值**：色相 120/240、饱和度/亮度 0/100；**border-style hidden**：不产生填充；**dirty 负坐标/链式合并**；**composite opacity=0、z_index 极值** | 21 |
| wasm-sandbox | **fuel 边界**：set_fuel(0) 即耗尽、fresh instance get_fuel；**memory round-trip**；**extreme i32**：MIN+MAX 溢出语义；**config/error display** | 7 |

### -4. 多 crate 测试覆盖率提升（前轮，2769 测试）

| 模块 | 实现内容 | 新增测试 |
|------|----------|----------|
| security | **mixed content data/blob/javascript URI**、**CORS wildcard + headers**、**same-origin 显式默认端口** | 13 |
| security | **CSP scheme-source 匹配**、**frame-src 限制**、**report-only 模式** | 3 |
| canvas | **变换组合非交换性**、**set_transform 替换验证**、**putImageData 边界** | 8 |
| canvas | **gradient 多 stop 排序**、**重复 offset**、**越界 offset** | 3 |
| dom | **shadow root closed 模式**、**compare_document_position 深度分支** | 4 |
| net | **URL fragment+query**、**IPv4 host**、**相对路径解析**、**组合边界** | 7 |
| storage | **Cache API CRUD**、**覆盖/keys**、**localStorage clear**、**session 隔离** | 5 |

### -3. z-index/is_sticky + 字体整形器 + 行内格式化增强（前轮，2661 测试）

| 模块 | 实现内容 | 新增测试 |
|------|----------|----------|
| layout-engine | **z_index/is_sticky 字段**：LayoutBox 新增 z_index 和 is_sticky 字段，engine 提取 z_index | 2 |
| layout-engine | **fixed 视口坐标调整**：adjust_fixed_to_viewport 递归修正 fixed 元素坐标 | 0 |
| layout-engine | **text-align center/right/justify**：InlineFormattingContext 支持居中、右对齐、两端对齐 | 30+ |
| layout-engine | **vertical_align**：TextRun 新增 vertical_align 字段，DOM 布局集成 | 多处 |
| render-foundation | **文本整形器（TextShaper）**：单行/多行整形，空格处换行，显式换行符支持 | 15 |
| css-parser | **float/clear 属性**：FloatValue/ClearValue 枚举，不区分大小写解析 | 4 |
| style-system | **float/clear 管线集成**：ComputedStyle、apply_property_value、PropertyRegistry | 4 |
| engine | **inline paint 增强**：paint 集成 InlineFormattingContext | 8 |

### -2. CSS 数学函数 + aspect-ratio + DOM 模块测试（前轮，2591 测试）

| 模块 | 实现内容 | 新增测试 |
|------|----------|----------|
| css-parser | **CSS min()/max()/clamp() 数学函数**：解析、求值、嵌套支持，LengthValue::Calc 变体 | 14 |
| style-system | **calc/min/max/clamp 管线集成**：parse_length_or_math 辅助函数，所有长度属性自动支持数学函数 | 6 |
| style-system | **aspect-ratio 属性**：ComputedStyle.aspect_ratio 字段，支持 auto/数字/w:h 斜杠语法 | 4 |
| layout-engine | **aspect-ratio 布局**：converter 传递 aspect_ratio 到 taffy | 2 |
| layout-engine | **box-sizing:border-box 布局验证**：确认 border-box/content-box 布局正确性 | 2 |
| style-system | **@supports selector() 条件验证**：连续组合器检测，无效选择器拒绝 | 4 |
| dom (serializer) | **ProcessingInstruction/Doctype PUBLIC+SYSTEM/Fragment/void 元素/转义 测试** | 8 |
| dom (document) | **PI 内容、set_text_content on Comment/Fragment、多 class 查找、quirks_mode 等** | 10 |
| dom (event) | **事件重用、Debug 格式、捕获阶段 prevent_default、深层嵌套传播** | 5 |

### -1. 关键功能修复 + 测试覆盖率提升（前轮，2359 测试）

通过工作流分析 14 个 crate 的高优先级功能缺口，并行修复并补充测试：

| 模块 | 修复内容 | 新增测试 |
|------|----------|----------|
| style-system | **var() 解析集成到样式计算管线**：级联值中的 var() 引用现在在继承/计算前被解析，包括嵌套自定义属性引用 | 4 |
| style-system | **@container 真实评估**：基于 ContainerContext 评估 min-width/max-width 等条件，无上下文时不应用 | 3 |
| canvas | **roundRect 圆角扁平化**：路径扁平化正确生成圆角弧线顶点（8段/角），而非退化为直角矩形 | 9 |
| storage | **IDB 事务缓冲/回滚**：事务操作缓冲到内存，commit 时应用，abort 时丢弃；tx_get 优先读缓冲区 | 10 |
| webview | **CSS 缓存持久化**：render() 不再丢弃 CSS，cached_css 字段在 load_html/inject_css 间保留 | 4 |
| dom | **节点生命周期测试**：移除后节点操作、重新挂载、错误路径 | 11 |

### 0. 全 crate 功能增强 + 测试覆盖率提升（前轮）

通过并行扫描 14 个 crate 识别出 158 个功能缺口和 133 个测试覆盖缺口，按优先级实现了 373 个新测试和对应功能：

| 模块 | 新增功能 | 新增测试 |
|------|----------|----------|
| engine | text/glyph 渲染、overflow clip、border-radius、z-index compositing、CSS transform、增量渲染 | ~18 |
| render-foundation | image cache + GC、GPU pixel readback、clipping/scissor | ~8 |
| security | CORS preflight、CSP nonce/hash/navigation/document、mixed content blocking、sandbox | ~20 |
| storage | IdbKeyRange、IdbIndex、IdbCursor、IdbTransaction、Cache API | ~25 |
| canvas | HSL/HSLA 颜色、gradient 解析 | ~20 |
| host-runtime | mouse/cursor/IME 事件、综合事件处理 | ~15 |
| net | HTTP send 集成、cookie 过期/SameSite enforcement | ~10 |
| wasm-sandbox | host function imports、fuel limiting | ~10 |
| webview | event callbacks、load_url fetch、execute_script | ~5 |
| css-parser | gradient 解析、bare 0 parsing、calc 改进 | ~40 |
| style-system | 所有属性初始值、grid e2e 测试、structural pseudo-class | ~40 |
| dom | 多 class selector 查询 | ~5 |

### 1. CSS 简写属性展开（style-system）

实现了完整的 CSS 简写属性展开模块（`shorthand.rs`），在级联之前将简写属性自动展开为长属性：

| 简写属性 | 展开为 |
|----------|--------|
| `margin` | `margin-top/right/bottom/left` |
| `padding` | `padding-top/right/bottom/left` |
| `border-width/style/color` | 4 边对应属性 |
| `border` | 12 个长属性（4 边 × width/style/color） |
| `border-top/right/bottom/left` | 3 个长属性 |
| `overflow` | `overflow-x` + `overflow-y` |
| `border-radius` | 4 个角半径 |
| `flex` | `flex-grow` + `flex-shrink` + `flex-basis` |
| `inset` | `top` + `right` + `bottom` + `left` |
| `transition` | `transition-property/duration/timing-function/delay` |
| `animation` | 8 个长属性（name/duration/timing/delay/iteration-count/direction/fill-mode/play-state） |
| `margin-block/inline` | `margin-block-start/end` 或 `margin-inline-start/end` |
| `padding-block/inline` | `padding-block-start/end` 或 `padding-inline-start/end` |
| `inset-block/inline` | `inset-block-start/end` 或 `inset-inline-start/end` |

### 2. CSS Transitions（css-parser → style-system）

- `TimingFunctionValue` 枚举：ease、linear、ease-in/out、cubic-bezier、steps
- `parse_time()` 支持 s/ms 时间值
- `ComputedStyle` 新增：`transition_property`、`transition_duration`、`transition_timing_function`、`transition_delay`
- `transition` 简写展开，正确处理 cubic-bezier() 和 steps() 内部逗号
- 21 个新测试

### 3. CSS Animations + @keyframes（css-parser → style-system）

- `KeyframesRule`、`KeyframeBlock`、`KeyframeSelector` AST 类型
- `@keyframes` 专用解析器：from/to/百分比选择器、逗号分隔、声明块
- `AnimationDirectionValue`、`AnimationFillModeValue`、`AnimationPlayStateValue` 枚举
- `ComputedStyle` 新增 8 个动画字段
- `animation` 简写展开（8 个子属性）
- matcher 中正确跳过 `Rule::Keyframes`
- 23 个新测试

### 4. CSS 逻辑属性（style-system）

- 12 个逻辑长属性：`margin-block-start/end`、`margin-inline-start/end`、`padding-block-start/end`、`padding-inline-start/end`、`inset-block-start/end`、`inset-inline-start/end`
- 水平书写模式映射：block→top/bottom，inline→left/right
- 6 个轴简写：`margin-block`、`margin-inline`、`padding-block`、`padding-inline`、`inset-block`、`inset-inline`
- 21 个新测试

### 5. CSS @media 媒体查询（css-parser → style-system）

- 媒体类型：`screen`、`print`、`all`
- 媒体特性：`width/min-width/max-width`、`height/min-height/max-height`、`orientation`
- `not` 取反和多条件 `and` 组合
- 无视口信息时 `@media` 规则不应用

### 6. CSS Transform 属性（css-parser → style-system）

- 变换函数：translate/translateX/translateY、rotate、scale/scaleX/scaleY、skew
- 多函数链式组合
- `ComputedStyle` 新增 `transform` 字段

---

## Tier 1 CSS 覆盖状态

| Tier 1 类别 | 状态 |
|-------------|------|
| 选择器全量 | ✅ ~95% |
| 盒模型 | ✅ 100%（含 **box-sizing: border-box** 布局测试） |
| Block/Inline/Flexbox 布局 | ✅ 已实现（行内格式化上下文已实现，**word-break: break-all/keep-all 渲染集成**） |
| Grid 布局 | ⚠️ ~65%（display + auto-flow + 项放置 + grid-area + repeat() + auto-rows/cols；缺 auto-fill 真实支持、命名区域） |
| 颜色 | ✅ ~98%（含 **148 种标准命名颜色**、hwb/hsl/rgb/rgba/hex 全格式） |
| 字体 | ✅ 100% |
| 定位 | ✅ 100% |
| Overflow | ✅ 100% |
| Transforms | ✅ ~90%（2D + 3D 函数、**transform-origin 渲染集成**、perspective、transform-style、backface-visibility、**TransformPrimitive 仿射矩阵**） |
| **Transitions** | ✅ 已实现 |
| 自定义属性 | ✅ ~90% |
| 媒体查询 | ✅ ~85%（only 关键字、逗号 OR、prefers-color-scheme、prefers-reduced-motion、pointer、resolution） |
| **逻辑属性** | ✅ 已实现 |
| **Animations/@keyframes** | ✅ 已实现 |
| **@supports** | ✅ 已实现（含 **selector() 条件验证**） |
| **@layer** | ✅ 已实现 |
| **@import** | ✅ 已实现 |
| **@container** | ✅ 已实现（解析 + 骨架评估） |
| **scroll-snap** | ✅ 已实现（scroll-snap-type/align/stop + scroll-margin/scroll-padding） |
| **CSS 数学函数** | ✅ 已实现（calc()/min()/max()/clamp() 解析、求值、样式管线集成） |
| **aspect-ratio** | ✅ 已实现（属性解析 + 布局引擎集成） |
| **float/clear** | ✅ 已实现（属性解析 + 样式管线集成） |
| **direction/unicode-bidi** | ✅ 已实现（属性解析 + 样式管线集成） |
| **tab-size** | ✅ 已实现（属性解析 + 样式管线集成） |
| **overflow-wrap** | ✅ 已实现（属性解析 + 样式管线集成） |
| **text-align-last** | ✅ 已实现（属性解析 + 样式管线集成） |
| **font-variant-numeric** | ✅ 已实现（属性解析 + 样式管线集成） |
| **text-shadow** | ✅ 已实现（属性解析 + 样式管线集成 + **渲染集成**，继承属性） |
| **box-shadow** | ✅ 已实现（属性解析 + 样式管线集成 + **渲染集成** ShadowPrimitive，含 inset） |
| **outline 简写** | ✅ 已实现（outline-width/style/color 简写展开） |
| **list-style-image** | ✅ 已实现（none/url，继承属性） |
| **column-gap** | ✅ 已实现（长度值，非继承） |
| **justify-items / justify-self** | ✅ 已实现（auto/normal/start/end/center/stretch/baseline，非继承） |
| **align-content** | ✅ 已实现（含 space-between/space-around/space-evenly，非继承） |
| **empty-cells** | ✅ 已实现（show/hide，继承属性） |
| **border-spacing** | ✅ 已实现（1-2 length 值，继承属性） |
| **gap 简写** | ✅ 已实现（gap → row-gap + column-gap） |
| **text-overflow** | ✅ 已实现（属性解析 + 样式管线 + **ellipsis 渲染集成**） |
| **filter** | ✅ 已实现（属性解析 + 样式管线 + **渲染集成 FilterPrimitive**，10 种滤镜函数） |
| **background-position** | ✅ 已实现（属性解析 + 样式管线 + **渲染集成**：关键字/长度/百分比/双值组合） |
| **background-size** | ✅ 已实现（属性解析 + 样式管线 + **渲染集成**：auto/cover/contain/长度/百分比） |
| **background-clip** | ✅ 已实现（属性解析 + 样式管线 + **渲染集成**：border-box/padding-box/content-box/text） |
| **background-origin** | ✅ 已实现（属性解析 + 样式管线 + **渲染集成**：border-box/padding-box/content-box） |
| **border-image** | ✅ 已实现（属性解析 + 样式管线 + **渲染集成**：9-region slicing，4角+4边+中心） |
| **mix-blend-mode** | ✅ 已实现（属性解析 + 样式管线 + **渲染集成**：16 种混合模式 BlendModePrimitive） |
| **background-repeat** | ✅ 已实现（属性解析 + 样式管线 + **渲染集成**：6 种平铺模式 repeat/repeat-x/repeat-y/no-repeat/space/round + tile 裁剪到 origin 区域） |
| **resize** | ✅ 已实现（属性解析 + 样式管线 + **渲染集成**：手柄指示器 stroke 图元） |
| **content** | ✅ 已实现（CSS `content` 属性 **渲染集成**：String + Counter 格式化 decimal/lower-alpha/upper-alpha/lower-roman/upper-roman） |
| **object-fit** | ✅ 已实现（CSS `object-fit` **渲染集成**：fill/contain/cover/none/scale-down 5 种图片适配模式） |
| **writing-mode** | ✅ 已实现（属性解析 + 样式管线 + **渲染集成**：vertical-rl/vertical-lr 字形旋转 90°，GlyphPrimitive.rotation 字段） |
| **word-break** | ✅ 已实现（属性解析 + 样式管线 + **渲染集成**：break-all 逐字符断行 + keep-all CJK 保持为单词，WordBreakMode 枚举） |

---

## 里程碑完成情况

| 里程碑 | 状态 |
|--------|------|
| M1 项目骨架 + 渲染基础设施 | ✅ |
| M2 HTML 解析 + DOM 树 | ✅ |
| M3 CSS 解析器 + 样式系统 | ✅ |
| M4 布局引擎 | ✅ |
| M5 渲染管线集成 | ✅ |
| M6 JavaScript 集成 (V8) | ✅ script-sandbox V8 引擎已集成，WebView.execute_script 可用 |
| M7 网络栈 + 导航模型 | ✅ |
| M8 多进程架构 (IPC) | ✅ (protocol crate) |
| M9 Canvas + Storage | ✅ |
| M10 WebView API | ✅ (webview + integration tests) |
| M11 浏览器应用 | ✅ 功能完成：Ctrl+快捷键（L/T/W/R/F/D/+/-/0/,）、鼠标滚动、右键菜单、书签栏、查找栏、缩放、自动补全、下载进度条、设置页面（zero://settings）、5 模块架构（均 <2000 行） |
| M12 高级 Web 能力 | ✅ 基本完成：Service Worker 集成（注册/安装/激活/注销/fetch 拦截 + navigator.serviceWorker polyfill）、WebAssembly JS API polyfill + WebView.execute_wasm() 真实执行、PerformanceObserver + performance API、QuickJS feature gate、Cache API、WPT runner（85 内建测试） |
| M13 性能优化 + 安全加固 | ✅ CSP 完整实现 ✅ Mixed Content 阻止 ✅ HSTS 支持 ✅ 增量布局计算 ✅ GPU Glyph Atlas ✅ 权限模型 ✅ 资源预加载 ✅ 站点隔离 ✅ V8 持久化 Context ✅ 渲染管线优化（填充批处理 + 视口剔除 + draw call 统计） ✅ TransformPrimitive 渲染 ✅ CSS 计数器渲染 |

---

## M12 进度详情

### -1. Service Worker 集成 + WebAssembly JS API + PerformanceObserver（本轮）

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| storage/service_worker | **Service Worker 注册表**：完整生命周期状态机（Registered→Installing→Installed→Activating→Activated→Redundant）、scope 匹配（路径/完整 URL）、fetch 拦截（缓存命中/PassThrough/NoWorker/Error）、Cache API 集成、origin 级管理、版本替换 | 12 |
| webview | **Service Worker 集成到 WebView**：fetch_url 拦截（SW 优先于网络请求）、register/install/activate/unregister API、service_worker_registry() 访问器、origin 提取辅助函数 | 8 |
| engine/dom_bridge | **WebAssembly JS API polyfill**：compile()、instantiate()、validate() 桩实现，记录模块和实例；**PerformanceObserver polyfill**：observe/disconnect/takeRecords、supportedEntryTypes；**performance API**：now()/mark()/measure()/getEntries() | 6 |
| wasm-sandbox | 边界测试：错误参数/global export/Display/config 链/空模块/多函数 linker | 6 |
| engine | dom_bridge.rs 文件拆分（2077→1145 + 948 行 tests） | — |

Total: 6219 → 6378 tests (+159)

### 已完成的 M12 交付物

- [x] Service Worker 基础（注册、生命周期、scope 匹配、fetch 拦截）— storage crate + webview 集成
- [x] Cache API — storage crate 已有
- [x] QuickJS feature gate — script-sandbox 已有（V8/QuickJS 双后端）
- [x] WASM 运行时 — wasm-sandbox 已有（wasmi/wasmtime 双后端）
- [x] WebAssembly JS API polyfill — engine dom_bridge
- [x] Observer API — MutationObserver/IntersectionObserver/ResizeObserver/PerformanceObserver polyfills
- [x] performance API — performance.now/mark/measure polyfill

### M12 剩余工作

- [ ] WPT 通过率持续追踪和扩展（当前 923 内建测试，25 分类，100% 通过率）
- [x] ~~页面级 WASM JS→wasm-sandbox 自动桥接~~ ✅ 已完成：JS WebAssembly.instantiate() 自动通过 base64 桥接到 wasm-sandbox，WebView.call_wasm_export() 调用缓存实例

---

## 下一步优先级（2026-08-04 更新）

> **2026-08-04 状态更新**：用户裁决工作切回父目标——**P1（DOM/JS Bridge 原生化）恢复为当前活跃主线，P1a 先行**。P0（字体栈）与 P2（渲染补齐）随渲染兼容性降频守成，等用户点名后切回；P3（GPU/Display）保持非紧急。

> 距离上一次优先级回顾（2026-06-06）已过去约 7 周，上一批 10 项优先级全部落地。以下优先级基于当前瓶颈和技术债重新评估，按战略重要性分层。

### P0 — 字体栈重建（战略破局）

> **⚠️ 2026-08-13 用户裁决**：字体栈方向**明确否决，zero-web 流不碰**（属 rendering-compat 流域，非本目标域）。本条此前记录的"2026-08-09 批准 / 2026-08-08 征询"等状态**已失效**——以文件顶部「⚡ 用户战略决策」块为准。本条保留仅作历史记录，**不作为推进依据**。

**为什么是 P0**：当前 reftest chromium-oracle 真一致 ~47%（FreeType-default 后），rendering-compat 赛道经 200+ 轮 clean-lever hunt 已基本穷尽（R2009 dormant-infra hunt 负，证实单 session lever 耗尽）。Headline ≥95% 的唯一战略杠杆是字体栈统一（fontdue → FreeType + HarfBuzz 单一权威度量/光栅/塑形管线）。

**当前状态**：
- [x] FreeType C-dep 已默认开启（R1159，+232 reftest，已验证可行）
- [x] 字体栈重建 RFC v0.2.3 已就绪（`docs/goal/rendering-compat/fontdue-replacement-scoping.md`）
- [x] 用户审批 RFC（2026-08-09，接受 HarfBuzz C 依赖），启动实施
- **2026-08-04**：rendering-compat 侧已将该方向列入待用户决策清单（font-stack C-dep rebuild，R2025 user-blocked），**等用户点名**；点名后切回渲染侧实施
- **2026-08-08**：已通过飞书 CLI 向用户征询（P1b RFC 审批 + 字体栈 RFC v0.2.3 审批 + 深结构点名三项合并一条消息，om_x100b6847a89f54a0c31f9749fe65ad9），等待回复；P1a 剩余切片（fetch byte-wire / customElements upgrade / rAF 帧驱动）不依赖此审批，可自主继续
- **2026-08-09**：**用户批准实施**（决策点汇总回复）——接受 HarfBuzz C 依赖；渲染侧恢复主动实施，深结构方向清单免点名（font-stack 项随本项消除）；第一刀按下方「后续步骤」选最小可度量收益切片，每片 kill-switch + 全量 oracle A/B net≥0 才落地

**后续步骤**（RFC 通过后）：
1. [x] 审查并通过字体栈重建 RFC（2026-08-09 已批准）
2. 拆分为独立可验证切片（度量统一 → 光栅统一 → 塑形 HarfBuzz → 字体回退逻辑）
3. 每个切片：kill-switch + 结构签名 gate + 全量 oracle A/B，net≥0 才落地
4. 第一刀选最小但可度量收益的切片（例如 FreeType 度量替 fontdue，保持光栅不变）

---

### P1 — DOM/JS Bridge 原生化（产品可用性破局）

**为什么是 P1**：字体栈决定"看起来对不对"，JS 绑定决定"能不能用"。当前 polyfill 字符串桥接模式存在三个硬限制：

| 问题 | 影响 |
|------|------|
| Polyfill 字符串序列化/反序列化 | 每个 DOM 操作 ~O(μs) 额外开销，SPA 不可用 |
| Observer 全为 stub | MutationObserver/IntersectionObserver/ResizeObserver 不触发回调，依赖这些 API 的框架（React/Vue 等）不可运行 |
| fetch() 为 stub + 事件循环简化 | AJAX/SSR/hydration 不可用 |

**分两阶段推进**：

| 阶段 | 范围 | 收益 |
|------|------|------|
| **P1a: 事件循环补全 + 核心 API 真实化** | HTML spec event loop（microtask queue / task queue / rAF / ric），fetch() 走真实 net crate，MutationObserver 真实触发 | 简单表单、AJAX 页面可用 |
| **P1b: V8 原生绑定** | 用 rusty_v8 `FunctionTemplate` 替换 polyfill 字符串桥接，Rust DOM 对象直接暴露给 V8 | 性能提升 10-100x，能跑 SPA 框架 |

P1a 低风险、可快速见效（主要改 `dom_bridge.rs` + `script-sandbox` + `net` crate）；P1b 是架构级改造，需要独立 RFC 和风险拆分（与字体栈 RFC 同级对待）。

**当前状态**：
- [x] **P1a: 事件循环补全 + fetch/MutationObserver/IO/RO 真实化 — 实质完成（2026-08-13 复核）**。详见下方「P1a 复核结论（2026-08-13）」。剩余仅为非阻塞 follow-up（IO/RO 持续 tick Slice 2b / 纯 host 侧 MO / A 代死代码「提及不删」）。
- [x] **P1b: V8 原生绑定 RFC — 全部完成（js-dom goal 已收官归档，2026-08-31）**（[`docs/specs/p1b-v8-native-bindings-rfc.md`](../../specs/p1b-v8-native-bindings-rfc.md)，方案 C 混合 DOM-Node；M1–M5 全达成，**R383/R384 双引擎 `native_dom` default-on land + kill-switch 删除（M7 QuickJS 同款 default-on）**，DC-1~8 全 ✅，goal 归档至 [`docs/goal/archive/js-dom/`](../archive/js-dom/master.md)；此前 2026-08-08 飞书征询（om_x100b6847a89f54a0c31f9749fe65ad9）、2026-08-09 用户批准 S0 PoC）

**P1a 复核结论（2026-08-13，本轮核对实际代码后纠正——此前 master 记 P1a 为活跃主线/4 切片待办，现已实质完成）**：

- **切片 1 事件循环 ✅**：rAF 帧驱动（`part01.js:1573` `requestAnimationFrame` + `__zw_raf_tick` host 回调 + `_rafPending`，render 后 host 派发，含 webkit/moz 别名 + cancelAnimationFrame + rAF budget）；requestIdleCallback ✅（`part01.js:886`，timeout 选项 + IdleDeadline{didTimeout,timeRemaining} shape，host 不可用时 microtask 兜底）；setTimeout/setInterval/clearTimeout 真实延迟 ✅（TimerBridge 子线程，tab_js_worker 测 811/831/852 e2e）。microtask checkpoint 经 `_defer` 派发。
- **切片 2 fetch 完整化 ✅**：shim 透传 method/headers/body（part05.js fetch + Response），host `FetchRequest{method,url,headers,body,body_bytes}` + `FetchResponse{status,status_text,headers,body,body_bytes}`（`fetch_bridge.rs`，R2923 + R3021 byte-wire）；AbortSignal/AbortController ✅（part02.js，signal 传 fetch + settled flag 防双 resolve + AbortError DOMException）；e2e 测 `tab_js_worker_fetch_post_method_headers_body_r2923`（762）。**fetch GET/POST/headers/body/abort 全链路真实**。
- **切片 3 Observer ✅（尽力而为单发，Slice 2b 持续 tick 为已知 follow-up）**：MutationObserver JS-proxy 拦截 ✅（part01.js `_mo_notify` + part04.js:613/644 proxy setter 触发，observe-options 全矩阵 R3025-R3028 闭合）；IntersectionObserver ✅（part01.js:1092 Slice 2a，gBCR + rootMargin R2966 + threshold + initial notification + DOMRect 身份 R3319）；ResizeObserver ✅（part01.js:1331 Slice 3）。**已知接受限制（非 bug）**：IO/RO 仅 observe 时算一次，scroll/resize/async-layout 后续通知 = Slice 2b（须 host render-loop tick）；纯 host 侧 DOM 变更不经 JS proxy 不触发 MO（B-gen 路径全经 proxy，影响小）。polyfill e2e 测 dom_bridge_polyfill.rs:702-771 覆盖 observe/unobserve/disconnect 计数。
- **切片 4 A 代路径 + wpt-runner 空洞 ✅**：web_api 空洞通过已由 **R3076 闭合**（`runner/mod.rs:158 check_js_executes_ok` 经 `wv.run_page_scripts_strict()` 真实执行内联 `<script>`，首个抛异常即 fail；R3329 扩 security/*、R3083 扩 interactive/*、R3332 加 native-dom parity 变体）；A 代 `DomCommand`/`parse_command` 仅单测引用 = 死代码（CLAUDE.md「提及不删」，不阻塞）。

**P1a 探查结论（2026-08-04 恢复推进前评估，历史记录）**：文档「fetch/Observer 为 stub、DomCommand 30+ 变体」描述**已过时**——生产路径已迁移到 B 代桥接（`crates/engine/src/js_dom_shim/part01-06.js` + `apps/browser/src/tab_js_worker.rs`），**fetch GET 已端到端真实**（`FetchBridge` + `AsyncResolver` + net crate，含真实 HTTP 测试）、**MutationObserver（JS 驱动）已真实触发**（shim Proxy trap 拦截）、setTimeout 已真实延迟（TimerBridge 子线程）。P1a 历史规划 = 4 切片：

1. **事件循环**（shim 内 JS 侧为主 + 1 条 host 命令）：建 macro-task 队列（microtask-before-macrotask 排序）；setTimeout 回调改投 task queue；rAF 改帧驱动（新增 host `FrameTick` 命令投递，tab_js_worker 命令枚举加变体）；requestIdleCallback 缺失补建（rAF 后空闲窗口）— **✅ 已完成（见上方复核）**
2. **fetch 完整化**：shim 透传 method/headers/body → host 返回 (status, headers, body)；AbortSignal 最小支持（`__zw_abort` 回调）；`FetchHandler` 签名升级（`fetch_bridge.rs:19`），用 `default_fetch_handler`（tab_js_worker.rs:289）适配器消化 browser/renderer/reftest 三处构造点 — **✅ 已完成（见上方复核）**
3. **Observer**：MutationObserver 补「host 侧变更不触发」缺口（`apply_dom_mutations` 后回注 `__zw_mo_host_*` 事件）；IntersectionObserver/ResizeObserver 新建（依赖 `__zw_get_rect` 回调 + 帧 tick）。**MutationObserver observe-options 矩阵已彻底收尾**（R3025 attributeFilter/attributeOldValue + R3026 subtree + R3027 characterData emission + **R3028 characterDataOldValue**——全记录类型 + 全 observe options spec 合规） — **✅ MO + IO + RO 已实现（尽力而为单发）；纯 host 侧 MO 触发 + IO/RO 持续 tick 为 Slice 2b follow-up（非阻塞）**
4. **A 代路径统一**：`DomCommand`/`parse_command`（dom_bridge.rs:200-319）仅剩单测引用 = 死代码（CLAUDE.md「提及不删」）；`webview.execute_script_with_dom`（A 代）或废弃或转注 B 代 shim；**wpt-runner web_api 用例为空洞通过**（runner 不执行内联 JS，test_cases_web_api.rs:161-181），P1a 后需补 JS 执行路径 — **✅ 空洞通过已由 R3076 闭合（见上方复核）**

切片 1-3 均低风险可独立 land；验证基线 = tab_js_worker 既有测试（fetch 端到端 663-810 / 定时器 811-852 / MutationObserver 五连测 906-1065，`wait_for_global` 轮询模式）+ 每切片 `make test` 零回归。P1b（V8 原生绑定）仍需独立 RFC。**P1a 主线实质完成 → 当前活跃推进面 = security/storage/net deep-review（自主域）+ P1b（S6/S7 等用户拍板 default-on）+ P3 GPU/Display（需物理环境）**。

> **2026-09-04 状态勘误**：上行「P1b 待 default-on 拍板」已过时——P1b 全部完成（R383/R384 双引擎 default-on land + kill-switch 删除，js-dom goal 2026-08-31 收官归档，见上方「当前状态」P1b 条）；security/storage/net deep-review 自主域亦经 R3388–R3398 逐文件审后收敛（见「最近完成的改进」）。媒体线三 goal（2026-09-05）与存储/Service Worker 两 goal（2026-09-06）均已收口整树归档（见 `docs/goal/archive/`）；keyboard/editing 三 goal 与 webdriver goal 已于 2026-09-08 收口整树归档（editing selection 终态 2994P/5F = 99.8%、隐式提交 3/3；webdriver 9→33 endpoint，DC-1~4 ✅）；webdriver-screenshot goal 已于 2026-09-09 收口整树归档（`zero-paint-convert` 公共转换层 + GET /session/{id}/screenshot，33→34 endpoint，DC-1~4 ✅）；storage-opfs goal 已于 2026-09-09 收口整树归档（DC-1~4 ✅）；web-components goal 已于 2026-09-11 收口整树归档（DC-1~5 ✅）；page-wasm goal 已于 2026-09-12 收口整树归档（DC-1~4 ✅，WPT jsapi 31 案 99.4%）；event-loop-spec goal 已于 2026-09-12 收口整树归档（P1a 遗留面闭合——IO/RO WPT 基线 + MO host 触发 default-on + per-task checkpoint default-on，DC-1~4 ✅，task queue 子项经用户决策缓行挂账）；android-browser goal 已于 2026-09-13 判定 DONE（DC-1~4 ✅——多标签槽位/视口真值/IME 输入/下载通知 SAF/双语无障碍/CI 模拟器冒烟 + chaos 断连恢复，真机门控冒烟项见其 master「下一步」，待模式 A 整树归档）；M12/M14 首批拆分六子 goal 全部达成；2026-09-12/13 用户决策新立十四个子 goal（cdp-protocol / devtools / desktop-browser / security-hardening / web-api-batch2 + 九个 WPT 分域 compat goal：svg / net-api / navigation / workers / timing-animation / encoding / html-syntax / uievents / webgl）；当前父目标活跃面 = 渲染兼容性收口（rendering-compat 流，R4744 轮 corpus default 15175/16594 = 91.45%——ruby-overhang 模型 flip default-on（R4422）后 vertical writing modes 线（R1043 深域）已开动推进 R4425–R4463，multicol-span-all / region-distribution 线 R4499–R4524、边框/裁剪与 background-clip:text 染色线 R4525–R4553、尺寸关键字与垂直 width 族线 R4556–R4569、corner-shape 延伸与 @container style() 线 R4570–R4582、font-size 计算值确定性线 R4642/R4643 持续收口，R4584–R4744 守成巡检维持）+ 上述十四个新子 goal 推进（devtools 已于 2026-09-17 达成 DONE 转守成——M1 Elements/Console + M2 Network/Application 四面板演示流全绿 + M3 GUI 模式 CDP 可开关，DC-1~3 ✅，GUI 标签页桥接等碰头协调项挂账；web-api-batch2 已于 2026-09-24 M4 收口——Clipboard API + Fullscreen API 语义修齐，WPT clipboard-apis 83.6% / fullscreen 88.0%，DC-1~4 ✅，DnD 维持排除挂账；security-hardening M1 导入基线 + M2-s1–s5 CSP 运行时强制接线推进中——script/style/img 检查点 + connect-src 阻止族 + `SecurityPolicyViolationEvent` 事件，csp corpus subtests 16.3%→28.9%；cdp-protocol 静默监测轮至 S1664，绿步 6→33 维持）与零星自主域复扫。

---

### P2 — CSS 渲染补齐 + rendering-compat 赛道继续（低风险填充）

字体栈决策到第一刀落地之间可能存在等待期。rendering-compat 的自主 hunt 继续作为低风险填充：
- **Print-layout**（当前 R2011 在做 @page 规则）——reftest yield 低（~6 用例），但产品价值存在（打印预览是真实浏览器需求）
- **CSS 属性"已解析→已渲染"补齐**——还有少量属性只有解析没有渲染（安全、低风险的 clean win）

这些工作**不与字体栈或 JS bridge 冲突**，可以并行推进。

**当前状态**：
- [x] @media print 完整管线（R1981-R1994：级联+测量+embed API+浏览器 Ctrl+P UI）
- [x] Print-layout Phase P1a-M1/M2（break-after + natural fill + nested promotion）
- [ ] Print-layout @page size/margin（R2010-R2011 进行中）
- [ ] CSS 属性渲染补齐（剩余未渲染属性扫描）
- **2026-08-04**：本项随渲染兼容性降频守成，不再作为自主推进面；等用户点名渲染深结构时一并恢复

---

### P3 — GPU/Display 验证

Done Criteria 唯一未勾选的项。需要实际 GPU 桌面环境验证 GPU 加速合成正常工作。非紧急，但在字体栈第一刀落地后应尽早跑一次完整验证。

**当前状态**：
- [ ] GPU 加速合成正常工作（需真实 GPU + Display 环境）
- [ ] 三平台（macOS/Linux/Windows）真实窗口渲染验证

## 浏览器质量测试体系推进计划

研究依据：[WebView 渲染合规测试：以 WPT Reftest 为主、像素基线为辅](../../research/research-webview-rendering-test-strategy-2026-06-04.md)

当前判断：`tests/wpt-runner` 已能解析 WPT manifest 类型并运行内置 HTML/CSS 用例，但执行层主要验证“不 panic、有 DOM/layout/primitives”。它适合作为 smoke/invariant runner，暂不能证明 CSS 排版、布局几何或最终渲染像素与规范或参考页一致。后续目标是把“WPT 测试持续扩展”升级为覆盖浏览器质量关键面的测试体系。

### 质量测试矩阵

| 测试层 | 状态 | 目标 | 后续动作 | 验收标准 |
|--------|------|------|----------|----------|
| 1. 标准合规测试 | 🔄 | 验证 HTML、DOM、CSSOM、CSS、Fetch、Storage、Workers、Modules、WASM、Security 等 Web 标准行为 | WPT 1341 用例（23 分类，100% 通过率），覆盖 HTML/DOM/CSS/JS/Canvas/Storage/Security/Navigation/Workers/ES Modules/CSS Layout Subset/CSS Advanced/Platform Input/Accessibility/A11y-I18n | 按标准模块输出通过率；PR 只阻断 unexpected fail |
| 2. 渲染正确性测试 | 🔄 | 验证 CSS 排版、layout geometry、paint order 和最终视觉等价 | Phase 1-3 ✅（layout/primitive snapshot + 16 reftest + 45 CSS/layout subset + 按分类通过率报告）；Phase 4 待推进 | 核心 CSS/layout 改动可被 snapshot 或 reftest 捕获 |
| 3. 真实网站兼容性测试 | ✅ | 验证真实页面组合能力，而不只验证单项标准 | Top 20 真实网站兼容性测试（24 个测试，20/20 站点通过），覆盖 fetch → parse → style → layout → paint 完整管线 | 每个站点有可复现结果，全部通过（需 `--ignored` 运行） |
| 4. 安全测试 | 🔄 | 验证浏览器安全边界不被绕过 | 52 个跨 crate 安全管线测试 + SecurityContext 集成 + HSTS preload + 混合内容执行引擎 + 19 WPT 安全扩展 | 安全边界测试进入 CI；fuzz/sanitizer 夜间报告无新增高危问题 |
| 5. 运行时和事件循环测试 | 🔄 | 验证 JS + DOM + Web API 的组合时序 | 当前：简化版 setTimeout/setInterval（polyfill 定时器管理），Observer 为 stub，fetch() 为 stub。P1 计划：实现 HTML spec event loop（microtask/task queue/rAF/ric），让 MutationObserver 真实触发回调，fetch() 走真实 net crate | DOM mutation 后 style/layout/paint 更新顺序可测试；关闭/导航不泄漏状态 |
| 6. 网络和导航测试 | 🔄 | 验证导航状态机、资源加载和历史行为 | 10 个 WPT 导航测试（Redirect/Hash/Cache/Cookie/HSTS/StateMachine/SW/Timeout/CORS）+ 13 个 URL+安全管线集成测试 | 导航和网络异常路径可复现；历史和资源状态稳定 |
| 7. 性能测试 | 🔄 | 从 crate benchmark 上升到页面级性能预算 | 16/16 crate 有 criterion 基准（78+ 个）；中等复杂度页面首屏 < 2s 验证通过；增量布局验证 | 页面级性能报告可比较；关键预算有阈值和趋势 |
| 8. 平台和输入测试 | 🔄 | 验证跨平台、字体、DPI、输入和 GPU/CPU fallback | 18 个 WPT 平台输入测试（键盘/鼠标/触摸/滚动/视口响应式/HiDPI/IME/CJK 输入/焦点管理）+ 15 个视口自适应集成测试（响应式重排/极端视口/resize 往复/CSS viewport 单位）+ 19 个字体回退国际化渲染管线集成测试（CJK/RTL/emoji/多语言混合/竖排文本） | 平台差异进入 expected/skip 管理；关键输入路径跨平台通过 |
| 9. 产品层测试 | 🔄 | 验证 ZeroBrowser 和 ZeroWebView 作为产品/API 可用 | 31 个 BrowserShell+WebView 产品级 smoke 测试（标签页生命周期/地址栏自动补全/书签CRUD+导航/历史记录搜索+清除/下载管理/设置/缩放控制/查找功能/会话保存恢复/上下文菜单） | 产品级 smoke 可在发布前阻断明显退化 |
| 10. 无头协议和自动化控制面 | ✅ | 支持外部自动化工具驱动 ZeroWeb，用协议统一真实站点、截图、性能和产品 smoke | Phase 1-5 全部完成（WebSocket 服务器 + JSON 消息路由 + CDP 命令 + 自动化测试 + 安全加固） | 外部客户端可连接、建上下文、导航、执行脚本、截图并收集网络/日志事件 |

### WebView 渲染合规测试阶段

| 阶段 | 状态 | 范围 | 验收标准 |
|------|------|------|----------|
| Phase 1: Layout/primitive snapshot | ✅ | `LayoutResult::snapshot()`、`RenderPrimitives::snapshot()` 稳定文本/JSON dump；精确几何断言系统 | CSS/layout 改动能产生可读 diff；核心用例可定位到 geometry、style 或 primitive 差异 |
| Phase 2: 最小 reftest harness | ✅ | ReftestCase（test/ref HTML pair）、ReftestConfig（viewport/fuzzy threshold）、run_reftest()（CPU framebuffer 像素比较）；16 个 CSS 布局 reftest（block/flex/position/color/box-model） | ✅ 能跑一组 reftest 并输出 pixel diff 统计 |
| Phase 3: 真实 WPT CSS/layout 子集 | ✅ | 45 个 CSS/layout 子集测试（按 CSS 规范领域组织：盒模型/VFM/Flexbox/Grid/排版/颜色/变换/逻辑属性/变量/综合布局）；CategorySummary 按分类通过率报告（文本+JSON）；expected metadata 已就位 | PR 只阻断 unexpected fail；报告按 CSS/layout 分类汇总并通过率排序 |
| Phase 4: WebView 产品级视觉 smoke | ✅ | 27 个 headless WebView smoke 测试（固定页面加载验证、视口 resize 重渲染、CSS 注入变化检测、多页面导航状态、脚本执行 DOM 修改、事件回调、prefers-color-scheme、WASM 执行、综合生命周期、渲染管线耗时分解） | 用于发现产品可见退化，但不替代引擎级 reftest 和 WPT 合规测试 |

短期不要直接扩大 golden screenshot 覆盖。像素基线只用于无法构造 reftest 的少量场景，并且必须固定 OS/font/DPI/scale 与 fuzzy 阈值。

### 推进优先级

1. **P0: 字体栈重建** — FreeType + HarfBuzz 统一度量/光栅/塑形，唯一能推动 headline refest 从 ~47% 到 95%+ 的战略杠杆。RFC v0.2.3 已就绪，待审批启动。
2. **P1: DOM/JS Bridge 原生化** — P1a 事件循环补全 + fetch/MutationObserver 真实化（低风险快速见效）；P1b V8 原生绑定（架构级改造，需独立 RFC）。
3. **P2: 构建与基础设施** — CI 3 平台全部绿色、WPT 持续扩展、覆盖率不退化、性能基准趋势追踪。
4. **P3: GPU/Display 验证** — 真实 GPU 桌面环境下验证加速合成；三平台真实窗口渲染验收。
5. **P4: 产品化打磨** — 可访问性深化、跨平台发布、真实网站兼容性矩阵持续扩展。

## 无头浏览器协议支持计划

协议依据：
- [W3C WebDriver BiDi](https://www.w3.org/TR/webdriver-bidi/)：标准化的双向浏览器自动化协议，基于 WebSocket。
- [MDN WebDriver BiDi connection](https://developer.mozilla.org/en-US/docs/Web/WebDriver/How_to/Create_BiDi_connection)：说明浏览器和客户端通过 WebSocket 连接，常见入口是 remote debugging port。
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)：Chrome/Puppeteer/Playwright 生态常用的调试和自动化协议。

协议策略：**WebDriver BiDi 优先，CDP 兼容子集辅助**。BiDi 是跨浏览器标准主线，适合作为长期协议；CDP 生态成熟，适合作为 Playwright `connectOverCDP`、Puppeteer 和调试工具的短期接入面。短期不追求完整 CDP，只实现能支撑导航、脚本、截图、网络事件和目标管理的最小域。

| 阶段 | 状态 | 范围 | 验收标准 |
|------|------|------|----------|
| Phase 1: 远程调试服务骨架 | ✅ | `--headless`/`--remote-debugging-port` CLI 标志、WebSocket 服务器（tungstenite）、JSON message id 路由、session 生命周期、6 个命令（session.status/new、browser.close、browsingContext.navigate、script.evaluate、captureScreenshot、getDOMSnapshot） | ✅ 能启动无窗口实例并接受 WebSocket 命令；10 个单元测试覆盖全部命令 |
| Phase 2: WebDriver BiDi 核心子集 | ✅ | ✅ browsingContext.create/getTree/close/reload ✅ script.callFunction ✅ /json/version HTTP 发现 ✅ 事件推送（browsingContext.load/log.entryAdded/contextCreated/contextDestroyed） ✅ 多客户端连接 | 事件推送系统已实现，12 个事件相关测试覆盖 |
| Phase 3: CDP 最小兼容子集 | ✅ | ✅ HTTP 发现服务器（/json/version、/json、/json/list） ✅ Page.navigate/captureScreenshot ✅ Runtime.evaluate ✅ Target.getTargets ✅ Network.enable 桩 | HTTP 发现和核心 CDP 域命令已实现；Playwright/Puppeteer 可发现和连接 |
| Phase 4: 自动化测试接入 | ✅ | ✅ HeadlessClient 协议客户端（parse_response/build_request/parse_event/parse_screenshot/parse_dom_snapshot） ✅ ProtocolTestRunner 全协议往返模拟器 ✅ DomSnapshotStats 图元统计 ✅ 8 个协议驱动冒烟测试（完整会话生命周期、CDP 命令序列、脚本执行变体、多浏览上下文管理、渲染管线验证、协议错误处理、重载事件序列） | 协议驱动冒烟测试可覆盖会话/导航/脚本/截图/上下文管理/错误处理；Phase 4 基础设施就位 |
| Phase 5: 隔离与安全加固 | ✅ | ✅ 默认绑定 `127.0.0.1`（不暴露公网） ✅ `HeadlessSecurityConfig`：可配置 auth_token（首个请求必须携带 token） ✅ Origin allowlist（HTTP + WebSocket 连接均检查） ✅ `extract_origin_header` 从 HTTP 请求头提取 Origin | 7 个安全单元测试覆盖 token/origin/localhost 绑定 |

协议实现边界：
- 先支持单 browser process + 多 browsing context，暂不承诺多进程 target attach 的完整语义。
- 先支持无头截图和脚本执行；输入事件、下载控制、权限控制、请求拦截后续扩展。
- CDP 只做兼容子集，不复制完整 Chrome DevTools Protocol。
- 协议层测试必须反向驱动质量计划：真实站点 smoke、性能预算、渲染 reftest 都应逐步迁移到协议控制面。

### Done Criteria 评估（2026-07-24）

| Done Criteria | 状态 | 说明 |
|---------------|------|------|
| 1. WebView 可嵌入 | ✅ | lib crate 可引入、load_url/execute_script/V8 集成、Builder API、事件回调、**Web Worker 管理**均就位。**多进程架构已实现**。**Top 20+ 真实网站全部验证通过**（20/20 站点 fetch → render 管线完整）。**55+ 真实网站兼容性测试就位** |
| 2. 浏览器日常可用 | ✅/❌ | 多标签页/地址栏/前进后退/收藏夹/历史/下载/查找/缩放/右键菜单/设置均就位。**真实网页渲染已验证**（55+ 真实网站通过完整管线）。**缺少：GPU/Display 环境下的真实窗口渲染验证**（P3）。**缺少：交互式网站可用性**——polyfill 桥接（fetch/MO/rAF 等核心 API 已真实，R3177 复核）+ 事件循环 microtask checkpoint 时序简化（非 spec 每 task checkpoint）+ native DOM 未成生产路径（escape-hatch/L2 polyfill-live 待用户点名，rule 11）（P1） |
| 3. Web 标准兼容性 | ✅ 大部分 | HTML/CSS/JS/DOM/Canvas/Network/Security/WebSocket/Storage 均已实现。WPT 1341 用例（23 分类，**100% 通过率**，**按分类通过率追踪就位**）。**Web Workers + ES Modules 已实现**。**WASM 自动桥接完整实现**。**安全管线集成测试 52 个**。**可访问性基础测试 19 个**。**DOM/JS Bridge**：polyfill 桥接模式（30+ DomCommand），Observer stub、fetch stub、事件循环简化。**rendering-compat**：reftest 自源 ~57% / chromium-oracle ~47%（字体栈是 headline ≥95% 的唯一战略杠杆） |
| 4. 性能基准体系 | ✅ | 78+ 个 criterion 基准覆盖所有 crate。**中等复杂度页面首屏 < 2s 已验证**。**增量渲染验证通过**（incremental_paint 图元 < 全量 20%）。GPU 加速验证待 GPU/Display 环境 |
| 5. 单元测试与质量 | ✅ | 12,001 测试全绿，95.46% 行覆盖率（函数 96.94%），clippy 零警告，**55+ 真实网站兼容性测试（ignored）**，**58 个产品级 smoke 测试**，**1341 WPT 用例（23 分类，100% 通过率）** |
| 6. 工程化 | ✅ | CI（3 平台）、CI 发布工作流（Linux/macOS/Windows 自动打包）、scripts/run-benchmarks.sh、scripts/check-coverage.sh、scripts/package-linux.sh、scripts/package-macos.sh、scripts/package-windows.ps1、18 个 crate 全部有 README、WebView demo 可编译、API 文档（cargo doc） |

**剩余阻塞项**（需 GPU/Display 桌面环境）：
1. GPU 加速合成正常工作验证

**已完成项**（本轮融资内完成）：
1. ~~Top 20 真实网站加载渲染验证~~ ✅ 20/20 站点通过完整管线
2. ~~中等复杂度页面首屏渲染 < 2s 性能验证~~ ✅ python.org 真实网站渲染 < 2s
3. ~~Web Workers（Dedicated Worker）实现~~ ✅ 已完成
4. ~~ES Modules（`<script type="module">`）实现~~ ✅ 已完成
5. ~~多进程架构实际运行~~ ✅ 已完成
6. ~~V8 快照优化（M13 剩余）~~ ✅ 已完成
7. ~~浏览器质量测试体系 P0~~ ✅ 全部完成
8. ~~无头浏览器协议 Phase 1-5~~ ✅ 全部完成
9. ~~WPT 按分类通过率追踪~~ ✅ CategorySummary + 文本/JSON 报告
10. ~~质量测试矩阵 Phase 3~~ ✅ 74 个 CSS/Layout 子集测试（15 个 CSS 规范领域）
11. ~~WebView 产品级视觉 smoke Phase 4~~ ✅ 27 个 headless smoke 测试（load/resize/CSS注入/导航/脚本/事件/缓存）
12. ~~产品层 smoke 测试~~ ✅ 31 个 BrowserShell+WebView 产品级 API 测试（标签页/书签/历史/下载/设置/缩放/查找/会话/上下文菜单）
13. ~~可访问性基础~~ ✅ FocusManager（Tab 导航 + tabindex 排序 + 13 单元测试）+ 19 个 ARIA WPT 测试
14. ~~跨平台打包脚本~~ ✅ Linux AppImage/deb + macOS .app + Windows .zip
15. ~~CI 发布工作流~~ ✅ GitHub Actions 多平台构建 + .deb + .app + Release 自动创建
16. ~~真实网站兼容性矩阵扩展~~ ✅ 35→45 站点（+10：rfc-editor/unicode/reqres/postman-owasp/openssl/chromium/deno/bun/csswg）
17. ~~WASM 自动桥接完整实现~~ ✅ WebAssembly.instantiate/compile/instantiateStreaming + 魔术字节验证 + _start 自动执行 + 导出函数调用队列 + 内存状态注入 + 6 WPT 测试
18. ~~WebView 单元测试覆盖提升~~ ✅ +20 WASM 桥接单元测试（base64/execute_wasm/call_wasm_export/_start/bridge integration）
19. ~~真实网站兼容性扩展至 55+~~ ✅ 45→55+ 站点（+10：tc39/webassembly/jsdelivr/npm/arxiv/owasp-top-ten/replit/caniuse/html-spec/ziglang）
20. ~~平台和输入测试 Layer 8~~ ✅ 18 个 WPT 平台输入测试 + 15 个视口自适应集成测试 + 19 个字体回退国际化渲染管线集成测试

---

## 归档记录

- **M1** ✅ → [archive/m1-skeleton-render-foundation.md](archive/m1-skeleton-render-foundation.md)
- **M2** ✅ → [archive/m2-dom.md](archive/m2-dom.md)
- **M3** ✅ → [archive/m3-css-parser-style-system.md](archive/m3-css-parser-style-system.md)
- **M4** ✅ → [archive/m4-layout-engine.md](archive/m4-layout-engine.md)
- **M5** ✅ → [archive/m5-rendering-pipeline.md](archive/m5-rendering-pipeline.md)
- **M7** ✅ → [archive/m7-network-security.md](archive/m7-network-security.md)
- **历史 CI / ZRG / GB 守护记录（2026-08 至 2026-11-19）** → [archive/guard-history-2026-08-to-11-19.md](archive/guard-history-2026-08-to-11-19.md)
