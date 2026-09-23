# M4 — Done Criteria 逐项判定 + 平台差异挂账定稿

**日期**: 2026-09-24
**判定范围**: docs/goal/web-api-batch2.md DC-1~DC-4（对应里程碑 M1~M4 全部执行完毕）

## DC 逐项判定

### DC-1: WPT 导入与基线 — ✅ 满足

- [x] 上游 `clipboard-apis/` + `fullscreen/` window 可执行面子集导入：fetch 脚本 ×2
  （pin 315976933870）+ runner 双子命令 + Makefile 四目标（test-guard 包裹）+
  `imported-testharness.txt` +88 行（M1，930add225 / b0ac7bf92）。
- [x] 分类通过率基线（文本 + JSON）落 evidence/：2026-09-23-m1-*-baseline.{md,json} ×2
  （clipboard 21.1% / fullscreen 63.0%）。

### DC-2: Clipboard 语义收敛 — ✅ 满足

- [x] navigator.clipboard 四方法 + ClipboardEvent：clipboard-apis 21.1%→**83.6%**
  （61/73，全绿案 25/33）——M2-s1 富 MIME 面（ClipboardItem/Clipboard/代际失效）→
  M2-s2 execCommand copy 桥 + MIME 映射 + 时限预算 → M2-s3 denied 拒绝 + read(options)
  校验 + write 图片魔数校验 + DataTransfer 'Files'。逐簇修齐三轮全部带 evidence 记档。
- [x] 剪贴板后端：headless 进程内内存后端可用且为全 corpus 验收路径；**平台后端差异
  如实记账**（见下「平台能力评估」）。

### DC-3: Fullscreen 语义收敛 — ✅ 满足

- [x] requestFullscreen/exitFullscreen（含 options）/fullscreenElement/fullscreenEnabled +
  fullscreenchange/error 事件：fullscreen 63.0%（立项起点）→ **88.0%**（132/150，全绿案
  37/55）——M3-s1 异步状态机（渲染机会 steps + ready check + 激活门/消费 + PermissionStatus
  + FullscreenOptions WebIDL 校验）→ M3-s2 节点移除联动。三轮 pass set 零丢失。
- [x] 全屏状态 viewport 语义联动验证：corpus 可观察面三案全绿（document-exit-fullscreen-
  timing 的 resize→change 事件序、element-request-fullscreen-timing 的 rAF 时序、
  screen-size 的 screen 尺寸不变断言）；真窗口 OS 级全屏差异记账（见下）。

### DC-4: 测试与质量不可退让 — ✅ 满足

- [x] `make test` 全绿（68/68 段零失败，test-guard 包裹）+ `cargo clippy --workspace
  --all-targets -- -D warnings` 全过 + `cargo fmt --all -- --check` 干净 + `cargo build
  --workspace` 通过（2026-09-24 本轮实测）。
- [x] 每语义切片带单测：WAB2-M2（rich MIME）/ WAB2-M2-s2（copy 桥）/ WAB2-M2-s3（denied
  + 校验）/ R2938（重写至新语义）/ WAB2-M3-s1（激活消费/权限豁免/双 exit/options/
  PermissionStatus）/ WAB2-M3-s2（removal 三形态）+ R2964/R2948/R2817 邻接保持。
- [x] `make reftest` 零回归：2026-09-24 实测 **failed 0**（真通过 533 + 近似 48，exit 0）。
- [x] `make bench-gate`（定向 zero-engine）：GATE PASS（26 指标全预算内；首调 3 项 paint
  超预算经复测归因兄弟流满载并发污染——R4702 同款模式，空载复测全绿，未改测量配置）。

## 平台能力评估与差异记账（挂账定稿）

1. **平台剪贴板后端**：OS 剪贴板能力已存在——workspace 依赖 `arboard = "3"`，接线点
   `apps/browser/src/clipboard.rs`（浏览器壳层）。缺口 = webview→engine 桥（navigator.clipboard
   内存 store ↔ arboard 读写互通）。**记账**：本 goal 以 headless 内存后端收口；桥接归
   webview/host-runtime 后续 goal（重入条件：桌面剪贴板互通需求点名）。
2. **真窗口 OS 级全屏**：winit `Fullscreen::Borderless` 已在 `crates/host-runtime/src/
   window.rs` 窗口创建层支持（`with_fullscreen` 配置）。缺口 = 运行时桥（engine fullscreen
   状态机 grant 路径 → host `window.set_fullscreen` 切换 + viewport resize 回灌）。**记账**：
   本 goal 语义面（状态机/事件/激活/权限）已完备可挂接；真窗口切换归 host-runtime 流域
   （重入条件：视频/演示场景真全屏需求点名）。

## 终态口径

| corpus | 立项基线 | 终态 | 全绿案 |
|---|---|---|---|
| clipboard-apis | 21.1%（M1 修复后口径） | **83.6%**（61/73） | 25/33 |
| fullscreen | 63.0% | **88.0%**（132/150） | 37/55 |

余留非绿项全部甄别归属（无未定位缺口）：rendering-compat 流域（`:fullscreen` 伪类/
display:contents UA 面）、js-dom 面（SVGElement instanceof、shadowRoot.getElementById）、
runner infra（window.event 持久性、`/common/` fetch、basics 探针停滞、isTrusted）、
iframe 文档管道（cross-origin/navigate/allowfullscreen）、clipboard tentative 面
（custom formats ×6、svg 净化、内容规范化）、栈模型（remove-last）。DnD 按 goal 契约
排除挂账（重入条件见入口文档）。
