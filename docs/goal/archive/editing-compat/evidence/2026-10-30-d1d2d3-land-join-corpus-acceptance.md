# D1/D2/D3 落地 + join-different 语料验收（2026-10-30）

前置：[ws-join 实现与管线阻塞](2026-10-30-ws-join-impl-and-pipeline-blocker.md)（D1/D2/D3 根因链与修复切入点）。

## 1. 落地清单

| 项 | 位置 | 内容 |
| --- | --- | --- |
| D1 标记剥离持久化 | part03.js `_zwSelStampParseTree`/`Subtree`/`CharData` + part04 hook | 解析文本节点 `data`/`nodeValue` 写经访问器路由：`__nv` 同步 + `SetChildTextByPath`/`RemoveChildAtPath` 变更入队（`parse_child_index_path` 全路径拒绝，`ZW_SEL_PATH_MUTATION` kill-switch） |
| D3 caret 重锚 | part01.js `_zwCeChainTop`/`_zwCeRemapEndpoint`/`_zwCeReanchorRange` | resync 后 live Selection `_ranges[0]` 持分离代理 → 注册表身份链上行走回 in-doc 顶点并写回 |
| 新缺陷（第 4 应用路径漏换代通知） | webview/user_actions.rs `apply_dom_script_result` | **根因**：execute_dom_script/用户动作脚本尾批 apply+换基座后未发 `__zw_apply_generation_bump`——base 已含 apply 结果而 `_zwApplyGeneration` 未换代，R380/R381 桶级代际门前提（「base 含 apply 结果 ⇒ gate 关」）被破坏：本脚本尾批入桶条目（CE join 壳 span）stamp === gen 判同 turn 继续并入融合视图 → **div1 双 span**（探针实证）。与其余 3 条应用路径对齐补 bump。 |
| R382 ghost 兜底 | part05.js 桶首登戳 `_zwPaGen`（first-only）+ `_zwOverlayPendingChildNodes` 跨代际 ghost 剔除 | 桶记账被 R322 查询物化重盖 stamp 后 R380 桶级门失效，按条目代际兜住：跨代际且 host 快照真含（`_zwHandleInHostSnapshot`）即剔出桶；年龄源 `_zwPaGen` 缺失回退 `_zwPaGen45`（slice45 全局表首登戳；探针实证 W1 仅携 `G453`） |

双修复并存的理由：bump 恢复 innerHTML 门（R381）不变式；R382 守的是 childNodes 融合列表（`_zwOverlayPendingChildNodes`）——该面无桶级 gate，同族 ghost（seen 恒 miss 的重同步前条目）只有条目级判定能拦。

## 2. 探针证据（真实管线，单变体临时页）

`zz-probe-join-one.html`（send 管线 + 三 promise_test + 故意 fail 携 dump；跑完即删）：

- 修复前（仅 D1/D3 + R382 前两笔）：`genAfterSend:3`（恒不换代）、`afterSend` = `abc<span…>def</span><span…>def</span>` **双 span**；kids = `[abc-text, SPAN(parsed,S), SPAN(H__n0,G453)]`——W1 壳已进 base 又被 overlay 并入。
- 修复后（+bump+年龄源回退）：`genSetup:0 → genAfterSend:4`、`afterSend` = `abc<span style="white-space:pre">def</span>` **单 span**；`rawNew` 同；W1（`_zwPaGen45=3 ≠ gen 4`）跨代际被剔出桶。caret `DIV@1`（join 后落 div1 内，精细落点归后续 caret 簇）。
- 对照 `zz-probe-append-only.html`（无 action 管线 append+读）：修复前后行为一致（单 X、gen 恒 0）——零回归信号。

## 3. join-different 语料验收（修复后二进制全量）

入口：逐文件 `zero-wpt-runner testharness-editing-other <file>`，`ZW_SCRIPT_TIMEOUT_MS=300000 ZW_CORPUS_CASE_TIMEOUT_SECS=60`，test-guard（per-proc 4G/total 8G/4200s）；test-guard 包裹、单文件 4.5-7.5min。

