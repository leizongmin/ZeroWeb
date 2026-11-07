# M3 R3 变异应用臂：`resolve_slots_all_hosts` 批末统一挂接

**日期**: 2026-11-08
**切片**: 45 号 M3 R3（动态 slot 分配接线——变异应用臂）
**提交**: 3744043e2
**结论**: 批末统一重解析落地。命令式 attachShadow/childList/innerHTML/slot
属性改名后分配不再过期；无 shadow 页 O(1) 快返零开销。shadow-dom 矩阵与
R4 收口态逐位一致（零漂移），release 侧门禁全绿；debug workspace 腿因本机
多会话并发风暴未完成（挂账，非本 diff 失败）。006 归因改判：缺口实为
manual slot 分配通道（R3b 独立切片）。

## 1. 本片交付（`3744043e2`）

- `crates/dom/src/document/shadow.rs`：`resolve_slots_all_hosts()`——重解析
  `shadow_roots` 表内全部宿主的 slot 分配（嵌套宿主逐层各自解析，与逐宿主
  调用语义一致）；`shadow_roots` 空表 O(1) 快返（非 shadow 页零开销）。
  收口 `materialize_declarative_shadow_roots` 处 FIXME（「命令式 attachShadow
  与宿主 childList 后续变异不触发重解析」）。
- `crates/engine/src/js_dom_bridge.rs`：`apply_dom_mutations_full` 批循环
  结束后（handle_selectors 构建前）统一挂接——全部 JS 面 DOM 变异
  （append/insert/remove/innerHTML/attachShadow/setAttribute 族，含
  `__zw_append_child` 等回调烘焙路径）都经本函数批队列落地，单点覆盖。
  布局树 shadow 分支经 `get_assigned_nodes` 在下帧布局消费新分配。
  https://dom.spec.whatwg.org/#assigning-slots-and-slots
- `crates/dom/src/tests/tests_3.rs`：两个新单测——命令式 attach 后统一重解析
  补齐分配；初始解析后 childList 追加 + slot 属性改名的重分配（旧分配清零
  重收集、默认/具名档迁移）。
- 附带：`runtime_flags.rs` 删除 R4 翻默认遗留孤儿 `opt_in`/`opt_value`
  （dead_code 2 处——R4 收口轮 make test 在 workspace 腿 Error 101 早退，
  clippy 腿未跑到，属潜伏门禁失败，本轮预检捕获）。

## 2. 门禁账（2026-11-08）

**绿（release 侧，diff 已编入 runner）**：
- `make reftest` 全量本地 725：725/725 零回归。
- `make reftest-upstream FILTER=shadow-dom`：32P/3F 与 R4 收口态逐位一致
  （004 ✓0.00% 保持、006 4.90% 不动、可信档 9、F 面恰 3 案具名挂账）——
  变异臂对既有通过面零漂移。语义受益面：layout-slot-no-longer-assigned /
  -fallback（此前靠「命令式 attach 后从未分配」的假通过；现走真动态语义
  同样通过——分桶未变但理由坐实）。
- 新单测 2/2（test-guard 包裹 `-p zero-dom --lib`）。
- `cargo fmt --check` / `cargo clippy -p zero-dom -p zero-engine
  -p zero-layout-engine --all-targets` 干净。

**挂账（debug workspace 腿 + 渲染产品面）**：
- `make test`（workspace debug 腿 + renderer/browser 腿）：三连跑均死于
  本机基础设施竞态，非测试失败——① incremental dep-graph 竞态 Error 101；
  ② 进程树外部 Terminated（Broken pipe）；③ test-guard 重编译竞踩
  Error 127（每次 `make test` 前置 rustc 重编 test-guard 同路径，并发
  `make test` 实例互踩）。根因：本机 19 个并发 agent 会话双 spawn 风暴
  （时间戳证据 /tmp/zr3-r3-arm-make-test-*.log 多实例重叠），与本 diff
  无关。机器余量恢复后补跑。
- `make product-smoke`、`make bench-gate`（定向 zero-dom/zero-engine）：
  同因未跑，挂账。

## 3. 006 归因改判（推翻本切片前置预期）

`slot-fallback-content-006` 的缺口**不是**命名 slot 动态重解析：
该案用 `attachShadow({slotAssignment:'manual'})` + `HTMLSlotElement.assign()`
手动分配通道。JS shim 侧语义完备（`__zwManualAssigned` 列表 + 抢占 +
slotchange + 跨 shadow 面），Rust 渲染侧零消费：
- `DomMutation::AttachShadow`/`OnHandle` wire 不携带 slotAssignment；
  `attach_shadow_full` 无该参数，`ShadowRootData.slot_assignment` 恒 Named；
- `assign()` 无 DomMutation 变体、无 Document 侧表；
- 渲染消费（`get_assigned_nodes` → 匿名叶/回退分支）不感知 Manual 模式。

→ R3b 切片：wire 携带 slotAssignment + `assign()` 桥 + Manual 模式下
resolve_slots 跳过 + 连通性过滤（已移除 slottable 不渲染、回退生效）。

## 4. 多会话并发注记（rally-double-spawn-race 实录）

本切片 diff 由两会话并行推进：本会话实现+单测+release 门禁全绿后转后台
make test；并行会话采认 staged 工作树先手提交推送（`3744043e2`，内容与
本会话 diff 一致）。goals 文档与 evidence 由本会话补记。教训归档：
同 clone 双 spawn 时 staged 工作树随时可能被采认提交，长门禁先行提交
（先手早提交避碰）再跑门禁。

## 5. 下一步

1. **R3b manual slot 分配通道**（见 §3）。
2. 挂账补跑：make test debug 腿 + product-smoke + bench-gate 定向。
3. B6 补充池通道。
