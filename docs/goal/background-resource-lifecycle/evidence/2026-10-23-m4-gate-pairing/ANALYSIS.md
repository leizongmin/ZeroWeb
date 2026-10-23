# M4 资源门禁阈值配对——证据分析（2026-10-23）

## 目标与产出

M4 收尾两项：资源门禁阈值多轮配对（DC-5）与 DC-2 音视频「合法工作不误停」补证。

产出：
- `scripts/background-resource-gate.py`：腿指标提取（report）/ 阈值导出（derive）/ 门禁检查（check）三模式。
- `docs/perf/baselines/background-resource-linux-x86_64.json`：7 场景阈值基线 v1（全部 n≥2，无 pairing_insufficient）。
- 本目录 8 条新鲜腿（6 场景 fresh + bg02-media ×2）+ 媒体缺口证据腿 `bg02-media-autoplay`。
- `tests/fixtures/background-resource/bg02-media.html`（媒体腿 fixture，显式 play 版）。
- `scripts/background-resource-observe.sh`：bg02-media 场景臂 + fixture server `--media-dir`（共享 `tests/fixtures/media`，不复制二进制）。
- `scripts/background-resource-probe.mjs`：bg02-media 采样臂（`__zw_media_ticks` 计数 + 终态/终末桥探测）。

## 阈值配对方法

- 协议腿形态：`background-resource-observe.sh <scenario> <out-dir> [wall]`——run-config.txt（场景/env/墙钟）+ samples.jsonl（250ms 进程树采样）+ probe.json（CDP 场景驱动）。
- 指标：renderer/browser/compositor 主实例（首现 pid）CPU cores（jiffies 差分/user_hz/窗口时长）、RSS last/hwm；probe 计数差值（rAF 泵/迭代/媒体存活 ticks）；nav_ok_ratio、stall_closed_ratio（probe.json nav_series 末次服务端累计快照）、close_error_count。
- 阈值带：CPU 上界 = 配对腿 max×1.5；RSS 上界 = max+64 MiB；工作下界（probe 计数）= min×0.5；close_error_count 固定 max=0。
- env 一致性：check 比对 run-config 的 ZW_*+gui 快照（display 除外）；run-config schema v1（M1/dc1 时代）未记录 `ZW_COMPOSITOR_ANIMATIONS`/`gui`（当时运行时默认即 0），gate 对缺失键回填默认值归一化。env 不一致不短路指标判定——错版与指标突破并列输出。

## 配对清单与范围决策

| 场景 | n | 配对腿（均修复后） |
|---|---|---|
| bg01-foreground | 2 | dc1-nested-clamp/bg01-nested-after + fresh |
| bg01-background | 2 | m3-dc2-visibility/bg01-background + fresh |
| bg02-anim | 3 | m3-dc2-raf-pump + m3-dc2-visibility + fresh |
| bg02-static | 3 | m3-dc2-raf-pump + m3-dc2-visibility + fresh |
| bg02-media | 2 | 本目录 bg02-media + bg02-media-b |
| bg03-nav | 4 | m2-s1-recheck + m2-s2-plain + m2-s3-residue + fresh（均 45s） |
| bg04-cycle | 2 | m1-baseline/bg04-cycle + fresh（静态页 close/reopen，未受后续修复影响） |

不入 v1：M1 负例腿（bg01-foreground/bg02-anim/bg03-nav 的 m1-baseline——修复前数据，仅作 check 负例）；GUI/compositor 腿（bg02-anim-compgui、bg02-onscreen-compgui、bg02-anim-gui、bg02-static-gui、bg03-nav-gui、bg04-close-gui——display 环境不一，present-chain 轮 display=none，one-off 诊断协议未固化）；bg01-interval（墙钟 20s/120s 不一致）；bg02-rafstub、bg01-timer、bg02-anim-xdo 等 one-off 场景标签。

## 门禁验证

正向（8 条 fresh 腿）：全部 PASS（exit 0）。

