# Web API 第二批 — 运行时控制面板（master.md）

**入口文档**: [../web-api-batch2.md](../web-api-batch2.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-23（M1 收口：两 corpus 导入 + 基线 + 基建修复）

---

## 当前状态

**专项定位**：M12 余面收编——Clipboard API + Fullscreen API 语义落地，WPT 两 corpus
为验收标尺。DnD 排除挂账（宿主拖拽输入管线深依赖）。

**与兄弟 goal 的边界**：
- security-hardening — 权限语义供数关系（本 goal 最小权限查询面，其 DC-4 落地时对齐）
- rendering-compat — `:fullscreen` 样式面跨域记账，`crates/style-system` 不碰
- cdp-protocol / devtools — 无协议面耦合
- android-browser / desktop-browser — 全屏窗口语义触 host-runtime 平台面时 git log 核对

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | clipboard-apis / fullscreen 两 corpus fetch 脚本 + 导入 + 基线 | ✅ M1（2026-09-23） |
| P2 | navigator.clipboard 四方法 + ClipboardItem/Clipboard 面 + ClipboardEvent + 内存后端 | ⏳ M2（基线 21.1%，主簇 = ClipboardItem ×13 案 + Clipboard 接口 ×2 案） |
| P3 | 最小权限查询面（security-hardening DC-4 对齐点） | ⏳ M2（基线已测出 denied 拒绝缺失 ×3 案） |
| P4 | Fullscreen 事件/状态面 + viewport 联动 | ⏳ M3（基线 63.0%，主簇 = 事件 target/栈时序） |
| P5 | 平台剪贴板后端（host-runtime 能力评估）或差异记账 | ⏳ M4 挂账定稿 |

## 已完成切片

- **M1（2026-09-23）**：`fetch-clipboard-apis-subset.sh` + `fetch-fullscreen-subset.sh`
  （pin 315976933870）+ runner 双子命令（`testharness-clipboard-apis` /
  `testharness-fullscreen`，observers 式内容级 skip）+ Makefile 四目标（test-guard 包裹）
  + 基线 evidence ×2（clipboard 21.1% / fullscreen 63.0%，账本 88 行）。
- **M1 基建修复（WAB2-M1）**：engine `script_gen.rs` strict-eval 全局发布扫描补
  `async function`/`function*`/`async function*` 形态（clipboard-apis helper 全 async
  形态 → 24 案 ReferenceError 的根因；修复前口径 13.0% → 修复后 21.1%）。单测 +
  R147/R198/R201 邻接回归全绿。
- **立项基线事实修正**：引擎已有部分 fullscreen 语义（viewport 桥遗产）——fullscreen
  基线起点 63.0% 显著高于立项预期（立项记「无 requestFullscreen 面」）。

## 下一步计划

1. **M2**：ClipboardItem + Clipboard 接口面（主簇 15 案）→ blob 回读类型 → 权限
   denied 拒绝语义（P3，security-hardening DC-4 对齐）→ 内存后端口径复核
2. **M3**：fullscreenchange/error target 与栈时序簇 → promises 拒绝形态 → Timeout 8 案
   甄别 → viewport 联动验证（消费 ④ viewport 桥）
3. **M4**：DC 逐项判定 + 平台后端差异挂账定稿

**跨域记账（回流不越界）**：
- `:fullscreen` 伪类/UA 渲染样式面 → rendering-compat 流域（rendering/ 9 案不导入 +
  api 内伪类断言簇）
- iframe 依赖面（fullscreen 25 案 + clipboard detached-iframe 6 案）→ 重入 = iframe
  文档管道
- runner infra：testdriver `bless`/`set_context` 越白名单（11 案 Unsupported）、
  `/common/` 绝对路径 fetch（1 案）

**待用户决策清单**：
- （暂无）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 导入与基线 | ✅ 2026-09-23 |
| M2 — Clipboard 语义 + 后端 | ⏳ |
| M3 — Fullscreen 语义 | ⏳ |
| M4 — 收口 | ⏳ |

## 验证基线

- 基线 evidence：`evidence/2026-09-23-m1-clipboard-apis-baseline.{md,json}`
  （33 案 / 15-71 = 21.1%）+ `evidence/2026-09-23-m1-fullscreen-baseline.{md,json}`
  （55 案 / 92-146 = 63.0%）；账本 `imported-testharness.txt` +88 行（WAB2-M1-baseline）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  engine 面 js_dom_bridge R147/R198/R201/WAB2-M1 单测绿；全量 `make test` 门禁见
  M1 提交说明

**碰撞管理**：碰 engine shim 面前与 security-hardening / cdp-protocol `git log` 互核。
