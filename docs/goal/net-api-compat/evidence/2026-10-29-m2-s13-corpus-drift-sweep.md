# M2-S13 — net-api 语料漂移巡检首轮 + request-upload 假红腿清偿（2026-10-29）

**切片**: [M2-S12](2026-10-29-m2-s12-preflight-redirect-corpus-drift.md) 记账 3
（corpus 漂移巡检立项）专项。**巡检 + 1 腿语料对齐清偿**，引擎/runner fixture
零改动；全通道 A/B 对直接前轮基线 `netapi-m2s12-r1.json` 逐 subtest（结果见
「验证」）。S12 教训（learnings/bugs：红腿先查上游漂移）的横向应用。

## 前提纠正：v1.10 周级漂移假设不成立

巡检原设计以语料仓 v1.10 tag（快照 2026-07-30）为漂移基准，预期三个月级漂移
面。mtime 取证**推翻前提**：fetch/url/mimesniff 三域全部 247 文件的 mtime 密集
于 **2026-10-29 00:19–00:39**——即语料在当夜被交互会话全量同步至上游 master
（同步动作无记录，属 S12 记账 3 所述「无同步时点记录」状态的实例）。旁证：
9 个差异文件对 v1.10 tag 全部 404——它们是 v1.10 之后新增/改写的面，本地既有
内容与周边 238 个同批文件同 mtime，说明同步源就是 master 近 HEAD。

因此实际命题变为「本地（master@00:30）vs 上游 master@13:22 的 13 小时窗漂移」。

## 巡检方法（可复跑）

- 清单：`find fetch url mimesniff -type f`（goal 三族全域，247 文件）。
- 拉取：gh-proxy 前置的 raw.githubusercontent.com 8 路并行抓 master HEAD
  （raw 无 GitHub API 配额问题；绕开 commits API 限流）。
- 比对：`cmp` 字节级；差异文件再抓语料仓 v1.10 分类（DRIFT / LOCALPATCH）。
- 脚本与产物：`/tmp/wpt-drift-sweep.sh`、`/tmp/wpt-drift/`（本地文件清单 /
  differ / missing / classify / 上游 HEAD 副本）。一次性诊断产物不入库。

## 结果：238/247 字节同，9 differ 全分类

| 文件 | 差异方向（上游 →） | 处置 |
|---|---|---|
| fetch/api/basic/request-upload.any.js | **删除**「POST text body 421 retried once」腿 | **对齐删除（本轮，见下）** |
| fetch/api/request/request-disturbed.any.js | **翻转**期望：`new Request(req, {body})` 后源 req `bodyUsed` true→false | **不对齐**，记账 1 |
| fetch/api/basic/gc.any.js | 新增 FormData body GC 腿 | 不追，记账 2 |
| fetch/api/cors/cors-preflight-cache.any.js | 新增通配/凭据缓存腿 | 不追，记账 2 |
| fetch/api/request/request-bad-port.any.js | 改 PAC 代理基建（`bad-port.wpt` + proxy-all.sub.pac） | 不追，记账 2（runner 无 PAC 基建） |
| url/url-setters-stripping.any.js | 新增 searchParams 剥离反射 26 腿 | 不追，记账 2 |
| url/resources/urltestdata.json | 数据翻新 | 不追，记账 2 |
| url/resources/IdnaTestV2.json | 数据翻新 | 不追（S2 以当前数据 2671/2671 收口） |
| mimesniff/mime-types/resources/mime-types-minimized.json | 数据翻新 | 不追（DC-4 口径以当前数据收口） |

mtime 异常点全部归因：05:44 × 3 = M2-S3c 的 `.txt` fixture；12:45 = M2-S12 的
cors-preflight-redirect 对齐 + 本轮 request-upload 对齐。无不明白改。

## 实现面

