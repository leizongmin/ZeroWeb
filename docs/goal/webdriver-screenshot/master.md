# WebDriver Screenshot — 运行时控制面板（master.md）

**入口文档**: [../webdriver-screenshot.md](../webdriver-screenshot.md)
**创建日期**: 2026-09-09（用户拍板方案①——抽公共截图转换层 crate，从已归档 webdriver
goal 挂账项立项）
**最后更新**: 2026-09-09（M1 完成：paint-convert crate 落地 + compositor/browser 双端
收敛 + workspace 登记同步；下一步 M2——webdriver ViewPainted 消费 + screenshot endpoint）

---

## 当前状态

**专项定位**：收敛 `PaintSnapshotParams → RenderPrimitives` 转换双份实现为公共 crate，
并落地 webdriver GET /session/{id}/screenshot。M1（抽取+收敛）→ M2（endpoint）→
M3（接线文档）。

**当前进展**：**M1 完成（2026-09-09，R4181 三切片全落地）**——
- 切片 1（`3b04866eb`）：新 crate `crates/paint-convert`（`zero-paint-convert`），
  compositor `convert.rs` 删除、lib.rs/rasterize_tests.rs 切换消费；既有测试平移
- 切片 2（`0bcac5fc6`）：browser `paint_ipc.rs` 597→271 行切换消费，`apply_paint_snapshot`
  对外签名不变；重复 `ipc_*_to_*` 函数族删除（grep 零残留）
- 切片 3：README（32 members）/AGENTS.md 架构树/zero-web master.md 登记同步

**对照测试记账（DC-3 抽取行为对照）**：转换语义逐字段一致由以下既有测试构成对照——
compositor `rasterize_tests`（24，含 DSF 物理尺寸/脏区增量）+ 平移入公共 crate 的
glyph intern/冲突拒绝、glyph raster metadata + 新增 text_control_boundaries 断言
（公共 crate 4）+ browser `paint_ipc` 3 测（hit-test 还原/glyph source+variations/
image payload）+ browser bin 全量 413/413。全绿即对照通过。

**与兄弟 goal 的边界**：
- rendering-compat — 本流已完成的抽取是**纯结构搬家**（R4139/R4059 语义注释原样
  迁移）；后续该流对转换语义的修复只需改 `crates/paint-convert` 一处
- event-loop-spec — apps/renderer 活跃域；本流**零 renderer 改动**（只读 ViewPainted
  通道，webdriver spawn 已发 SetFramePublishMode(Legacy)）
- keyboard-* / editing-contenteditable / 其他 webdriver 消费者 — 本流提供截图能力，
  不替它们写用例

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 新公共 crate（转换 + 光栅化 + PNG） | ✅ M1 切片 1（转换已落地；光栅化+PNG 随 M2 webdriver 消费面抽入——rasterize 仍留 compositor，PNG 编码待 M2 按消费形态定） |
| P2 | compositor 切换消费 + 重复删除 | ✅ M1 切片 1 |
| P3 | browser paint_ipc 切换消费 + 重复删除 | ✅ M1 切片 2 |
| P4 | workspace/文档登记 | ✅ M1 切片 3 |
| P5 | webdriver ViewPainted 消费 + ImageCache 累积 | ⬜ M2 |
| P6 | GET /session/{id}/screenshot + 全链路测试 | ⬜ M2 |
| P7 | 验证通道文档（视觉回归使用说明） | ⬜ M3 |

## 下一步计划

1. **M2**：webdriver session.rs 消费 ViewPainted（`handle_renderer_message` 增加
   `IpcMessageKind::ViewPainted` 臂，跨帧累积 ImageCache——sent_keys 去重语义要求
   renderer 只发一次像素，webdriver 必须按 image_key 累积保活）→ 验证：单测 +
   http_session 模式全链路测试
