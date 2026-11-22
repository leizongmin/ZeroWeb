# DC-3 · 批十七：批十六在飞 run 收账＋WebIDLParser 生成路由勘误与别名补档

2026-11-22。run 44996021189（5024fa97a）aggregate 快照落 main（7a34ff3dd，
09:13Z）逐通道收账；fix＝57/59 两脚本 WebIDLParser 拉取改真身落别名（+13 行）。

## 批十六 run 逐通道收账（44985566993 → 44996021189）

| 通道 | 批十五 | 批十六 run | Δ | 判定 |
|---|---:|---:|---:|---|
| html/webappapis | 438 | 586 | +148 | sab.js 兑现：structured-clone.any.js 1/0P→152/148P 唯一变更键 |
| html/dom | 59,634 | 59,634 | 0 | 三查询键未兑现（见勘误） |
| html/syntax | 2,790 | 2,747 | −43 | speculative 整面消失形（瞬态网络窗，见下） |
| html/infrastructure | 635 | 629 | −6 | messagechannel.any.worker.js 抖动带同键（115P→109P＋1TO） |
| html/semantics | 10,467 | 10,466 | −1 | dialog-overlay.html 单子测试翻转（2P→1P/1F，挂观察） |
| browsers/rendering/xhtml | 257/2,793/25,606 | 同 | 0 | 持位 |

目标级账（快照口径）：html 面 106,296→**106,394**（净 **+35,298**）；全量
1,917,413→1,917,532（+119）。DC-3 验收 ≥152,096 → **余量 45,702**。
批十六计划面（webappapis +1,067＋dom 三键 +~1,394）实际兑现 +148，
**计划缺口 −2,313 全归 WebIDLParser 勘误**（−919−1,394）＋base64 +32
（勘误补档 b29107e7e 晚于派 run，预期内未含）。

## WebIDLParser 生成路由勘误（确定性 404，纠正批十六前提）

`resources/WebIDLParser.js` 上游**不是独立 Git blob**——是 wptserve 对
`resources/webidl2/lib/webidl2.js` 的生成路由（archive/service-workers
2026-08-19 idlharness 审查证据在册先例）。实证：pin rev 315976933 与
master 双 ref，jsdelivr／raw.githubusercontent／contents API／git trees
四路全无此文件；本地 `wpt-data/resources/WebIDLParser.js`（125,228B）
与 webidl2 blob `bae0b204` 字节级相同（cmp 实证）。

由此纠正批十六证据两处：

1. 「resources/WebIDLParser.js 本身已在盘（批十五补档兑现）」为假——批十五
   run 同样 fetch 失败（该路径恒 404），8 页在两轮 run 零注册是同一根因；
   html.idl 是第二缺件，本轮 CI 拉取成功（日志无失败行）。
2. 「GITHUB_TOKEN 幂等重跑 FAILED_FETCHES=0」对该文件无验证力——本地在盘
   跳过路径掩盖拉取必然失败（批十六证据已自注此风险，本轮坐实）。

CI 日志旁证：syntax-dom job 07:04:43/07:07:27 两次
`fetch failed: resources/WebIDLParser.js`（57/59 脚本各一）。

## 瞬态网络窗（07:06–07:08Z，catch-all-4）

- `63-wptfyi-html-speculative-parsing.sh`：git trees 列「html」4× curl
  (28) 超时 → 目录枚举跳过 → **语料 0 文件入盘** → 套件 `[]`（126 键
  43P/83TO 全消失）＝ syntax −43。
- `fetch-wpt-import-maps`：git clone 连接 github.com:443 失败（133s）→
  make 128 → import-maps −37（六个子组 37P，非 html 计分域）。
- 两形均为优雅降级设计路径，下批 run 自愈；非本流码面变更。

## 邻居观察（不入本流账）

dom/ranges +53（批十六 −13 抖动回位后净增，wptfyi-dom 账）、editing/run
−130（editing 域账本）、css/css-sizing +89／css/cssom +30／
css/css-animations +14（渲染流正向位移另账）、shadow-dom/declarative −1、
web-animations/interfaces −1、其余组 ±1–3 抖动。

## 修复与验证

57/59 两脚本：`fetch_raw "resources/WebIDLParser.js"` 改为拉真身
`resources/webidl2/lib/webidl2.js` 后 `cp` 落别名（fetch 失败仍只计
FAILED_FETCHES 不中断，脚本退出行为不变）。`bash -n` ×2 通过；本地功能
验证：挪走别名文件后跑 57/59，别名由真身重建且与 webidl2.js
`cmp` 字节级相同，exit=0、fetch 失败 0。归档 goal 脚本
99-webgl-compat.sh:39 同款恒 404 拉取（无活跃流，记档不动）。

批十七 coverage run 已派（本 HEAD，含 b29107e7e 勘误补档＋本修复），
预期：webappapis 586→~1,537（+919 event-handler face＋+32 base64 atob
尾）、html/dom 三键 1→~5,822 子测试（+~1,394P；CI 90s 页预算观察点）、
syntax +43／import-maps +37 瞬态回位、infra/semantics 抖动自愈；html 面
~108,800（净 ~+37,700，余量 ~43,300）。

[逐文件 JSON](2026-11-22-dc3-batch17-webidlparser-alias-and-rerun.json) ·
[批十六证据](2026-11-22-dc3-batch16-reconciliation-and-closure.md) ·
[前置账](2026-11-22-dc3-goal-accounting-and-corpus-runway.md)
