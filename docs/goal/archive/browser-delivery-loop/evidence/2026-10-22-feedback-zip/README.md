# 反馈包 zip 形态（--export-format=zip）

**日期**：2026-10-22
**对应**：[M2 RFC](../../m2-delivery-rfc.md) §4.4/IF-4 原始形态（zip）；S4 落地时因依赖图无 zip crate 回退目录形态，§3 约定"优先 zip crate 若依赖图已有"——依赖图至今没有，且根 Cargo.lock 是并行流共享碰撞面，故零新依赖自实现 **stored（不压缩）ZIP writer**。目录形态保持缺省（S4/M4 证据可复现），zip 为显式 opt-in。
**产物与脚本**：[run-feedback-zip-acceptance.sh](run-feedback-zip-acceptance.sh) / 原始输出 [zip-acceptance-run.log](zip-acceptance-run.log)（**19 断言全 PASS**，构建时 commit `5c1ba767797a`）

## 1. 实现面

| 文件 | 内容 |
|---|---|
| `crates/browser-shell/src/feedback.rs` | `export_feedback_zip`（与目录形态同签名）；`prepare_feedback_payload` 共用载荷（校验+报告字节+environment.json+可选 summary 一次备齐，两形态字节一致）；`zip_store_bytes` 手写 stored ZIP（local header + central directory + EOCD，PKWARE APPNOTE §4.3，全小端手写，零新依赖）；`crc32_ieee` 逐位 CRC-32/ISO-HDLC（成员 ≤10×64KiB 小文件，免查表）；`civil_from_unix` 从 `format_date_ymd` 拆出复用（DOS date 需数值年月日）。已存在同名 zip 拒绝不覆盖；失败清理半成品；target 父目录缺失时递归创建（与目录形态 `create_dir_all` 语义对齐——首轮验收真实抓到此缺陷并修复） |
| `apps/browser/src/main.rs` | `ExportFormat { Dir, Zip }` + `--export-format=` 解析（仅 `dir`/`zip`，非法值报错 exit 2）；`run_export_feedback` 按形态分发；usage 文本行 |
| 单测 | CRC 已知向量（`""`→0、`"123456789"`→0xCBF43926）；zip 魔数/EOCD 计数/stored method 字段断言；zip 与目录形态成员逐字节一致 + 重复导出拒绝；target 目录缺失递归创建；`cargo test -p zero-browser-shell` 11/11 PASS |

## 2. 真实进程验收（三腿 19 断言全 PASS）

隔离：`--profile` + 环境变量兜底均在 work-dir；守卫 `systemd-run -p MemoryMax=8G timeout 300`。

| 腿 | 场景 | 断言 | 结果 |
|---|---|---|---|
| leg A | GUI + `--profile` → SIGKILL 唯一 tab renderer 产生真实崩溃报告（G3 leg B 模式）→ `--export-format=zip` 导出 | 真实报告落盘；导出 zip 命名 `<version>-<date>.zip`；python3 zipfile **真解压语义深校验**（`testzip()` CRC 完整性=自实现 crc32 端到端证明、成员名清单与 environment.reports 一致、报告逐字节一致、version/commit 与 `--version` 贯穿）；summary 成员仅显式 `--include-session-summary` 时出现且无 URLs；整包隐私 grep（`https?://\|cookie\|token\|authorization`）全 0；重复导出拒绝且既有 zip sha256 不变 | PASS ×11 |
| leg B | 缺省回归 | 不带 `--export-format` 恒目录形态；同参数目录包与 zip 包成员一一对应且逐字节一致（2 文件） | PASS ×3 |
| leg C | 负例 | 空 profile 无报告 → `FEEDBACK_REJECTED NO_REPORTS` 且零文件；`--export-format=tar` 非法值 → 报错带 usage（exit 2）；勾选不存在报告 → 拒绝且零文件 | PASS ×5 |

**排障记档**（不遮掩）：首轮验收 leg A 导出步 `FEEDBACK_REJECTED No such file or directory`——zip 形态直接 `fs::write`，target 父目录不存在即失败；目录形态靠 `create_dir_all(feedback_dir/reports)` 连带创建父目录而未暴露。修复为 zip 形态显式 `create_dir_all(target_dir)`，附单测。同轮脚本硬化：`set -eu` 下命令替换非零退出会**静默终止脚本**（首轮只看到 4 个 PASS 就断流），关键 CLI 捕获改 `|| true` + 显式断言。

## 3. 门禁

fmt PASS / 单测 11/11 PASS / release 构建（守卫）PASS / clippy `--workspace --all-targets -D warnings` / 全量 `make test`（test-guard 包裹）/ 验收守卫 `systemd-run -p MemoryMax=8G timeout 300` / 提交前 lei-pre-commit-guard PASS。

## 4. 限制与挂账

- stored 不压缩：崩溃报告为 ≤64KiB JSON 小文件，压缩收益有限；不引入 zip/压缩依赖（Cargo.lock 共享碰撞面）。包体 <1KiB/报告。
- RFC §3"优先 zip crate 若依赖图已有"以零依赖自实现满足；§4.4 GUI 导出入口仍挂账（需目录选择器基建，成本超切片）。
- 真机安装/签名验收待 P1/P4 授权（master.md 下一步）。
