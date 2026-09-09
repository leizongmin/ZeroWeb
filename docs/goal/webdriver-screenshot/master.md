# WebDriver Screenshot — 运行时控制面板（master.md）

**入口文档**: [../webdriver-screenshot.md](../webdriver-screenshot.md)
**创建日期**: 2026-09-09（用户拍板方案①——抽公共截图转换层 crate，从已归档 webdriver
goal 挂账项立项）
**最后更新**: 2026-09-09（M2 完成：GET /session/{id}/screenshot 全链路落地 + 全门禁
验证；下一步 M3——验证通道文档 → DC 全满足判定）

---

## 当前状态

**专项定位**：收敛 `PaintSnapshotParams → RenderPrimitives` 转换双份实现为公共 crate，
并落地 webdriver GET /session/{id}/screenshot。M1（抽取+收敛）→ M2（endpoint）→
M3（接线文档）。

**当前进展**：**M1 + M2 完成（2026-09-09，R4181）**——
- M1 切片 1（`3b04866eb`）：新 crate `crates/paint-convert`（`zero-paint-convert`），
  compositor `convert.rs` 删除、切换消费；既有测试平移
- M1 切片 2（`0bcac5fc6`）：browser `paint_ipc.rs` 597→271 行切换消费，重复
  `ipc_*_to_*` 函数族删除（grep 零残留）；`text_control_boundaries` 并入公共转换
- M1 切片 3（`9a6964bc2`）：README（32 members）/AGENTS.md/zero-web master.md 登记
- M2（`a09e0d711`）：GET /session/{id}/screenshot 全链路——webdriver 消费 ViewPainted
  （两路：await_load_complete + request_automation handle_other）+ 跨帧累积 ImageCache
  （S8 sent_keys 去重语义正确处理）+ CPU 光栅化 + PNG base64；零 renderer/protocol 改动

**对照测试记账（DC-3 抽取行为对照）**：转换语义逐字段一致由以下既有测试构成对照——
compositor `rasterize_tests`（24，含 DSF 物理尺寸/脏区增量）+ 平移入公共 crate 的
glyph intern/冲突拒绝、glyph raster metadata + 新增 text_control_boundaries 断言
（公共 crate 4）+ browser `paint_ipc` 3 测（hit-test 还原/glyph source+variations/
image payload）+ browser bin 全量 413/413。全绿即对照通过。

**与兄弟 goal 的边界**：
- rendering-compat — M1 抽取是**纯结构搬家**（R4139/R4059 语义注释原样迁移）；
  后续该流对转换语义的修复只需改 `crates/paint-convert` 一处
- event-loop-spec — 本流零 renderer 改动（只读 ViewPainted 通道），已兑现
- keyboard-* / editing-contenteditable / 其他 webdriver 消费者 — M3 提供截图
  使用说明，不替它们写用例

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 新公共 crate（转换层） | ✅ M1（光栅化/PNG 暂留原位，见决策记录——不推测性抽象） |
| P2 | compositor 切换消费 + 重复删除 | ✅ M1 |
| P3 | browser paint_ipc 切换消费 + 重复删除 | ✅ M1 |
| P4 | workspace/文档登记 | ✅ M1 |
| P5 | webdriver ViewPainted 消费 + ImageCache 累积 | ✅ M2 |
| P6 | GET /session/{id}/screenshot + 全链路测试 | ✅ M2 |
| P7 | 验证通道文档（视觉回归使用说明） | ⬜ M3 |

## 下一步计划

1. **M3**：验证通道文档（`docs/goal/webdriver-screenshot/evidence/`）——兄弟 goal
   （rendering-compat 视觉回归、keyboard-* 行为验证、android-browser 截图通道）如何
   消费 screenshot endpoint 做视觉回归：spawn zero-webdriver → New Session → Navigate
   → GET screenshot → base64 解码 PNG → 与基线/另一渲染路径比对；W3C 语义注记
   （视口截取、无帧 unable to capture screen）
2. **M3 验证**：DC-1~4 逐条判定 → 满足则输出 DONE 归档

**碰撞管理**：M3 纯文档轮，零代码改动、零碰撞面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 公共 crate 抽取 + 双端收敛 | ✅ 完成（2026-09-09） |
| M2 — screenshot endpoint | ✅ 完成（2026-09-09） |
| M3 — 接线收尾 | ⬜ 未开工（下一步，纯文档） |

## 关键决策记录

