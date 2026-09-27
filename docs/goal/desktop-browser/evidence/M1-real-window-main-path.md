# M1 — Linux 真窗口主链路演示流 + GPU 合成显示验收（记账）

**日期**: 2026-09-27
** goal**: [docs/goal/desktop-browser.md](../../desktop-browser.md)（M1 真窗口主链路验收）
**环境**: Linux x86_64（本机开发环境）；无人值守无 display → `xvfb-run -a`（X11 真窗口，
winit X11 后端）；GPU 面经 wgpu Vulkan adapter（本机 `/dev/dri/card0 + renderD128`，
Xvfb 下 adapter 为软件光栅化，链路与真 GPU 同管道——frame_flow 17/17 与
form-visual-browser-gpu-smoke 先例佐证）。

## 1. 演示流入口（可脚本重放）

```sh
bash scripts/desktop-browser-m1-smoke.sh        # 无 DISPLAY 自动 xvfb-run；有 display 直跑
SKIP_BUILD=1 bash scripts/desktop-browser-m1-smoke.sh
OUT_DIR=… bash scripts/desktop-browser-m1-smoke.sh
```

- 驱动状态机：`apps/browser/src/gui_smoke.rs`（真实窗口事件驱动：导航 → 首个可视
  compositor 帧 → 滚动输入 → 缩放输入（Ctrl +）→ 刷新，逐步截图 + 像素签名断言）
- 本地 fixture：`examples/m1-real-window/index.html`（本地 python3 http.server 随机
  loopback 端口；**file:// 会短路输入交互腿**——gui_smoke 对 file:// 只验加载，故滚动/
  缩放/刷新必须 http）
- 每腿经 `./target/test-guard --time-limit 300` 包裹（run-rules 内存/墙钟门禁）

## 2. 三腿覆盖与判定锚

| 腿 | 配置 | 验收锚 |
|----|------|--------|
| a-cpu-compositor | `--renderer=cpu` + compositor 默认旗标 | 步骤 ×4 passed + 滚动/缩放 visual_change + compositor 事件链（Healthy→frame_submitted→frame_committed→frame_completed→compositor_bitmap_adopted）+ 零 fallback/panic |
| b-gpu-window | `--renderer=gpu`（默认旗标） | 同上 + `GPU renderer initialized`（真窗口 surface，含 wgpu 30 display handle 路径）|
| c-gpu-direct | `--renderer=gpu` + `ZW_COMPOSITOR_SCROLL_TRANSFORM=0` | 同上 + `compositor_dmabuf_adopted`（compositor GPU 纹理导出 → browser 导入，`TabSnapshot.compositor_frame.gpu_direct=true`）|

腿 C 说明：默认产品旗标 `ZW_COMPOSITOR_SCROLL_TRANSFORM=on` 时 compositor 走侧滚动
变换快速路径（bitmap 采纳）；`GPU 导入链`在该旗标关闭时启用（`apps/compositor/src/lib.rs`
导出门：`!scroll_transform && gpu_texture_export && gpu_image && gpu_enabled &&
try_export_headless`）。两形态均为产品真实路径，分腿记账。

### 判定锚日志样例（2026-09-27 实测，HEAD 1ee482191 + 本切片资产）

```
zero_browser::app: GPU renderer initialized (format: Bgra8Unorm)                    # 腿 b/c
zero_browser::compositor_client: SMOKE_EVENT … status=Healthy
zero_browser::compositor_client: SMOKE_EVENT … event=frame_submitted surface=1 …
zero_compositor: SMOKE_EVENT component=zero-compositor event=frame_committed …
zero_browser::compositor_client: SMOKE_EVENT … event=frame_completed …
zero_browser::process_backend: SMOKE_EVENT … event=compositor_bitmap_adopted …      # 腿 a/b
zero_browser::process_backend: SMOKE_EVENT … event=compositor_dmabuf_adopted …      # 腿 c
zero_browser::gui_smoke: GUI_SMOKE_ASSERT action=scroll visual_change=passed changed_samples=43/64
zero_browser::gui_smoke: GUI_SMOKE_ASSERT action=zoom_in visual_change=passed changed_samples=7/64
zero_browser::gui_smoke: GUI_SMOKE_COMPLETE url=http://127.0.0.1:…/index.html steps=load,scroll,zoom_in,reload
```

## 3. 结果（2026-09-27 实测）

- **三腿全绿**（连续两轮复跑通过，含加固后的 fallback 断言）
- 截图证据：`.acceptance/desktop-browser-m1/<腿>/01-loaded.png … 04-reloaded.png`
  （全窗 + page 区双份；`.acceptance/` gitignored，重放脚本即再生）
