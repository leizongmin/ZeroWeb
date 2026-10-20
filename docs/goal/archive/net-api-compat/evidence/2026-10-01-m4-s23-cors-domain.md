# M4-S23 — fetch/api/cors 域页拉取 + preflight 语义修齐

**日期**: 2026-10-01
**基线**: M4-S22（2026-10-01，9059/11338 = 79.9%，fetch 1258/1760 = 71.5%）
**结果**: **9248/11658 = 79.3%**（分母 +320——cors 域 21 页 321 腿导入；fetch
**1449/2081**，cors 域新解锁 **+191 腿** 零回归；xhr 557/584 与 S22 持平 /
mimesniff 100% / streams 88.6% / url 73.0% / eventsource 33/34——
request-cache-control 页全量跑争用超时，隔离复跑双 subtest 全绿，在册争用窗族）。
make test 19536 passed / 0 failed（68 套全绿）+ clippy `-D warnings` 干净 + fmt 干净。

## 变更清单

### 资产腿

- `20-net-api-compat.sh` DIRS + `NET_API_CORPUS_SUBDIRS` 增 `fetch/api/cors`
  （M1 记账重入条件兑现——preflight cache/OPTIONS 三前置 S22 已落）；21 页 321 腿
  导入，基线 110/321。

### shim — preflight 语义修齐

- **preflight 请求带 `Accept: */*`**（浏览器行为；上游 preflight.py 校验该头）。
- **ACAM/ACAH 覆盖判定 spec 化**：非 safelisted method 须 ACAM 覆盖（缺 → 拒——
  「server refuses」面）；自定义头须 ACAH 覆盖（缺 → 拒）；`*` 通配两者。
- **Max-Age 0/负 → 不缓存**（原实现落默认 5s——cors 域 `max_age=0` 用例全组
  连锁误命中，单测页首个 preflight 污染全组根因）。
- **跨源请求被重定向跨源 → Origin opaque（"null"）**：当前跳 origin ≠ 文档 origin
  且 Location origin ≠ 当前跳 origin 才置 opaque（same-origin→cross-origin 保留
  文档 Origin——xhr redirects-async-same-origin 面）；跳间 Origin 头**替换**而非
  跳过（陈旧 Origin 残留根因）。
- **探针定位回归**：`_preNames` 前置重构漏了 `content-type` 跳过——CT 入 ACAH 覆盖
  检查致非 safelisted method + CT 请求全组 preflight 误拒（xhr PUT async 腿红斑；
  preflight 日志探针一轮定位）。eventsource format-bom 族未涉。

### runner fixture

- `fetch/api/resources/preflight.py`（上游行为等价——多 ACAO/clear-stash/
  credentials/OPTIONS 校验 + allow_methods/allow_headers/max_age/preflight_status +
  stash 四字段定序编码记录 did-preflight/ACRH/referrer/UA + x-did-preflight 等
  暴露头回读 + take-then-put 回存）+ `clean-stash.py`。**两 fixture 须置于 S18
  泛化 OPTIONS 兜底门之前**（原插入位被兜底拦截——preflight OPTIONS 恒 200，
  「server refuses」全组假阴性根因，探针 A/B/C 分解定位）。
- `fetch/api/resources/redirect.py` 对齐上游：OPTIONS + `redirect_preflight` →
  **落穿重定向逻辑**（preflight 命中重定向 → fetch 判失败）；Location 保留全量原
  query（上游 urlencode(url_parameters) 语义）。
- 路径点段归一：handler 入口 `..`/`.` 段归一（cors 域 `dirname/../resources/x.py`
  端点形态——不归一则 fixture 分支失配）。

## 解锁明细（cors 域 +191 对照 S22）

- cors-no-preflight 15 / cors-preflight 13→多数 / cors-preflight-star 11 /
  cors-preflight-status 26→多数 / cors-preflight-not-cors-safelisted 8 /
  cors-preflight-referrer 12→多数 / cors-basic 7→5 / cors-origin 9→多数 /
  cors-filtering 19→部分 / cors-redirect 10 / cors-redirect-preflight 15→部分 /
  cors-multiple-origins 全组 / cors-keepalive、cors-cookies*、cors-preflight-cache
  等零散腿（逐腿明细见 evidence JSON 对照）。

## 残余（cors 域 ~130 腿）

- **重定向再 preflight 语义**（fetch spec cors-preflight-fetch 递归主 fetch——
  cors-redirect-preflight「after preflight failed」族 ~15 腿）：重定向后的请求
  携非 safelisted 头须对新 URL 重跑 preflight——shim hop 循环未递归主 fetch，
  需 spec 重读后专设。
- **credentials 面腿**（cors-cookies* / cors-redirect-credentials——cookie 管道
  stash 形态）、**cors-filtering 剩余**（PNA/policy 邻界面）、**cors-preflight-
  referrer 剩余**（Referer 策略面）。
- **eventsource/request-cache-control 全量争用超时**——隔离复跑绿（在册争用窗族），
  暂不立案。
