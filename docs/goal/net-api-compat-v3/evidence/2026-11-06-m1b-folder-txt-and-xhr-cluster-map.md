# M1b（部分）：xhr/folder.txt 补拉 + xhr 解锁面失败簇归因图

日期：2026-11-06。前置：M1a 枚举器窗体解锁（+1,624P 零回归）。

## 一、folder.txt 补拉 + A/B

上游 pin rev `xhr/folder.txt`（blob `bf1a1fde`，内容单行 `top`）补入本地语料
（`GOAL_PULL_TXT=1` 既有过滤器的漏网文件——M1 只拉了 `.py/.headers/.txt`
helper 于 resources/ 下，根面 `folder.txt` 不在任何已列子目录）。

A/B（`make testharness-net-api FILTER=xhr/`，M1a log vs folder.txt 后）：
交集 1,254 对，**CHANGED 2，全部 Fail → Pass**：

- open-url-base-inserted-after-open.htm（`<base>` 插入后 open 解析）
- open-url-fragment.htm（fragment 标识符解析）

其余 1,252 对字节一致——零回归。xhr 腿 903P → **905P**/344F/5T。

## 二、xhr 解锁面失败簇归因图（M1a 后 143 案 351 F+T 全量盘点）

M1a 让 201 个此前从未运行的 xhr 案落结果——以下为 DC-3 收敛对象的簇级
归因（按 subtest 数降序；簇名 → 案数/子测试数 → 归因 → 批次）：

| 簇 | 代表案（子测试数） | 归因 | 批次 |
|---|---|---|---|
| **redirect 面** | send-redirect.htm(10)、send-redirect-to-cors(12)、send-redirect-post-upload(5)、send-redirect-bogus*.sub.htm(7)、send-redirect-infinite×2、preserve-ua-header-on-redirect(2) | xhr 侧 302/307/308 redirect 语义 + `send-redirect*.htm` 引用的服务端 redirect 端点缺失（fixture 无 redirect.py 等价） | **M1b**：redirect.py shim（与 fetch/api/redirect 批次同端点，一次落地两族） |
| **setRequestHeader 面** | setrequestheader-content-type.htm(31)、headers-normalize-response(15)、setrequestheader-header-allowed(10)、bogus-name/value、allow-whitespace/empty-value | 头禁用表/值规范化语义缺口 + `inspect-headers.py` 等回显端点缺失混合；逐案二次归因后分语义/端点两路 | M2/M3 |
| **open() 方法校验面** | open-method-insecure(9)/bogus(8)/case-sensitive(7)/responsetype-set-sync(5) | method token 校验（CONNECT/TRACE 禁用、大小写保真、bogus token InvalidAccessError）引擎缺口 | M3 |
| **解码面** | responsetext-decoding(15)、responsedocument-decoding(5)、send-receive-utf16、overridemimetype 族(6)、send-content-type-charset(7) | overrideMimeType/charset 接线（encoding 底座耦合，37 号划界侧只收 xhr 接线） | M3 |
| **status 面** | status.h2.window.js(7)、status-error(4)/async(3)/basic(3) | status-line 语义 + **h2 形态案（HTTP/2 面，runner 无 h2——候选排除记账）** | M3（h2 面甄别后排除） |
| **responseXML 面** | responsexml-non-well-formed(7)/non-document-types(5)/media-type(4)/basic(2)/document-properties | XML 解析耦合（engine XML 管道） | M3/挂账 |
| **getResponseHeader 面** | getallresponseheaders(1)、getresponseheader-server-date/chunked-trailer/special-characters/cookies-and-more 等(~8) | 需带 Server/Date/Trailer/Set-Cookie 头的受控响应（fixture 响应头控制面） | M2 |
| **事件序面** | send-no/send/sync-* response-event-order 族(~8)、open-during-abort 族(4)、open-open-send 族(3) | 事件触发序（open() re-entry 冗余事件已实锤：expected [1,4] got [1,1,2,3,3,4]）——引擎状态机缺口，非语料缺失 | M3 |
| **auth 面** | send-authentication-basic 族(8)+competing-names-passwords(13) | HTTP Basic 401 challenge 端点（resources/auth1-11 未拉，fixture 需 auth 语义——M0 甄别档既定） | **M2** |
| **CORS preflight 面** | access-control-preflight-* 族(17)、access-control-basic-*(7)、response-with-body/exposed-headers(3) | corsca.py/preflight 端点缺失为主（fixture 已有部分 access-control-* 先例可复用扩表） | M2 |
| **杂项单案** | gzip/deflate 响应体(2)、send-conditional(2)、timeout-sync(2)、cookies.http.html 挂起(1)、permissions-policy(2 排除) 等 | 逐案归因随批次顺带 | M2/M3 |

排除面（沿 M0 既定口径）：permissions-policy 2 案（策略面）、status.h2 形态
（runner 无 HTTP/2，重估后正式排除或挂账）、manual/worker/iframe skip 照旧。

## 三、M1b 剩余项（下一步）

1. **redirect.py shim**（fetch/api/redirect 16 案 + xhr redirect 面 ~45 子测试
   同端点收益）——redirect 端点 Rust fixture 实现 + fetch/api/redirect 语料
   补拉 + 双族 A/B。
2. h2 形态案正式甄别（status.h2.window.js 等 7+ 子测试——排除记账落档）。
3. M2 auth/getResponseHeader 响应头控制面 fixture 设计稿。
