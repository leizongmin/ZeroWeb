# M0 甄别一：fetch 未导入目录逐目录对账

日期：2026-11-06。方法：上游 pin rev `315976933870b34d6ea30e3f6643403edae678ba`（lib.sh
WPT_REV）Git Trees API 全量清单（fetch/ 1,043 blob）× 本地语料逐文件比对 × 556 个缺失
case 文件全量下载做内容级甄别（`.py` 端点引用 / worker / iframe / `.https.` / wss），
分母侧取 coverage-2026-11-05（wpt.fyi aligned @1eecaa0c30，fetch 13,288 子测试分母）。

## 一、通道既有账（M0 实测基线）

`make testharness-net-api` 全量通道实跑（test-guard 包裹，2026-11-06）：

- **websockets 面**：107 文件，377P / **0F**（详见 websockets 甄别档）
- **xhr 面**：91 文件，586P / 7F / 1T（详见 xhr 残余簇档）
- fetch/streams/url/mimesniff/eventsource 各腿数字见全量通道日志（本轮 evidence 落档）

通道既有通过率（数值门倍率用，M0 实测下限）：**98.5%+**（websockets 100%、xhr
98.7%，fetch 腿全量日志为准）。

## 二、逐目录甄别表（fetch 未导入/未收满面）

「case 数」= 上游 window 可执行 case 文件（.html/.any.js/.window.js，不含 resources
helper 与纯 .py/.headers/资产）；「py 引用」= case 源码引用 `.py` 端点的文件数；
「den」= wpt.fyi 子测试分母（含变体膨胀，> 通道可枚举面为常态）。

| 目录 | case 数 | py 引用 | den 子测试 | 甄别结论 | 批次归属 |
|---|---:|---:|---:|---|---|
| fetch/api/redirect | 16 | 12（redirect.py 为主） | ~（归 api 8,049 内） | redirect.py 语义可 fixture shim（302/307/308 + location，v2 echo 族先例） | **M1b 批次**（pull + redirect.py shim） |
| fetch/api/policies | 32 | 6 | ~（同上） | CSP connect-src / referrer-policy 请求面——策略头面，引擎 CSP 策略通道域 | **排除**（security-hardening 域，沿 v2 记账） |
| fetch/metadata | 137 | 25（record-header.py 等） | 2,336 | Sec-Fetch-* 元数据头策略面；且 56 文件 `.https.` 页锚 | **排除**（策略/头面，security-hardening 域） |
| fetch/http-cache | 19 | 2 | 554 | HTTP 缓存语义本体 | **排除**（47 号 http-cache goal 承接，划界） |
| fetch/h1-parsing | 4（全在本地） | 5（resources py） | 234 | **case 文件已全本地但 0 被跑**：3 个 `.window.js` 形态通道枚举器不认（M0 根因②）；resources 108 资产 + 5 py 未拉 | **M1a 批次**（枚举器窗体解锁 + resources 补拉） |
| fetch/orb、fetch/corb | 58 | 4 | 259 | ORB 读取算法面 | **排除**（划界「HTTP 栈本体/ORB」） |
| fetch/fetch-later | 54 | 3 | 154 | fetchLater 新 API，引擎未实现 | **排除**（新 API 面） |
| fetch/range | 13 | 8（long-wav.py 等） | 123 | Range 请求栈语义 | **排除**（HTTP 栈本体） |
| fetch/content-encoding | 13 | 3 | 117 | gzip/br/zstd 内容编码栈 | **排除**（HTTP 栈本体） |
| fetch/compression-dictionary | 41 | 16 | 110 | CompressionDictionary 新 API，34 文件 `.https.` | **排除**（新 API 面） |
| fetch/cross-origin-resource-policy | 10 | 9 | 107 | CORP 响应头策略面 | **排除**（策略/头面，security-hardening 域） |
| fetch/security（dangling-markup） | 14 | 1 | 92 | dangling-markup 溢出策略面 | **排除**（策略面，security-hardening 域） |
| fetch/nosniff | 7 | 6 | 65 | X-Content-Type-Options 策略面 | **排除**（划界：nosniff 策略面归 security） |
| fetch/local-network-access | 26 | 5 | 66 | LNA 许可面（新 API + 策略） | **排除**（策略/新 API 面） |
| fetch/stale-while-revalidate | 7 | 6 | 9 | 缓存 revalidation 语义 | **排除**（47 号缓存层承接） |
| fetch/connection-pool | 7 | 1 | 9 | 连接池栈语义 | **排除**（HTTP 栈本体） |
| fetch/redirect-navigate | 3 | 2（/common/redirect.py） | 121 | 导航级 redirect 编排（window.open + 302 链） | **排除**（多页导航编排，runner 无多文档管道——multi-globals 同因） |
| fetch/origin | 1 | 1（redirect-and-stash.py） | 43 | 单案 request-origin 语义；stash 跨请求编排依赖 | **挂账**（单案 43 子测试，stash 端点 shim 成本/收益不成比；47 号缓存层或后续小片） |
| fetch/content-disposition | 3 | 0 | 12 | Content-Disposition 头透传面 | **M1b 批次候选**（静态面，随 redirect 批次顺带盘点） |
| fetch/data-urls/content-type/content-length/images/redirects | 已入通道 | — | 828 | 已在 DIRS；余量是 resources helper 与 `.window.js` 形态未枚举 | **M1a 批次**（窗体解锁自然收） |
| fetch/api/crashtests | 5 | 0 | — | 崩溃回归面 | **排除**（v1 起既定口径） |
| fetch/api/idlharness.* | 1 | 0 | — | WebIDLParser build 期资产 | **排除**（skip 规则既有） |

## 三、已入通道目录的残余缺口（api/ 内）

已导入目录（headers/request/response/body/basic/abort/credentials/cors + 自包含面）
case 文件基本齐平，缺的是：

1. **服务端 .py 端点**（api/resources 20 个未拉：redirect.py、inspect-headers.py、
   method.py、authentication.py、cache.py、preflight.py、status.py、trickle.py 等）——
   runner 以 Rust fixture 等价承接（v2 先例：echo-content / access-control-*），**不是
   语料缺口**；未 shim 的端点被引用的案按真实缺口失败，属 M1b/M2 shim 批次面。
2. **api/request/destination**（iframe/worker 面 20 资产）——runner 无多 frame/worker
   管道，排除口径不变。
3. **api/request|response/multi-globals**（5 文件）——多全局面，排除口径不变。

## 四、DC-4 数值门判定输入

地址池（fetch 侧 v3 范围内可导入面）= api/redirect 批次 + 窗体解锁残余
（h1-parsing/content-type/content-length/data-urls）+ content-disposition 小面，
估算 ≤ 500 子测试（den 口径；通道实跑口径更低）。三族合计地址池 < 10k ⇒ 走
**决策规则 <10k 分支**：优先非 py 端点目录、py 密集面按 fixture 可行性分批、
worker/iframe/多文档面如实排除。数值门条目化回填见主文档 DC-4 节。
