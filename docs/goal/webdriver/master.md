# WebDriver 服务完善 — 运行时控制面板（master.md）

**入口文档**: [../webdriver.md](../webdriver.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-08（M2 切片 1+2 收口——find_elements/source + 元素状态族全绿）

---

## 当前状态

**专项定位**：`zero-webdriver` 从 9 endpoint 最小子集补齐到「驱动自动化验证够用」+
接线为 CI 可用验证基建。W3C 协议为准，每 endpoint 一切片（实现 + wire format 测试 +
HTTP 全链路测试）。

**M1 已收口（2026-09-08）**：endpoint 9 → 16（+status/session readback/url/back/forward/
refresh/timeouts GET+POST）；CI 接线完成（v8 app tests + quickjs 矩阵均含
`-p zero-webdriver`）；W3C 兼容性清单基线落地（evidence/endpoint-matrix.md）。

**M2 切片 1+2 已收口（2026-09-08）**：endpoint 16 → 25。+GET /source、find_elements、
text/rect/enabled/selected/attribute/property/css value、clear。协议新增
`AutomationOperation::FindElements`/`ElementState`/`ElementClear` +
`AutomationResult::Elements` + `AutomationStateQuery`（wire roundtrip 测试同步）。
集成测试 8 个全绿。

**与兄弟 goal 的边界**：
- rendering-compat — 渲染流域 crate 域零重叠
- event-loop-spec — **apps/renderer 属该流活跃域**；本流只碰 renderer 的 Automation
  消息处理段，不碰事件循环/observer tick 段；发现要碰即暂停记入本表（碰头信号，run-rules §9）
- keyboard-* / editing-contenteditable — 本流为其提供验证通道能力，不替它们写用例
- 共享面：crates/protocol（Automation 消息族）、apps/renderer（处理端）——碰之前
  `git log --since="14 days ago" -- crates/protocol/ apps/renderer/` 核对

## 实测基线（2026-09-08 M1 收口时）

### 已实现（25 endpoint）

- ✅ 会话：POST /session、DELETE /session/{id}、GET /session/{id}（capabilities 回读）、GET /status
- ✅ 导航族：POST /url、GET /url、back、forward、refresh、GET /title
- ✅ 超时：GET/POST /timeouts（字段可选、部分更新、400 校验）
- ✅ 元素定位：POST /element、POST /elements（复数）、GET /element/active
- ✅ 元素交互：click、value（send keys）、clear
- ✅ 元素状态族：text、rect、enabled、selected、attribute/{name}、property/{name}、css/{name}
- ✅ 执行：POST /execute/sync、GET /source
- ✅ 基建：零依赖 HTTP（loopback-only、默认 9515）、元素引用映射 4096 上限 +
  history_epoch 跨导航守卫、parse_webdriver_keys、错误码映射（400 invalid argument /
  404 no such session·element·stale / 408 timeout / 500 unknown）+ W3C 错误包络 stacktrace
- ✅ 集成测试：tests/http_session.rs 8 个全链路用例（真实 TCP + 真实 renderer 子进程）+
  session.rs 3 + renderer automation 3 单元测试

### 缺失（M2 剩余 + M3 面）

- ⬜ 执行族：execute/async
- ⬜ 窗口/截图：window 全族、screenshot（能力待评估）
- ⬜ 定位策略扩展（link text / tag name / xpath——css 之外）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | CI 接线（集成测试入 CI） | ✅ M1（ci.yml v8 app tests + quickjs 矩阵） |
| P2 | W3C 兼容性清单基线 | ✅ M1（evidence/endpoint-matrix.md） |
| P3 | 会话/导航族 endpoint | ✅ M1 |
| P4 | 元素状态族 + 执行族 endpoint | 🟨 M2 大半收口（25/26 类；余 execute/async） |
| P5 | 窗口/截图能力评估 + 兄弟流验证通道文档 | ⬜ M3 |

## 下一步计划

1. **M2 切片 3**：execute/async（renderer 侧 async 脚本语义 + 完成回调轮询）
2. **M2 收口** → 转 M3：window handle(s) 评估（单窗口架构最小实现）+ screenshot
   能力摸底（renderer 有 frame 导出链路，评估经既有绘制管线截帧的可行性）+
   兄弟 goal 验证通道使用文档 → DC 全满足判定

**碰撞管理**：碰 protocol/renderer 前 `git log --since="14 days ago" --
crates/protocol/ apps/renderer/` 核对 event-loop-spec 流活跃面（apps/renderer/src/runtime.rs
有 keyboard-default-actions 流活跃提交——**不碰 runtime.rs**；automation.rs 2026-09-08
实测 14 天零提交，已安全扩展 FindElements/ElementState/ElementClear 三操作）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 门禁就位 + 会话/导航族 | ✅ 2026-09-08 |
| M2 — 元素交互族 + 执行族 | 🟨 元素族收口；余 execute/async |
| M3 — 窗口/截图评估 + 接线收尾 | ⬜ |

## 关键决策记录

| 决策 | 理由 |
|------|------|
| back/forward 历史边界 no-op 返回成功 | renderer 栈边界直接回 Ok 不发生加载；与 ChromeDriver 边界行为一致，W3C 未规定边界必须报错 |
| 会话默认超时 10s script / 15s pageLoad / 0s implicit（偏离 W3C 300s 默认） | 自动化驱动 fail-fast 更安全；既有集成测试依赖；记录于 SessionTimeouts doc comment |
| 元素引用加 webdriver 本地 `history_epoch` 守卫 | renderer back/forward 路径（`reload_history_entry`）不 bump `document_generation`，同 node handle 跨历史条目会被误判存活——本层保守判 stale（W3C 正确错误码），避免误触新文档元素；不碰 renderer（event-loop-spec 活跃域） |
| GET /url 用会话态而非 ExecuteScript location.href | 页面 shim location.href 可被脚本重写/不可靠；会话态由 renderer UrlChanged 消息维护（重定向跟随真值），零 renderer 改动 |
| refresh 走 Reload IPC + epoch+1 | renderer Reload 内部已带 `navigation_epoch.wrapping_add(1)`（runtime.rs:2381），与 navigate 语义一致 |
| timeout 错误映射 HTTP 408 | W3C §processing-model 错误码-状态映射表；此前 500 是 wire format 偏差 |

## 待用户决策

| 项 | 状态 | 说明 |
|----|------|------|
| screenshot 链路 | ⬜ 评估中 | M3 摸底 renderer 截图能力后定 |
| alert 全族 / print | ⬜ 排除 | 依赖 host 对话框/打印能力 |
| actions 完整语义 | ⬜ 排除 | 依赖输入管线深化；keyboard 够用子集先做 |
| renderer back/forward 不 bump document_generation | ⬜ 待决策 | 本层 history_epoch 守卫已兜底（保守 stale）；若要 renderer 侧根治需碰 runtime.rs（event-loop-spec 活跃域），记待决策不阻塞 |

## 验证基线

- 测试基线：M1 收口全绿（6 集成 + 3 单元，`make test` 入口经 test-guard 包裹；禁止裸跑 cargo test）
- W3C 兼容性清单：evidence/endpoint-matrix.md（16 endpoint 落账，随切片更新）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
- CI：v8 + quickjs 矩阵均含 zero-webdriver（ci.yml）
