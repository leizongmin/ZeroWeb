# WebDriver 服务完善 — 运行时控制面板（master.md）

**入口文档**: [../webdriver.md](../webdriver.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-07（立项——M1 待启动）

---

## 当前状态

**专项定位**：`zero-webdriver` 从 9 endpoint 最小子集补齐到「驱动自动化验证够用」+
接线为 CI 可用验证基建。W3C 协议为准，每 endpoint 一切片（实现 + wire format 测试 +
HTTP 全链路测试）。

**与兄弟 goal 的边界**：
- rendering-compat — 渲染流域 crate 域零重叠
- event-loop-spec — **apps/renderer 属该流活跃域**；本流只碰 renderer 的 Automation
  消息处理段，不碰事件循环/observer tick 段；发现要碰即暂停记入本表（碰头信号，run-rules §9）
- keyboard-* / editing-contenteditable — 本流为其提供验证通道能力，不替它们写用例
- 共享面：crates/protocol（Automation 消息族）、apps/renderer（处理端）——碰之前
  `git log --since="14 days ago" -- crates/protocol/ apps/renderer/` 核对

## 实测基线（2026-09-07 立项时）

### 现有实现（9 endpoint）

- ✅ POST /session（create_session，spawn zero-renderer 子进程）、DELETE /session/{id}
- ✅ POST /session/{id}/url、GET /title
- ✅ POST /element（find_element）、POST /element/{ref}/click、POST /element/{ref}/value
  （send_keys）、GET /element/active
- ✅ POST /execute/sync
- ✅ 基建：零依赖 HTTP（loopback-only、默认 9515）、元素引用映射 4096 上限、
  parse_webdriver_keys（修饰键/特殊键）、错误码 no such session / no such element /
  stale element reference → 404
- ✅ 集成测试：tests/http_session.rs（真实 TCP 全链路：New Session / Navigate / Get Title /
  Delete Session）

### 缺失（25+ 类）

- ⚠️ 会话与状态：GET /status、GET /session/{id}、timeouts
- ⚠️ 导航族：GET /url、forward、back、refresh
- ⚠️ 元素状态族：find_elements、text、rect、enabled、selected、attribute、property、css value、clear
- ⚠️ 执行族：execute/async、GET /source
- ⚠️ 窗口/截图：window 全族、screenshot（能力待评估）
- ⚠️ 接线：Makefile/CI 零命中（唯一消费者是自身集成测试）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | CI 接线（集成测试入 CI） | ⬜ M1 |
| P2 | W3C 兼容性清单基线 | ⬜ M1 |
| P3 | 会话/导航族 endpoint | ⬜ M1 |
| P4 | 元素状态族 + 执行族 endpoint | ⬜ M2 |
| P5 | 窗口/截图能力评估 + 兄弟流验证通道文档 | ⬜ M3 |

## 下一步计划

1. **M1 切片 1**：W3C 兼容性清单基线（对照规范全集逐项标注——纯文档）
2. **M1 切片 2**：GET /status + GET /session/{id}（不碰 renderer，最薄切片）
3. **M1 切片 3**：导航族（Automation 消息按需扩展）+ CI 接线

**碰撞管理**：碰 protocol/renderer 前 `git log --since="14 days ago" --
crates/protocol/ apps/renderer/` 核对 event-loop-spec 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 门禁就位 + 会话/导航族 | ⬜ 待启动 |
| M2 — 元素交互族 + 执行族 | ⬜ |
| M3 — 窗口/截图评估 + 接线收尾 | ⬜ |

## 待用户决策

| 项 | 状态 | 说明 |
|----|------|------|
| screenshot 链路 | ⬜ 评估中 | M3 摸底 renderer 截图能力后定 |
| alert 全族 / print | ⬜ 排除 | 依赖 host 对话框/打印能力 |
| actions 完整语义 | ⬜ 排除 | 依赖输入管线深化；keyboard 够用子集先做 |

## 验证基线

- 测试基线：立项时点全绿（`make test` 入口，经 test-guard 包裹；禁止裸跑 cargo test）
- W3C 兼容性清单：无基线（M1 建立）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
