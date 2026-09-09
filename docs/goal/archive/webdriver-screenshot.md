# WebDriver Screenshot — 公共截图转换层 crate + GET /session/{id}/screenshot

**版本**: v1.0
**日期**: 2026-09-09
**状态**: ✅ Completed（2026-09-09 归档至 [archive/webdriver-screenshot/](archive/webdriver-screenshot/)；M1+M2+M3 全完成，DC-1~4 逐项判定全满足——公共转换层 crate `zero-paint-convert` 落地且 compositor/browser 双端收敛（重复 `ipc_*_to_*` 函数族零残留）、GET /session/{id}/screenshot 全链路（ViewPainted 双路消费 + 跨帧 ImageCache + PNG base64 + W3C 错误路径）+ 全链路集成测试（PNG 魔数/800×600 尺寸/像素采样）、验证通道文档落账；门禁 `make test` 各阶段全绿（唯一 FAIL 为多轮在案 etag 负载敏感 flake，隔离复跑 781/0）+ 全 workspace clippy `-D warnings` 零告警 + fmt 无 diff，终态见 archive master.md）
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/archive/webdriver.md`（screenshot「待用户决策」项，方案①拍板 2026-09-09）

> **说明**
> 本文档是 ZeroWeb「WebDriver Screenshot」专项目标执行契约。已归档的 webdriver goal
> 在 M3 摸底结论中明确：GET /session/{id}/screenshot 技术路径存在，但 `IPC →
> RenderPrimitives` 转换层在 `apps/browser/src/paint_ipc.rs`（browser 域），webdriver
> 复用需跨域依赖或抽库——当时记「待用户决策」。**2026-09-09 用户拍板方案①：抽公共
> 截图转换层 crate**。本文定义 Mission、边界、Done Criteria、执行协议和文档治理规则，
> 供后续 `rally run` 会话作为稳定输入。日常进展、evidence、active milestone 更新写入
> `master.md`。
>
> **▶ 拆分动机（2026-09-09 用户决策——screenshot 征询三方案中的方案①）**：从已归档
> webdriver goal 的挂账项起新 goal。理由：① `PaintSnapshotParams → RenderPrimitives`
> 转换逻辑目前**双份维护**（browser `paint_ipc.rs` 与 compositor `convert.rs` 同逻辑
> 两份拷贝）——实证：rendering-compat 流 R4139（`552ab2701`）与 R4059（`8cfabf4f4`）
> 两笔渲染修复均被迫同批改两处，收敛为单一公共层消除长期维护税；② compositor 已有
> 完整可复用实现（`convert.rs::to_render_primitives` 404 行 + `rasterize.rs` CPU 光栅
> 化），但 compositor 是纯 binary crate（模块全私有 `mod`），抽取是搬家而非重写；③
> 落地后 webdriver GET /screenshot 顺理成章（snapshot → primitives → render_full_scene
> → PNG，headless.rs R1601 已有验证路径），为兄弟 goal（rendering-compat 视觉回归、
> android-browser 截图通道）提供能力。
>
> **▶ 基线事实（2026-09-09 实测）**：
> - **转换层双份拷贝**：`apps/compositor/src/convert.rs`（404 行，
>   `to_render_primitives(&PaintSnapshotParams) -> RenderPrimitives`，模块私有）与
>   `apps/browser/src/paint_ipc.rs`（597 行，`apply_paint_snapshot(&mut TabSnapshot,
>   PaintSnapshotParams)` + `ipc_*_to_*` 函数族）——同逻辑（IpcRect/Color/Gradient/
>   Glyph/LineCap/LineStyle/Filter/BlendMode/DrawOp 映射）双实现。
> - **光栅化已有**：`apps/compositor/src/rasterize.rs`（160 行，
>   `rasterize_into_back(...)` → FrameBuffer，走 `render_full_scene*`）；browser
>   headless 路径用 `zero_render_foundation::cpu::render_full_scene` +
>   `framebuffer_to_png_base64`（R1600/R1601，`headless.rs:43`）。
> - **lib+bin 先例**：`apps/image-decoder` 同 crate 双 target（`[lib]` +
>   `[[bin]]`），抽取形态有据可依。
> - **renderer 发布通道**：webdriver spawn renderer 后**已**发
>   `SetFramePublishMode(Legacy)`（`session.rs:118`）→ renderer 每帧发
>   `ViewPainted(Box<PaintSnapshotParams>)`；`RendererHandle` 已有
>   `recv`/`try_recv`/`poll` 消息面（`crates/protocol/src/process.rs`）。**但 webdriver
>   目前丢弃未消费的 ViewPainted**（session.rs 零 ViewPainted 引用）。
> - **图像 payload 去重约束**：renderer `fetch_image_payloads_with_cache` 带
>   `sent_keys` 去重（S8 优化）——像素只发一次，webdriver 侧必须**跨帧累积**
>   ImageCache，否则二次导航后图片缺失。
> - **依赖现状**：`png = "0.18"` 已在 workspace deps（browser 用）；`base64 = "0.22"`
>   仅 browser 直依赖（同版本引入新 crate 即可，不加新第三方依赖）。
> - **撞车面核对（2026-09-09）**：`apps/compositor/` 14 天内有渲染流提交（R4139 等
>   但均为图元语义修复、非 crate 结构改动）；`apps/browser/src/paint_ipc.rs` 同批被
>   渲染流改过——**M1 抽取切片开工前须按 run-rules §9 再核对**，发现撞头即记
>   master.md 暂停该切片。

---

## Mission

以方案①（用户拍板 2026-09-09）为纲：**抽出公共截图转换层 crate**（`PaintSnapshotParams
→ RenderPrimitives` 转换 + CPU 光栅化 + PNG 编码），收敛双份维护，并基于它落地
webdriver **GET /session/{id}/screenshot** endpoint（W3C Screen Capture），使
zero-webdriver 获得像素级端到端验证能力。

阶段校准：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **crate 抽取** | 新建公共 crate，compositor 与 browser 双端切换消费，行为零变化 |
| 第二阶段 | **screenshot endpoint** | webdriver 消费 ViewPainted + 累积 ImageCache + PNG 输出 + 全链路测试 |
| 第三阶段 | **接线收尾** | 验证通道文档更新（兄弟 goal 如何用截图做视觉回归） |

**关键约束**：抽取切片必须**行为零变化**（转换结果逐字节一致——用既有测试作对照，不
新写语义）；不引入新第三方依赖（png/base64 复用既有 workspace 版本）；screenshot
endpoint 以 W3C Screen Capture 语义为准（base64 PNG、全文档还是视口按规范最小面）。

覆盖范围：

1. **公共 crate** — `PaintSnapshotParams → RenderPrimitives` 转换 + 光栅化辅助 + PNG
   编码（新 crate，落 `crates/` 或 `apps/` 下按其消费形态定；compositor/browser/webdriver
   三端消费）
2. **双端收敛** — compositor `convert.rs`、browser `paint_ipc.rs` 切换为消费公共层，
   删除重复实现
3. **screenshot endpoint** — webdriver 消费 ViewPainted（Legacy 模式已在发）、跨帧
   ImageCache 累积、GET /session/{id}/screenshot 全链路测试
4. **接线文档** — 兄弟 goal 视觉回归使用说明（evidence/）

不在范围内（明确排除）：

- **element screenshot**（元素裁剪）— 依赖页面截图先行，页面级落地后评估，不阻塞 DC
- **GPU 光栅化路径下沉公共层** — compositor `gpu_raster.rs` 维持原状，公共层先收敛
  CPU 路径
- **CDP captureScreenshot / beyond-wire 语义** — headless 已有实现不动
- **screenshot 性能优化**（增量编码、脏区截图）— 先正确后快

### Support Envelope

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| 新 crate | 转换 + 光栅化 + PNG 的公共实现 | 照 image-decoder lib+bin 先例；名称立项时定（建议 `zero-paint-convert`） |
| compositor | `convert.rs`/`rasterize.rs` 改为薄封装或直呼公共层 | 行为零变化，测试面平移 |
| browser | `paint_ipc.rs` 转换函数族切换消费 | `apply_paint_snapshot` 对外签名不变 |
| webdriver | session.rs 消费 ViewPainted + 新 endpoint | 照既有 endpoint 切片模式 |
| protocol | 原则上零改动（ViewPainted/payload 已有） | 若确需新消息，先按 run-rules §9 核对 |

依赖约束（run-rules §9 碰撞管理）：

- **rendering-compat 流**：`apps/compositor/`、`apps/browser/src/paint_ipc.rs` 近期有
  该流的图元语义提交（R4139/R4059 同批双改两份拷贝）——**每轮开工前 `git log
  --since="7 days ago"` 核对两处**，有活跃编辑先做零碰撞面（webdriver HTTP 层、新
  crate 骨架、测试）；抽取切换提交尽量小（一次一个消费者），撞头即暂停记 master.md。
- **event-loop-spec 流**：`apps/renderer/` runtime/lib 属该流活跃域。本流**只读**已有
  ViewPainted 通道，**不改 renderer**；发现必须改 renderer 即暂停记「待用户决策」。
- **protocol 共享面**：原则上零改动；确需动 Automation 消息族时照 webdriver goal 旧例
  核对 14 天活跃。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: 公共转换层 crate

- [ ] 新 crate 落地，包含 `PaintSnapshotParams → RenderPrimitives` 转换 + CPU 光栅化 +
      PNG 编码三能力，公共 API 有 `///` 文档注释
