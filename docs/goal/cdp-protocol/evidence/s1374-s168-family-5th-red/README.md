# S1374 — S168 形态第 5 例 RED 捕获与 #0 升级取证（2026-09-20）

## 来源

S1374 轮（树变化刷新轮 + 到期期限轮，双腿活跑）门腿**首调即红**，二调收口绿。
本目录为红例保全取证：`gate-call1-red.log`（首调 make cdp-e2e 全输出）、
`gate-call2-closure.log`（复跑收口全输出）。steps-report.json / determinism-report.json
被复跑覆盖，红例二跑明细在覆盖前转录于本 README（实测转录，非推测）。

## 时间线

| 时刻 | 事件 |
|------|------|
| 08:48:56 | 开工环境快照：load 0.95/1.41（低位段）、盘 448G、零 zombie、922x 全 free、五 jsonl 组合 md5 2c2cdb6d 恒值 |
| 08:49:16 | 门腿首调起跑（R4549 新树态：3 Compiling = zero-engine → zero-page-runtime → zero-browser，与 S1367 门腿同型；dead_code warning match_media_to_json 真重编第七次再发射实证） |
| 08:56:53 | 首调收口 **EXIT=2，wall ~7min37s（延长形态），deterministic NO** |
| 08:58:57 | 复查残留：零 zombie、922x 全 free；load 1.54/2.95 |
| 08:59:01 | 复跑（复跑收口）起跑：0 Compiling（全缓存） |
| 08:59:31 | 复跑收口 **EXIT=0，wall ~30s（常态），deterministic YES，green 33 = expected 33 对称差 none，regressions 空** |
| 09:00:16 | mt 腿（make test 后台跑法）错峰起跑 |

## 红例形态（首调 run 对比，覆盖前转录）

- **run 1：全绿** — ok 33，failed 仅 `frames.click+evaluate`（期望失败步骤既有形态）。
- **run 2：8 步失败级联** — ok 24，failed 9 步：
  - 常态 locator 超时（10s）×2：
    - `screenshot.element` — `locator.screenshot: Timeout 10000ms exceeded`（waiting `#btn`，10001ms）
    - `page.setContent` — `page.setContent: Timeout 10000ms exceeded`（setting frame content, waiting "load"，10002ms）
  - **watchdog 60s 显式超时 ×6**（连续聚簇）：
    - `viewport.verified`（60003ms）/ `page.second.lifecycle`（60002ms）/ `target.getTargets`（60013ms）/ `target.attachDetach`（60011ms）/ `runtime.releaseObjectGroup`（60000ms）/ `emulation.userAgentOverride`（60013ms）
  - 以及 `frames.click+evaluate`（期望失败步骤，10007ms locator timeout 既有形态）。
- green_steps = 33（两跑并集恰为 expected 33，对称差 none，regressions 空）——
  deterministic NO 的成因 = run 2 独有 8 步新增失败（run 1 同步骤全绿）。
- wall 核算：6×60s watchdog + 2×10s locator + 常态 ~30s ≈ 7min37s 与实测吻合。

## 四点归因

1. **形态**：run 2 中段起单点卡死（`screenshot.element` 起连续不恢复），两超时形态
   （Playwright locator 10s + 步骤 watchdog 60s）叠加，六步连续 watchdog = 页面/CDP
   服务端整体无响应，非单步逻辑错。run 1 同树同码全绿 → 非确定性回归。
2. **wall 伴生形态**：~7min37s 延长形态，与 S1313 停滞（~7min33s）、缺步第 7 例
   （~7min41s）同量级；复跑 ~30s 常态回绿。
3. **负载窗相关性**：起手低位（load 0.95/1.41）、窗尾 1.54/2.95（五分钟均值含本门
   自身 watchdog 空转贡献）——低-中负载窗发作，负载相关性不足以单因解释（S218/S228
   先例负载样本口径记档，对 #0 取证有价值）。
4. **机械成因**：未定位（同 #0 长期挂账）。R4549 paint 路径相关性**排除**：
   run 1 全绿含全部 paint/screenshot 步骤 + 复跑全绿同码；且 S168 形态早于 R4549
   约 1200 轮已存在（S168/S268/S1342/S1366 先例）。

## 家族记账更新

- **S168 形态族第 5 例**（S168 run1 11 步 → S268 run2 单步 → S1342 run2 单步 →
  S1366 run2 三步聚簇 watchdog 亚型 → **本例 run2 8 步聚簇 watchdog 亚型扩大**）。
  间隔序列 ~100 → ~1074 → 24 → **8 轮**——**收窄至个位数触发「升级 #0 排查」条款**，
  且三步聚簇 watchdog 亚型再现并扩大（3 步 → 6 步 watchdog + 2 步 locator 超时）。
- **缺步/停滞族**：本例为 run 2 步骤级显式 watchdog FAIL 形态，按 S1366 先例归
  S168 形态族（缺步/停滞族口径 = 静默缺步 + flow 级 spawnSync ETIMEDOUT 停滞），
  S1314-S1373 连续六十轮零再现计数**不受本例打断**（S1374 门腿复查窗口内归因完成）。

## #0 升级取证价值

- 新样本形态：**run 2 专属**（第 3/4/5 例均 run 2；run 1 仅 S168 首例）——双跑第二跑
  发作偏置在案，指向跨跑残留态/资源面（headless 进程池、端口、临时态）而非纯随机。
- 发作点：中段（screenshot.element 起步），此前步骤（goto/evaluate/click/network/
  cookies/dialog/frames.access/emulation.media/screenshot.viewport+fullPage）全绿。
- watchdog 六连 = 服务端事件循环/渲染管线整体停摆窗口 ≥6min，test-guard --time-limit
  600 未触发（wall 7min37s < 600s 上限——注意该上限按 invocation 计，贴近值需关注）。
- 保全产物：`gate-call1-red.log`、`gate-call2-closure.log`、本 README 转录段。

> 注：两份 gate log 为 /tmp 原始日志的保全副本；工作区绝对路径已按「绝对路径零入控制面」纪律通用化为 <repo-root>（S1343 先例口径），其余内容逐字保留。
