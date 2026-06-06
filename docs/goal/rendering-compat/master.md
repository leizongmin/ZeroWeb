# 渲染兼容性目标 — 运行时控制平面

**最后更新**: 2026-06-06
**当前活跃里程碑**: M1 — WPT Reftest 基础设施搭建

---

## 当前状态概览

| 维度 | 状态 | 说明 |
|------|------|------|
| 渲染管线 | ✅ 全链路贯通 | HTML→CSS→Style→Layout→Paint→Composite 完整可用 |
| WPT Runner | ⚠️ smoke 级 | 1,341 个手写 TestCase，验证"不 panic + 有 primitives"，不证明渲染正确 |
| Reftest Harness | ⚠️ 最小可用 | 16 个内建 reftest case，像素对比基础设施已有 |
| Manifest Parser | ⚠️ 基础可用 | 解析 WPT MANIFEST.json，支持按类型/路径过滤，不支持 fuzzy 注解 |
| CPU 软件渲染 | ✅ 可用 | 仅支持 FillPrimitive（矩形填充）+ GlyphDraw（位图文字） |
| GPU 渲染 | ✅ 可用 | wgpu + WGSL shaders |
| JS 执行 | ❌ 未集成 | RenderPipeline::render_html() 不执行 JS |
| Chromium 参考截图 | ❌ 不存在 | 无自动化 headless Chromium 截图工具链 |
| 上游 WPT 导入 | ❌ 不存在 | 无从上游 WPT 仓库导入真实 reftest 的能力 |
| #[ignore] 测试 | ⚠️ 保留 | 59 个真实网站测试保留 #[ignore]，因本地网络不稳定。其余测试零 #[ignore] |

---

## Done Criteria 进度

### DC-1: WPT Reftest 基础设施就位

| 条目 | 状态 | 说明 |
|------|------|------|
| fetch 上游 WPT 仓库 | ❌ | 未实现 |
| 解析 fuzzy() 元数据 | ❌ | manifest.rs 需扩展 |
| CPU 渲染截图 | ✅ | render_scene_to_framebuffer() 可用 |
| GPU 渲染截图 | ❌ | 截图持久化未实现 |
| Chromium 参考截图 | ❌ | 无 Puppeteer/Playwright 脚本 |
| Viewport 对齐 | ⚠️ | ReftestConfig 有 viewport 字段但无强制对齐机制 |
| JS 执行集成 | ❌ | render_html() 不执行 JS |
| 分类容差机制 | ❌ | 当前只有固定容差（1%/5ch） |
| 范围外过滤 | ❌ | 无 skip list |
| 通过率报告 | ⚠️ | report.rs 有报告框架但未接入 reftest |
| 单一命令运行 | ❌ | 无 cargo run --bin wpt-reftest |
| CI 集成 | ❌ | 未接入 |

### DC-2: CSS 2.1 核心通过率 ≥ 95%

| 条目 | 状态 | 说明 |
|------|------|------|
| 导入 reftest 子集 ≥ 50 | ❌ | 未导入任何上游 reftest |
| 通过率 ≥ 95% | ❌ | 无数据 |

### DC-3 ~ DC-5: Flexbox+Grid / 布局模式 / 文字排版

- 全部 ❌ — 依赖 M1 基础设施

### DC-6: Quirks Mode

| 条目 | 状态 | 说明 |
|------|------|------|
| CSS parser quirks | ❌ | DOM parser 存储了 quirks mode 但 CSS parser 忽略 |
| Style system quirks | ❌ | 未实现 |
| Layout engine quirks | ❌ | 未实现 |
| Quirks mode 传递链 | ❌ | 未建立 |

### DC-7: 测试与质量

| 条目 | 状态 | 说明 |
|------|------|------|
| cargo test 零失败 | ⏳ | 正在验证（59 个真实网站测试保留 #[ignore]） |
| 零 #[ignore] 测试 | ⚠️ | 59 个真实网站测试保留 #[ignore]（本地网络不稳定），其余零 #[ignore] |
| 新修复有单元测试 | — | 尚未开始修复 |
| cargo clippy 零警告 | ⏳ | 待验证 |
| Reftest 报告持久化 | ❌ | evidence/ 目录已创建，无报告 |

---

## M1 里程碑详情

**目标**: 建立能够导入、运行、对比和报告 WPT reftest 的完整基础设施。

### M1 完成标准 (14 项)

1. ❌ fetch 上游 WPT 仓库
2. ❌ 扩展 manifest.rs 解析 fuzzy() 元数据
3. ✅ CPU 软件渲染截图（已有 render_scene_to_framebuffer）
4. ❌ GPU 渲染截图
5. ❌ 自动化 Chromium 截图工具
6. ❌ Viewport 对齐机制
7. ❌ JS 执行集成
8. ❌ 分类容差机制
9. ❌ 范围外 reftest 过滤 (skip list)
10. ❌ 按目录分类通过率报告
11. ❌ 单一命令运行全部 reftest
12. ❌ 导入 CSS 2.1 核心 ≥ 50 个 reftest
13. ❌ 记录初始通过率
14. ⚠️ 确认 #[ignore] 标记状态：59 个真实网站测试保留 #[ignore]（本地网络不稳定），其余零 #[ignore]

### M1 影响范围约束

- **主要修改**: `tests/wpt-runner/`
- **新增文件**: Chromium 截图脚本、reftest skip list
- **可能修改**: `crates/render-foundation/src/surface.rs`、`crates/render-foundation/src/cpu/`、`crates/engine/src/pipeline.rs`
- **确认状态**: `tests/integration/src/real_website_compat.rs`（59 个真实网站测试保留 #[ignore]，因本地网络不稳定）
- **不允许修改**: `crates/css-parser/`、`crates/style-system/`、`crates/layout-engine/`（M1 只建基础设施）

---

## 已知关键缺口

| 缺口 | 影响范围 | 优先级 |
|------|----------|--------|
| Float 布局算法 | CSS 2.1 核心 | M4 |
| Table 布局算法 | 表格渲染 | M4 |
| Multi-column 布局算法 | 多列布局 | M4 |
| OpenType shaping | 文字排版质量 | M5 |
| BiDi 算法 | RTL 文本 | M5 |
| Vertical writing-mode | 竖排文本 | M5 |
| Quirks mode | CSS 2.1 兼容性 | M2 |
| CPU 渲染路径不完整 | reftest 可行性 | M1 |

---

## 技术决策记录

| 日期 | 决策 | 理由 |
|------|------|------|
| 2026-06-06 | 保留真实网站测试的 #[ignore] | 本地网络不稳定，这些测试不可执行 |
| 2026-06-06 | 扩展而非重写 manifest.rs 和 reftest.rs | 目标文档明确要求扩展现有模块 |

---

## 下一步

1. 验证 cargo test 全绿（59 个真实网站测试保留 #[ignore]，不计入）
2. 扩展 manifest.rs 添加 fuzzy 元数据解析
3. 扩展 ReftestConfig 添加分类容差和 per-test fuzzy 注解
4. 创建 reftest skip list 和过滤机制
5. 创建 Chromium 截图脚本
6. 实现 reftest runner CLI
7. 导入 CSS 2.1 核心 ≥ 50 个 reftest
8. 运行初始 reftest 基线测试
