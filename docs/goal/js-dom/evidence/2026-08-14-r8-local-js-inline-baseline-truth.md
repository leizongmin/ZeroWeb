# R8 — testharness 本地 .js 内联 + 基线真实化（M4 / DC-3）

**日期**: 2026-08-14
**轮次**: R8
**里程碑**: M4（WPT dom 上游基线扩展与真实化）
**commit**: 见 `git log`（R8 land commit）

## 测试命令

```bash
# polyfill 路径
./target/test-guard --per-proc-mem 10 --total-mem 28 --time-limit 870 -- \
  ./target/release/zero-wpt-runner testharness-dom --json

# native 路径（ZW_NATIVE_DOM=1）
ZW_NATIVE_DOM=1 ./target/test-guard --per-proc-mem 10 --total-mem 28 --time-limit 870 -- \
  ./target/release/zero-wpt-runner testharness-dom --json
```

## 基线结果（dom/nodes，178 用例 / 4490 subtest）

| 路径 | Pass | Fail | Timeout | Unsupported | Pass% |
|------|------|------|---------|-------------|-------|
| polyfill | 1698 | 2768 | 23 | 1 | **37.82%** |
| native   | 1688 | 2778 | 23 | 1 | **37.59%** |

**双路径对等**: 差 0.23pp（polyfill 37.82% vs native 37.59%），与 R7 对等水平（差 0.33pp）一致。

完整 JSON 快照: `2026-08-14-r8-dom-nodes-polyfill.json` / `2026-08-14-r8-dom-nodes-native.json`。

## 本轮改动（R8）

**testharness 运行时本地 .js 内联**（`tests/wpt-runner/src/testharness.rs`）：

- 新增 `inline_local_scripts(html, wpt_root, case_path)`：用例引用的本地 .js 测试体（如 `<script src="attributes.js">`、`<script src="Document-createProcessingInstruction.js">`、相对 `../constants.js`）经 `extract_page_scripts` 不加载，故从 wpt-data 读文件内容内联为 inline `<script>`。仅内联相对路径（非 `/resources/`、非 `http(s)`）；文件缺失则移除该标签（best-effort）。
- `prepare_harness_html` 新增 `wpt_root` + `case_path` 参数，调用 `inline_local_scripts`。
- `run_testharness_html` / `run_canvas_testharness_html` / `run_testharness_html_inner` 贯穿 `wpt_root: &Path`（统一非 Option：dom 与 canvas 两路径都需要本地资源——dom 需 .js 内联，canvas 需 image fetcher）。

**与并行 canvas 流（`ad0ef686` G5 headless 图片）合并**：canvas 流独立把 `wpt_root` 贯穿到 inner（其设计 `Option<&Path>`），本轮合并统一为 `&Path`（双路径都需要），同时保留 canvas 流的 `wpt_data_image_fetcher` 并让 dom 路径也启用（dom 用例也可能加载图片资源）。

## 为何通过率"下降"（R7 51.12% → R8 37.82%）— 非回归

R7 只补了用例引用的 .js 文件（用例主 .js + dom 根共享 .js），但 **testharness 运行时仍不内联这些 .js**（那是 R8 的工作）。R7 基线的 51.12% 是"用例 .js 在磁盘上但运行时没加载 → 引用 .js 的用例因 `attr_is is not defined` 抛错被计为单条 Fail，掩盖了用例内部数十个 subtest"。

R8 让 .js 真正内联执行后，这些用例**真正跑起来**，暴露了用例内部的真实 gap（`before/after/replaceWith/prepend/append unscopable`、`cloneNode`、`getRootNode`、`createProcessingInstruction is not a function`、`createElementNS` namespace 处理等），subtest 数从 ~2696 → 4490（+1794 真实 subtest），通过率分母变大故百分比下降——这是**基线进一步真实化**（同 R7「.js 缺失致跳过→真实化」逻辑的延续），更诚实，非回归。

**双路径对等不变**（差 0.23pp），证明 native 与 polyfill 行为仍等价。

## 失败聚类（下一步 ROI 排序）

| 错误模式 | Fail subtests | 主要用例 |
|----------|---------------|----------|
| assert_equals mismatch | 913 | Document-createElementNS(596)、case(259) |
| cannot read properties | 726 | querySelector/getRootNode/document 链 |
| other | 695 | — |
| is not a function | 284 | createProcessingInstruction/createEvent 等 |
| instanceof | 89 | HTMLElement/Element 原型链（R7 已记录 ~88） |
| is not defined | 61 | 部分已由 .js 内联解决 |

**下一步 ROI**：
1. **createElementNS namespace 处理**（596 subtest，最大块）
2. **createEvent / createProcessingInstruction 方法实现**（284 is not a function）
3. **instanceof HTMLElement/Element 原型链**（89）
4. **扩 DOM_TEST_SUBDIRS**（dom/events 等纯资产）

## 验证

- `cargo build -p zero-wpt-runner` ✅
- `cargo fmt --all -- --check` ✅
- `cargo clippy -p zero-wpt-runner --all-targets -- -D warnings`（v8）✅ 零警告
- `cargo test -p zero-wpt-runner`（v8）✅ 168 passed
- `cargo test --no-default-features --features quickjs -p zero-wpt-runner` ✅ 103 passed（双矩阵）