2. **M2**：GET /session/{id}/screenshot endpoint——快照缺失返回 W3C unable to capture
   等价错误；有快照走 to_render_primitives + render_full_scene（CPU）+ PNG + base64
   （照 browser headless.rs R1601 `framebuffer_to_png_base64` 形态；该编码函数届时
   评估下沉公共 crate，一个消费者则不抽）
3. **M2 验证**：`make test` 全绿 + clippy 零告警 + PNG 可解码/尺寸断言全链路测试
4. **M3**：验证通道文档（兄弟 goal 视觉回归使用说明）→ DC 全满足判定

**碰撞管理**：M2 只动 `apps/webdriver/`（本流独占）+ 只读 renderer 消息面；开工前
照例 `git log --since="7 days ago" -- apps/renderer/` 核对 event-loop-spec 流活跃度
（只读不动，理论零冲突）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 公共 crate 抽取 + 双端收敛 | ✅ 完成（2026-09-09） |
| M2 — screenshot endpoint | ⬜ 未开工（下一步） |
| M3 — 接线收尾 | ⬜ 未开工 |

## 关键决策记录

| 决策 | 理由 |
|------|------|
| 方案①抽公共 crate（用户拍板 2026-09-09） | 双份维护实证：R4139/R4059 两笔渲染修复同批改 paint_ipc.rs + convert.rs 两处；收敛消除长期税 |
| 转换逻辑以 compositor `convert.rs` 为搬家基准 | compositor 主链路在用的真值；纯函数形态更易抽；browser 版有 TabSnapshot 副作用耦合，搬家后由调用方组合 |
| 新 crate 落 `crates/`（库形态），compositor/browser/webdriver 三端消费 | image-decoder 先例是 app 内 lib+bin；本层是纯库无进程入口，落 crates/ 更合工作区分层 |
| **`to_render_primitives` 取值签名（消费所有权）** | 两版搬家基准在字段级等价；值语义省 clone；需要保留 `PaintSnapshotParams` 的调用方（compositor 光栅化后续用 paint、browser 用 viewport/dirty_rects/hit_test）自行 clone——调用点语义不变 |
| **text_control_boundaries 并入公共转换** | 搬家时发现的字段级分歧（browser 版携带、compositor 版不携带）：属非绘制元数据（CPU/GPU 光栅化零消费），并入公共层后 compositor 行为零变化、browser 既有测试保绿；符合「以 compositor 为准、分歧点记录」协议，判定为字段映射缺口而非行为分歧 |
| 光栅化暂留 compositor `rasterize.rs` | M1 收敛转换层即可消除双份维护税；光栅化唯一消费者是 compositor，待 M2 webdriver 需要时再评估是否抽（避免推测性抽象） |
| PNG/base64 编码暂留 browser `headless.rs` | 同上：现唯一消费者是 headless CDP；M2 webdriver 落地时若两消费者成形再下沉公共层 |
| 不动 GPU 路径（gpu_raster.rs 留 compositor） | 公共层先收敛 CPU 转换+光栅化；GPU 是 compositor 专属主链路，无第二消费者 |
| 零 renderer/protocol 改动 | ViewPainted + SetFramePublishMode(Legacy) 通道已在（session.rs:118）；sent_keys 去重在 webdriver 层用跨帧 ImageCache 处理 |

## 待用户决策

（暂无）

## 验证基线

- M1 验证（2026-09-09，HEAD `0bcac5fc6`）：compositor 24 + paint-convert 4 +
  browser bin 413/413 全绿；fmt 无 diff；clippy `-p zero-paint-convert
  -p zero-compositor -p zero-browser --all-targets -D warnings` 零告警
- 立项基线（2026-09-09，HEAD `23830995b`）：main 全绿（webdriver goal 归档时
  `make test` 66 套件 19037/0 + clippy 零告警）
- 双份维护实证：`git show --stat 552ab2701`（R4139）与 `8cfabf4f4`（R4059）均含
  `paint_ipc.rs` + `convert.rs` 双文件改动
- **M1 收尾门禁待跑**：`make test`（全量）——M2 切片开工前执行；M1 各切片定向测试
  已全绿
