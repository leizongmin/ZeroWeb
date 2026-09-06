# M1 剩余 WPT 裁决

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：M0 evidence（零 runtime 源码改动）
**逐案裁决**：[final review TSV](2026-08-19-m1-final-review.tsv)
**资源闭包**：[final resources TSV](2026-08-19-m1-final-resources.tsv)

## 来源分级

| 来源 | 覆盖 | 类型 | 置信度 |
|------|------|------|--------|
| WPT manifest + 固定 revision 正文 | 25 case、40 个关键传递资源、blob SHA | 一手事实 | 高 |
| 页面运行时 test 生成逻辑 | 99 个 subtest | 一手事实 | 高 |
| Goal support envelope | 单页面边界、基础生命周期/fetch 目标 | 项目契约 | 高 |
| decision 分类 | gated/skip 与实施顺序 | 作者综合 | 待运行验证 |

## 0. 执行摘要

- 从 message-channel 批次后逻辑剩余的 120 个 `review` 中，审计最后 25 个
  `milestone=M1` 文件。
- 25 案实际产生 **99 个 subtest**，其中：
  - **19 case / 88 subtest** 为 `gated`：依赖动态 WPT server、HTTP cache/update、
    request metadata、navigation/fetch 或 controller 能力；
  - **6 case / 11 subtest** 为 `skip`：依赖 partitioned storage、多窗口/closed global、
    Clear-Site-Data 或 static routing，超出当前 support envelope。
- 本批没有可加入静态 core 的完整上游 case。`multiple-register` 虽含两个基础断言，但同文件
  还验证 iframe global identity；不拆分上游验收单位。
- 至此初始 inventory 中 **57/57 个 M1 review 全部完成人工传递审计**：
  14 no-signal + 11 iframe + 7 message-channel + 25 final。
- 全量 inventory 的逻辑剩余 review 从 120 降为 **95**；这些文件属于 M2/M3 或
  `other-api/security`，不再是 M1 分母的不确定项。

TSV SHA-256：

- review：`003955a2ed074e10f940a5b9afc079bc7432ba441bf83d6d62b8d3d3a7876a3b`
- resources：`0759f0accddea50172b8eab801270369796a321fe49f6473569ed28575ce3783`

## 1. 审计方法

1. 从初始 inventory 筛出 `disposition=review`、`milestone=M1` 的 57 个文件。
2. 扣除前三批 32 个已裁决路径，得到 25 个互斥剩余 case。
3. 读取全部页面断言；展开 `for`/`forEach` 生成的测试，不以 test 函数文本出现次数代替
   实际 subtest 数。
4. 追踪动态 handler、worker import、iframe/window fixture、server stash、cookie、
   redirect、response header 和 streaming response。
5. 对 25 个 case 和 40 个关键传递资源计算 Git blob SHA。

`registration-updateviacache.https.html` 实际生成 25 个 subtest（4 + 16 + 4 + 1），
`update-bytecheck*.https.html` 各生成 8 个，`Service-Worker-Allowed-header.https.html`
通过 helper 调用生成 8 个。浅层声明计数会把本批错误低估为 60。

资源清单共 40 个对象、39,498 bytes。固定 commit 的 jsDelivr 对
`partitioned-service-worker-third-party-window.html` 返回 301 跳转；审计使用跳转后的
raw GitHub 正文计算 blob SHA，没有把 CDN 跳转提示文本记作 fixture。

## 2. 动态 update 与 HTTP 语义

### Update job（11 case / 69 subtest）

| Case 组 | Case | Subtest | 关键服务端语义 |
|---------|-----:|--------:|----------------|
| activation/multiple update | 2 | 5 | 时间/随机字节、并发 update、等待 worker |
| updateViaCache | 1 | 25 | max-age、Last-Modified、main/import cache 矩阵 |
| bytecheck | 2 | 16 | main/import 字节变化、classic/module、跨源 CORS |
| update() / importScripts | 3 | 14 | stash 次数、MIME/redirect/404/语法错误、文件切换 |
| request metadata | 2 | 2 | UUID 注入、ETag、请求头捕获和消息回传 |
| registration script type | 1 | 7 | stash 驱动 classic/module 脚本交替 |

这些 handler 的输出由请求次数、时间、随机值、缓存状态或请求头决定。把 Python 源码作为
静态 JavaScript 返回，或预生成单一响应，都会改变被测语义。

