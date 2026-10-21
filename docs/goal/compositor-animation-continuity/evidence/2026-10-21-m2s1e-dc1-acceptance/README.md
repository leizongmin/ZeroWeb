# M2-S1e：DC-1 生产合成帧验收（真实窗口 + frame_dump 采集）

日期：2026-10-21 ｜ 里程碑：M2-S1e ｜ 判据：DC-1

## 判定

**有条件成立**。DC-1（「页面运行有界长 JS，opacity/transform 持续动画 → 生产合成帧持续更新」）在非忙窗成立；JS 忙窗存在 ~1s 采样空洞，归因为浏览器主线程阻塞在 Wayland 读取（既有呈现路径问题），非采样架构缺陷。证据为生产合成帧采集（compositor swap 后落盘），非 CPU 截图。

## 环境

| 项 | 值 |
|---|---|
| 平台 | Linux (WSL2/WSLg Weston)，窗口 600×1092 |
| fixture | tests/fixtures/compositor-animation/comp01-transform-opacity-under-js-load-autostart.html（load+1.5s 自动 `__zwStartLoad()`；comp01-fade 1s / comp01-spin 1.3s 互质时长） |
| 采集 | frame_dump（compositor swap 后 front buffer 序号命名，commit 48c1d7297） |
| 对照 | flag OFF（ZW_COMPOSITOR_ANIMATIONS=0）vs ON（=1），各 14s |
| 埋点 | ZW_LOG_LEVEL=DEBUG（本批新增 env，默认 INFO 零行为变化）+ tick 心跳/keep-alive/Get 采样诊断行 |

## 实测

### OFF 对照（run-config-off.txt，manifest-off.jsonl）

14s 仅 2 帧：#2（t=0，sha `746b4d0b3b85`，与 M1 冻结基线逐字节同）→ #5（t=2214ms，负载帧：marker 变更 + from 关键帧一次性应用），此后 11.8s 零提交。动画不推进，与 M1 FROZEN 一致。

### ON 主实验（run-config-on.txt，manifest-on.jsonl）

| 指标 | on | on2（复跑） |
|---|---|---|
| 采样帧数 | 256 | 256 |
| 帧哈希全异 | 256/256 | 256/256 |
| dt 中位 / p90 | 13ms / 23ms | 17ms / 24ms |
| 内容帧 frame_id | 2→5→8（3 个） | 同 |
| 红盒（opacity） | 均值色 R:200→232→222 连续扫描（alpha 混白底相位） | 同 |
| 蓝盒（transform） | bbox 连续旋转包络（96²→128²+ 平移外接） | 同 |

frame_id 冻结在 5→8 期间 compositor 独立采样 251 帧（动画覆盖逐帧应用同一内容帧的 clone）——采样架构端到端成立，动画相位由合成器时钟推进。

### 忙窗空洞归因（diag3，~1s，4 次复现确定性）

- 帧空洞：dump 序列 dt=1010ms（on2：1008ms），位于 seq5→seq6（frame_id 5→8 提交点）。
- Get 静默窗互括：最后一次采样 51.303s（`有声明=true now_active=true`）→ 恢复 52.322s，静默 1.019s；空洞 51.319→52.303（985ms）落在其中，窗内 compositor 0 条 Get。
- 窗内 browser 0 条 keep-alive、0 条 worker 发送、0 条 tick 心跳；全程唯一 >0.45s 心跳间断 48.800→52.401（3.600s）覆盖空洞。
- gdb 主线程栈：`wayland_client::conn::blocking_read()`（WSLg Weston 同步往返）。
- 上游诱因：renderer 每次文本 DOM 更新触发全文档重解析（解析 token 爆发与忙片 1:1 锁定），忙窗内 renderer 不提交内容帧。
- 恢复：忙结束帧 #8 提交，dt 立即回到 7-25ms；动画相位不跳（合成器单调钟）。

已排除：空集声明清理（diag 埋点显示声明全程 true）、帧看门狗（10s 未触发）、worker 排空、队列替换语义、compositor 侧故障（零 Get 到达）。

## 结论

1. DC-1 架构主张成立：flag ON 时合成动画由 compositor 采样推进，生产合成帧持续更新（256 帧连续、双通道像素归因）。
2. 忙窗空洞的根因在浏览器主线程与 Wayland 呈现耦合（tick 挂起 → keep-alive 停），不在 S1b-S1d 采样链路；修复候选（呈现/tick 与 Wayland 解耦）记入缺口表，不在本切片修。
3. flag 默认仍关（翻默认需 perf-gate 证据，RFC 既定）。

## 产物

| 文件 | 说明 |
|---|---|
| analysis.txt | off/on/on2 三组帧序列分析（sha256/红盒/蓝盒/marker 行哈希，零依赖脚本生成） |
| manifest-on.jsonl / manifest-off.jsonl | frame_dump manifest（帧序号/surface/epoch/frame_id/尺寸/墙钟） |
| run-config-on.txt / run-config-off.txt | 运行配置（flag/wall/fixture） |
| frame-on-n0000-f2.png | ON 首帧（t=0，与 OFF/M1 冻结基线同源状态） |
| frame-on-n0002-f5-sampled.png | ON 采样早期（f5 + 动画覆盖） |
| frame-on-n0100-mid.png | ON 中段（红盒半透、蓝盒旋转中、`load=done slices=4`） |
| frame-on-n0255-last.png | ON 末帧 |
| frame-off-n0001-f5-frozen.png | OFF 冻结帧（对照：f5 一次性应用后 11.8s 不动） |

.bin 原件不入库（2.6MB×帧），sha256 见 analysis.txt，可用 committed 工具 + fixture 复现。原始运行日志（browser.log 含 renderer/compositor 透传）在 target/dc1-s1e/ 各 run 目录，不入库。

## 局限

- 空洞归因基于 WSLg Weston 环境；原生 Wayland/X11 下主线程阻塞形态可能不同，未测。
- 采样 dt 中位 13-17ms 受 winit tick 节奏（每 Wayland 事件迭代一次闭包，~4ms/tick）与 Get 往返影响，不代表独立刷新率上限。
- 首帧（加载期）走 budget 路径不导出声明 → 发空集（S1b 边界），故加载窗（~2.2s）无采样属预期，两种模式一致。
