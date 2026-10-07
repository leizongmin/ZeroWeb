---
date: 2026-10-06
modules: engine,tests,wpt-runner
---

# 残留 http.server 占端口导致合并树集成验收假 FAIL

## 问题描述

slice36 合并树（687b12ecd）集成验收的活体三件套（timeline2 / baidu RP-6 / s32 全驱动）出现整面失败：multimatch、nameface、live-3frame 三面 FAIL，timeline2 round 1 即 GHOST。失败面恰好全部依赖本地 http.server 8123 提供的 fixture 页；oracle（14/14）与 baidu RP-6（真实站点）PASS。

## 根因分析

集成脚本自身的 http.server 启动即崩（`Address already in use`，traceback 落在 zw36-http.log）——8123 被 **slice36 worker 两小时前跑最小复现时遗留的 `python3 -m http.server 8123`**（cwd = `slice36/repro`）占住。残留 server 根目录是 repro 目录，集成 fixture（`s30-multimatch.html` 等）404，页面渲染为「页面加载失败 HTTP 404」，driver 探针读到的是错误页 → `window.s30mm` 等 named access 全 undefined → 判为 FAIL/GHOST。

三点叠加使假象很像真回归：

1. 失败面（named access/HC 集合）恰好是本切片改动面，方向上「像」合并碰撞；
2. exit code 全 0（driver 是诊断脚本，face FAIL 不改退出码），`LIVE_EXIT=0` 造成已通过错觉；
3. oracle/baidu PASS 证明「浏览器基本功能正常」，进一步把怀疑推向 shim 语义层。

定位关键一步：**看失败面的截图**（`integ-mm.png` 直接写着 404）——比读 JSON verdict 快得多。归属验证用 `/proc/<pid>/cwd` 确认残留进程属于本任务系后才能 kill。

## 解决方案

- 清残留进程（`kill <pid>`，先 `/proc/<pid>/cwd` 验归属）。
- 集成脚本加**预检守卫**：驱动前对每个 fixture 页 `curl -w "%{http_code}"` 断言 200，否则 `PREFLIGHT_FAIL` exit 2——端口被占时秒级暴露，不再烧完整轮后才从 verdict 反推。
- worker/探针的进程回收不能只 `trap cleanup`：脚本内启动的 server 若因端口占用直接死亡，trap 无可回收；反过来 worker 自己的成功启动进程也可能在会话结束后残留。**收尾一律 `pgrep -af 'http[.]server'` 按任务系 cwd 核对清零**。

## 如何避免

- 任何依赖本地 http.server 的验收脚本，第一步先预检 fixture 可达（200），第二步用 `pgrep` 核对 8123 无非本轮 server。
- face FAIL 时先看截图/HTTP 码排除加载面，再怀疑代码面——「失败面恰好=改动面」的巧合会诱导错误归因。
- 无人值守长流程的每段收尾把「杀本任务系 http.server」列入固定清理项（参考 zw33/35/36 live 脚本的 cleanup trap + pgrep 复核组合）。
