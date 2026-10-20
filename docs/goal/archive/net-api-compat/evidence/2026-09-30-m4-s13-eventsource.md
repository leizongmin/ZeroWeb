# M4-S13 — eventsource 跨源半 + cors.py fixture（eventsource 收口 100%）

**日期**: 2026-09-30
**切片**: M4-S13（上一轮 CONTINUE 指向的 eventsource 残余跨源 + runner 多源能力）
**原始数据**: [eventsource](2026-09-30-m4-s13-eventsource.json)

## 结果（六 corpus，对照 M4-S12）

| corpus | M4-S12 | M4-S13 | Δ pass |
|---|---|---|---|
| fetch | 1229/1766（69.6%） | 1229/1766（69.6%） | 0 |
| xhr | 240/451（53.2%） | 240/451（53.2%） | 0 |
| url | 4431/6065（73.1%） | 4431/6065（73.1%） | 0 |
| mimesniff | 1898/1898（100%） | 1898/1898（100%） | 0 |
| streams | 903/1019（88.6%） | 903/1019（88.6%） | 0 |
| eventsource | 33/34（97.1%） | **35/35（100%）** | **+2** |
| **合计** | **8734/11233（77.8%）** | **8736/11234（77.7%）** | **+2** |

**eventsource 六 corpus 首个 100% 收口**。request-cache-control 页 4/4（跨域
`www2.wpt.test` 两案转绿——runner 多源能力首次实战验证）；五 corpus 全量复核与
M4-S12 逐字节同值（零回归）。

## 修齐内容

- **runner fixture `eventsource/resources/cors.py`**（testharness.rs，上游逐字等价——
  https://github.com/web-platform-tests/wpt/blob/3159769/eventsource/resources/cors.py）：
  ACAO ← `?origin=`（缺省回显请求 Origin 头）+ ACAC ← `?credentials=`（默认 true）；
  `?run=cache-control` → text/event-stream + cache-control 模板体（回显请求
  Cache-Control 头，即上游 `pipes.sub` 的 `{{headers[cache-control]}}` 面）；其余
  `run` 值上游返空体。
- **跨源链路零改动即通**：runner fetch handler 本就 path 匹配（origin 无关，
  `{{domains[www2]}}` 替换面既有）；shim fetch 对跨源 cors mode 注入 Origin 头 +
  按 ACAO 过滤响应（M2/R3044 既有面）——EventSource 侧无新改动。

## 门禁

- `make test` 全绿（19506 passed / 0 failed）；`cargo clippy --workspace
  --all-targets -- -D warnings` 干净；`cargo fmt --all` 干净。

## 残余记账（fetch/api/cors 域评估结论）

- **fetch/api/cors 域重入为独立切片**：域内 fixture 与 eventsource cors.py 语义
  不同（fetch/api/resources/cors.py 为多模式端点：`?cors=`/`?location=`/
  `?checkResponse=`/preflight 断言面），且需 runner 支持 OPTIONS preflight 请求
  （现 fetch handler 非 GET 直接拒绝）+ engine CORS-preflight 语义。域 helper
  （corspreflight.js / not-cors-safelisted.json）已在本地（DIRS 预拉），测试页
  （.any.js）仍按记账不拉，待 fixture 通道扩展后解锁。
- transfer/detach 族结构性挂账不变（streams——需宿主 V8 detach，14 腿）。