| 文件 | 修复前 | 修复后 |
| --- | --- | --- |
| join-different-white-space-style-paragraphs | 0P（全簇 7,518F 预期） | **531P** / 2,925F |
| join-different-white-space-style-left-paragraph-and-right-line | 0P | **72P** / 1,383F |
| join-different-white-space-style-left-line-and-right-paragraph | 0P | 0P / 2,607F |
| **合计** | **0P / 7,518F** | **603P / 6,915F**（总行数 7,518 不变，Pass 0→603） |

残余失败族（下一簇素材，非本轮范围）：

1. **inline run 合并语义**（paragraphs 文件主导）：`abc<b>[]def</b>` 型 join 后 `<b></b>` 空壳残留 + 后续内容被 normal-ws span 重包（期望并入 `<b>` run）。
2. **host 级 dst 插入位次**（left-line-and-right-paragraph 全 2,607F）：host 行 + 右块形态 join 后 span 落 div 之后（期望 `abc<span>def</span><div>ghi</div>`），单族 `assert_in_array`。
3. 块间 `\n` 文本节点（源格式换行）使 previousSibling 链命中非元素节点，join 几何退化为容器尾插。

## 4. 契约与回归钉（本提交新增）

- `execute_dom_script_path_generation_bump_call_site_present_d2`（webview.rs 源码钉）：第 4 应用路径 bump 调用点删行即红；判别子串取独立脚本形态（`();"` 结尾）不混计 s143 format! 前拼位点。
- `test_send_pipeline_join_mutation_queue_d1d2`（js_dom_bridge_tests/part03.rs）：turn1 标记剥离变更持久化 + turn2 join 变更应用无空 ws-span；视图层去重不在沙箱测试范围（沙箱手工换基序列与 runner 优先通道重同步包装体身份语义不同，实测不可迁移），由语料常驻文件 + 源码钉承担。
- `test_sel_parse_subtree_chardata_path_d1`：D1 访问器单元行为。

## 5. 验证

- `cargo fmt --all -- --check` 干净。
- clippy `-p zero-engine -p zero-webview --all-targets -- -D warnings` 零告警
  （修掉重复 `let mut handles` 声明——遮蔽合法但触发 unused_mut/unused_variable
  双告警，本轮杂音已清）。
- join 探针双页 + 语料三文件见 §2/§3。

## 6. 测试门禁补账

- **make test**（DISPLAY=:20）：workspace 腿止于 zero-engine lib
  **3617P/9F**——9 失败与已知红集逐一对应（fetch/Request/Response/
  iframe-service-worker 域），零新增；cargo test 无 `--no-fail-fast` 在首个
  失败二进制中止，renderer/browser/quickjs-test 腿未达（与既往轮次记账口径
  相同）。新测试在腿内确认：`test_sel_parse_subtree_chardata_path_d1` ok、
  `test_send_pipeline_join_mutation_queue_d1d2` ok（turn3 收窄后转绿）。
- **webview lib 定向腿**（guarded，make test 中止面之外）：822P/1F——
  `test_document_scrolling_element_r3259` **HEAD 上同样失败**（stash 验证）：
  scrollingElement 依据断言 compatMode 得 BackCompat 期望 CSS1Compat，预存红，
  非本轮引入；此前不可见因 make test 恒止于 engine lib。挂账 html-semantics
  域（doctype/compatMode 分支），本轮不扩 scope。新钉
  `execute_dom_script_path_generation_bump_call_site_present_d2` ok。
- **bench-gate**（ZERO_WEB_BENCH_CRATES=zero-engine 定向）：见 §7。

## 7. bench-gate 补账

`ZERO_WEB_BENCH_CRATES=zero-engine make bench-gate`：**GATE PASS**——19 指标
（NEW=7）全部在预算内（page/medium first_paint p95 324.84ms ≤362.35 预算、
peak_rss 192.52MB ≤359.02）。part05 融合列表逐条目分支 + 桶首登戳、part01
join 文本入壳复制路径无性能回退。
