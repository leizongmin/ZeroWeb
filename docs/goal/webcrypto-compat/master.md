# Web Crypto 兼容二期运行时控制面板

**状态**: DC-1~4 全部落账有据并推送（`5894374c1`，2026-10-31）；门禁
build/clippy 绿 + webcrypto 锚在最终 HEAD 逐字节复验一致；唯一缺口
`cargo test` 10F 全部为 html-semantics 并行流在清的引擎既有基线（域不相
交、本 goal 零代码变更）——收口归档待该基线清零。M1/M2 里程碑已归档
（`archive/2026-10-31-m1-m2.md`），入口文档状态行已同步。
**收口前提变化（2026-10-31 12:55 核查）**：goal-25（html-semantics，s 系
账本）rally 进程**不在跑**（ps 仅存 editing ZeroWeb-3 与本流 ZeroWeb-2；
tmux 无 server、crontab 空），s 系 shim 推进止于 10:52 `3b25bb2c7`；此后
远端 main 仅 rendering 流与本流 docs 推进，10F 组合态无变化依据。处置
路径登记待用户决策表 **D2**，期间维持低成本轮询（`git fetch` + 账面），
基线清零即当场收口（详见
[evidence 追记](evidence/2026-10-31-gates-and-closure-status.md)）
**最后更新**: 2026-10-31（门禁与收口状态见
[evidence](evidence/2026-10-31-gates-and-closure-status.md)）

---

## 当前状态

DC-1~4 已全部达成（入口文档 [`../webcrypto-compat.md`](../webcrypto-compat.md)
v2.1 契约）：

- **DC-1（语料补全）✅**——实质结论更正：pin 语料一期已补至全量（上游 202
  blob，198 文件在库、0 失败，4 个为非测试元数据被扩展名分流正确排除）；
  「derive_bits_keys 32/38 未拉」系 coverage universe 口径假象（转正重排 +
  window/worker 变体路径），pin 目录事实是 11 案全在库全执行。证据：
  [M1 语料与基线](evidence/2026-10-31-m1-corpus-and-baseline.md)。
- **DC-2（转正甄别清单）✅**——一期 7,320F 挂账域在上游 master
  （cb6cc97a，2026-10-31 查证）**全部转正**（`.tentative.` 50→0，迁入
  modern-algos/ + secure-curves/ 两族；META.yml 无 tentative 声明）。
  逐域切片建议（域面/分母/范围/依赖候选）落
  [DC-2 甄别清单](evidence/2026-10-31-dc2-promotion-screening.md) 并登记
  下方待用户决策表 D1。
- **DC-3（扩面基线）✅**——全量复跑 123 案 / 33,109 子测试 = 25,788P
  （77.89%）+ 7,320F + 1T，双跑逐字节一致；与一期终态锚
  （m6-residual-full.json.gz）逐 (case, subtest) 对照**状态变化 0 例**，
  33,109 分母零回归底线保持。分域分类通过率 + tentative/非 tentative
  失败归因见 M1 证据文档。
- **DC-4（coverage 复测）✅**——同工具同缓存对 M1 结果复测，与官方
  coverage-2026-10-31 快照逐位一致（分子确定性证实）；分子 ≠ 实跑的
  12,541P 差值与 265 个未覆盖文件逐桶具名归因（分片变体 / 转正族 /
  worker 腿 / 新案 / 记账排除），tentative / 非 tentative 分列归因表落
  [DC-4 对账](evidence/2026-10-31-dc4-coverage-attribution.md)。

**W4 定性**：algorithm-discards-context 1T = runner/shim DOM 域边界
（iframe 跨帧注入 + `top` 回调链），非 WebCrypto 语义域、非语料边界，
维持 DOM 域挂账。

## 与兄弟 goal 的边界

- 33 号 editing-compat（ZeroWeb-3 工作区并行在跑）——无共享面。
- 23 号 selection-compat——无共享面。
- workers 通道结构性挂账（worker 腿 window-only，coverage worker 桶
  13,812 子测试）——归 runner/workers 通道，不在本 goal。

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | 已转正域按域立项：ML-KEM（12 案/3,480）、ML-DSA（10/1,206）、X448/Ed448（26/3,700）、AES-OCB（10/2,652）、ChaCha20-Poly1305（10/1,044）、KMAC（10/1,418）、Argon2（4/92）、XOF digest（8/838）、Hybrid-KEM（7/2,742）、门禁面（12/880），合计 109 案/18,052 子测试（universe 口径）；切片建议与依赖候选见 [DC-2 甄别清单](evidence/2026-10-31-dc2-promotion-screening.md)（依赖含新 crate：ml-kem/ml-dsa/ocb3/x448/ed448-goldilocks/kangarootwelve 等，选型须用户拍板） | 待用户拍板；不 BLOCK 本 goal 收口 |
| D2 | 引擎基线 10F 处置路径（收口唯一缺口）：① 等 goal-25 rally 重启后清零（10F 全为 fetch/Request/XHR/ua_display 域，s 系账本面；12:55 核查该流驱动缺位）；② 授权本 goal 按 standing exclusion 口径收口（先例 net-api request-cache 98F——10F 归因已在 [gates evidence](evidence/2026-10-31-gates-and-closure-status.md) 落档，非本 goal 域、33,109 分母零回归不受影响，DONE 的「cargo test 全过」按归因口径豁免并行流账面）。**本 goal 不越界代修**（Mission 边界维持） | 待用户拍板；期间维持低成本轮询，命中 ① 即当场收口 |

## 碰撞管理记录（run-rules §9）

- 2026-10-31：立项时无同域在跑碰撞；33 号 editing-compat 在 ZeroWeb-3 并行
  推进，域不相交。
- 一期零回归底线：33,109 分母逐 subtest 对照，锚
  [`../archive/webcrypto-compat/evidence/2026-10-17-m1-baseline.md`](../archive/webcrypto-compat/evidence/2026-10-17-m1-baseline.md)
  （逐 subtest 实际对照用终态锚 m6-residual-full.json.gz，同目录）。
