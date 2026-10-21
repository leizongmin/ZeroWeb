# M2-S2c：DC-2（COMP-02）验收——查询一致性 + 结束事件 + 导航换代（真实窗口）

日期：2026-10-21 ｜ 里程碑：M2-S2c ｜ 判据：DC-2

## 判定

**有条件成立**。DC-2 四场景在真实窗口（生产合成帧采集）成立：(a) 忙窗前后 gCS(opacity) 与红盒 alpha 同相位（稳态 median dphase 0.080）；(b) 忙窗内/空闲期结束各恰一次 animationend、elapsedTime=1.000s；(c) flag OFF 对照全零（条带恒级联值、零事件、帧收敛冻结）；(d) 导航换代旧事件不重放。条件：像素级相位差受条带内容发布延迟约束（内容帧提交节奏 100-1300ms），逐帧 dphase≤0.15 覆盖 221/343——查询正确性由单一采样器结构等价（renderer/compositor 消费同一 `sample_declaration`，单测锁定）保证，像素通道为同相位佐证；RFC §3 Assumption 行已按实测更新。

## 环境

| 项 | 值 |
|---|---|
| 平台 | Linux (WSL2/WSLg Weston)，窗口 600×1092 |
| fixture | tests/fixtures/compositor-animation/comp02-script-state-and-end-event.html（+ comp02-page2.html；autostart：load+1.5s 自动 4×220ms 负载；page1 断言完成后 @7.0s 导航 page2） |
| 断言外显 | 窗口模式无 automation 通道：gCS 值编码为 #q-strip 灰度（level = 255 − round(opacity×240)）、事件回执四格色条（白未发生/绿恰一次且 elapsedTime 合格/红重复或不合格） |
| 采集 | frame_dump（compositor swap 后落盘）；本批 `ZW_COMPOSITOR_FRAME_DUMP_MAX` 上限 256→512 |
| 对照 | flag ON（=1）/ OFF（=0）各 12s |
| 埋点 | renderer animationend 逐事件 info 行（真相通道）；query override 命中/表键 debug 行（ZW_LOG_LEVEL=debug 时，本 run 为 info） |

## 实现变更（本切片）

1. **browser Get 前台 key 漂移两段修复**（compositor GetCompositorFrame 处理只取 surface_id、回实际前台 key；内容帧推进滞后时响应 frame_id 旧于请求 key。忙窗后 keep-alive 排队连发 Get(20)/Get(21)、前台仍 20 实测命中）：
   - demux Data 臂按 (surface, epoch) 接受（原 key 全等 fatal 断链，错误 `mismatched compositor response id`）；
   - `CompletedFrameChannel::try_recv` 按 (surface, epoch) 取该 surface 最新完成帧（原 `>= frame_id` 匹配令漂移响应永不可采纳 → 重采样像素滞留缓存、呈现静帧饥饿；on3 实测 compositor 277 次 Get 全 now_active=true 而画面静止）。
   - 回归测试 `get_accepts_front_frame_id_drift_from_requested_key`（zero-browser bin target）。
2. renderer `check_compositor_animation_finishures` 逐事件 info 行（`animationend dispatched selector=… elapsed=…`，exactly-once 与 elapsedTime 口径核对）。
3. frame_dump 上限 clamp 256→512（12s×~30fps≈360 帧需覆盖 page1 全程 + page2 换代后回执）。

## 实测

### ON 主实验（analysis-on.txt，manifest-on.jsonl）

| 指标 | 值 |
|---|---|
| 采样帧数 / distinct hash | 357 / 321（12s 无截断） |
| 内容帧 | 16 个（frame_id 1→76，含 page2 两个） |
| animationend | 恰 3 行，各元素一次、elapsed 均 1.000s（animationend-on.txt） |
| 事件间隔 | 3.031s / 3.649s，与 origin 相对预期（≈1.9s→4.95s→8.5s，差 3.05/3.55）一致 |
| 回执条首见 | busy @2256ms（负载窗 1.5-2.7s 内 → 结束事件负载后派发=排队语义）、idle @5979ms、replay @7389ms（page2）、own @8956ms（page2）；全部 pass，357 帧零 fail/qfail |
| (d) 导航换代 | nav 签名 @7232ms 翻转为 page2；replay 条绿 = page2 document 级监听未收到 comp02-end-a/b（旧事件不重放）；renderer 全程仅 3 行（无 re-dispatch） |
| 相位 | 稳态对（o_pixel<0.999 排除提交瞬态）343 对：median dphase 0.080，≤0.15 覆盖 221/343 |

