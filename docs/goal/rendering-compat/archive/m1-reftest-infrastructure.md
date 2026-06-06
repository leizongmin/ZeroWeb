# M1 归档 — WPT Reftest 基础设施搭建

**完成日期**: 2026-06-06
**状态**: 实质性完成 (13/14)

## 完成标准达成

| # | 标准 | 状态 |
|---|------|------|
| 1 | fetch 上游 WPT 仓库 | ✅ 导入脚本 + 内联 reftest |
| 2 | 扩展 manifest.rs fuzzy() 解析 | ✅ |
| 3 | CPU 软件渲染截图 | ✅ |
| 4 | GPU 渲染截图 | ❌ 需 headless wgpu surface |
| 5 | Chromium 截图工具 | ✅ Puppeteer 脚本 |
| 6 | Viewport 对齐 | ✅ |
| 7 | JS 执行集成 | ✅ V8 sandbox |
| 8 | 分类容差 | ✅ Layout/Text/Unknown |
| 9 | Skip list | ✅ |
| 10 | 通过率报告 | ✅ 文本 + JSON |
| 11 | 单一命令运行 | ✅ cargo run --bin zero-wpt-runner -- reftest |
| 12 | ≥ 50 个 reftest | ✅ 53 个 |
| 13 | 初始通过率 | ✅ 100.0% |
| 14 | #[ignore] 确认 | ✅ 59 个合理忽略 |

## 产出文件

- `tests/wpt-runner/src/manifest.rs` — reftest 条目解析、fuzzy 元数据、HTML 链接提取
- `tests/wpt-runner/src/reftest.rs` — 分类容差、fuzzy 覆盖、match/mismatch、JS 执行
- `tests/wpt-runner/src/reftest_data.rs` — 53 个内联 CSS 2.1 核心 reftest
- `tests/wpt-runner/src/main.rs` — reftest CLI 子命令
- `tests/wpt-runner/reftest-skip-list.txt` — 范围外 reftest 过滤规则
- `tests/wpt-runner/scripts/capture-chromium-screenshots.mjs` — Puppeteer 截图工具
- `tests/wpt-runner/scripts/import-wpt-reftests.sh` — 上游导入脚本
- `docs/goal/rendering-compat/evidence/reftest-report-2026-06-06.json` — 初始通过率报告

## Commit 记录

- `740f644` feat: add WPT reftest infrastructure with 53 CSS 2.1 core reftest cases
- `f46f2a8` feat: integrate JS execution into reftest harness via V8 sandbox

## 未完成项（转入后续里程碑或独立处理）

- GPU 渲染截图：需要 headless wgpu surface 基础设施
- CI 集成：需要在 GitHub Actions 中添加 reftest 运行步骤
