# WebDriver Screenshot — 运行时控制面板（master.md）

**入口文档**: [../webdriver-screenshot.md](../webdriver-screenshot.md)
**创建日期**: 2026-09-09（用户拍板方案①——抽公共截图转换层 crate，从已归档 webdriver
goal 挂账项立项）
**最后更新**: 2026-09-09（立项 bootstrap：goal 契约 + master.md + 启动脚本；基线事实
以入口文档「基线事实」节为准，不重复抄写）

---

## 当前状态

**专项定位**：收敛 `PaintSnapshotParams → RenderPrimitives` 转换双份实现为公共 crate，
并落地 webdriver GET /session/{id}/screenshot。M1（抽取+收敛）→ M2（endpoint）→
M3（接线文档）。

**当前进展**：立项完成，M0 尚未开工。首个切片 = M1 切片 1（新 crate 骨架 + compositor
`convert.rs` 搬家）。

**与兄弟 goal 的边界**：
- rendering-compat — `apps/compositor/`、`apps/browser/src/paint_ipc.rs` 近期有该流
  图元语义提交（R4139/R4059）；本流做的是**结构搬家不改语义**，每轮开工前 `git log
  --since="7 days ago"` 核对两处，撞头即暂停该切片
- event-loop-spec — apps/renderer 活跃域；本流**零 renderer 改动**（只读 ViewPainted
  通道，webdriver spawn 已发 SetFramePublishMode(Legacy)）
- keyboard-* / editing-contenteditable / 其他 webdriver 消费者 — 本流提供截图能力，
  不替它们写用例

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 新公共 crate（转换 + 光栅化 + PNG） | ⬜ M1 切片 1 |
| P2 | compositor 切换消费 + 重复删除 | ⬜ M1 切片 1 |
| P3 | browser paint_ipc 切换消费 + 重复删除 | ⬜ M1 切片 2 |
| P4 | workspace/文档登记 | ⬜ M1 切片 3 |
| P5 | webdriver ViewPainted 消费 + ImageCache 累积 | ⬜ M2 |
| P6 | GET /session/{id}/screenshot + 全链路测试 | ⬜ M2 |
| P7 | 验证通道文档（视觉回归使用说明） | ⬜ M3 |

## 下一步计划

1. M1 切片 1：新 crate（建议名 `zero-paint-convert`，落 `crates/`）骨架 +
   compositor `convert.rs` 转换逻辑搬家 + compositor 切换 + 测试平移 → 验证：
   `cargo test -p zero-compositor` 全绿 + workspace clippy 零告警
2. M1 切片 2：browser `paint_ipc.rs` 切换 → 验证：browser 测试全绿 + grep 无重复函数
3. M1 切片 3：workspace 登记同步 → 验证：`make test` 全绿
4. M2：webdriver ViewPainted 消费 + screenshot endpoint（注意 sent_keys 去重：跨帧
   累积 ImageCache）

**碰撞管理**：开工前核对 `git log --since="7 days ago" -- apps/compositor/
apps/browser/src/paint_ipc.rs`（rendering-compat 流）与 `apps/renderer/src/{runtime,
lib}.rs`（event-loop-spec 流，只读不动）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 公共 crate 抽取 + 双端收敛 | ⬜ 未开工 |
| M2 — screenshot endpoint | ⬜ 未开工 |
| M3 — 接线收尾 | ⬜ 未开工 |

## 关键决策记录

| 决策 | 理由 |
|------|------|
| 方案①抽公共 crate（用户拍板 2026-09-09） | 双份维护实证：R4139/R4059 两笔渲染修复同批改 paint_ipc.rs + convert.rs 两处；收敛消除长期税 |
| 转换逻辑以 compositor `convert.rs` 为搬家基准 | compositor 主链路在用的真值；纯函数形态更易抽；browser 版有 TabSnapshot 副作用耦合，搬家后由调用方组合 |
| 新 crate 落 `crates/`（库形态），compositor/browser/webdriver 三端消费 | image-decoder 先例是 app 内 lib+bin；本层是纯库无进程入口，落 crates/ 更合工作区分层 |
| 不动 GPU 路径（gpu_raster.rs 留 compositor） | 公共层先收敛 CPU 转换+光栅化；GPU 是 compositor 专属主链路，无第二消费者 |
| 零 renderer/protocol 改动 | ViewPainted + SetFramePublishMode(Legacy) 通道已在（session.rs:118）；sent_keys 去重在 webdriver 层用跨帧 ImageCache 处理 |

## 待用户决策

（暂无）

## 验证基线

- 立项基线（2026-09-09，HEAD `23830995b`）：main 全绿（webdriver goal 归档时
  `make test` 66 套件 19037/0 + clippy 零告警）
- 双份维护实证：`git show --stat 552ab2701`（R4139）与 `8cfabf4f4`（R4059）均含
  `paint_ipc.rs` + `convert.rs` 双文件改动
