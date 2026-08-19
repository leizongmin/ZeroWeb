# Service Worker IDL Harness 分母与裁决

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：M0 evidence（零 runtime 源码改动）
**Context 摘要**：[idlharness contexts TSV](2026-08-19-idlharness-contexts.tsv)
**逐项分母**：[idlharness subtests TSV](2026-08-19-idlharness-subtests.tsv)

## 来源分级

| 来源 | 覆盖 | 类型 | 置信度 |
|------|------|------|--------|
| 固定 revision case、IDL 与 harness 字节 | 11 个 Git 输入资产 | 一手事实 | 高 |
| WPT 生成 wrapper | window/dedicated/shared/service worker | 上游生成协议 | 高 |
| Chrome 127 localhost 执行 | 787 个 URL/subtest 对 | 独立运行证据 | 高 |
| context 裁决 | runtime/Cache/cross-global 实施边界 | 作者综合 | 待 ZeroWeb runner 验证 |

## 0. 执行摘要

- `idlharness.https.any.js` 的 manifest 生成 4 个 URL，不是 1 个普通 test：
  - window：**175 subtest**
  - dedicated worker：**155 subtest**
  - shared worker：**155 subtest**
  - service worker：**302 subtest**
- 总分母是 **787 个 generated-URL/subtest 对**，共有 324 个唯一名称。
- dedicated/shared 的 155 个名称完全相同，但属于两个独立生成 URL，不能去重。
- 该文件同时覆盖 Service Worker、Cache/CacheStorage、window/worker global exposure、
  event/client/controller/navigation preload 等接口，不能放入静态生命周期 core。
- 初始 inventory 的最后一个 no-signal review 已完成裁决；全量逻辑 review 从 71 降为 **70**。

机器清单 SHA-256：

- contexts：`49c8136516f6f0d8913d636f59440b7b3969dc6581668fdda5dba6aea2d0ef03`
- subtests：`e075612906d9c2f543a80330d8104c2a794fe62934789ba17bf1c509349586f9`

## 1. 固定输入

执行前逐字节比较 wpt.live 安全源与 pinned revision。case、3 份 IDL、idlharness 和
WebIDL parser 六项全部匹配；实际执行使用下列固定输入：

| Asset | Bytes | Git blob SHA |
|-------|------:|--------------|
| `resources/testharness.js` | 198,291 | `c7ce4f51e1db075809f861255417c27478b2c179` |
| `resources/testharnessreport.js` | 1,231 | `405a2d8b06f00fe8292e0e9d5b917e193cfd416c` |
| `resources/webidl2/lib/webidl2.js` | 125,228 | `bae0b2047595d0742f58384f01a6af5f69c3519e` |
| `resources/idlharness.js` | 145,966 | `57cefedc22a182704076f3d3bde7eb2592a733d5` |
| `interfaces/service-workers.idl` | 8,761 | `34af3372401eed53328b8bc5e7ea42b87ddd9b20` |
| `interfaces/dom.idl` | 23,163 | `1ddc084b949df64f36041f3a0c533468765dceb0` |
| `interfaces/html.idl` | 107,062 | `748cb63e0e953308a7b058e23e20e48861d8a3d7` |
| `cache-storage/resources/test-helpers.js` | 9,266 | `050ac0b542455ceb53ed36038af5b9b0810977cf` |
| `service-worker/resources/test-helpers.sub.js` | 10,485 | `74301523e7355ad8d62bcb568280edbc23fdaacf` |
| `service-worker/resources/empty-worker.js` | 15 | `49ceb2648a93410bdd5ee53ef0e114146210741b` |
| `service-workers/idlharness.https.any.js` | 1,966 | `8db5d4d10ff7b90f990d77394dac2dddbd2aa45f` |

合计 11 个 Git 对象、631,434 bytes。`resources/WebIDLParser.js` 是 WPT server 对
`resources/webidl2/lib/webidl2.js` 的生成路由，不是独立 Git blob。

## 2. 执行方法

1. 按 WPT `.any.js` 生成协议构造 4 个 wrapper。
2. window wrapper 注入 `GLOBAL.isWindow()`；worker script 注入 `GLOBAL.isWorker()`，
   按 META 顺序加载 testharness、WebIDL parser、idlharness 和两个 helper。
