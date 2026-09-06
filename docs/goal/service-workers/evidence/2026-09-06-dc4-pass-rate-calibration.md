# DC-4 通过率校准报告（收口）

- Date: 2026-09-06
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- 分母: disposition contract 294 testharness 源 / 331 URL
  （`make audit-wpt-service-workers-disposition` 可确定性重建）

## Lane 分布与执行面

| Lane | 源数 | 处置 |
|---|---:|---|
| core | 78 | 纳入三 runner 常驻断言集 |
| fetch | 3 | fetch runner 专属 lane |
| defer | 22 | 逐案注记（更新/多客户端/动态服务器语义等后续切片） |
| gated | 149 | 依赖 https 服务环境/多客户端/动态 fixture，逐案理由见 evidence |
| skip | 42 | 超出支持边界（多 iframe 客户端枚举等），skip list 已注明 |

## Runner 基线（2026-09-06 实测，双跑 deterministic）

| Runner | cases | subtests | Pass |
|---|---:|---:|---:|
| testharness-service-workers（core） | 65 | 249 | 249 |
| testharness-service-workers-fetch（fetch/message） | 31 | 85 | 85 |
| testharness-service-workers-cache-storage（CacheStorage） | 25 | 318 | 318 |
| **合计** | **121** | **652** | **652** |

- core lane 78 源 + fetch lane 3 源全部有 runner 覆盖（65+31+25=121 case
  大于源数因 `.any.js` 双 variant 与 wrapper 变体成 case）。
- 校准口径：**core+fetch lane 覆盖率 100%**（81/81 源全部常驻执行）；
  defer/gated/skip 214 源为当前环境不可执行或待后续切片面，逐案理由在
  disposition 与各批 evidence。

## 质量门禁（本日）

- 三 runner 全绿（上表），clippy -D warnings（SW 四 crate）、fmt 干净
- 全工作区 `make test` 独立验证；zero-net `stale_etag_revalidation_is_coalesced`
  在满载 sweep 有一次环境时序 flake（隔离通过；net crate 本轮双流均未触碰）
- 既有 webview 集成 `update_permissions...` 本地超时项见 master.md CI 守护记录
  （CI 通过，待架构级处理）
