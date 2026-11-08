# M4 复测收口：DC-4 数值门逐批 + B6 门值回填 + rendering-compat 零回归

**日期**: 2026-11-08
**切片**: 45 号 M4（append sel 通道修复落地后的复测收口）
**提交**: 本链无代码改动——纯复测收口 + 落账（文档提交行见 git log）
**树**: `03900b511`（146e8252c append sel 修复 + 渲染流 R5460 + editing 流
docs 落账的组合态，pull --rebase 后净树 dirty=0）

## §1 DC-4 数值门逐批（矩阵 33P/2F，双跑一致）

矩阵 `make reftest-upstream FILTER=shadow-dom`（rc=2 = 在册 2F 预期形态）：
链 B 10:21:06 与链 A 10:21:09 独立两跑 P/F 逐位一致。逐批映射：

| 批 | 案面 | 结果 |
|---|---|---|
| B1 slot 回退/分配（8） | slot-fallback-content-001..008 | **8/8 ✓**——004 0.00%（M3 翻绿保持）、**006 0.07%（append sel 通道修复翻绿保持）**、余案 0.00-0.65% |
| B2 布局失效/静态分配（5） | imperative-slot-layout-invalidation-001（0.07%）、invalidate-sibling-different-slots（0.00%）、layout-slot-no-longer-assigned / -fallback（0.00% 恒白守卫）、manual-slot-assignment-no-wrong-unassign（0.00% 恒白守卫） | **5/5 ✓** |
| B3 样式失效/继承（3） | directionality-001（0.59%）/ 002（0.00%）✓；shadow-style-invalidation-vw-units ✗ 2.08% | **2/3 + 1 具名挂账**（vw-units 归因 iframe 载体面，M1 evidence §4，具名不变） |
| B4 `:host` 伪类族（3） | focus/focus-pseudo-on-shadow-host-1/2/3 全 ✓ 0.00% | **3/3 ✓**（M1 翻绿保持） |
| B5 交互耦合（1） | nested-hover-pseudo-class-removal ✓ 0.00% | **1/1 静态面 ✓**（真交互面具名挂账不变） |

**DC-4 = 19/20 全绿 + vw-units 具名挂账**。邻域 selection/shadow-dom
**9/9 不回落**（cross-shadow-boundary-1..6 / -img / -select-document /
-select-root，0.00-0.22%）；他域误中 display-contents-shadow-dom-1 ✗ 1.97%
在册（非本池分母，css/display 面）。

## §2 rendering-compat 全量零回归

`make reftest` rc=0，**725/725**（Layout 521/521 + Text 204/204；链 B
10:21:29，链 A 10:20:55 同账双跑一致）。

## §3 B6 门值回填（M0 §4 挂账项闭环）

补充池 10 案 101 子测 **15P/86F 零位移**：链 B `testharness-shadow-dom`
输出 86 条 `Fail` 逐行与基线 `zr3-b6-baseline-082500.log` 排序 diff 零差异
（`diff` rc=0）。门值落定：

- 几何命中簇（elementFromPoint 41F / caretPositionFromPoint 14F /
  highlightsFromPoint 4F / offsetX-offsetY 3F）0 过——runner 无输入注入与
  布局几何查询管道，按 M0 具名挂账口径不变；
- 滚动/焦点面已过：focus-within-shadow 1P、scroll-restore-shadow 1P、
  scroll-to-the-fragment-in-shadow-tree 8P/2F；
- offsetParent 簇 12F/10P、offsetTop-offsetLeft 2P/1F、accesskey 2F
  （`pressAccessKey` helper 解析缺口，静态等价挂账）；
- 回池条件：webdriver 输入管线接 reftest/testharness 通道（不改）。

## §4 make test 账（legs10 规范入口 + legs11/11b 补腿）

- **workspace v8 腿**（legs10 规范 `make test`，head=146e8252c；→03900b511
  仅 docs-only delta，代码面等价）：**3815P/6F**——6F 与 R3b/R4 evidence
  在册名册逐名恒一（img_runtime_deferred_settle_r5275 /
  svg_img_settle_density_r5277 / test_fetch_filtered_response_type_matrix /
  test_iframe_service_worker_messages_dispatch_to_iframe_container /
  test_t8k_websocket_error_nonterminal /
  test_t8k_websocket_event_flow_open_message_close），net-api 外域在册债
  非本流域——该簇在册归因见
  [html-semantics m2s329 evidence](../../html-semantics-compat/evidence/2026-11-07-m2s329-canvas-zero-dim-to-dataurl-blob.md)
  （stash 法证「6 红全既有」，早于本流全部改动）；3814→3815 为测试自然
  增长，**零新增红**。fmt ✓、quickjs clippy 零错、standalone binaries
  构建 ✓ 同链出账（target/ 被磁盘守卫全量清空后冷重建 670 crate）。
  workspace 腿在册 6F 使规范 make test 结构性止于该腿（fail-fast），属
  html-semantics/engine 域既有账不属本 goal（不硬解，规则 9）。
- **product-smoke rc=0**（11b 10:23:01）：welcome 17.23% < 20 阈 +
  struct-check PASS + form-input-perf-gate PASS。