### Navigation 与 recovery（4 case / 4 subtest）

- `update-after-navigation-redirect`：三段动态 redirect chain 上的三个 registration 都须更新。
- `update-after-oneday`：依赖浏览器测试开关，把 registration 视为超过 24 小时。
- `update-on-navigation`：`trickle.py` 延迟并流式写 response，和 update 并发。
- `update-recovery`：cookie 在坏 fetch interceptor 与恢复版本之间切换。

### Worker global update（1 case / 1 subtest）

`ServiceWorkerGlobalScope/update.https.html` 由时间戳 handler 生成 worker；worker 在消息事件中
调用 `self.registration.update()`，两个 iframe fetch 用响应正文验证
updatefound/activate/fetch/message 顺序。它同时依赖动态 update、worker registration 投影和
M2 fetch interception。

## 3. Server policy 与 scope

### Service-Worker-Allowed（1 case / 8 subtest）

WPT server pipe 动态设置 `Service-Worker-Allowed`，覆盖相对、绝对、父路径和跨 origin
header 值。该文件必须在支持 response-header pipe 和跨源 WPT host 的 runner 中执行。

### Soft update Fetch Metadata（1 case / 3 subtest）

handler 把每次 classic/module/main/import 请求的 `Service-Worker` 与 `Sec-Fetch-*` 头写入
server stash；页面 navigation 触发 soft update 后轮询 JSON 记录。静态资产无法表达请求记录。

### Mixed iframe registration（1 case / 3 subtest）

`multiple-register.https.html` 同时验证同 global 重复注册、iframe global 的 wrapper identity
和 10 路并发注册。iframe 通过动态 404 text response 建立独立 global，因此等 runner 的
Document/container bridge 与动态 server 后整体纳入。

## 4. 明确 skip

| 类别 | Case | Subtest | 排除原因 |
|------|-----:|--------:|----------|
| HTTP/cross-origin popup redirect | 1 | 2 | HTTP remote origin、popup、动态 redirect |
| Partitioned multi-context | 1 | 1 | 三方 window + nested iframe + 分区 SW 身份 |
| Closed-window global | 1 | 1 | 从关闭 popup 保存 container 后调用 register |
| Clear-Site-Data | 2 | 5 | 立即清 registration/controller 与中止 extendable fetch |
| Static routing | 1 | 2 | `InstallEvent.addRoutes()`、Cache API、module worker |

这些不是当前单页面基础 SW 生命周期/fetch 目标的缺失测试。若未来扩大 support envelope，应
恢复完整上游 case，而不是复制其中部分断言。

## 5. 证据矩阵

| 结论 | 来源 1 | 来源 2 | 一致性 | 置信度 |
|------|--------|--------|--------|--------|
| 本批为 25 case / 99 subtest | 页面及循环生成逻辑 | TSV 机器求和 | 一致 | 高 |
| 19 gated / 6 skip | handler/fixture 正文 | support envelope | 一致 | 高 |
| 动态 handler 不可静态化 | stash/time/random/header/cookie 逻辑 | 页面断言依赖响应变化 | 一致 | 高 |
| M1 review 已全部裁决 | inventory 中 57 个 M1 review | 四批路径互斥且合计 57 | 一致 | 高 |
| 逻辑剩余 review 为 95 | 初始 152 | 152 - 14 - 11 - 7 - 25 | 一致 | 高 |

## 6. 后续输入

1. RFC 批准后，M1 runner 先执行已资产化的 11 个 core case。
2. MessagePort 结果通道落地后加入 2 个 message-lifecycle case。
3. 单 iframe bridge 落地后恢复单 iframe defer case。
4. 动态 WPT server adapter 若进入支持范围，再按本清单恢复 19 个 gated case。
5. M2/M3 与 other-api/security 的 95 个逻辑 review 继续按里程碑分批裁决。

## 7. 质量审查

- [x] 25/25 case 正文已读，manifest blob SHA 匹配。
- [x] 40/40 关键 handler/worker/fixture 已读取并记录 blob SHA。
- [x] 循环生成测试已展开，99 个 subtest 无文本计数低估。
- [x] server-dynamic gated 与 support-envelope skip 已分开。
- [x] M1 的 57 个 review 路径互斥且全部有裁决。
- [x] 未修改 runtime 源码、WPT 数据或既有 inventory 初筛记录。
