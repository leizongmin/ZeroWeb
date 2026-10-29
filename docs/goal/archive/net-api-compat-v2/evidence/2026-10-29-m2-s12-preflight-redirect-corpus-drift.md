# M2-S12 — cors-preflight-redirect 语料漂移收口：上游 651ffcb03 五腿改写（2026-10-29）

**切片**: [M2-S11](2026-10-29-m2-s11-cors-preflight-cache-coverage.md) 记账 1 /
[M2-S11b](2026-10-29-m2-s11b-window-origin-var-shadow.md) 记账 1
（cors-preflight-redirect「after preflight failed」5F）专项。**归因反转 + 语料
对齐收口 +5 零回归**，全通道 10995 → **11000**（对一期基线 9354 累计 +1646）。
**零代码改动**——引擎/runner fixture 均未动，只对齐语料。

## 归因纠正（S11/S11b 记账 1 的 runner stash 互写假设不成立）

两档记账曾假设：上游 redirect.py（写双键 stash dict）与 preflight.py（六键
schema）跨 fixture 互写同一 token → KeyError → 500 无 CORS 头 → fetch 拒绝，
runner 缺该拒绝链仿真。本轮上游源码核对**证伪**：

- **wptserve Stash 按（请求 path, uuid）作用域**——`request.server.stash` 每请求
  以 `Stash(request.url_parts.path)` 构造（tools/wptserve/wptserve/request.py），
  键为 `(isomorphic_encode(path), uuid)`。redirect.py 与 preflight.py 路径不同，
  同 token 条目互不可见——跨 fixture 互写冲突在真实 wptserve 里不存在。
- **真根因是语料漂移**：上游提交 651ffcb03（2026-09-12，"Fetch: redirects after
  a CORS-preflight are followed"）把五腿整体改写——「after preflight failed」
  （期望 promise_rejects_js TypeError）→「after preflight succeeded」（期望
  200 + x-did-preflight=1）。语义依据：fetch spec 现行文本里 HTTP-redirect fetch
  递归重入 main fetch，unsafe-request flag 与 CORS-unsafe 头仍在 → 重定向跳对
  新目标**重新预检**（旧版「tainted-origin 抑制重预检」模型已演进）。
- **浏览器实证**：wpt.fyi master run
  `9df53f9d847f7d3b15e1b4d1615f167af39a3357`（Chrome 158.0.8127.2）该测试
  10/10 全绿（summary `{'s':'O','c':[10,10]}`）——本地快照的旧期望与现代
  浏览器行为相悖。

定位手法落
[learnings/bugs](../../../learnings/bugs/2026-10/2026-10-29-wpt-red-leg-upstream-drift-first.md)
（红腿排查先证上游漂移，再动引擎/runner；wpt.fyi run id → GCS summary 实证链）。

## 实现面

- `tests/wpt-runner/wpt-data/fetch/api/cors/cors-preflight-redirect.any.js`
  逐字对齐上游 HEAD：五腿改期望（200 + `x-did-preflight=1`，断言文案
  "Preflight request has been made for the redirect target"）+ 腿改名
  （failed → succeeded）+ `HTTP_REMOTE_ORIGIN` → `REMOTE_ORIGIN`（本地
  get-host-info.sub.js 已有该 getter，http 页下同值）。
- 引擎侧无需改动：shim 在 M4-S24 已实现重定向跳重预检（递归 main fetch 语义，
  cors-redirect-preflight 30/30 的既有绿腿即同一机制）。

## 验证

- 单文件探针（`testharness-net-api preflight-redirect`）：**10/10 收口**，原
  「on preflight failed」五腿零扰动。
- 全通道 A/B（`netapi-m2s12-r1.json` vs **直接前轮**
  `netapi-m2s11-final.json`，逐 subtest，对账工具
  `.acceptance/net-api-compat/abdiff-m2s12-r1.py` / `abdiff-m2s12-r1.txt`）：
  **+5 增益 / 0 回归 / MISSING 0**（10995 → 11000）。5 条旧名
  「after preflight failed」（基线 Fail）与 5 条新名「after preflight
  succeeded」（本轮 Pass）精确对应——改名 Fail→Pass；per-file Δ 仅
  cors-preflight-redirect（5/10 → 10/10），其余 487 文件零变化；stderr
  0 panic（0 字节）。

## 记账

1. S11 记账 1 / S11b 记账 1 **清偿（归因纠正，非代码修复）**；两档原文保留
   假设并已加纠正指针。教训入 learnings：快照语料红腿先查上游最近提交。
2. 语料持久化：wpt-data 为独立版本化仓（当前 ref v1.10，升级即删目录重建），
   本补丁按既有约定记入 `scripts/update-wpt-data.sh` 头部缺口清单（升级新
   ref 后需重新核对；新 ref 快照 ≥ 2026-09-12 则无需补）。是否推进语料仓
   滚动新 tag（使该文件原生含修复）挂待决项，未动。
3. 本地 wpt-data 无同步时点记录：后续如建 corpus 漂移巡检（对上游 HEAD 抽查
   net-api 相关文件），另行立项。
   **[2026-10-29 M2-S13 首轮巡检已执行并结论不常态化立项：语料实际已于
   2026-10-29 00:19–00:39 被全量同步至上游 master（mtime 取证），实测漂移面
   仅 9 文件（13.5h 窗口），1 对齐清偿 / 8 分类挂账；见
   M2-S13 档](2026-10-29-m2-s13-corpus-drift-sweep.md)。**
