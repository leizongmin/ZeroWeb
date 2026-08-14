# M0 Slice R30 — dom_bindings 独立 coverage 口径收口

**日期**: 2026-08-14
**里程碑**: M0 — 基线建立 + polyfill-live 合一起刀（must-complete 项 4 收口）
**切片**: R30
**前置**: R29（M0 项 1/2/3/5/6 ✅，项 4 待 cargo-llvm-cov）

## 切片选择（决策记录）

M0 must-complete 项 4「补齐 dom_bindings 独立 coverage 口径」自 R0 起记录为「待 `cargo-llvm-cov` 安装」。R30 开工核实：**cargo-llvm-cov 0.8.7 + llvm-tools-preview 已安装**（环境前提已满足，推翻 R0「本地未装」记录）。缺口纯为「未建口径」，非环境阻塞——按入口文档「覆盖率策略」："若当前缺少 dom_bindings 独立 coverage 口径……这不是 BLOCK 理由，而是要继续推进的 active milestone"。

本切片零碰撞（纯脚本+度量，无生产代码改动），关闭 M0 唯一未完成项，收口 M0 里程碑（6/6 项）。

## 根因

`dom_bindings` 是 `zero-engine` 的**子模块**（`crates/engine/src/dom_bindings/`），非独立 crate。`scripts/check-coverage.sh` 的 `cargo llvm-cov --workspace --summary-only` 仅按 crate 报告，dom_bindings 被 fold 进 zero-engine 总数，无独立行覆盖率数字——DC-4「dom_bindings 覆盖率持续提升、不退化」缺乏度量基线。

## 修复

### `scripts/check-dom-bindings-coverage.sh`（新建）

经 `cargo llvm-cov -p zero-engine --lib --lcov` 生成 lcov，内联 Python 解析 `SF:`/`DA:` 记录：

- **源码/测试分离**：`/tests_` 前缀的 5 个测试文件（tests_ab_compare/tests_collections/tests_dom_api/tests_events/tests_html_setters）与 15 个源码文件分开统计——测试文件恒 100%，混入会虚高源码覆盖率
- **逐文件明细**：按覆盖率排序输出，标出 <90% 提升候选
- **双 feature 可参数化**：默认 `--features v8`（dom_bindings 测试点全在 v8 矩阵，tests_* 均 `#[cfg(feature="v8")]`），透传 `--no-default-features --features quickjs` 支持 DC-4 双路径
- **`--json` 模式**：机器可读，供 evidence 持久化与趋势追踪
- **工具 guard**：cargo-llvm-cov / llvm-tools 缺失时提示安装并 `exit 0`（覆盖率口径非 CI 硬门禁，不阻断）

### `scripts/check-coverage.sh` 集成

加 `--dom-bindings` flag：workspace 摘要后追加 dom_bindings 子模块口径（cov run 额外 ~15s，flag 开启不阻默认快路径）。

## 验证

- **脚本实测**：`bash scripts/check-dom-bindings-coverage.sh` 输出 dom_bindings 源码 93.14%（4561/4897，15 文件）/ 全部 95.15%（6590/6926，20 文件）+ 逐文件明细
- **JSON 模式**：`--json` 经 `json.load` 校验有效（source rate / per_file entries 正确）
- **基线 JSON 持久化**：`docs/goal/js-dom/evidence/2026-08-14-r30-dom-bindings-coverage-baseline.json`
- **`git diff --check`**：clean

## dom_bindings coverage 基线（R30 首建）

| 文件（源码） | 覆盖率 | 命中/总行 |
|---|---|---|
| html_element.rs | 99.1% | 228/230 |
| gc.rs | 97.9% | 285/291 |
| document.rs | 97.8% | 225/230 |
| event.rs | 97.7% | 295/302 |
| factories.rs | 96.0% | 316/329 |
| event_target.rs | 95.5% | 276/289 |
| mod.rs | 94.9% | 502/529 |
| dom_token_list.rs | 93.0% | 277/298 |
| namednodemap.rs | 92.8% | 233/251 |
| element.rs | 92.7% | 662/714 |
| node.rs | 91.6% | 536/585 |
| dataset.rs | 91.6% | 141/154 |
| **custom_elements.rs** | **89.0%** | 154/173 |
| **css_style_declaration.rs** | **86.7%** | 331/382 |
| **dom_exception.rs** | **71.4%** | 100/140 |

**提升候选**（<90%）：dom_exception 71.4% / css_style_declaration 86.7% / custom_elements 89.0%（纯补测试，零碰撞，下轮候选 b）。

## 决策记录

- **为何用 lcov 而非 --summary-only**：summary-only 仅按 crate，dom_bindings 作为子模块无独立数字。lcov 含每个源文件的 `DA:<line>,<count>` 记录，Python 解析即可得子模块 + 逐文件口径。
- **为何源码/测试分离**：测试文件（`tests_*.rs`）本身被 100% 执行，计入会虚高（混入后 95.15% vs 纯源码 93.14%）。DC-4「dom_bindings 覆盖率」应度量源码被测试覆盖的程度，故主指标用源码口径（93.14%），同时报告全部口径（95.15%）供参考。
- **为何 --lib 而非全量**：dom_bindings 测试点全在 `--lib`（unit tests in src），聚焦免跑 integration/binary 减少噪声 + 加速。
- **非 CI 硬门禁**：脚本工具缺失 `exit 0`（不阻断 CI）。覆盖率口径是度量基础设施，不是 land gate；land gate 仍是 `make test` + clippy。

## 残留（转 R31+）

- **dom_bindings coverage 提升**：dom_exception 71.4% / css_style_declaration 86.7% / custom_elements 89.0% 纯补测试（下轮候选 b，零碰撞）
- Event-dispatch 系列（深结构）/ 双路径差收口 / 主线 M1 L2 / M6 QuickJS native（均见 master.md 剩余聚类）