负向（M1 修复前腿，各自命中预期指标，exit 1）：
- `m1-baseline/bg01-foreground`：`renderer_cpu_cores=1.4628 > max 0.0814`——嵌套 timer 无钳制（DC-1 修复前）。
- `m1-baseline/bg02-anim`：`probe_delta:bg02-anim=0.0 < min 620.5`——rAF 泵死（M3 修复前）。
- `m1-baseline/bg03-nav`：`stall_closed_ratio=0.0 < min 0.4688`——fetch 无取消（M2-S1 修复前）。

门禁抓历史回归三类（CPU 上界/工作下界/连接清理下界）各验证一次成立。

## DC-2 媒体维度：播放时钟缺口（上游，未在本 goal 修）

`bg02-media` 腿协议：audio（loop mp3）+ offscreen video（loop webm/H.264），load 后显式 `play()`，采样 500ms `__zw_media_ticks`（!paused 计数）与 `__zw_media_pos`（currentTime 高水位），终态直读元素态 + 桥真值。

三条腿实测：
1. `bg02-media-autoplay`（autoplay 属性版 fixture）：a/v 元素 paused=false、ticks 正常推进，但 currentTime 恒 0.00、pos=0.00——播放态活着、时钟从未走。
2. `bg02-media`（显式 play v2）：video 全链路通（元素 2.04 = 桥 2.043，泵推进）；audio 元素 0.00、桥 currentTime 0、桥 isPlaying false。
3. 终末桥探测（t+25s 直接 `__zwVideoBridge.play(a.src,0)`）：返回 true，1.5s 后桥 cursor 1.999 推进；再 +2.5s 冻在 1.999（loop 未置位）——**原生音频泵/解码/时钟全链路健康**。

根因定位（三处独立缺口，均 engine/media-playback 面资源）：
1. **autoplay 路径绕桥**：`_zwMediaAutoplay`（`crates/engine/src/js_dom_shim/part06.js:9959`）直接置 `ms.playing` 发 play/playing/timeupdate，不经 `__zwVideoBridge.play` 武装原生 registry——无原生播放，时钟无源。
2. **非桥墙钟推进钩子死代码**：`_zwMediaTimeMarchesOn`（`part06.js:16225`，M3 扩批 XXVII headless 时钟 + 桥真值拉取）全仓零调用方（仅定义 + `part01b.js:727` 注释提及）——即使 autoplay 置了 playing 态也无泵推进 `ms.currentTime`。
3. **显式 play() 的 load 时桥武装失败（audio 特有）**：play 桥武装路径（`part03.js:23145` 起）video 命中（bridgeOn 置位 + setLoop + getter 镜像走桥 `part04.js:445`）；audio 未命中——桥 play 返回 true 但 loop_on 未置位（终末探测跨流末不回卷）证明 load 时未走到命中路径的 setLoop（`part03.js:23246`）。失败点在 `_pSrcRaw`→`_pAbs` 解析或重试耗尽之间，未再深挖。

判定：
- video 维度 DC-2 通路成立（显式 play 后原生泵推进、后台场景不误停的面在 M3 visibility 切片已盖）。
- audio + autoplay 维度的「不误停」**当前不可测**——播放时钟本身不走（上游缺口），先于后台节流问题。
- 处置：归 media-playback goal 面修复（autoplay 镜像 play 桥武装 ≈60 行或重构加载序列，有 WPT 事件时序回归风险，不属本 goal 资源生命周期范畴）。bg02-media 场景的资源阈值配对不受影响（CPU/RSS/ticks 存活计数均与播放时钟解耦）；媒体时钟下界待上游修复后作为 v2 追加。

## 质量门禁说明

本轮改动面：python/sh/mjs 脚本 + fixture html + 基线 JSON + 文档，无 Rust 代码——未跑 make test（cargo 面无变化）；py_compile / sh -n / node --check 全过；门禁脚本逻辑由正负 11 腿批量验证覆盖。无人值守腿全部包 systemd-run MemoryMax=8G scope。
