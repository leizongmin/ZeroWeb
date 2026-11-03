# DC-4 前置：coverage 复测机制验证 + 桶账预编（2026-11-02，对 m2s4 锚）

## 目的

M 末 DC-4（HEAD 布局全量 coverage 清零路径 + worker 变体分列归因）的
前置去险：① `wptfyi_gap.py coverage` 在 360 案新布局上机制验证（二期
DC-4 时只有 123 案）；② worker 腿接线（M2-3/3b）后 universe worker
变体是否解析入分子——二期 `uncovered·worker 腿 13,812` 桶的清账判定；
③ 缺口桶全部具名预编。对 [m2s4 锚](2026-11-02-m2s4-full.json.gz) 离线
复测，产物只落 /tmp（诊断产物不入库），wptfyi-cache 零写入（mtime
不变）。

## 工具机制验证

- 命令：`wptfyi_gap.py coverage --results <m2s4 --json> --cache
  tests/wpt-runner/wptfyi-cache`（与二期 DC-4 同工具同缓存，
  wpt.fyi aligned stable @ ce5ca39646）。
- 跑通：可疑条目 **0**（双腿命名 `.any.js` / `.any.worker.js` 不撞
  同名去重）、跨套件重复 0、108 条无 universe 对应路径单列。
- 分子（WebCryptoAPI 行）：宇宙 334 路径 / 70,756 子测试，**覆盖 252 /
  全过 149 / 通过 27,209**（二期 DC-4：覆盖 69 / 全过 68 / 13,247，
  语料预置 + 双腿接线后 ×2.05）。

## ZW 360 条解析分类（工具自身 `resolve_to_universe` 逐条复算）

| 类 | 条 | 子测试 | P | 说明 |
|---|---:|---:|---:|---|
| window 变体（`.any.html`） | 125 | 21,057 | 13,600 | |
| **worker 变体（`.any.worker.html`）** | 125 | 21,053 | 13,596 | **二期 worker 挂账结构性闭合**：runner `.any.worker.js` 命中既有 `.worker.js → .worker.html` 变体规则，双腿全量入分子 |
| exact（`.window.html` 等直名） | 2 | 19 | 13 | algorithm-discards（13P）+ modern-algos hybridkem window（6F，域门控） |
| unmapped（无 universe 路径） | 108 | 39,722 | 25,572 | 见下 |

- unmapped 108 = **98 tentative**（P 530 + F 14,150，与
  [DC-5② 预编账](2026-11-02-dc5-continuity-precheck.md) 精确吻合）+
  **10 分片变体主文件**（hkdf / pbkdf2 / successes_RSA-OAEP / PSS /
  RSASSA-PKCS1-v1_5 双腿，**25,042P 全绿**——universe 只收录
  `?N-M` 分片路径，整文件结果无映射落分子外。二期「分片变体 12,521P
  隐形」桶保持，双腿后翻倍至 25,042P；通道基建域挂账不变，非本 goal）。
- universe 侧未命中 82 = sliced 74（同上基建挂账）+ idlharness ×6
  （根 + 两新族 × 双变体，runner 记账排除先例）+ secure_context
  sub.html ×1（include 资产，排除先例）+ `historical.any.sharedworker`
  ×1（**新形态**：ZW 无 SharedWorker 通道，window/worker 双腿已跑，
  sharedworker 变体按通道域排除记账——M 末终账沿用此口径）。

## 分子内非 P（对宇宙分母 45,463）

- **F 14,916 + T 4**，全部 D1 域门控新族：F 双腿对称 7,455 + 7,455 +
  hybridkem exact 6；T 4 = Hybrid-KEM 双腿 × failures/successes。
  与 F 扫查闭式互证：29,066 − 14,150（tentative，分子外）= **14,916**。
- 形态差 3,334 = 宇宙分母 45,463 − ZW 实跑 42,129（三家 wrapper 子测试
  数与 ZW 形态差，二期同型 449 的放大版，非引擎失败）。

### 顺带更正

F 归因扫查 prose 的新族聚合数 **14,912 系笔误，实为 14,916**
（11,252 + 3,664；本预检独立复算 mapped F 精确吻合）——已同批修正
[扫查 evidence](2026-11-02-m2s4-f-attribution-sweep.md) 与控制面。

## M 末 DC-4 履行清单（对终态锚重算为准）

1. 域切片完成后对终态锚重跑本 coverage 对账；
2. 与周报 cron 快照（`docs/compat/wptfyi/coverage-*.md`）对账；
3. 桶复核：tentative 维度随 98 案退役归零（universe 侧本就 0）、
   worker 变体分列在账、sliced/排除项挂账口径延续。
