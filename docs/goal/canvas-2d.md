# Canvas 2D 兼容性 — WPT 驱动的 Canvas API 正确性目标

**版本**: v1.0
**日期**: 2026-08-13
**状态**: Active
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（ZeroWeb 总体目标）

> **说明**
> 本文档是 ZeroWeb Canvas 2D 兼容性的专项目标执行契约。目标是以 WPT `html/canvas` 真实用例通过率为验证标准，将 ZeroWeb 的 Canvas 2D API 行为与像素输出对齐到 Chromium 水平。本文定义了使命、边界、完成标准、执行协议和文档治理规则，供后续 `rally run` 会话作为稳定输入使用。
>
> **拆分动机（2026-08-13 用户决策）**：zero-web 父目标与 rendering-compat 两个并行 goal 之外，用户要求再拆一个独立专项。经筛选从 zero-web.md Tier 3「Canvas 2D 完整 API（Path2D、OffscreenCanvas、ImageBitmap）」立项——理由：① zero-canvas 单 crate 边界清晰，与 CSS 渲染（rendering-compat 域）零工作重叠；② 有独立验收面（WPT `html/canvas` 分类 + 像素 oracle）；③ 与父目标 JS/DOM 主线仅通过 `js_dom_shim` canvas 段共享，碰撞可经 run-rules §9 管理。
>
> **▶ 基线事实（2026-08-13 实测）**：Rust 层 `CanvasContext` ~90 方法（路径/文本/变换/渐变/图案/阴影/合成/drawImage/getImageData 等）+ `Path2D` 完整（21 方法含 `from_svg`/`is_point_in_path`），~737 测试全绿；JS 侧（`js_dom_shim` part01/04/05.js + engine `CanvasRegistry`）经 `__zw_canvas_op(handle, op, ...args)` 串参派发已接通：getContext/toDataURL/toBlob（R2795/R2797/R3296/R3077）、Path2D（R3306/R3307）、ImageBitmap+createImageBitmap（R3309/R3311）、OffscreenCanvas+transferControlToOffscreen+transferToImageBitmap（R3312/R3313）、显示链路（R3268，painter 把 canvas 像素桥接为页面图元）。**WPT canvas 用例仅 40 个内建 smoke**（`test_cases_canvas.rs`，断言 render_completes/js_executes_ok），**无上游真实用例导入、无像素级 oracle 对比**。详见 [`canvas-2d/master.md`](canvas-2d/master.md)。

---

## Mission

以 **WPT `html/canvas` 目录真实用例通过率为验证标准**，将 ZeroWeb 的 Canvas 2D API 行为与像素输出对齐到 Chromium（Chrome/Edge）水平。分阶段里程碑校准执行预期（数字在首次导入后按实测校准）：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 2026 年内 | **基线建立** | 导入上游 `html/canvas` 全部范围内用例 + 像素对比 harness，记录通过率基线（当前无基线） |
| 中期 | **80%** | 轻量修复 + API 语义对齐为主 |
| 长期 | **90%+** | 覆盖 OffscreenCanvas Worker 等深结构后 |

**关键约束**：所有验证必须基于从上游 WPT 仓库（`https://github.com/web-platform-tests/wpt`）导入的**真实用例**，不允许用手写 inline 用例替代或充数。通过率统计的分母是上游 `html/canvas` 目录中所有属于范围内、不在 skip list 中的用例。

**验证可信度**：像素级断言优先复用 rendering-compat 已建立的 Chromium Oracle 机制（`scripts/capture-oracle-per-dir.mjs` / `make reftest-oracle` 同源工具链）；当前环境无 Chromium 可执行文件时（R3344 实测记录），先以 self-source + 非平凡性检查建立基线，oracle 环境可用后补齐 A/B 验证。**同源通过率不作达标依据**（anti-false-pass 原则同 DC-14）。

覆盖范围：