- 全窗截图核验（人工查看腿 b `01-loaded.png`）：标签栏（fixture 标题 + favicon 位）、
  地址栏（URL）、导航钮、页面内容（section alpha 蓝底 + beta 红底）均真实呈现

## 4. DC-1 判定映射

| DC-1 条目 | 状态 | 证据 |
|-----------|------|------|
| Linux 真窗口端到端演示流绿（启动→加载→渲染→输入交互） | ✅ | 本文档 §2/§3（三腿可重放） |
| GPU 合成显示验收记账（联动父 DC-4.4） | ✅ | 腿 b（GPU 窗口 surface 呈现）+ 腿 c（GPU 导入链 compositor_dmabuf_adopted）；父 DC-4.4 证据归档见本节下方注记 |
| 三平台可编译启动（CI 矩阵佐证） | ✅（接线完成，CI 绿证据待下一次 workflow_dispatch 取证） | 可编译：`.github/workflows/ci.yml` 矩阵（linux x86_64/aarch64、macos x86_64/aarch64、windows x86_64/aarch64，clippy `--all-targets` + nextest 编译覆盖 zero-browser bin）+ `weekly.yml` 11 个 target release 构建 + 打包产物（macOS .app / Windows zip）。**启动冒烟**（2026-09-27 补）：`matrix.launch_smoke` 步接入 `build-and-test` job 的 linux-x86_64 / macos-aarch64 / windows-x86_64 三条目——构建 zero-browser 本体 bin（测试链 lib 不产 bin）后跑 `scripts/browser-launch-smoke.sh`：headless 模式不开窗（runner 无需 display），轮询 `/json/version` 断言 `"Browser":"ZeroWeb/"` + `webSocketDebuggerUrl`、`/json` 断言 page target 枚举；本地 Linux release 目录全链路 PASS（headless boot → 发现端点 → 探活 → 收尾）。CI 侧 `build-and-test` 矩阵三平台全绿（2026-09-26 run 36264551226 实测），新增两步在其中执行；红的是**独立的 benchmarks job**（perf-gate 5 指标超预算，CI 共享 runner 噪声域，与本步骤无关，归因记账见 master.md） |

> 父 DC-4.4 联动注记：compositor dma-buf 链路的进程级 round-trip 由
> `apps/compositor/tests/frame_flow.rs`（`compositor_gpu_dmabuf_browser_import_round_trips`）
> 常驻守成；本 goal 补齐的是**真窗口显示端**消费证据（腿 b/c），两证据合并即父
> DC-4.4 的「GPU 加速合成显示」链路完整闭环。

## 5. 观察挂账（跨流，不在本流处置）

### 5.1 正文空格间歇性塌缩（product 可见，归 rendering-compat 流）

M1 全窗截图可见正文词间空格部分塌缩（如 "M1 Real Window Demo Fixture" 渲染为
"M1 RealWindow DemoFixture"）。**已定域非本流/非 compositor/GPU/Xvfb**：同 fixture 经
in-process 路径（`zero-wpt-runner product-smoke`，零 compositor 零 GPU）逐字节复现。

特征（最小复现见下）：普通 normal 文本、单空格分隔；塌缩位置**与字体族无关**
（sans-serif / Liberation Sans / monospace / serif 同一位置集合）、与字符对无关
（同字母不同位置表现不同）、随词序位置呈块状分布（如 "aa bb … ss tt" 中第 6–11 个
空格保留、其余塌缩）。

最小复现（任一路径渲染即见）：

```html
<!DOCTYPE html><html><head><meta charset="utf-8"><style>p{font-size:16px}</style></head><body>
<p>aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt</p>
</body></html>
```

**归属**：与 rendering-compat R4096-N 已定案的「layout↔paint advance 双源分叉 → 词距
塌缩」（pre/nowrap mono 域暴露，挂账 font-stack 统一专项，user-gated）同族疾病的新
暴露面（普通 normal 文本域、字体族无关）。按 run-rules §9 crate 零重叠纪律，本流
**不触** engine/render 子 crate，仅记档移交；两处证据（R4096-N 四层 probe + 本文
normal 文本最小复现）合并可供该专项收口时定界。

## 6. 复现指引

```sh
# 演示流三腿
bash scripts/desktop-browser-m1-smoke.sh
# 空格塌缩最小复现（in-process，无需窗口）
./target/release/zero-wpt-runner product-smoke examples/m1-real-window/index.html \
    --base-dir examples/m1-real-window --out /tmp/m1.png --width 1024
```
