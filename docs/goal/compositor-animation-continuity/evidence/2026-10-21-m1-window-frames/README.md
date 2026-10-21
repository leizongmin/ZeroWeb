# M1 真实窗口帧序列观察——合成器提交节奏实测（2026-10-21，接续 run-20261021b）

**Run ID**：2026-10-21-m1-window-frames（同日第二次窗口观察；首轮 INCONCLUSIVE 的补测）
**判定**：真实窗口（compositor 进程）路径下动画 **FROZEN（不推进）**，内容归因已证实——首轮 INCONCLUSIVE 的两个缺口（内容归因、多帧采样工具）均已闭合。M1 真实窗口基线至此完成。

## 环境与隔离

| 项 | 值 |
|---|---|
| 构建 | 本轮新增 frame_dump 工具后的 release 构建（`make browser-build`，MemoryMax=24G scope） |
| 启动 | `zero-browser http://127.0.0.1:<port>/comp01-...html`（CLI 启动 URL 位置参数）+ `ZW_COMPOSITOR_BIN` 指向 release zero-compositor |
| 会话隔离 | `XDG_CONFIG_HOME=<run>/profile`（一次性配置目录）——首轮的会话恢复混淆（nodejs.org 噪声、恢复页归因未证）本轮不复现：恢复的是空配置 |
| 采样工具 | `ZW_COMPOSITOR_FRAME_DUMP_DIR=<run>/frames ZW_COMPOSITOR_FRAME_DUMP_MAX=64`——compositor 在 `backing.swap()` 后把 front buffer 落盘（本轮新增 `apps/compositor/src/frame_dump.rs`，env 门控默认关闭） |
| 墙钟上界 | `timeout -s TERM 25` 包裹浏览器；HTTP 服务（127.0.0.1 python3 http.server）随脚本退出清理 |
| 内存上界 | 整个观察跑在 systemd-run scope（MemoryMax=16G）内 |

## 实测结果（manifest.jsonl + analysis.txt + browser.log）

25 秒观察窗内 compositor 对 surface 1 仅提交 **2 帧**（epoch=2，600×1092）：

| 帧 | 时刻（UTC） | fills | sha256 前 12 | 像素归因 |
|---|---|---|---|---|
| #2 | 02:45:55.007 | 3 | `746b4d0b3b85` | red=16384px（=128×128 `#opacity-box` 满格→opacity=1）、blue=9216px（=96×96 `#transform-box` 满格→初始 transform） |
| #3 | 02:45:57.196 | 3 | `746b4d0b3b85` | 与 #2 **逐字节相同**（wall_ms 差 2189ms） |

此后至 25s TERM 终止无任何提交。frame-f2.png（由 .bin 经零依赖 PNG 编码转换）目视确认为 COMP-01 fixture 的 t=0 状态：红盒全不透明、蓝盒初始位、`load=idle` 标记。

## 结论

1. **内容归因证实（闭合首轮缺口）**：红/蓝盒像素数与 fixture 几何精确相等（128²/96²），PNG 目视三重印证——surface 1 显示的就是 fixture，非恢复页。
2. **动画 t=0 冻结在真实窗口复现**：两帧间隔 2.19s 像素逐字节同，动画属性全停在 keyframe `from`；与 headless 基线（evidence/2026-10-21-m1-comp01-baseline）及静态代码结论（`render_html` R3882 静态 t=0、时钟泵无调用方）三方一致。
3. **提交节奏即帧生产节奏**：加载收敛后（帧 #3 后 ~22s）合成器零提交——renderer 事件驱动、无时钟泵，合成器无动画状态也不会自行产帧。「合成动画连续性」的缺口在两端：renderer 不推进 t，compositor 不重采样（M2 切片的设计输入；`GetCompositorFrame` 的 scroll 重光栅是既有的 compositor 取数时重光栅先例）。
4. **采样工具可用**：frame_dump 落盘 2/64 帧（未触上限），manifest 时间戳与日志帧序一致，工具达标并留作 M2 观察复用。

## 产物

| 文件 | 说明 |
|---|---|
| frame-f2.png | 帧 #2 的 PNG 转换（目视归因；.bin 原件 2.6MB×2 不入库，sha256 见 analysis.txt，可用 committed 工具+fixture 复现） |
| manifest.jsonl | compositor 落盘清单（surface/epoch/frame_id/尺寸/wall_ms） |
| browser.log | 浏览器+renderer+compositor 合并日志（RUST_LOG=info） |
| analysis.txt | sha256 + 像素统计 + 逐字节比较 |
| run-observe.sh | 观察脚本（隔离 XDG、env 门控、双超时上界） |

## 局限

- 未在窗口模式下触发 `__zwStartLoad()` 负载（窗口模式无 automation 通道；负载通道证据以 headless 基线为准）。DC-1 完整验收（含负载时序）仍待 M2 切片。
- 帧 #1（epoch 1，启动 newtab）未落盘（dump 从 epoch 2 帧 #2 开始采集，时序上 dump 启用先于首帧提交，帧 #1 未达 compositor 或属启动期未提交），不影响判定。
- 本轮产物未经 `lei-pre-commit-guard` 之外的用户复核；.bin 原件保留在 `target/`（gitignore）至会话结束。
