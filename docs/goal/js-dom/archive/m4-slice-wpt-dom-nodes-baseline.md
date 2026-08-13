# 归档：M4 首切片 — WPT dom/nodes 上游通过率基线

**日期**: 2026-08-13
**轮次**: R1
**Milestone**: M4（WPT dom 上游基线建立与扩展）
**切片**: M4 切片 1（上游用例导入 + 分类通过率报告，零源码行为改动）
**基线**: `b12ea3e5`（R0 land 后）

## 切片目标

DC-3 硬缺口「WPT dom 上游用例 0 导入，无通过率基线」的首切片：从上游 `web-platform-tests/wpt` 导入 `dom/nodes/` 真实 testharness 用例，建按子分类通过率基线，立即暴露 native/polyfill 真实规范差距。

## 实现产物

镜像 `testharness-canvas` 模式（同构）：

1. **`tests/wpt-runner/scripts/fetch-dom-subset.sh`**（新，~70 行）：pin WPT `31597693`，经 GitHub API 列 `dom/nodes/` 目录 + raw 拉 .html；wpt-data gitignored，用例按需 fetch 不入库。`SUBDIRS=("dom/nodes")`，后续 M4 切片追加 `dom/events` 等。
2. **`tests/wpt-runner/src/testharness.rs`**：
   - `pub const DOM_TEST_SUBDIRS: &[&str] = &["dom/nodes"]`
   - `pub fn run_dom_cases(wpt_root, filter)`：扫描 `DOM_TEST_SUBDIRS` 下 .html，复用 `run_testharness_html`（dom 不需 canvas-tests.js，比 canvas 更简）
3. **`tests/wpt-runner/src/main.rs`**：`testharness-dom` 子命令 + `cmd_testharness_dom`（输出 text/json/tap，exit 1 = 有失败，与 html/canvas 一致）+ help 文本
4. **Makefile**：`fetch-wpt-dom` + `testharness-dom` 目标（test-guard 包裹，支持 `FILTER=`），加 `.PHONY`
5. **evidence**：`evidence/2026-08-13-r1-wpt-dom-nodes-baseline.md`（聚合 + 失败聚类 + ROI 分析）

## 通过率基线（首跑即基线）

| 指标 | 值 |
|------|-----|
| 用例文件数 | 141 |
| subtest 总数 | 2696 |
| Pass | 1112（**41.25%**） |
| Fail | 1572 |
| Timeout | 12 |
| 全 Pass 用例 | 14 / 141 |

## 失败聚类（重排后续 ROI）

| 失败次数 | 类型 | 根因方向 |
|----------|------|----------|
| 414 | `assert_throws_dom`（非法操作未抛 DOMException） | createElement 非法标签 / appendChild 闭环——**最大缺口** |
| 98 | `documentElement` of undefined | XML/XHTML document 模型缺失 |
| 80 | `name` of null | Attr/namedNodeMap 在 XML 上下文 |
| 60 | token 校验 | DOMTokenList/classList 应抛 InvalidCharacterError |
| 49+39 | instanceof HTMLElement/Element | native 对象原型链 |
| 44 | createProcessingInstruction 未实现 | 单一 API 缺失 |

## 关键决策

1. **选 dom/nodes 作首批**：直接对应 native DOM 绑定的 Node/Element/Document 核心，与 M1/M2 迁移目标最相关；178 个顶层 .html（拉到 141，部分 curl 限流超时，够建基线）。
2. **镜像 canvas 模式而非 html-interaction 白名单**：canvas 用目录扫描（`CANVAS_TEST_SUBDIRS`），适合 dom 多文件批量；html-interaction 用固定白名单（7 文件），不适合 dom 扩展。
3. **基线非 land 门禁**：`testharness-dom` exit 1 仅记录通过率，不阻塞 land（基线本就预期大量失败）；agent 经 `--format json` 捕获后写 evidence。

## 验证

- `cargo build --release -p zero-wpt-runner` ✅ 编译通过
- 单用例 smoke：`testharness-dom Document-createElement.html` 正确报 subtest 差距 ✅
- 全量基线：141 用例 / 2696 subtest 跑通（test-guard 包裹）✅
- clippy / fmt / test：land 前 make test 矩阵验证（见 master.md 测试基线段）

## 下一步（M4 切片 2 候选）

按 ROI：① DOMException 抛出语义（~474 失败）② createProcessingInstruction（44）③ 扩 dom/events 子目录。