- **renderer bin/lib v8**：rc=0 / rc=0（11b 10:23:08 / 10:25:45）。
- **gpu adapter + consistency**：rc=0 / rc=0（链 C 10:23；11b 10:30 复跑同账）。
- **quickjs 三腿全绿**：crates rc=0（链 C 10:25:18 + 11b 10:32:37 双跑）、
  ren-bin rc=0（双跑）、ren-lib **370P/0F**（11b 10:38:42，362.71s）。链 C 的
  ren-lib 369P/1F（automation::evaluate_accepts_statement_sequences）与 11b
  同树 370P/0F 矛盾——其运行窗（10:25:57-10:32:29）与 11b browser 双腿
  （10:25:45-10:30:21，真浏览器进程 + Xvfb）重叠并发，判并发资源挤占污染，
  以 11b 干净单跑为准（同 §5 互踩判读）。
- **browser bin v8**：**742P/1F**（11b 10:27:28）——1F =
  `default_actions_work_without_javascript`（R5384 净 HEAD 恒红，对端
  zero-browser 面在册同型，零新增）。链 C 的 rc=127 为磁盘清空竞态
  基础设施伪红（test-guard 二进制瞬时缺失），不作账。
- **browser tests v8**：9 个集成测试二进制全 ok（含 cdp 三件套）；bin
  harness 随 `--tests` 重跑同 742P/1F 在册同型（rc=101 唯一来源即它）。
- **bench-gate（dom/engine 定向）**：rc=0（链 C 10:35:28 + 11b 10:40:08
  双跑一致）。
- **cdp_focus_governance 净树复核（R3b §6 挂账项②关闭）**：净树净 bin 跑
  **6P/0F**（18.82s；cdp_focus_governance.rs 恰 6 个 #[test]，与 M1 在册
  6P/0F 一致）——mtime 守卫拒跑形态消除，行为面确认无恙。

## §5 竞态与伪红判读记录（多实例 spawn 碰撞，2026-11-08 10:18-10:26）

上一 session rate limit 中断后 rally 驱动三重 spawn，10:18-10:22 产生 5 个
同任务收口链（1019/1020/1023/1025/1035）。处置：采纳 1035（M4 三件套，
matrix 先出账）+ 1023（product-smoke 起余腿补完）+ legs11-1035（余腿 +
bench），停止 1019（与 1035+1023 完全重叠）与 1020（同质）。

两个判读教训：
1. **磁盘守卫连锁清空**：1019 链 `make reftest` 入口 target-disk-guard
   检出 86GB 超阈全量清空 target/，吞掉 legs10 刚建好的 debug 工件，并使
   并发链 C 的 renderer/browser 腿 rc=127（`./target/test-guard` 瞬时缺失
   = 基础设施伪红，重跑覆盖即消）。
2. **同二进制并发互踩**：1020 与链 C 的 quickjs-crates 腿并发跑同一测试
   二进制（1020 rc=101 vs C rc=0）——并发同二进制双跑的失败面不作账，以
   干净单跑为准。

## §6 M4 收口判定

DC-4 条款逐项：20 池逐批不回落 ✓（19/20 + vw-units 具名挂账）；邻域 9/9
✓；rendering-compat 全量零回归 ✓（725/725 双跑）；B6 门值回填 ✓（15P/86F
零位移）；make test 全腿 ✓（legs10 workspace + 11b/链 C 余腿，红面全部
在册同型零新增）；头条不回落 ✓（slot-fallback-content-006 0.07% 保持
翻绿）。**M4 收口**，45 号 goal 里程碑 M0-M4 全部完成。

## §7 门禁链总账（legs10/legs11/legs11b 汇总）

| 腿 | 结果 | 来源 |
|---|---|---|
| fmt / standalone build / workspace v8 / quickjs-clippy | ✓ / ✓ / 3815P/6F 在册 / 零错 | legs10（146e8252c，代码面与 03900b511 等价） |
| DC-4 矩阵 | 33P/2F（双跑一致） | 链 A + 链 B（03900b511） |
| reftest 全量 | rc=0 725/725（双跑一致） | 链 A + 链 B |
| B6 门值 | 15P/86F 零位移 | 链 B（diff 基线复核） |
| product-smoke | rc=0 | legs11b |
| renderer bin/lib v8 | rc=0 / rc=0 | legs11b |
| gpu adapter/consistency | rc=0 / rc=0 | 链 C + legs11b |
| browser bin/tests v8 | 742P/1F 在册 / 9 集成二进制全 ok | legs11b |
| quickjs crates/ren-bin/ren-lib | rc=0 ×3（ren-lib 370P/0F） | 链 C + legs11b |
| bench-gate（dom/engine） | rc=0（双跑一致） | 链 C + legs11b |

logs：`.acceptance/zr3-appendfix-maketest-1001.log`（legs10）、
`zr3-m4-close-1035.log{,.matrix,.b6}`（链 B）、`zr3-m4-closeout-1020.log`
（链 A）、`zr3-m4-close11b-1023.log`（legs11b）、
`zr3-appendfix-legs11-1035.log`（链 C）。

## §8 对端移交验收（s331 closedby-complex，挂账待复核）

M0 记录的对端移交项（s331 `dialog-popover-closedby-complex` 18F shadow-slots
变体，勘根 = R3 动态 slot 分配缺位，归 M3 变异应用臂批一并验收）：本轮诊断
探针（`make testharness-html-semantics FILTER=closedby-complex`）在 runner
层秒败（`Using 8 test job(s)` 后 make Error 1，无逐案输出）——**取证无效，
非逐案红**，验收结论挂账待净态重跑；该案语料属 html-semantics runner 通道，
复核由该通道净态出账后回填。
