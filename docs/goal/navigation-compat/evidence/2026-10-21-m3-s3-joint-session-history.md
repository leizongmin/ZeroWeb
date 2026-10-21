# M3-S3 联合 iframe session history 最小面（2026-10-21）

M3 第三切片：S2 记录的推进位移案 srcdoc_change_hash（step 3/4 父窗
history.back()/forward() 驱动子窗片段状态）收口——**the-iframe-element
27P→28P（+1 案翻绿）**，srcdoc_change_hash Timeout→Pass。两处 shim 变更，
零 runner 产品面变更。

## 实现

- **联合序列独立结构**（part02，声明于 `_hist_cursor` 旁）：`_hist_jointSeq`
  （标记两形：`{top: <_hist_entries 索引>}` / `{frameKey, url}`）+
  `_hist_jointCursor`。**null = 无子同文档条目**——惰性播种前全部路径与既有
  行为逐字节相同（M2 收口面零漂移的关键约束）。设计约束兑现：顶层
  `_hist_entries`/Navigation records **不膨胀**（Navigation entries() 只覆盖
  顶层自身条目——spec
  [joint session history](https://html.spec.whatwg.org/multipage/nav-history-apis.html#joint-session-history)
  联合序为遍历面概念，非顶层账本；测试断言 history.length 播种前后恒 1）。
- **播种与登记**：`__zwPushJointHistoryEntry(frameKey, url)`——首个子条目
  push 时 1:1 镜像当期顶层列表（cursor 对齐），此后前向截断 + 追加 + 当前位
  随移。两处顶层 push 站点（pushState `_histApplyNav`、anchor/href/assign
  `_pushHistNav`）在 joint 在场时镜像追加 `{top}` 标记（`_jointPushTop`）；
  `_histTrimOldest` 50 上限镜像出列（顶层标记索引同步前移）；跨文档重置
  `__zw_reset_history` 清联合序。
- **联合遍历路由**：`_hist_applyTraversal` 变路由入口（joint 在场 →
  `_hist_applyJointTraversal`，否则原体更名 `_hist_applyTopTraversal` 零改动）。
  frame 标记 → `__zwApplyIframeHashEntry`（URL 槽 + 子窗异步 trusted
  hashchange；父窗不派 hashchange——顶层 URL 未变；spec hashchange 派条目
  所属 browsing context）；顶层标记 → 同步顶层 cursor 后复用既有顶层面
  （navigate/popstate/滚动恢复面全保留）。入队可达性快照按联合序界
  （`_hist_queueTraversal` atEnd）。ctrl 结算镜像顶层非 intercept 提交块。
- **子写侧合流**（part05 `iframeLocation` hash setter）：URL 槽 + hashchange
  派发改经 `__zwApplyIframeHashEntry`（与父遍历落地同一路径，oldUrlHint 保
  oldURL 取写前地址）；子窗自身会话条目 push 维持（子窗 back/forward 面）；
  登记联合序。S2 测试面（part63 m3s2 子 hash 案）逐断言不变，证实合流等价。

## FIXME 记录（最小面边界）

- `FIXME(history.length)`：joint 在场时父窗 history.length 仍报顶层长度
  （spec 为 joint 长度）——M2 长度断言面零回归优先。
- `FIXME(joint-popstate)`：spec 联合遍历还在父窗派 popstate——最小面不派。
- `FIXME(joint)`：顶层标记落地遇 preventDefault/抢占时联合序不回滚。
- 父驱动遍历不动子窗自身 history 列表（子局部 back/forward 不反映父遍历）。

## 验证

- 集成测试：`js_dom_bridge_tests/part63.rs` 新增
  m3s3_parent_back_forward_routes_child_hash_entries（子连写 #1/#2 → 父
  back 落 #1 + 子窗 trusted hashchange → 父 forward 落 #2 + 父窗零
  hashchange + history.length 恒 1）1/1 过；全量 `cargo test -p zero-engine
  --lib` **3354/3354**（含 S2 三案不变——写侧合流等价性）。
- 域内：`make testharness-navigation FILTER=the-iframe-element`
  （TIME_LIMIT=2700）**28P/32F/27T**；对 S2 域内账本逐路径 diff **恰 1 路径
  变化** = srcdoc_change_hash Timeout→Pass，其余 71 路径恒等。
- 全量 corpus：`make testharness-navigation` **460P/69F/51T**（580 案行）对
  S2 全量（459P/69F/52T）逐路径 diff：428 路径中**恰 1 路径变化**（同上），
  **其余 427 路径逐案恒等**（含 navigation-api 5 案既有 Fail 面与 M2 全部
  断言面——null 惰性播种约束兑现）。
- fmt 干净；clippy -p zero-engine --all-targets -D warnings 零 warning。
- reftest 未跑：变更面（session history 记账与遍历路由）零布局/绘制触达
  （S1/S2 同理由）。

原始输出：[2026-10-21-m3-s3-joint-session-history-full.txt](2026-10-21-m3-s3-joint-session-history-full.txt)
（全量）、[2026-10-21-m3-s3-joint-session-history-filtered.txt](2026-10-21-m3-s3-joint-session-history-filtered.txt)
（域内）。
