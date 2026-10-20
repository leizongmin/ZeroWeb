# M4-S24 — cors 域重定向再 preflight + Origin opaque 语义修齐

**日期**: 2026-10-01
**基线**: M4-S23（2026-10-01，9248/11658 = 79.3%，fetch 1449/2081 = 69.6%）
**结果**: **9267/11658 = 79.5%**（fetch **1468/2081 = 70.5%**，+19 腿零回归；xhr
557/584 / mimesniff 100% / streams 88.6% / url 73.0% 持平；eventsource 33/34——
request-cache-control 页全量争用超时再现，隔离复跑绿，在册族）。make test 19538
passed / 0 failed（68 套全绿）+ clippy `-D warnings` 干净 + fmt 干净。

## 变更清单

### shim — 重定向再 preflight（fetch spec cors-preflight-fetch 递归主 fetch 语义）

- hop 循环重定向后：请求仍携非 safelisted method/自定义头 → 对新 URL **重跑
  preflight**（ACRM/Accept: */*/ACRH + Origin 按 opaque 语义；ACAO（含 opaque
  `null` 形态）/ACAM/ACAH 覆盖判定；preflight cache 命中跳过；结果入库；失败 →
  network error——cors-redirect-preflight「after redirection」success/failure
  双族 +12 腿）。
- `_zwPreNamesOf(headersWire)` 抽取共享（主 fetch 与 hop 再 preflight 同规；
  skip 名单补 user-agent）。

### shim — Origin opaque 判定修齐

- 「当前跳相对文档是否跨源」改 **scheme 宽化比较**（`_zwUrlOriginLenient`——
  https→http 归一）：runner 页面锚 https://wpt.test vs WPT 宇宙 http://* 端点——
  严格比较把同源跳误判跨源（cors-redirect「same origin to cors」5 腿 Origin 误
  null 根因）；跨源判定（Location ≠ 当前跳）保持严格。

### shim — preflight `*` 通配 Authorization 例外面

- 主 fetch 与 hop 再 preflight 的 ACAH 覆盖检查：`*` 不覆盖 authorization（须
  字面列出——cors-preflight「authorization should not be covered by wildcard」面）。
- preflight cache 命中：star 条目对 authorization 同样不放行（cors-preflight-cache
  「does not reuse wildcard header entries for Authorization」面）。

## 解锁明细（+19 对照 S23，REGRESSED 0）

- cors-redirect-preflight 12（301/302/303/307/308 的 same-origin→cors、cors→another
  cors、cors→same origin 的 success/failure 双族部分）
- cors-redirect 5（same origin to cors 的 Origin 保留）
- cors-preflight-star 1（credentials: false POST `*` method 面）
- cors-preflight 1（authorization wildcard 拒绝面）

## 残余（cors 域 ~111 腿）

- **Referer 面**（cors-preflight-referrer ~10 腿）：preflight 请求须带文档 Referer
  （x-preflight-referrer 回读；referrer-policy 变体深水）。
- **非法头名校验面**（cors-preflight-not-cors-safelisted「accept/」族 ~8 腿）：
  Headers 非法名应 TypeError 而非静默丢弃。
- **credentials/cookie 管道腿**（cors-cookies* / cors-redirect-credentials）。
- **重定向后 opaque origin 的 ACAO `null` 匹配**（cors to another cors 部分腿——
  需 cors check 对 opaque origin 序列化 'null' 的字面匹配语义）。
- **eventsource/request-cache-control 全量争用超时**——隔离绿（在册族）。
