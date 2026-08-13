# M4 R8 切片 — testharness 本地 .js 内联 + 基线真实化

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线扩展）
**前置**: R7（fetch .js 依赖 + native createProcessingInstruction API）
**commit**: 见 `git log`（feat(wpt-runner): testharness local .js inline + baseline truthing）

## 背景

R7 把用例引用的 .js 文件拉到 wpt-data，并交付 native createProcessingInstruction API。但 R7 末尾记录了一个未完成项：**testharness 运行时仍不内联这些 .js**（`extract_page_scripts` 不加载外部 `<script src>`），故用例引用的 `attributes.js` / `Document-createProcessingInstruction.js` 等仍不执行 → `attr_is is not defined`、测试体 not defined。R7 基线（51.12%）因此虚高（引用 .js 的用例因顶层 not defined 抛错被计为单条 Fail，掩盖用例内部数十个 subtest）。

R8 = 在 testharness 运行时把用例引用的本地 .js 内联为 inline `<script>`，让用例真正跑起来，基线真实化。

## 改动

**文件**: `tests/wpt-runner/src/testharness.rs`（+111 / -6 行）

1. **`inline_local_scripts(html, wpt_root, case_path)`**（新）：扫描剩余 `<script src="...">`，仅相对路径（非 `/resources/`、非 `http(s)`、非 `//`），从 wpt-data 读文件内容内联为 `<script data-inline="...">content</script>`；规范化 `./` 前缀与 `../` 上溯（`dom/nodes/../constants.js` → `dom/constants.js`）；文件缺失则移除该标签（best-effort，不注入空）。
2. **`extract_script_src`** / **`normalize_relative`**（新 helper）。
3. **签名贯穿**：`prepare_harness_html` + `run_testharness_html` + `run_canvas_testharness_html` + `run_testharness_html_inner` 全部收 `wpt_root: &Path`（dom/canvas 双路径都需要本地资源）。

## 并行流碰撞与合并（run-rules §9/§10）

stash pop 时 testharness.rs 与远端 main 冲突——canvas 流（`ad0ef686` G5 headless 图片）独立把 `wpt_root` 贯穿到 inner（其设计 `Option<&Path>` + `wpt_data_image_fetcher`），与 R8 同一问题域（testharness 运行时访问 wpt-data 本地资源）。

**合并裁决**: 统一用 `&Path`（非 Option）。理由：
- canvas 路径需要 image fetcher（图片资源），dom 路径需要 .js 内联 → **两条路径都需要真实 wpt_root**。
- `Option<&Path>` 的 `None` 分支在 dom 路径下会让 `inline_local_scripts` 失效（读不到 .js 目录），破坏 R8 的核心能力。
- 用 `&Path` + image fetcher 始终启用，消除 Option 的复杂性（准则 §2 简单至上），且 dom 用例也可能加载图片资源。

冲突解决 4 处（canvas 调用格式 / canvas inner 调用 / dom inner 调用 / inner 签名 + image fetcher 调用）+ 1 处远端遗漏调用点（`run_html_interaction_cases` 的 `run_testharness_html` 仍用旧 4 参签名，补 wpt_root）+ 3 处单元测试调用点补 wpt_root（用 `/nonexistent-wpt-root-for-tests` 占位，这些测试用内联 html 不引用外部 .js）。

## 验证

| 门禁 | 命令 | 结果 |
|------|------|------|
| 编译 | `cargo build -p zero-wpt-runner` | ✅ |
| fmt | `cargo fmt --all -- --check` | ✅ 无 diff |
| clippy v8 | `cargo clippy -p zero-wpt-runner --all-targets -- -D warnings` | ✅ 零警告 |
| 单测 v8 | `cargo test -p zero-wpt-runner` | ✅ 168 passed |
| 单测 quickjs | `cargo test --no-default-features --features quickjs -p zero-wpt-runner` | ✅ 103 passed |

## 基线结果（dom/nodes，178 用例 / 4490 subtest）

| 路径 | Pass% | Pass | Fail | Timeout | Unsupported |
|------|-------|------|------|---------|-------------|
| polyfill | **37.82%** | 1698 | 2768 | 23 | 1 |
| native   | **37.59%** | 1688 | 2778 | 23 | 1 |

- **双路径对等**: 差 0.23pp（与 R7 的 0.33pp 一致）。
- **为何从 R7 51.12% 降到 37.82%**: 非回归。R8 让引用 .js 的用例真正跑起来，subtest 从 ~2696 → 4490（+1794 真实 subtest），分母变大故百分比下降——基线进一步真实化（同 R7 逻辑延续）。详见 evidence `2026-08-14-r8-local-js-inline-baseline-truth.md`。

## 下一步

- 聚类 ROI：① createElementNS namespace（596）② createEvent/createProcessingInstruction 方法（284）③ instanceof 原型链（89）④ 扩 DOM_TEST_SUBDIRS（dom/events）。
- 主线仍 M4（按聚类驱动修复），native/polyfill 双路径对等守住。

## 未解决问题（遗留）

- 无新增遗留。R7 的 PI 用例超时（ProcessingInstruction 构造器缺失）仍在，createProcessingInstruction 方法在部分用例仍 `is not a function`（284 块的一部分），下一轮 ROI 候选。