- `tests/wpt-runner/wpt-data/fetch/api/basic/request-upload.any.js` 逐字对齐
  上游 HEAD：净删「Fetch with POST with text body on 421 response should be
  retried once on new connection.」整腿。该腿依赖
  `fetch/connection-pool/resources/network-partition-key.py`（runner 语料无此
  fixture，本地恒 `expected 421 but got 0` 假红）——与 M2-S12 同型：本地红腿
  实为上游已弃测试面。
- 引擎零改动；runner fixture 零改动。
- `scripts/update-wpt-data.sh` 头部缺口清单补两条：request-upload（新 ref
  原生含删除，无需重补）+ request-disturbed 翻转预警（新 ref 落地时会显影
  1 条新红腿，属预期非回归）。

## 验证

- 单文件探针（`testharness-net-api request-upload`）：request-upload.any.js
  **18/18 Pass**（删除腿消失）；h2 变体 10P/1F/1T 与基线逐条一致（ReadableStream
  421 Fail 与 h2 Timeout 为既有引擎/基建面，零扰动）；stderr 0 字节。
- 全通道 A/B（`netapi-m2s13-r1.json` vs 直接前轮 `netapi-m2s12-r1.json`，逐
  subtest，对账工具输出 `.acceptance/net-api-compat/abdiff-m2s13-r1.txt`）：
  **0 增益 / 0 回归 / 0 翻转 / MISSING 1**——唯一 removed subtest 即被删腿
  「Fetch with POST with text body on 421 response should be retried once on
  new connection.」（基线 Fail）；pass 11000 → 11000，分母 12078 → 12077
  （红腿 1078 → 1077）；per-file 分母 Δ 仅 request-upload.any.js，其余 487
  文件逐条零变化；stderr 0 panic（0 字节）。

## 记账

1. **request-disturbed 期望翻转挂起**：上游按 fetch spec 演进改写——
   `new Request(inputRequest, {body: "..."})` 供入新 body 时**不再扰动**源
   request（本地语料旧期望 `bodyUsed === true`，引擎与旧期望一致、该腿
   Pass）。引擎翻新与语料对齐**必须同步**，单独动任一边都互斥翻红；挂
   goal「后续可推进小片」，建议与语料仓滚动新 tag 同批执行（届时该腿自动
   显影，属预期）。
   **[2026-10-29 M2-S14 已双侧同步翻转收口（引擎 + 语料同批），见
   M2-S14 档](2026-10-29-m2-s14-request-disturbed-init-body.md)。**
2. **additive/数据面不追 HEAD 的口径**：追平 = 分母上涨 + 上游新特性红腿
   （PAC 代理、FormData GC、searchParams 反射、preflight 缓存通配），与
   「残余失败收敛」方向相反；统一留给语料仓滚动 tag 时按 M2-S13 分类表
   结算（哪些新增面立项、哪些记账）。
3. **S12 记账 3 清偿**：巡检已执行，结论**不常态化立项**——13 小时窗漂移面
   仅 9 文件且全部分类处置后，常态化巡检性价比低；语料对账收敛到「语料仓
   滚动 tag 时一次性结算」单点。S12 档记账 3 已加指针。
   **[2026-10-29 M2-S14 事实修正：「13h 窗」推断不准——net-api 语料有正式
   pin（goals/lib.sh `WPT_REV=3159769`，上游 2026-08-12 快照；幂等快路径
   已存在即跳过，本地 mtime 只是首拉时刻非内容时点）。9 个 differ 实为
   pin→master HEAD（2026-10-29）约 2.5 个月漂移，巡检分类与处置结论不变；
   见 M2-S14 档](2026-10-29-m2-s14-request-disturbed-init-body.md)。**
4. **巡检范围**：goal 三族（fetch/url/mimesniff，247 文件）。xhr/streams 在
   通道分母内但不在 goal 三族，未巡检；如后续在该域见可疑红腿，按 S12
   learnings 流程单点核查即可，无需扩巡检。

