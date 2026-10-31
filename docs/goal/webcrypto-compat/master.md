# Web Crypto 兼容二期运行时控制面板

**状态**: Active——M1 未启动（2026-10-31 控制面建立；入口文档 v2.1 无人值守收口定稿）
**最后更新**: 2026-10-31

---

## 当前状态

待启动。入口文档 [`../webcrypto-compat.md`](../webcrypto-compat.md)（v2.1）为
唯一目标契约。

**与兄弟 goal 的边界**：

- 33 号 editing-compat（ZeroWeb-3 工作区并行在跑）——无共享面。
- 23 号 selection-compat——无共享面。
- workers 通道结构性挂账（encoding 二期归档登记，runner 基建域）——扩面中
  若命中 runner 通道限制（worker 腿等），归 runner 挂账登记，不在本 goal
  修通道。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| W1 | derive_bits_keys 族 32/38 文件未拉（24,918 子测试主扩面） | 待 M1 |
| W2 | 其余族 233/296 文件未拉 | 待 M1 |
| W3 | `.tentative.` 挂账域转正状态未甄别（7,320F：ML-KEM/ML-DSA/X448/Ed448/AES-OCB/kmac/chacha20_poly1305/argon2/cSHAKE 等） | 待 M1 |
| W4 | algorithm-discards-context 1F 未定性 | 待 M1 |
| W5 | coverage 复测未做（未覆盖 57,509 清零路径，10-31 快照） | 待 M2 |

## 里程碑状态表

| 里程碑 | 内容 | 状态 |
|---|---|---|
| M1 | 语料补全 + 扩面基线 + 转正甄别清单（DC-1/2/3） | 未启动 |
| M2 | coverage 复测收口（DC-4） | 未启动 |

## 碰撞管理记录（run-rules §9）

- 2026-10-31：立项时无同域在跑碰撞；33 号 editing-compat 在 ZeroWeb-3 并行
  推进，域不相交。
- 一期零回归底线：33,109 分母逐 subtest 对照，锚
  [`../archive/webcrypto-compat/evidence/2026-10-17-m1-baseline.md`](../archive/webcrypto-compat/evidence/2026-10-17-m1-baseline.md)。

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | 已转正域按域立项（M1 甄别后逐域出切片建议：域面 / 分母 / 建议范围 / 依赖需求；依赖含新 RustCrypto crate 时须用户选型） | 待 M1 产出 |
