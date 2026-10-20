# M4-S26 — CORS-safelisted request-header 值面（preflight 触发规则 spec 化）

**日期**: 2026-10-01
**基线**: M4-S25（2026-10-01，9283/11658 = 79.6%，fetch 1484/2081 = 71.3%）
**结果**: **9290/11658 = 79.7%**（fetch **1491/2081 = 71.6%**；+9 零回归 + **2 腿
spec 必要回归记账**）；xhr 557/584 / mimesniff 100% / streams 88.6% / url 73.0%
持平；eventsource 33/34——request-cache-control 页全量争用超时再现，隔离复跑绿
（在册族）。make test 19547 passed / 0 failed（68 套全绿）+ clippy `-D warnings`
干净 + fmt 干净。

## 变更清单

### shim — CORS-safelisted request-header 值面（fetch spec §CORS-safelisted）

- **`_zwPreNamesOf` 重写为 CORS 变量头单一事实源**：accept（值长 <128 且无
  forbidden 字节且无 `"`）与 content-type（essence 三形 + 长度 <128 + 无 forbidden
  字节）**值条件入列**；accept-language/content-language **2024 spec 移出安全名单
  恒入列**；range 恒入列（强制 preflight 面）；origin/content-length/access-control-
  */referer/last-event-id/user-agent 恒跳过。
- `_zwFetchNeedsPreflight` 收敛为 `_zwPreNamesOf(...).length > 0`（unsafe method 恒
  preflight）——preflight 触发 / ACRH / ACAH 覆盖三面单一事实源。
- **preflight 请求头最小集**（origin + referer + ACRM + Accept: */* + ACRH——OPTIONS
  不转发原请求自定义头；转发使原 accept 值遮蔽 UA Accept: */*、preflight.py 校验
  400 误拒——accept 值规则腿根因，探针 A/B 对照定位；首版最小集漏 referer 致
  referrer 全组回归，一轮补齐）。

## 解锁明细（+9 对照 S25）

- cors-preflight-not-cors-safelisted 9 腿全绿：accept/`"`、accept 超长、accept-
  language ×2、content-language ×2、content-type text/html、content-type 超长、
  range bytes 0-。

## 回归记账（−2，spec 必要）

- cors-no-preflight「[Accept-Language: fr]」「[Content-Language: fr]」：2024 fetch
  spec 将 accept-language/content-language 移出 CORS-safelisted 名单（任值 →
  preflight）——两腿为上游 stale-failing 形态（pinned corpus 未随 spec 翻新），
  按 spec 判失败。

## 残余（cors 域 ~92 腿）

- **credentials/cookie 管道腿**（cors-cookies* / cors-redirect-credentials）。
- **cors to another cors 部分腿**（opaque origin 的 ACAO `null` cors check 字面
  匹配语义）。
- **cors-filtering 剩余**（PNA/policy 邻界面）。
- **eventsource/request-cache-control 全量争用超时**——隔离绿（在册族）。