1. **CanvasRenderingContext2D 全 API 语义** — 路径、文本、变换、渐变/图案、阴影、合成、图像绘制、像素操作、状态栈、命中测试
2. **Tier 3 三大件完整化** — Path2D / OffscreenCanvas / ImageBitmap 的规范完整语义（含异常路径、属性反射、options 支持、Worker 集成）
3. **canvas 元素显示链路** — canvas 内容经 R3268 链路正确桥接为页面图元，`<canvas>` 尺寸语义（属性/样式/bitmap 同步）
4. **像素正确性** — 绘制结果与 Chromium 参考像素一致（布局类严格容差，WPT fuzzy 注解覆盖）

执行方式：**交替推进** — 每轮同时扩展上游 WPT `html/canvas` 导入范围和修复发现的 API/像素缺口，直到通过率达标。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| Canvas 2D API 语义 | CanvasRenderingContext2D 全部属性/方法的行为、异常、边界 | 以 WPT `html/canvas` 用例为准 |
| Path2D | 构造（空/复制/svgString）+ CanvasPath 方法 + 命中测试 | Rust 层完整（`path.rs`），JS 接线 R3306/R3307 |
| OffscreenCanvas | getContext/transferToImageBitmap/transferControlToOffscreen + 尺寸语义 | JS 侧 R3312/R3313；**Rust 侧 `offscreen.rs` 为桩** |
| ImageBitmap | createImageBitmap 源类型 + options（crop/imageOrientation/premultiplyAlpha）+ width/height | 基础 R3309/R3311；options 部分 defer |
| canvas 元素 | getContext/toDataURL/toBlob、width/height 反射、显示链路（R3268） | 已实现，WPT 驱动收尾 |
| WPT 基础设施 | `html/canvas` 真实用例导入、testharness 执行、通过率报告、像素对比 | 复用 `tests/wpt-runner` + `make import-wpt` 资产化机制 |
| 像素验证 | Chromium Oracle 对比（环境可用时）、self-source + 非平凡性检查 | anti-false-pass 同 rendering-compat DC-14 原则 |
| 单元测试与覆盖率 | canvas crate 每项修复带单测；覆盖率 ≥ 70%（核心六 crate 之一） | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **WebGL / WebGPU**：父目标非目标，本专项同样不做
- **CSS 渲染兼容性**：属 rendering-compat 目标域，本专项不碰（字体栈、布局、绘制管线差异归其管理）
- **Canvas 性能优化**：父目标性能基准体系覆盖，本专项关注正确性
- **`<video>` 帧绘制（drawImage from video）**：媒体播放为父目标非目标
- **实验性/非标准 Canvas API**：hit region、OffscreenCanvasRenderingContext2D 扩展等
- **新 crate 依赖的大规模引入**：最小化新依赖，仅在必要时引入许可证兼容 crate

### 依赖约束