- [ ] compositor 全部消费点切换（`convert.rs`/`rasterize.rs` 删除或变薄封装），既有
      测试平移后全绿
- [ ] browser `paint_ipc.rs` 切换消费公共转换（`apply_paint_snapshot` 对外签名不变），
      既有测试全绿
- [ ] 双份实现删除（grep 确认无 `ipc_rect_to_rect` 等重复函数残留）

### DC-2: screenshot endpoint

- [ ] GET /session/{id}/screenshot 落地，返回 W3C 语义 base64 PNG
- [ ] webdriver 侧消费 ViewPainted + ImageCache 跨帧累积（sent_keys 去重语义正确处理）
- [ ] wire format + 错误路径测试（无快照时行为对齐 W3C——unable to capture 或等价）

### DC-3: 每切片全链路测试

- [ ] screenshot 有 HTTP 全链路集成测试（真实 TCP + 真实 renderer 子进程，照
      http_session.rs 模式），断言 PNG 可解码 + 尺寸正确
- [ ] 抽取切片有行为对照测试（转换结果抽取前后一致——复用既有测试即视为满足，须在
      master.md 记明哪些测试构成对照）

### DC-4: 测试与质量不可退让

- [ ] `make test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] `cargo fmt --all -- --check` 无 diff
- [ ] workspace 成员变更同步文档（README 架构图、goal master.md、CHANGELOG 如适用）

---

## 活跃里程碑

### M1 — 公共 crate 抽取 + 双端收敛

**目标**：新 crate 落地，compositor + browser 双端切换，重复实现删除。

**切片建议**：
1. 新 crate 骨架 + 转换逻辑搬家（以 compositor `convert.rs` 为准——更接近纯函数形态）
   + compositor 切换 + 测试平移
2. browser `paint_ipc.rs` 切换 + 双份删除
3. workspace/Cargo.toml/文档登记同步

### M2 — screenshot endpoint

**目标**：webdriver 消费 ViewPainted、GET /session/{id}/screenshot + 全链路测试。

### M3 — 接线收尾

**目标**：验证通道文档更新（截图做视觉回归的使用说明）→ DC 全满足判定。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~4 全部满足；`make test` + clippy 全通过；master.md 内部自洽。
element screenshot 按「待用户决策/后续评估」明确记录不算未满足 DC。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**公共层 API 形态（以最小改动让三端消费为准，不预先设计大而全接口）
2. **自主补齐**切片，实现 + 测试同步交付；抽取切片严守行为零变化
3. **自主验证**：`make test` + clippy + 全链路测试确认行为正确
4. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量切片**：一个消费者一个切片（compositor → browser → webdriver），改动
   面小、可独立验证。
2. **永不停**：遇需拍板事项（如确需改 protocol/renderer）记「待用户决策」清单并跳过，
   继续下一个切片。
3. **碰撞管理**：每轮开工前 `git log` 核对 compositor/paint_ipc/renderer 三处活跃面；
   有活跃编辑先做零碰撞面。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。当作当前任务的一部分修复，直到稳定可重复。
2. **抽取发现行为分歧**（两份拷贝逻辑不一致处）：以 compositor 版本为准搬家（它是
   compositor 主链路在用的真值），分歧点记录到 master.md 关键决策；若分歧影响 browser
   行为，browser 侧切换后其测试自然暴露——此时停该切片，记「待用户决策」。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。
  **修改条件**：仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/webdriver-screenshot/master.md`：当前真实状态的唯一
  控制面板。治理规则：持续演进、不允许无限增长（过时内容压缩或归档）、各章节必须自洽。
- **归档区域** `docs/goal/webdriver-screenshot/archive/`：只追加不修改。
- **证据区域** `docs/goal/webdriver-screenshot/evidence/`：对照测试清单、兼容性注记，
  持续追加。
