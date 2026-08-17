# M1 + M2 里程碑归档（2026-08-15）

> 归档时间：2026-08-15。本文档只追加不修改；当前状态以 master.md 为准。

## M1 — WPT canvas 基线建立（✅ 完成，2026-08-15）

**目标**：导入上游 html/canvas 范围内真实用例，跑通 testharness 执行与像素对比，记录通过率基线。

**过程摘要**：
- 导入面：`html/canvas/element/*`（9 目录 919 文件）+ `html/canvas/offscreen/*`（worker 变体）
  + `resources/testharness.js` + `html/canvas/resources/canvas-tests.js`，经
  `tests/wpt-runner/scripts/fetch-canvas-subset.sh` 资产化（wpt-data 独立 repo，
  gitignored；P1 修订版 315976933870b34d6ea30e3f6643403edae678ba）。
- 基线演化：718 Pass（2026-08-14 首轮）→ 828 Pass（v6，float16 options）→
  **832 Pass / 0 Fail**（v7，G7 全灭，2026-08-15）。65 Timeout 全为 reftest-format
  文件（`rel="match"`/`-ref`/`-expected`，无 testharness——非 canvas 面）。
- 通过率报告：文本 evidence/r34xx-batch-*.md + JSON
  evidence/r34xx-batch3-canvas-main-2026-08-15.json。

**关键决策**：
- 资产化机制 = fetch-canvas-subset.sh + imported-resources.txt 账本（canvas
  testharness 面；reftest 面的 make import-wpt 机制不适用）。
- 像素验证环境受限（无 Chromium 可执行，R3344 记录）→ GPU 路径测试
  （gpu_path.rs，lavapipe 软件适配器）作为像素回读通道；oracle A/B 待环境。

## M2 — API 语义补齐（✅ 完成，2026-08-15）

**目标**：按 WPT 驱动顺序补齐 Tier 3 三大件 + 全 API 语义。

**过程摘要**：
- Path2D：构造三态（空/复制/svgString）+ CanvasPath 全方法 + isPointInPath 命中
  （R3306/R3307）。
- OffscreenCanvas：Rust 侧真实化（offscreen.rs）+ transferToImageBitmap +
  transferControlToOffscreen + worker 变体全通（G6，715 Pass）。
- ImageBitmap：createImageBitmap 源类型（Blob/ImageData/canvas/img）+
  options（crop/flipY/premultiplyAlpha/colorSpace/pixelFormat float16）。
- CanvasRenderingContext2D：drawing.style 全属性面、渐变 OKLab 插值、alpha 序列化
  最短回滚、ImageData 构造器 WebIDL 重载回退、text 全系（真字体光栅/基线/对齐/
  maxWidth/rtl/letterSpacing/wordSpacing/measure 真度量/TextCluster/
  getIndexFromOffset 中点边界/selectionRects）、variationSelectors 呈现感知字体
  选择、float16 覆盖层像素往返、外链 CSS url 绝对化。
- G7 全灭：testharness 面 **0 Fail**。

**关键决策**：
- cmap14（Unicode variation sequences）为 rustybuzz 0.20 原生支持
  （handle_variation_selector_cluster），无需自实现；VS 语义以单测锁定。
- variationSelectors 宽度差经「呈现感知字体选择」实现（VS15 → 文本字体回落，
  VS16 → emoji 字体），近似为整串选择（逐字回退为 Chromium 细化，文档化）。
- float16 越界值（2/-1）经 JS 侧覆盖层往返（u8 缓冲无法存），写像素操作失效。

**commit 索引**：
- 2810a207 float16 options（put.basic 通过）
- a1b82b97 float16 覆盖层补全（主+worker）
- ab298950 G7 全灭（variationSelectors/ctor.basics/index-from-offset-edge-cases）
- 47fc3205 evidence/master 定稿 v7（832/715，91.18% 覆盖率）

## 验证基线（归档时点）

- WPT：主线程 832 Pass / 0 Fail / 65 Timeout（全 reftest-format）；
  worker 715 Pass / 0 Fail
- Rust：canvas 774 / render-foundation 640 / engine 2129 / wpt-runner 171 全绿
- 覆盖率：canvas 91.18%（≥70% 达标）
- clippy --workspace --all-targets -- -D warnings 零警告
- GPU 路径：4 测试 lavapipe 真实执行通过