- **原则**：最小化新依赖引入
- **许可证**：仅接受 MIT / Apache-2.0 / BSD
- **与 html-compat 流碰撞管理**：`js_dom_shim` part04/05.js 的 canvas 段与 html-compat 流共享活跃面。开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/` 核对；若该流最近编辑过相关段落（rule 9 碰头信号），先做零碰撞面（canvas crate 本体、WPT 导入、Rust 侧），碰头段等其告段落

---

## 当前能力/缺口基线

**详见** [canvas-2d/master.md](canvas-2d/master.md)（运行时控制面板，唯一真实状态来源）。

**关键摘要**（2026-08-13 实测）：

- ✅ **Rust 层**：`CanvasContext` ~90 方法（context_impl.rs）、`Path2D` 完整（path.rs，含 `from_svg`/`is_point_in_path`）、变换/渐变/图案/阴影/合成/像素操作全支持、~737 测试全绿、依赖 `zero-render-foundation`
- ✅ **JS 接线**：`getContext('2d')` proxy（R3077）、`__zw_canvas_op` 串参派发、Path2D（R3306/R3307）、ImageBitmap+createImageBitmap（R3309/R3311）、OffscreenCanvas（R3312/R3313）、toDataURL/toBlob（R2797/R3296）、显示链路（R3268）
- ⚠️ **缺口 1 — WPT 覆盖为零**：40 个内建 smoke 用例非上游真实导入，无通过率基线
- ⚠️ **缺口 2 — OffscreenCanvas Rust 桩**：`offscreen.rs` 为 API 桩，`transfer_to_image_bitmap` 返 ImageData 而非 ImageBitmap
- ⚠️ **缺口 3 — createImageBitmap options defer**：imageOrientation/premultiplyAlpha 未支持（part05.js:916 注释）
- ⚠️ **缺口 4 — ImageBitmap 源类型受限**：HTMLImageElement 等源未接
- ⚠️ **缺口 5 — OffscreenCanvas × Web Worker 未集成**：worker 线程 OffscreenCanvas 不可用
- ⚠️ **缺口 6 — 像素级验证缺失**：无 canvas 像素 oracle 对比集

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT canvas 用例导入与通过率基线

- [ ] 从上游 WPT 仓库 `html/canvas` 目录导入**全部**范围内真实用例（排除 skip list 中的范围外 case），进入 `tests/wpt-runner`
- [ ] 建立按子目录分类的通过率报告（文本 + JSON），记录基线
- [ ] 每项修复的 driving WPT 用例经 `make import-wpt` 常驻断言集并记入 `imported-tests.txt` 账本（CLAUDE.md 测试资产化规则）
- [ ] 通过率报告持久化到 `docs/goal/canvas-2d/evidence/`，历史可追溯

### DC-2: Canvas 2D API 语义完整（Tier 3 三大件）

- [ ] Path2D 全 API 语义（构造三态/CanvasPath/命中测试）与规范一致
- [ ] OffscreenCanvas 完整：Rust 侧真实实现（非桩）+ JS 侧 transferControlToOffscreen/transferToImageBitmap 语义
- [ ] ImageBitmap 完整：createImageBitmap options（crop/imageOrientation/premultiplyAlpha）+ 全部规范源类型
- [ ] CanvasRenderingContext2D 属性/方法行为、异常路径、属性反射与规范一致（以 WPT 用例为准）

### DC-3: 像素正确性

- [ ] canvas 绘制结果与 Chromium 参考像素一致（有 oracle 环境的用例：严格容差真通过；环境受限期：self-source + 非平凡性检查，oracle 可用后补齐）
- [ ] canvas 显示链路（R3268）与页面合成结果正确，无 glyph/图元重排类回归

### DC-4: 测试与质量不可退让

- [ ] `cargo test` 全绿（含 canvas crate 单测），零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] canvas crate 行覆盖率 ≥ 70%
- [ ] 每项修复有对应单元测试 + driving WPT 用例资产化

---

## 活跃里程碑

### M1 — WPT canvas 基线建立（进行中）

**目标**：导入上游 `html/canvas` 范围内真实用例，跑通 testharness 执行与像素对比，记录通过率基线。

**切片建议**（每片可独立落地）：
1. 上游用例导入 + 分类通过率报告（零源码改动，纯资产）
2. 失败聚类分析 → 首个轻量修复队列
3. 像素对比 harness（复用 reftest canvas 显示链路 R3268 与 Oracle 工具链）

### M2 — API 语义补齐

**目标**：按 WPT 驱动顺序补齐缺口 2-5（OffscreenCanvas Rust 真实化 → createImageBitmap options → 源类型 → Worker 集成），每项 kill-switch + A/B 零回归。

### M3 — 像素正确性冲刺

**目标**：驱动用例全部通过 + Chromium Oracle A/B（环境可用后），达到 Mission 阶段目标。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足，目标能力达到 production-ready 水平 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进，还有未完成的工作 | `CONTINUE: <下一步>` | **这是默认输出** |
| 遇到真正的外部阻塞（依赖不可用、平台不支持） | `BLOCK: <原因>` | 罕见使用 |
| verify 发现未满足条件但进展仍可推进 | `CONTINUE: <下一步>` | 返回执行，不是 DONE |

### DONE 允许条件

**同时满足**：DC-1~4 全部满足；验证基于上游真实 WPT 用例（无内建 inline 充数）；像素断言满足 DC-3 可信度要求；`cargo build` + `cargo test` + `cargo clippy` 全通过；master.md 内部自洽，archive 已建立。

### 禁止输出 DONE 的情况

- ❌ 仅通过内建 inline 用例，未导入上游真实 WPT 用例
- ❌ 通过率含同源假通过而未做非平凡性检查
- ❌ 分母为子集，非上游全量
- ❌ 无 reftest/像素证据，或存在未分析的失败项
- ❌ 无实际代码/测试进度（仅有文档和计划）

### BLOCK 策略

- "未完成、证据不足、暂时无法验证通过率、文档状态不一致" 都是**继续推进的信号**，不是 BLOCK 的理由
- 只有在真正无法继续（外部依赖不可用且无替代方案、平台根本性不支持）时才输出 BLOCK

---

## Execution Protocol

### 自主执行原则

执行代理必须：

1. **自主探索**当前 canvas 管线状态，识别 API/像素缺口
2. **自主导入** WPT `html/canvas` 用例，扩大覆盖范围
3. **自主运行**用例，分析失败原因（API 语义？host 接线？像素差异？）
4. **自主修复**，不等待用户逐步指令；每修 net≥0 即 land
5. **自主添加测试**，新修复必须有对应单元测试 + WPT 用例资产化
6. **自主验证**：`cargo test` + clippy + WPT 通过率确认修复有效
7. **自主归档**，完成的里程碑记录到 archive
8. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先（借鉴 rendering-compat 2026-07-29 裁决）

1. **主线 = 轻量修复**：WPT 驱动、根因清楚、改动面小、A/B 无新失败。
2. **永不停**：遇需拍板事项记「待用户决策」清单并跳过，继续下一个轻量修复。
3. **深结构护栏**：OffscreenCanvas × Web Worker 集成、ImageBitmap 全源类型等跨面深改不自主开工，记待决策清单等用户点名。
4. **碰撞管理**：碰 html-compat 共享面（`js_dom_shim` canvas 段）前先 `git log` 核对；有活跃编辑则转零碰撞面。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。遇到 flaky test、遗留失败、环境脚本问题时，当作当前任务的一部分修复。
2. **用例失败分析**：每个失败 case 必须分析根因（API 语义缺失？JS 接线错误？Rust 层 bug？像素差异？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。
4. **依赖问题**：优先自行解决；只有真正无法解决时才 BLOCK。
5. **范围变更**：如果发现目标需要调整，在 master.md 中记录并说明理由，但不修改本文件（除非 Mission 本身变化）。

---

## Document Control / Archive Policy

### 文档控制平面

本项目采用**两层文档控制平面**：

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件；日常进度、证据、活跃里程碑更新写入 master.md。
- **运行时控制平面** `docs/goal/canvas-2d/master.md`：当前真实状态的唯一控制面板（活跃里程碑、通过率数据、已导入用例数量、缺口清单、下一步计划、待决策清单）。治理规则：持续演进、不允许无限增长（过时内容压缩或归档）、各章节必须自洽。
- **归档区域** `docs/goal/canvas-2d/archive/`：存储已完成里程碑的详细过程与历史证据，只追加不修改。
- **证据区域** `docs/goal/canvas-2d/evidence/`：存储通过率报告、失败分析等验证证据，持续追加。

### 文档治理原则

1. master.md 各章节必须自洽 — 活跃里程碑、Done Criteria、通过率数据不能互相矛盾
2. 如果发现矛盾，执行代理必须先纠正文档再继续
3. master.md 不允许无限增长 — 过时内容必须压缩或归档
4. archive 是只追加的 — 不修改已归档内容
5. 所有验证证据必须以结构化形式持久化
