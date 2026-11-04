# M2 s277 — below/in-viewport-dynamic 收口（曾-lazy once 口径 + 解析期在飞 fetch 存活 + noBlock 派发面后移，shim 2 文件 5 刀）

日期：2026-11-04。切片号 s277（承 s276 下一步候选①：below/in-viewport-
dynamic Chrome-only 口径——s272 挂账延续）。shim 2 文件 5 刀（engine
单车道，零 runner 改动、零语料改动）。

## 勘根

1. **红面定谳**：`image-loading-lazy-below-viewport-dynamic.html` /
   `image-loading-lazy-in-viewport-dynamic.html` 双页同构断言面——window
   load listener 记 `has_window_loaded`，img onload `step_func_done` 断言
   `has_window_loaded`（「window load 先发、img 后至」）。body script 解析
   期 `img.loading = 'eager'`（runner 无 trickle 时早于静态 settle）。r1 红
   签名 = `assert_true: The window load event should have fired before ...
   got false`——settle 非 lazy 分支落穿 host 快照同步派 load（早于 window
   load）。
2. **刀 A 断链**：初版刀（mutation 臂登记 `_zwLazyEver` + settle 曾-lazy
   跳过）solo 仍红——part05 IDL set trap 汇流点传 `_zw5277PreAttr`，而
   part04 trap 头预捕获集仅 src/srcset/sizes/media/type/crossorigin/
   referrerpolicy（R45 高频反射集），loading 不在集内 → IDL setter 路径
   preRaw=undefined → 登记 `preRaw=='lazy'` 恒假（双页均 IDL setter）。
3. **深勘探针实证装载悬空**（zz-s277-probe 验后删）：捕获集修复后红签名
   变 Timeout（pending:1）——dump `complete=false nw=40 fetching={}
   pend=null hold=[] evreg={#p:true}`。nw=40 非 fetch settle（
   `_zwImgNaturalDim` state 缺失 → attr eager → host 快照 `fetch_page_
   images` 回落），complete=false 直证 `_resourceStates` 无记录——
   **mutation 面启动的 fetch 从未完成**（无 state 写、无事件派、键册空）。
4. **根因（fetch 孤儿化）**：settle 重置块（part04 s224/s274 段）每轮置空
   `_zwImgDecodeEpoch5277` + `_zwImgFetching5277`；解析期 body script 的
   mutation run 启动 fetch 后，完成回调落点在 settle 后（trickle(d2) 慢
   流/异步链），代际表被清使 `_imEpochStale()`（`_ep[_imKey] !== _imEpoch`
   → undefined !== 1）把**本代**完成判 stale 丢弃（3341/3344/3371
   `_imInflightEnd(); return;`）——s224 跨案隔离语义误伤同文档在飞链。
   r1/r2/r3 全史自洽：r1 阻塞 fetch 孤儿 + settle 落穿同步派 → Fail；
   r2/r3 刀 B 拦落穿后仅剩孤儿 fetch → Timeout。
5. **第二缺口（派发序）**：pending 事件 drain（settle 尾）同步派发，位于
   window listener 之前（runner loadStep5275 settle → dispatchEvent 内
   6612 settle → 6675 listeners → 6677 after-load drain 序）——s273
   noBlock 只滤 `__zwImgLoadBlocked` 阻塞、不控派发序，no-trickle 时 lazy
   装载 load 事件早于 window listener（断言反向红）。
6. **第三缺口（bisect 定谳）**：持队若随 settle 重置清空，会被同文档第二
   轮 settle（dispatchEvent 内 6612）抹掉——6677 drain 前清空，
   zero-intersection-area / empty-src 两页 Pass→Timeout 实证（族 r1 见
   位移，bisect 关持队臂两页复绿）。持队生命周期须跟随其源
   `_zwImgPendingEvents`（惰性建、drain 清，不随 settle 重置——跨文档未
   drain 残留语义与 pending 队列既有面一致）。

## 施工（shim 2 文件 5 刀）

