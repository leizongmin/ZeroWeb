# Canvas 2D 运行时控制面板

**最后更新**: 2026-08-13（专项立项——入口 goal `docs/goal/canvas-2d.md` v1.0 创建，基于 zero-canvas 实测基线盘点）。

---

## 当前状态

**专项定位**：从 zero-web.md Tier 3「Canvas 2D 完整 API（Path2D、OffscreenCanvas、ImageBitmap）」拆出的独立目标，WPT `html/canvas` 真实用例驱动。

**与兄弟 goal 的边界**：
- rendering-compat（CSS 渲染/字体/布局）— 零工作重叠，canvas 像素输出差异归本专项管理（经显示链路 R3268 到达页面图元后即属 rendering-compat 域，若修复面落在 paint 管线则以 reftest 为准协商归属）
- zero-web 父目标（JS/DOM 桥主线）— 仅 `js_dom_shim` part04/05.js canvas 段共享，run-rules §9 碰头管理

## 实测基线（2026-08-13）

### Rust 层（crates/canvas，~1700 行 + context 子模块 ~4500 行）

| 面 | 状态 | 证据 |
|----|------|------|
| CanvasContext | ✅ ~90 方法：路径（begin/move/line/arc/arcTo/ellipse/quadratic/bezier/rect/roundRect）、fill/stroke/clip（含 Path2D 变体）、文本（fill/stroke/measure）、变换（set/getTransform/translate/rotate/scale/reset/transform）、渐变（linear/radial/conic/pattern）、阴影四属性、合成、像素（get/put/createImageData）、drawImage（3 签名）、isPointInPath/isPointInStroke、save/restore、lineDash | `context/context_impl.rs` |
| Path2D | ✅ 21 方法：moveTo/lineTo/closePath/arc/arcTo/quadratic/bezier/ellipse/rect/roundRect/addPath/from_svg/is_point_in_path/flatten | `path.rs` |
| OffscreenCanvas | ⚠️ **桩**：`offscreen.rs` 仅 new/get_context/width/height，`transfer_to_image_bitmap` 返 ImageData 非 ImageBitmap（docstring 自述"API 桩"） | `context/offscreen.rs` |
| 测试 | ✅ ~737 全绿（master 记录 734→737，R3359 后），覆盖率在核心 crate ≥70% 口径内 | `make test` |
| 依赖 | ✅ 仅 zero-render-foundation / thiserror / bytemuck | `Cargo.toml` |

### JS 接线层（crates/engine/js_dom_shim + js_dom_bridge/canvas.rs）

| 面 | 状态 | 证据 |
|----|------|------|
| getContext('2d') proxy | ✅ DOM 元素 proxy + standalone `_zwMakeCanvas`（R2795/R3077），per-element 缓存 `_zwCanvasCtx`，host 未注册 lenient 回落 | part01.js:97 / part04.js:140 |
| host 派发 | ✅ `__zw_canvas_op(handle, op, ...args)` 串参 → engine `CanvasRegistry`，~40+ op | part05.js:787 |
| 显示链路 | ✅ R3268：ctx id 写入 `data-zw-canvas-ctx`，painter 桥接 canvas 像素为页面图元；wpt-runner reftest 同链路 | part04.js:159 / reftest.rs:597 |
| toDataURL / toBlob | ✅ R2797 / R3296（PNG 导出 + Blob） | part05.js:813/827 |
| Path2D | ✅ R3306/R3307：new()/new(other)/new(svgString)，host createPath 走 `Path2D::from_svg` | part05.js:871-908 |
| ImageBitmap / createImageBitmap | ✅ 基础 R3309/R3311：Blob/ImageBitmap/ImageData 源 + 子矩形裁剪 + 参数校验（0 尺寸 RangeError/InvalidStateError） | part05.js:910-1040 |
| OffscreenCanvas | ✅ JS 侧 R3312/R3313：transferControlToOffscreen（复用 host handle）+ transferToImageBitmap | part04.js:204 / part05.js:1087 |

### WPT 面

| 面 | 状态 | 证据 |
|----|------|------|
| 内建用例 | ✅ 40 个 canvas smoke 用例（分类 "canvas"），R3079 断言全部 js_executes_ok | `test_cases_canvas.rs` |
| 上游真实导入 | ❌ **零**——无 `html/canvas` 真实用例导入，无通过率基线 | — |
| 像素 oracle | ❌ 无 canvas 像素对比集；Oracle 工具链存在但当前环境无 Chromium（R3344 记录） | — |

## 缺口清单（按建议处理顺序）

| # | 缺口 | 证据 | 建议 |
|---|------|------|------|
| G1 | WPT `html/canvas` 真实用例覆盖为零 | 40 内建 smoke 非上游导入 | M1 切片 1（纯资产，零源码） |
| G2 | 像素级 canvas 验证缺失 | reftest.rs:597 有链路无对比集 | M1 切片 3（复用 Oracle 工具链） |
| G3 | OffscreenCanvas Rust 桩 | `offscreen.rs` docstring 自述桩 | M2，轻量可先行 |
| G4 | createImageBitmap options defer（imageOrientation/premultiplyAlpha） | part05.js:916 注释 | M2 |
| G5 | ImageBitmap 源类型受限（HTMLImageElement 等） | part05.js:929 仅 Blob/ImageBitmap/ImageData | M2，深结构待点名 |
| G6 | OffscreenCanvas × Web Worker 未集成 | worker.rs 无 OffscreenCanvas 路径 | **深结构，待用户点名** |
| G7 | API 语义细节（异常路径/属性反射/边界） | 待 WPT 驱动发现 | M1 切片 2 聚类后逐项修 |

## 待用户决策清单

- 格式：`- [ ] <事项> — 为何需用户（深结构 / 许可证 / 破坏性操作 / 改 Mission / 超大下载）— 建议 — 追加时间`
- [ ] OffscreenCanvas × Web Worker 集成（G6）— 深结构（worker 线程上下文 + 消息传递语义），需独立设计 — 等点名 — 2026-08-13
- [ ] ImageBitmap 全源类型（G5 扩展至 HTMLImageElement/HTMLVideoElement/VideoFrame）— 跨面深改（依赖 img 解码/视频链路）— 等点名 — 2026-08-13

## 下一步计划

1. **M1 切片 1（零源码）**：上游 `html/canvas` 真实用例导入 `tests/wpt-runner`，跑 testharness 执行，记录分类通过率基线（当前无基线）
2. **M1 切片 2**：失败聚类 → 首个轻量修复队列（预期集中在 G7 语义细节）
3. **M1 切片 3**：canvas 像素对比 harness（复用 reftest canvas 链路 + Oracle 工具链，环境可用后）
4. **M2**：G3（Rust 桩真实化）可零碰撞先行；G4/G5 随 WPT 驱动

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/ crates/engine/src/js_dom_bridge/canvas.rs` 核对 html-compat 流活跃面；活跃则本流只做 canvas crate 本体 / WPT 资产 / Rust 侧。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT canvas 基线建立 | ⏳ 未开工（切片 1 为首个动作） |
| M2 — API 语义补齐 | ⏳ 未开工 |
| M3 — 像素正确性冲刺 | ⏳ 未开工 |

## 验证基线

- 测试基线：canvas crate ~737 全绿（含 R3354-R3359 溢出/命中测试加固家族）；workspace `make test` 全绿（既有 real-HTTP 并发时序失败为已知跨流限制）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` + `make test`
- 资产化：修复必须 `make import-wpt TEST=html/canvas/...` 常驻断言集并记入 `imported-tests.txt`