尾差归因（analysis-on.txt 中 ost 变化行与内容帧提交点一一对应）：条带像素只在内容帧提交时上屏（16 个内容帧间隔 100-1300ms），JS 采样值在提交间隔内正确但像素滞留。例：seq24 @3645ms ost=0.454 与同帧 box 相位差 0.035（采样即正确），其后 600ms 像素滞留至下一内容帧 f41 才刷新。查询正确性证据 = 单测 `query_output_matches_shared_sampler_across_scan`（renderer 输出 ≡ 共享采样器扫描等价）+ compositor `animation_overrides::apply` 消费同一 `sample_declaration` + 早期诊断 run（debug 级）hit=true 连续。

忙窗：seq7→8 dt=1013ms 采样空洞，复现 S1e 已知主线程-Wayland 呈现耦合（缺口表既定，不在本切片修）。窗内条带 ost=1.000：相位 2s 边界处 o=1.000 的精确采样（2256ms ≈ origin+2.000s）与级联回落两种解释像素上不可分（本 run info 级无 hit= 行），记入局限。

### OFF 对照（analysis-off.txt，场景 c）

40 帧、distinct hash 仅 5 → 帧序列收敛冻结；o_pixel = o_strip = 1.000 恒定（条带恒为级联值，灰度 level 15）；蓝盒 bbox 冻结 (252,96)-(347,191)（初始 transform 一次性应用后不推进）；零 animationend（renderer 0 行）；回执条 busy/idle 全程 none。JS 负载与导航仍按页内脚本执行（marker/nav 签名照常变化），仅动画链路全停——对照面干净。

## 结论

1. DC-2 判「有条件成立」：查询一致性（稳态同相位 + 结构等价双证据）、结束事件 exactly-once（含忙窗内排队语义与空闲期及时派发）、flag OFF 对照、导航换代不重放，四场景均落证据。
2. S2a（查询覆盖 hook + 采样 provider）与 S2b（惰性终结判定 + 落定唤醒）在真实窗口端到端成立；Get 前台 key 漂移缺陷在本切片发现并修复（fatal 断链与静默饥饿两段）。
3. flag 默认仍关（翻默认需 perf-gate 证据，RFC 既定）。

## 产物

| 文件 | 说明 |
|---|---|
| analysis-on.txt / analysis-off.txt | 逐帧分析（sha12、o_pixel/o_strip/dphase、回执条分类、内容帧/签名首见、相位统计；零依赖脚本生成） |
| manifest-on.jsonl / manifest-off.jsonl | frame_dump manifest（seq/surface/epoch/frame_id/尺寸/墙钟） |
| run-config-on.txt / run-config-off.txt | 运行配置（flag/wall/fixture） |
| animationend-on.txt | renderer animationend 真相通道 3 行（脱 ANSI；`.log` 后缀被仓库 gitignore，故记 .txt） |
| frame-on-n0000-f1-initial.png | ON 首帧（基线，与 OFF 冻结态同源） |
| frame-on-f24-bar-busy-pass.png | 忙窗内回执：bar-busy 绿 @2256ms（排队语义） |
| frame-on-f59-bar-idle-pass.png | 空闲期回执：bar-idle 绿 @5979ms |
| frame-on-f75-page2-replay-pass.png | page2 换代后：replay 条绿（旧事件未重放） |
| frame-on-f76-page2-own-pass.png | page2 own 动画恰一次回执（末帧） |
| frame-off-f56-frozen.png | OFF 冻结帧（条带级联值、动画不推进） |

.bin 原件不入库（2.6MB×帧），sha12 见 analysis-*.txt，可用 committed 工具（observe + analyze 脚本）+ fixture 复现。原始运行日志在 target/dc2-s2c/{on6,off}/browser.log（gitignored），renderer 透传由 `ZW_RENDERER_STDERR_ECHO=1` 开启。

## 局限

- 像素相位差受条带内容发布节奏（内容帧提交 100-1300ms）约束，逐帧 dphase 不能全量≤0.15；同相位结论以稳态 median 0.080 + 提交点即对齐（ost 变化帧 dphase 立即回落）为准。
- 忙窗内条带 ost=1.000 的两种解释（相位边界精确采样 vs 级联回落）未插桩区分；查询正确性的忙窗专项验证留待后续（debug 级 hit= 行可判）。
- 忙窗采样空洞为主线程-Wayland 呈现耦合（S1e 既定缺口），本切片未修，影响忙窗内像素采样密度但不影响事件与查询结论。
- 证据基于 WSLg Weston 环境；原生 Wayland/X11 未测。