1. **part04 set trap 头预捕获集扩 loading**：`p === 'loading'` 入名门（
   IMG/SOURCE tag 门既有）——IDL setter 路径 preRaw 捕获写入前值。
2. **part03 loading mutation 臂登记曾-lazy**：preRaw 归一 'lazy' →
   `_zwLazyEver5277[_elKey]=true`（Chromium once-lazy 口径：曾处 lazy 态
   的图装载不阻 window load；已装载图守卫既有，restart 语义不动）。
3. **part04 settle 非 lazy 分支曾-lazy 跳过**：loading 动态 eager 化图不
   落穿 host 快照同步装载（防双通道——runupdate fetch 承接装载）。
4. **part04 settle 重置在飞存活**：置空前捕获旧在飞键册/代际表，旧册有账
   的键照搬代际账目与在飞登记（同文档在飞 fetch 完成不再误判 stale）；
   无在飞账的异文档残留照旧置空，跨案隔离语义不变。
5. **pending noBlock 事件持队 + window listener 后派发**：settle drain 对
   `noBlock` 事件改持 `_zwImgPendHold5277`（惰性建、不随 settle 重置、
   drain 清）；part03 `__zwImgDrainAfterWindowLoad5277` 顶部置持队 drain
   （6677 相位 = window listener 全部跑完后；error 型补 broken 终态与既有
   drain 同判据；置前于 after-load 队列 early return 防短路）。
   `_zwImgDispatch` 入队注释补派发面后移注记。eager 事件照旧同步派发
   （delay-load-event 阻 load 断言序不动）。

## 验证

- **solo**：双页 Pass（日志 s277-solo-below-viewport-dynamic.log /
  s277-solo-in-viewport-dynamic.log）。
- **族**（FILTER=image-loading-lazy，31 全键）：FIXED = below/in-viewport-
  dynamic 2 键，REGRESSED=∅，余 29 键零位移（日志 s277-family.log）。族
  r1 曾见 empty-src / zero-intersection-area 2 键 Pass→Timeout（持队被
  settle#2 重置抹除），bisect 定谳 + 持队生命周期修正后 r2 复归 Pass。
- **全树 A/B**（stash 法，同语料）：A/B 8348 行 1230 全键，FIXED = 双页
  2 键，REGRESSED=∅（日志 s277-fulltree-{a,b}.log）。
- **img 单测定向**：31 passed / 0 failed（含 s276 五钉
  img_lazy_viewport_gate / img_runtime_deferred_settle /
  img_sweep_selection_density / svg_img_settle_density /
  img_relevant_mutations）。
- **make test**：71 test-result，engine 3724 passed + 5 failed 既有同集
  （R-t2qq adopted-stylesheets 5 红——p56_adopted_snapshot_restore_hook /
  p99s_rule_style_preserves_custom_property_case /
  stylesheet_add_rule_remove_rule_r3276 / stylesheets_write_r2809 /
  stylesheets_rule_style_r2810）；clippy -D warnings 零 error；
  `cargo fmt --all --check` 干净（日志 s277-maketest.log）。
- 探针页 zz-s277-probe（验后删）。

## 挂账更新

- below/in-viewport-dynamic **双页收口销账**（2 文件 2 子测 Fail→Pass）。
- 承前不动：multicol 窗序反向面（解除面 = runner trickle/async 慢流时序）
  + multicol 上游主干状态待补查（wpt.fyi 三轮网络超时）；multiple-times
  主干双红销案维持；runner 全量钉内存大户二分（s270 挂账①）；root
  margin 净速分档（s271 挂账①）；dynamic import() referrerpolicy 穿参；
  decode() 兜底读点裸 URL 元组细化面（s276 注记）。

## 下一步候选

1. 型 D 余红（multiple-times 主干双红）复核销案口径。
2. multicol 上游主干状态补查（wpt.fyi API 恢复时）。
3. runner 全量钉内存大户二分（s270 挂账①）。