| 决策 | 理由 |
|------|------|
| 方案①抽公共 crate（用户拍板 2026-09-09） | 双份维护实证：R4139/R4059 两笔渲染修复同批改 paint_ipc.rs + convert.rs 两处；收敛消除长期税 |
| 转换逻辑以 compositor `convert.rs` 为搬家基准 | compositor 主链路在用的真值；纯函数形态更易抽；browser 版有 TabSnapshot 副作用耦合，搬家后由调用方组合 |
| 新 crate 落 `crates/`（库形态），compositor/browser/webdriver 三端消费 | image-decoder 先例是 app 内 lib+bin；本层是纯库无进程入口，落 crates/ 更合工作区分层 |
| **`to_render_primitives` 取值签名（消费所有权）** | 两版搬家基准字段级等价；值语义省 clone；需保留 `PaintSnapshotParams` 的调用方自行 clone（compositor 光栅化后续用 paint、browser 用 viewport/dirty_rects/hit_test、webdriver screenshot 用 viewport）——调用点语义不变 |
| **text_control_boundaries 并入公共转换** | 搬家发现的字段级分歧（browser 版携带、compositor 版不携带）：非绘制元数据（CPU/GPU 光栅化零消费），并入后 compositor 行为零变化、browser 既有测试保绿；判定为字段映射缺口而非行为分歧（协议执行条款 2），公共 crate 新增断言测试 |
| **光栅化暂留 compositor `rasterize.rs`、PNG 编码暂留 browser `headless.rs`（M2 修订：PNG 编码已在 webdriver 侧独立实现）** | M2 后 PNG 编码实际已有两份（headless.rs / webdriver session.rs 各 ~15 行薄封装）——该重复是有意的：编码函数极薄且两处错误类型不同（headless 用 ProtocolError、webdriver 用 DriverError），抽公共层收益低于抽象成本；若未来第三消费者出现再收敛。光栅化唯一重消费者仍是 compositor（webdriver 走裸 `render_full_scene` 无脏区增量需求），不预抽 |
| 不动 GPU 路径（gpu_raster.rs 留 compositor） | 公共层先收敛 CPU 转换；GPU 是 compositor 专属主链路，无第二消费者 |
| 零 renderer/protocol 改动 | ViewPainted + SetFramePublishMode(Legacy) 通道已在（session.rs:118）；sent_keys 去重在 webdriver 层用跨帧 ImageCache 处理（renderer 导航时 clear sent_image_keys → 新页同 key 重发 payload → `insert_with_key` 覆盖即正确语义） |
| screenshot 视口语义（viewport 截取） | W3C Take Screenshot 以 window（=会话视口 800×600）为准；initial about:blank 无 ViewPainted 帧 → unable to capture screen（规范等价错误），与错误路径测试锁定 |

## 待用户决策

（暂无）

## 验证基线

- **M2 全门禁（2026-09-09，HEAD `a09e0d711`）**：
  - `make test` 分阶段复跑全绿：workspace 主矩阵 780+1 后隔离复跑 integration lib
    781/0（唯一 FAIL `stale_etag_revalidation_is_coalesced` 为多轮在案的
    TcpListener 负载敏感 flake，R4093-G/R4056-F 同签名，隔离复跑即时通过）、
    renderer bin 0-fail、browser bin（xvfb）413/413、GPU adapter 96+1、
    quickjs webdriver/script-sandbox 组全绿
  - 全 workspace clippy `-D warnings` 零告警；`cargo fmt --all -- --check` 无 diff
  - `make product-smoke`：23.49%（112736/480000 px）+ struct PASS——与 ZRG-2026-09-09-01
    巡检在案值逐字节同 px 数（chronic ZRG-2026-08-17-01 oracle 陈旧域，非本轮回归；
    M2 零渲染路径改动）
  - webdriver 全链路：11 http_session 测试（含新增 screenshot 错误/成功双路径）+ 3 单测
- M1 验证（2026-09-09，HEAD `0bcac5fc6`）：compositor 24 + paint-convert 4 +
  browser bin 413/413 全绿；切片级 clippy 零告警
- 立项基线（2026-09-09，HEAD `23830995b`）：main 全绿（webdriver goal 归档时
  `make test` 66 套件 19037/0 + clippy 零告警）
- 双份维护实证：`git show --stat 552ab2701`（R4139）与 `8cfabf4f4`（R4059）均含
  `paint_ipc.rs` + `convert.rs` 双文件改动
