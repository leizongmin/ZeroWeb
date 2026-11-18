# selection-face 分片零页执行根因定谳 + runner selection 通道 filter 竖线支持修复

日期：2026-11-17（20:0x 收账）
归属：CI 执行面修复（runner 通道语义），非 css 码面——不入五 corpus 账；css corpus 通道（testharness_css_corpus）不经 selection 通道，零影响。

## 现象

weekly run 44017408145（11-17）聚合守卫拦快照回写：`selection: 34041→0（−34041，跌幅 100%）`。coverage 工件对比：selection 组 covered_files 90→0（执行零落盘，非断言回归）。

## 诊断链

1. **双工件形态**：zw-results-selection-face 工件 171 字节（`testharness-selection.json` = 3 字节 `[]` 空数组）；zw-results-selection-ws 13KB（423KB 真内容）。同 run 同套件（suite=selection）两片一死一活。
2. **job 133818637747（selection-face）日志**（`gh run view --job` 逐行）：
   - fetch 步正常：`fetch-wpt-selection`（23 号 lib.sh deep 拉取）+ `fetch-wpt-editing` 双拉，语料盘点 `selection 182 案` 在 wpt-data 本地（fetch 耗 ~48s）——**语料不缺**；
   - Run suites 步：`./target/release/zero-wpt-runner "testharness-selection" $filter --json` 在 **0.28 秒内**退出非零（`##[warning]testharness-selection 退出码非 0`），JSON 落 `[]`——「用例集为空 → 1」（main.rs 通道文档明文）＝零匹配空真绿形态（[[runner-page-path-wpt-data-relative]] 同族，此处为 filter 语义形）。
3. **活片对照**：selection-ws 片 filter=`editing/whitespaces`（单字面子串）→ 423KB 真账；selection-face 片 filter=`selection/|editing/other|editing/event.html|editing/plaintext-only`（竖线联合）→ `[]`。唯一差异＝filter 串形。
4. **runner 语义分通道法证**（grep 全量）：
   - `testharness_editing_corpus.rs:123`、`test262.rs:506`：`filter.split('|').any(|part| relative.contains(part))`——**竖线联合任一命中**（editing-run 11 分片 pipe 组 fb30d4f08 的实现惯例）；
   - selection 通道两收集器（`testharness.rs` `collect_selection_flat_dir_cases` / `collect_selection_dir_cases`）：单字面 `relative.contains(filter)`——竖线串永不命中任何路径。
5. **断点提交**：`ef429d0e9`（11-15 02:50，whitespaces/plaintext-only/event 语料接入 selection 通道）给 selection-face 写了竖线联合 filter（沿 editing-run 惯例），但漏了给 selection 通道补竖线支持 → 该提交起 face 片恒零匹配恒 `[]`。11-14 快照（covered_files 90 / passes 34041）为断点前末代基线；44017408145 为断点后首个 weekly run。聚合守卫正确拦截（环比回归守卫职能兑现——快照未回写 main）。

## 修复（两文件四行）

- `tests/wpt-runner/src/testharness.rs`：两收集器 filter 条件补 `split('|').any`（editing 通道同款一行式）。单子串行为不变（split 无竖线串得单元素，向后兼容）——ws 片与本地 `make testharness-selection FILTER=<子串>` 用法零漂移。
- `tests/wpt-runner/src/main.rs` + `run_selection_cases` doc：通道文档同步竖线语义注记。
- 配置面零改动：ci-shards.yml 的 face 片 filter 在修复后即回归作者本意。

## 验证（verify-selfix-filter-ab.sh，flock 单写者 + pgrep + crates/ 脏检查放行 es_module + test-guard 包裹）

生产形 filter 双备选探针（recursive + flat 各一页）：
`selection/collapse-00.html|editing/other/typing-space-in-editable-summary.tentative.html`

- **A 腿**（修复前 bin，bin-selfix-prefix `5005a82d4b4549c04107f92de86890d2` = j23r3 19:42 HEAD 态构建）：**pages=0，rc=1**——CI 零匹配形态本地复现 ✓
- **B 腿**（修复后 bin `362918276b610ebeb149cfd40a412a59`，20:07:31 构建 48.78s）：**pages=2**——`selection/collapse-00.html` **2655/2655 pass**（recursive 收集器 ✓）、`editing/other/typing-space-in-editable-summary.tentative.html` 0/4（flat 收集器 ✓ 页命中并执行；该页本身为兼容缺口，0 过预期内非本修复面）。竖线联合匹配双收集器实证，修复闭环 ✓

## 恢复预期

修复入 main 后下轮 weekly selection 组恢复入账：face（selection/ 递归 + editing/other + editing/event.html + editing/plaintext-only）+ ws（editing/whitespaces）并集 ≈ 前基线 34041 以上（ef429d0e9 同批扩面文件此前从未跑过，数字可升）。
