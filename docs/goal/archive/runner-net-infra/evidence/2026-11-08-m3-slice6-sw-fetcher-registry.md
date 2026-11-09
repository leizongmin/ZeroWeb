# M3 切片六（6a）：SW script fetcher 注册表化——21 臂闭包链 → py_sw_fetch_routes 分表

任务文档 [runner-net-infra.md](../../runner-net-infra.md)；普查蓝图 [切片六普查档](2026-11-08-m3-slice6-fetcher-tables.md) §三。
基线锚 HEAD = d19247694（切片五 lint 收敛态），落库基线 HEAD = 9f2b7cdef（含 23:03 合流件 9afb87f69 webview s348——归因见 §五）。

## 一、交付面

- `tests/wpt-runner/src/testharness.rs`：`wpt_data_service_worker_script_fetcher` 闭包内 459 行 `let body = if…else` 链（21 端点臂 + 静态尾）拆除，替换为：
  - `PySwRoute { suffix, handle }` 行型 + `SwRouteHandler` fn 指针签名（外层 `Result` 承载臂内 `?` 直通语义——闭包返回型同形；内层 `Option` = 落穿，None → 续表 → 静态尾）。
  - `ServiceWorkerRouteCtx<'a> { root, state }`——案级上下文，Mutex 于闭包构造点创建、handler 经 ctx 借用（共享不可断，普查档 §二设计）。
  - `py_sw_fetch_routes()`：24 行注册表（21 handler / OR 对两行同 handler），表序 = 既有链臂序，新端点 = 注册项声明。
  - 闭包 dispatch 收口为「headers/ctx 构建 → 表循环（`ends_with` + let-chain）→ 静态尾」。
- 21 handler 全部 private fn 就地表旁（辅助依赖 `service_worker_fixture_*` 族原地复用）。
- 静态尾（非 SW 端点回退）留守闭包，逐字节未动。

## 二、机械等价变换（转换器 `/tmp/m3s6_sw_convert.py`，一次性工具不入库）

- **T1** dedent 8（臂体 12→4）。
- **T2** `state`→`ctx.state`、`root`→`ctx.root`——词边界 + 字符串字面量外（`_segments` 分段态机）；**guard 感知特判**：臂含 `let mut state = state` 守护绑定时，仅该行 RHS 指锁本体改写，其余 `state` 为 guard 引用不动。
- **T3** `return Ok(`→`return Ok(Some(`（多行收尾 `});`/`));` → `}));`/`)));`，组外 `;` 补齐），`return Err(` 不变。
- **T4** 落穿臂 2（mime-type-worker 空 body、import-scripts-404-after-update 读盘 body）手写按尾包裹同形重构（`std::mem::take(headers)`）。
- **T5** 未用参数 `_` 前缀——由 `cargo check` 逐 handler 裁决固化映射（格式捕包 `{clean}` 计为使用，故以编译器为准而非文本扫描；fn 指针类型不受参数名影响，承切片五 `_ctx` 先例）。
- **T6** fn 尾 `return EXPR;` → 尾表达式（承切片五 handler 收尾先例；体内嵌套 return 缩进 ≥8 不动；let-chain dispatch 承 testharness.rs:9609 先例）。

## 三、等价性证据

- **验证器** `/tmp/m3s6_sw_verify.py` ALL PASS：A——19 早答臂 handler 体 vs 原臂 T1-T6 变换后 token 序全等 + 2 落穿臂共享段 needle 断言；B——24 行表 suffix 序 = 链序扁平序；C——循环在场 + 链区无残留 else-if + 静态尾 token 全等。
- cargo fmt --check 净；cargo check 0 警告；clippy `-p zero-wpt-runner --all-targets -D warnings` 净（systemd scope 包裹）。
- runner 单测：298P/0F（`test-guard --compile-first` 定向，终态源码实跑）。
- 全量 A/B：net-api 双跑 multiset 空集（§五）+ SW-core 双跑 multiset 空集（§四协议、§五实录）。

## 四、A/B 协议修正（普查档 §三.2 的一处计划缺陷，随证据档订正）

