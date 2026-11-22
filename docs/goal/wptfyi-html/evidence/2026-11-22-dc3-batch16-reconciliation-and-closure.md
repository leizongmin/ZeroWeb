# DC-3 · 批十六：run 44985566993 逐通道收账 + CI 专有缺档两文件补档

2026-11-22。DC-3 对账 run 44985566993（cde99ad26，04:33Z 派）五分片产物逐键
A/B 收账，无 Rust 码笔，fix = 57 脚本 +13 行（两 `fetch_raw`）。

## 逐通道收账（控制点 44972755575 → 44985566993）

| 通道 | 控制 | 本轮 | Δ | 判定 |
|---|---:|---:|---:|---|
| infrastructure（+links/obsolete/scripting/select） | 82 | 674 | +592 | 批十四兑现：132 键 674/376/138 与批十三本地 after 逐项相同 |
| webappapis | 3 | 438 | +435 | 批十五 fetch 兑现；距本地 face 1,537 缺 1,099（见缺档定谳） |
| browsers | 44 | 84 | +40 | 批五 face 逐键兑现；zero-reg 键与本地同形 |
| semantics | 8,032 | 9,991 | +1,959 | 批六 face 兑现：231 delta 键逐键零漂移 |
| syntax | 2,748 | 2,791 | +43 | speculative 勘误后首跑 43P/83TO 同形；html5lib/template 基底零漂移 |
| xhtml / dom / rendering / 邻居 CE+SD | 25,606 / 59,644 / 2,793 / 11,361 | 同 | 0 | 持位 |

邻居观察（不入本流账）：dom/ranges −13＝2 键子测试注册抖动形
（dataChange 950→934 子测试 −16 全 Pass 形、replaceData +3 回冲），归
wptfyi-dom goal 账本。

## webappapis 缺 1,099 定谳（批十五扫描盲区）

207 delta 键中 198 键 P 值与本地逐一相同；差全在 10 键：

1. **event-handler 8 页（−919 P）**：页面运行期
   `fetch("/interfaces/html.idl")` 失败 → promise 拒绝 → 零注册 TO
   （`tests:0`）。批十五闭包扫描只抓 `<script src>` 静态引用，运行期
   fetch 不可见——方法论盲区，本轮补勘。`resources/WebIDLParser.js`
   本身已在盘（批十五补档兑现），单缺 idl 文件。
2. **structured-clone.any.js（−148 P）**：`META: script=/common/sab.js`
   load `os error 2`。62 脚本虽有同文件，但 62 不在 syntax-dom 分片
   fetch 清单——各自自足原则在此补。
3. **base64.any.js（−32 P，非缺档）**：`META: global=window,dedicatedworker`
   双 scope——本地 380 子测试（310P/70F 双 scope 合并）vs CI 286 子测试
   （278P/8F 单 scope），执行面差异，不补档。

agent-clusters ~55 键 / open-features non-integer 13 键 /
cross-origin-objects-on-new-window 的零注册 TO 与本地 batch-5 逐键同形
（TO=1 sub=1）＝本地行为面（dispatcher/页面挂起域），非 CI 缺档，维持
批五挂账不重计。

## 修复与验证

57 脚本批十六块：`fetch_raw "interfaces/html.idl"`（107KB）＋
`fetch_raw "common/sab.js"`，注释记两形根因与扫描盲区勘误。
`bash -n` 通过；GITHUB_TOKEN 幂等重跑 FAILED_FETCHES=0（两文件本地在盘
跳过）。预期 webappapis CI face 438→~1,505（余 −32 base64 差异）。

## DC-3 状态

- **44985566993 aggregate 快照已落 main**（d38d2f116，06:42Z，三家
  a3830cd8f7）：快照级逐组与键级收账全对齐——infrastructure 82→635
  （+553；套件五组合计 673，scripting 键清单外 0 入账）、webappapis
  3→438（+435）、browsers 217→257（+40）、semantics 8,508→10,467
  （+1,959）、syntax 2,747→2,790（+43）、xhtml 25,606／dom 59,634／
  rendering 2,793 持位；全量 zw_passes 1,914,204→1,917,413（+3,209），
  分母 2,276,004→2,276,010（+6 上游漂移）。
- 批十六码需再派 coverage run：承接 webappapis face 修正
  （+1,067）并作为 DC-3 最新周跑产物，下轮取 aggregate 快照做
  目标级对账（预期 webappapis face ~1,505，余 −32 base64 归双 scope
  差异；rendering/cssom 面含渲染流 80a64cdda CSSOM-INSETS-V3 刀的
  跨流正向位移，另账不入本流）。

[逐文件 JSON](2026-11-22-dc3-batch16-reconciliation-and-closure.json) ·
[批十五证据](2026-11-22-dc2-batch15-ci-shard-corpus-channels.md) ·
[批十三证据](2026-11-22-dc2-batch13-query-encoding-py-fixtures.md)
