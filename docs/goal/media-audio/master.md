# 媒体音频 — 运行时控制面板（master.md）

**入口文档**: [../media-audio.md](../media-audio.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-08-17（立项——M0 环境验证待启动）

---

## 当前状态

**专项定位**：媒体方向三拆之三（门控最深）。音频输出（解码→混音→设备）+ A/V 同步 +
volume/muted 真控制。**双重启动门控**：① M0 音频环境验证与验证策略成文（自持）；
② media-playback M0 解码选型 RFC 获批（解码路线绑定其选型）。齐备后才动源码。

**与兄弟 goal 的边界**：
- media-playback — 视频/解码选型归其管；A/V 同步接口对齐（audio clock 主时钟——契约记录
  于两流 master.md）
- media-elements — 语义面归其管；volume/muted 本目标接真增益（反射已有）
- js-dom — volume/muted 反射段共享，`git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项时）

### 现有实现

- ✅ 反射底座：muted/volume 属性反射（R3040，真控制未接）
- ✅ 时钟底座：rAF 帧驱动（P1a）——音频时钟对齐可挂
- ⚠️ 零音频管线（无 cpal/音频依赖，无解码/混音/输出代码）
- ⚠️ 环境未验证：cpal 可用性/dummy 回退/headless 验证策略未探测成文（M0）
- ⚠️ 选型未对齐（待 media-playback M0 落地）
- ⚠️ 无音频 e2e 资产（真输出 + 混音总线断言双轨）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| A1 | 音频环境验证 + headless 验证策略未成文（M0 自持门控） | 🔄 M0（当前活跃） |
| A2 | 解码选型未对齐（外部门控：media-playback M0） | ⬜ 等待 |
| A3 | 零音频管线（解码/重采样/混音/输出） | ⬜ M1（双门控解除后） |
| A4 | A/V 同步机制缺失 | ⬜ M2（依赖 media-playback M2 视频时钟） |
| A5 | 音频 e2e 资产为零 | ⬜ M0 期间可先设计断言形态 |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | AudioContext（Web Audio）最小面可行性 RFC → 是否实施 | ⬜ M3 时点提交 |

## 下一步计划

1. **M0 切片 1**：本地音频设备探测 + cpal PoC（正弦波播放；无声卡/dummy 回退行为记录）
2. **M0 切片 2**：headless 验证策略设计成文（混音总线可观测断言形态）
3. **M0 切片 3**：与 media-playback M0 选型的依赖对齐记录（解锁条件清单更新）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 环境验证 + 验证策略（门控） | 🔄 待启动（当前活跃） |
| M1 — 首个声音输出 | ⬜ 门控：M0 成文 + media-playback 选型落地 |
| M2 — A/V 同步 + 控制 | ⬜ |
| M3 — `<audio>` 全路径 + Web Audio 评估 | ⬜ |

## 验证基线

- 测试基线：立项时点全绿；clippy 零警告
- 音频 e2e 面：无资产（策略待成文）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