普查档原案「SW 面（bytecheck、update-worker、import-scripts-*、redirect.py）四域锚从 net-api A/B 日志提取」——实测 **net-api 套件不行使 SW fetcher 臂**（锚日志四域提取全 0，唯 redirect 652 行属 fetch/api/redirect 面，与本批臂无关）。SW 面行为验证改由 **`testharness-service-workers-core` 双跑**承载（干净基线 worktree @9f2b7cdef vs 主树含本批，同资产根，multiset 空集）；net-api 双跑保留为 py 路由面零扰动回归。

## 五、基线锚漂移归因（合流扰动收账）

- 锚 `/tmp/m3s6-base-224834.log` 录于 22:48–22:56 @ d19247694；B 跑 @ 9f2b7cdef——期间落库代码件仅 `9afb87f69`（webview s348 模块包装级 parse 失败记录归属，其提交语预告 choice-of-error-1/compilation-error-2 双案 F→P）+ 纯 docs 件（c084c49bc/4b99267f5/9f2b7cdef）。
- 准则：net-api A/B 差集若含且仅含预告双案 F→P → 归因 9afb87f69，本批零行为增量成立；其余任何差集行 → 本批引入，硬门不过。
- SW-core 对比双侧同 HEAD（worktree A 与主树 B 均 9f2b7cdef），差集应为纯空集。
- 实测结果（net-api，`/tmp/m3s6-base-224834.log` vs `/tmp/m3s6-b-netapi-232521.log`）：**计数敏感 multiset 空集**——P=13,614 / F=1,305 / T=8 双侧全等（分母 14,927 一致），四域锚全等（wss 741=741、S3b 016 2=2、S12 10=10、inspect-headers 70=70）；预告双案不在本套件，零归因需要，9afb87f69 离面确认。
- 实测结果（SW-core，worktree A `/tmp/m3s6-swcore-a.log` @9f2b7cdef 干净基线 vs 主树 B `/tmp/m3s6-swcore-b.log` @9f2b7cdef+本批，同资产根 `.service-workers-tier-a-root`，双侧 test-guard 同形）：**multiset 空集**——246P / 3F / 0T 双侧全等（3F 预存失败双侧同形）；页面直方图双侧逐项全等，本批臂行使面在账：update-bytecheck 8+8=16、import-scripts-mime-types 23、update.https 7（含 mime type invalid / malformed / redirect 子测）、update-registration-with-type 7、registration-script-url/module 面在场。SW 资产为本 clone 首次拉取（45 波 fetch 全绿，exit 0）。

## 六、过程记录

- 转换器四轮迭代全部是**双向同错盲区**（验证器与转换器共享 transform_arm，同错互证不拦）：① T2 首版把字符串字面量内 `state` 也改写（"…lock is poisoned" ~15 臂全中）→ `_segments` 分段态机；② guard 绑定行 LHS 被改写（`let mut ctx.state = ctx.state` ×15 编译错）→ guard 感知 t12；③ pending 收行正则组 `(…\))` 只捕 `})`、组外 `;`，拼装公式产出 `}))` 丢分号——`return EXPR` 无分号恰为合法块尾，**编译照过**，直至 clippy `needless_return` 才暴露 → 公式修正 `g2 + ");"`。结论再证：token 级验证不能替代编译器 + clippy 终验。
- B 跑三次 systemd service 形态修正：transient service 不继承 shell cwd（显式 `cd`）；service 环境无 cargo PATH（`source $HOME/.cargo/env`）；A 锚真命令为 `make testharness-net-api`（首版 B 误用 `make test`，日志形态完全不同——js_dom_bridge 腿早夭，作废重录）。
- 主树 `make test`（作废 B 跑）暴露 engine js_dom_bridge 7 案失败（websocket event flow / img settle / fetch filtered matrix 等）——与本批无关（wpt-runner 变更不进 engine 依赖向），疑涉 9afb87f69 或并发实例在飞件，不在本批范围，已留观不入账。
- SW 波资产（45 fetch 目标）本 clone 缺失，本批首次拉取（gitignored 数据面）；A/B 双侧共用同一资产根，缺失件双侧同报不计差集。
