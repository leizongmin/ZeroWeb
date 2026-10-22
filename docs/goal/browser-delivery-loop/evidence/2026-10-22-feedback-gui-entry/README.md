# FR-5 GUI 导出入口（zero://feedback 内部页）

**日期**：2026-10-22
**对应**：[M2 RFC](../../m2-delivery-rfc.md) §4.4「GUI 菜单入口视既有菜单基建接入成本为可选增强」——勘察发现无需菜单系统：既有 `zero://settings` 内部页 + 动作 URL 截获（`try_apply_internal_url`）+ 附件下载管线（书签导出先例）三件基建齐备，GUI 入口成本回到切片量级，本轮落地。
**产物与脚本**：[run-feedback-gui-entry.sh](run-feedback-gui-entry.sh) / 原始输出 [run-feedback-gui-entry.log](run-feedback-gui-entry.log)（**5 断言全 PASS**，构建时 commit `ca2bbbf7b65a`）；app 级测试 4 项 + pages 单测 3 项 + shell 单测 1 项

## 1. 实现面

| 文件 | 内容 |
|---|---|
| `crates/browser-shell/src/feedback.rs` | 拆出 `build_feedback_zip_bytes` → `FeedbackZipPayload { file_name, bytes }`（GUI 内存导出与 CLI 落盘共用，字节一致）；`export_feedback_zip` 改为其消费方，CLI 语义不变 |
| `apps/browser/src/pages.rs` | `generate_feedback_html`：报告清单（文件名/角色/相对时间/大小）+ 每条删除链接（`zero://feedback/remove/<file>`）+ 导出动作（`zero://feedback/export`）+ 空态 + 隐私说明（包不含 URL/页面内容；会话摘要与 URL 注明走 CLI）；文件名 HTML 转义；双语（UiLanguage）。设置页隐私卡新增「崩溃报告 → 查看与导出」入口行 |
| `apps/browser/src/app.rs` | `open_feedback_page`（`list_reports` → 生成 HTML → `open_internal_list_page`）；`try_apply_feedback_url`（挂入 `try_apply_internal_url`；删除走 `delete_report` 信任校验；导出 = 全部报告 → `build_feedback_zip_bytes` → `save_attachment_download` 落盘下载目录，同名去重归下载层——对齐书签导出先例）；`load_local_tab_url` 挂 F5 刷新分派 |

**设计取舍**：GUI 面不提供目录选择器（无此基建）与 URL/会话摘要导出（NFR-1 隐私红线，页面注明走 CLI `--include-session-summary`/`--include-session-urls`）；导出目标 = 下载目录（设置页已有下载目录配置项），FR-5「确认目标路径」语义由页面明示 + 下载管理器回执承担。

## 2. 验收

### 真实进程（守卫 8G/300s，5 断言全 PASS）

| 腿 | 场景 | 断言 |
|---|---|---|
| leg A | GUI + `--profile` 启动 → SIGKILL 唯一 tab renderer → SIGINT | 真实报告落盘；GUI 进程正常启动/优雅退出（feedback.rs/pages.rs/app.rs 改动整体加载无回归） |
| leg B | 同 profile CLI `--export-format=zip` | GUI 导出动作与 CLI 共用 `build_feedback_zip_bytes`：zipfile 真解压深校验（CRC/成员逐字节/version-commit 与 `--version` 贯穿） |

### app 级（门禁 `cargo test -p zero-browser` 686/686 PASS）

- `feedback_page_lists_reports_deletes_and_exports_zip_via_download`：`ZERO_CONFIG_DIR` 隔离 + 真实 BrowserApp（CPU 渲染）——页面打开、列表渲染（报告名/删除链接/导出链接）、导出动作经下载管线落盘（PK 签名 + 下载管理器 Completed）、删除动作文件消失、穿越文件名被动作层消费但 `delete_report` 拒绝（目录内无误删）。
- `generate_feedback_html_*` 3 项：列表渲染/空态无导出/敌意文件名转义。

**排障记档**：app 级导出测试首轮将「导出落盘」断言为字节全等——两次 `build` 各取 `now()`，跨秒边界 DOS 时间字段必不同；改按成员名+内容比对（忽略时间戳字段）。同轮全 bin 套件出现 1 例 view-source 本地 fixture 网络失败与上轮 service worker 并发 SIGSEGV 同类（单跑必过、同树偶发，非本切片域）。

## 3. 门禁

fmt PASS / `cargo test -p zero-browser --bin` 686/686 PASS（含 4 新测试）/ `zero-browser-shell --lib` 371 PASS（feedback 12/12 × 3 稳定）/ release 构建（守卫）PASS / clippy `--workspace --all-targets -D warnings` / 全量 `make test`（test-guard）/ 验收守卫 `systemd-run -p MemoryMax=8G timeout 300` / lei-pre-commit-guard PASS。

## 4. 限制与挂账

- GUI 导出不带会话摘要与 URL（CLI 面 `--include-session-summary`/`--include-session-urls` 承担）；目录选择器仍缺（导出目标固定为下载目录）。
- RFC §4.4 GUI 入口项闭合；真机安装/签名验收仍待 P1/P4 授权。