3. 在 `http://127.0.0.1` 可信源运行，保持 CacheStorage 与 Service Worker 可用。
4. 使用 Chrome DevTools Protocol 按真实墙钟等待 harness 完成，导出 result/name。
5. 以 Chrome for Testing `127.0.6533.119` 执行；浏览器状态仅证明测试注册和完成，
   **不作为 ZeroWeb 通过率或兼容性结论**。

直接用非 localhost IP 会使 CacheStorage 不可用；用 virtual-time budget 会在 SW 注册完成前
冻结页面。这两种不完整运行均已排除，不计入证据。

## 3. Context 分母

| Context | Generated URL | Subtest | Chrome 127 状态 | 裁决 |
|---------|---------------|--------:|-----------------|------|
| window | `idlharness.https.any.html` | 175 | 175 pass | defer-window-idl |
| dedicatedworker | `idlharness.https.any.worker.html` | 155 | 115 pass / 40 fail | defer-cross-global-idl |
| sharedworker | `idlharness.https.any.sharedworker.html` | 155 | 115 pass / 40 fail | defer-cross-global-idl |
| serviceworker | `idlharness.https.any.serviceworker.html` | 302 | 270 pass / 32 fail | defer-serviceworker-idl |

Chrome 127 早于 pinned IDL revision，失败主要表示旧浏览器缺少较新的 exposure/member；不能用
这些数字给 ZeroWeb 建立 red baseline。稳定验收分母是 URL 和 subtest 名称，而不是 oracle 状态。

## 4. 能力边界

### Window（175）

页面创建真实 registration、ServiceWorker 和 Cache 实例，覆盖：

- ServiceWorker/Registration/Container 对象、prototype、member 与实例投影
- NavigationPreloadManager
- Cache/CacheStorage
- window Navigator 上的 serviceWorker exposure

因此必须等 M1 runtime、页面 bridge 和 storage-cache-api 接线后执行。

### Dedicated/shared worker（155 + 155）

两者不创建 ServiceWorker 实例，但验证 Service Worker/Registration/Container 等接口在
generic worker global 中应暴露或不暴露，并执行 Cache/CacheStorage IDL。它们属于跨 global
exposure 兼容面，不是多客户端控制语义，也不应永久 skip。

### Service worker（302）

除基础接口外，SW global 还实例化并验证：

- `self` / `registration` / `serviceWorker`
- Clients、Client、WindowClient
- ExtendableEvent、InstallEvent、FetchEvent、ExtendableMessageEvent
- Cache/CacheStorage 和 NavigationPreloadManager

它横跨 M1 生命周期、M2 fetch/cache 和 M3 clients/message，须在 typed runtime 与 result
channel 完成后作为综合 IDL gate。

## 5. 证据矩阵

| 结论 | 来源 1 | 来源 2 | 一致性 | 置信度 |
|------|--------|--------|--------|--------|
| 总分母为 787 | 四个完成的 harness | contexts/subtests TSV | 一致 | 高 |
| dedicated/shared 各为 155 | 两个独立 wrapper | 名称集合逐项相等 | 一致 | 高 |
| serviceworker 为 302 | SW global 实例设置 | 完成的 remote harness | 一致 | 高 |
| 输入对应 pinned revision | Git blob SHA | wpt.live 六项逐字节比较 | 一致 | 高 |
| 不属于静态 core | IDL 对象实例要求 | Goal runtime/cache/client 分层 | 一致 | 高 |

## 6. 后续输入

1. M1 typed runtime 落地后先启用 window 中 ServiceWorker/Registration/Container 子面。
2. storage-cache-api 接线后启用 Cache/CacheStorage 子面。
3. worker result channel 落地后执行 serviceworker generated URL。
4. M3 clients/message 后以 302 项 serviceworker context 作为综合 IDL gate。
5. dedicated/shared exposure 作为跨 global 兼容切片，不计入单页面首批通过率。

## 7. 质量审查

- [x] case、IDL 与 harness 关键输入匹配 pinned revision。
- [x] 四个 generated URL 均在可信 localhost 完成。
- [x] 787 行逐项清单与四个 context 数字可反算。
- [x] oracle 状态与 ZeroWeb 通过率明确分离。
- [x] 未修改 runtime 源码、WPT 数据或既有 inventory 初筛记录。
