---
date: 2026-10-29
modules: wpt-runner,net-api-compat
---

# WPT 红腿排查先证上游漂移，再动引擎/runner——一次 stash 冲突假设的证伪

## 问题

net-api 二期把 `cors-preflight-redirect.any.js`「after preflight failed」5F 记账为
runner 缺口：假设上游 redirect.py（写 2 键 stash dict）与 preflight.py（6 键
schema）跨 fixture 互写 token → Python KeyError → 500 无 CORS 头 → fetch 拒绝，
runner 未仿真该拒绝链。

## 根因

两个假设都错：

1. **wptserve Stash 按（请求 path, uuid）作用域**：`request.server.stash` 每请求
   构造 `Stash(request.url_parts.path)`，键为 `(isomorphic_encode(path), uuid)`。
   redirect.py 与 preflight.py 路径不同 → 同一 token 的条目互不可见，跨 fixture
   互写冲突在真实 wptserve 里根本不存在。
2. **语料漂移才是真根因**：上游 2026-09-12 提交 651ffcb03「Fetch: redirects
   after a CORS-preflight are followed」把这 5 条腿整体改写——重命名
   「after preflight failed（期望 reject）」→「after preflight succeeded（期望
   200 + x-did-preflight=1）」，因 fetch spec 已改为重定向递归重入 main fetch
   （unsafe-request flag 仍在 + CORS-unsafe 头仍在 → 重定向跳**重新预检**）。
   本地 wpt-data 快照早于该提交，持有旧期望。

## 定位手法（可复用）

红腿动手前按序取证：

1. **上游实证**：wpt.fyi API 可达（`/api/runs` 拿 run id → GCS 下载该 run 的
   `summary_v2.json.gz`——注意实际是明文 JSON，格式 `{'s':'O','c':[P,T]}`）。
   Chrome 158 该测试 10/10 全绿 → 与「runner 缺口」假设矛盾。
2. **测试文件考古**：GitHub commits API 按 path 列该测试的提交历史，最近提交
   直接揭示语料改写（本例 6 周前）。
3. **基建语义核实**：怀疑 runner 仿真缺口的，先读 wptserve 对应实现
   （stash.py/request.py），确认「上游行为」再设计仿真。

## 解决方案

语料文件对齐上游 HEAD（`REMOTE_ORIGIN` getter + 新期望 200 + x-did-preflight=1）。
引擎无需改动——shim 在 M4-S24 已实现重定向跳重预检（递归 main fetch 语义），
新期望五腿一次通过；原「on preflight failed」五腿零扰动。

## 避免方式

- 跑批红腿的修复对象排序：**上游语料漂移 → runner fixture 仿真缺口 → 引擎语义**；
  引擎改动放最后。对着快照语料修引擎/runner 可能是把浏览器已经改掉的行为再抄回来。
- 「上游两条 fixture 共享 token 会冲突」这类推演，先查 wptserve 的实际作用域规则
  （stash path 域、preflight cache (origin, URL, credentials) 键等），不要按
  直觉字典语义推。
- 本仓 wpt-data 快照的同步时点记在语料文件与上游 HEAD 之间；红腿收敛轮先抽查
  该文件上游最近提交。
