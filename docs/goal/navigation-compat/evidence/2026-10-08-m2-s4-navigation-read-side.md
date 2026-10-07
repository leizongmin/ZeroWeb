# M2-S4 — Navigation API 最小面（read side + currententrychange）（2026-10-08）

**通道**: `make testharness-navigation`（test-guard 包裹；全量批 TIME_LIMIT=2700——见「运行时注记」）
**全量运行日志**: [2026-10-08-m2-s4-full-corpus.txt](2026-10-08-m2-s4-full-corpus.txt)
**前序**: [2026-10-08-m2-s3-history-queue.md](2026-10-08-m2-s3-history-queue.md)

## 切片内容（master 下一步计划 ①——M2 ④ 评估切片的第一可达子集）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| `window.navigation`：currentEntry / entries() / updateCurrentEntry() / back() / forward() / addEventListener / on\*（navigate/navigatesuccess/navigateerror/currententrychange） | `crates/engine/src/js_dom_shim/part02.js` | Navigation API spec；WPT currententrychange-event、updateCurrentEntry-method |
| `NavigationHistoryEntry` 公共对象（index/url/key/id/sameDocument/getState()）——per-record 单例保 entries()[i] === currentEntry 身份面；index 在 detach 后返 -1 | 同上 | WPT current-basic / entries-array-equality / location-api `from.index===-1` |
| **entry 生命周期**：pushState→push（fresh key/id）；replaceState 与 location.replace→replace（**保 key** 新 id + 旧 entry detach）；**location.hash setter→replace 保 key**（WPT current-basic sixth + location-api）；href/assign 同文档→push（WPT sameDocument-after-fragment `location = "#hash"`）；traverse→按 session entry 反查 record 还原 key/id（WPT key-id-back-same-document） | 同上（hook 进 S3 的 pushState/replaceState/_setLocationHash/_setLocationPart/assign/replace/applyTraversal） | 同上 |
| entry `navState` 槽**独立于** history.state：updateCurrentEntry 只改 navState（history.state 不动）；pushState/replaceState 清 navState；getState 每次返结构化克隆 | 同上 | WPT state/history-pushState、updateCurrentEntry basic |
| `NavigationCurrentEntryChangeEvent` 构造器（`from` 必填缺省 TypeError；navigationType 缺省 null；属性反射） | `crates/engine/src/js_dom_shim/part05.js` | WPT constructor.html 四断言 |
| `location` 全局属性改 accessor——`location = v` setter 走 href 写侧导航（spec WindowLocation 拦截 `location=` 赋值） | `crates/engine/src/js_dom_shim/part01.js` | WPT sameDocument-after-fragment-navigate |
| `_setLocationPart` href 相对值回落解析：URL part setter 无 base 上下文（URL impl 已知限制）→ `new URL(value, oldHref)` 显式解析，仍失败才 SYNTAX_ERR | `crates/engine/src/js_dom_shim/part02.js` | spec location-href-setter 相对值合法（`location.href='#hash'` 片段导航） |

**已知限制（记录，slice ④-B 及后续）**：navigate 事件 / navigate() / reload() /
traverseTo() / transition.finished 拦截面未实现（onnavigate 仅存储不派发）；
sameDocument 恒 true；跨文档 entry list 重置面未建模（traverse 落到无 record 的
session entry 时 fallback 新建，key 不保）；`state/*` 两案依赖 `<meta name=variant>`
URL 变体（runner 不展开 variant——runner 缺口记账）。

## 数字

| corpus 域 | S3 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api | 1/228 = 0.4% | 19/222 = **8.6%** | +18 Pass |
| history/the-history-interface | 45/49 = 91.8% | 45/49 = 91.8% | 0 |
| history/the-location-interface | 31/36 = 86.1% | 31/36 = 86.1% | 0 |
| traversal/history-traversal | 42/45 = 93.3% | 42/45 = 93.3% | 0 |
| 全量 | 130 P = 30.4% | **148 P = 35.1%** | +18 |

零回归：S3 全部 Pass 案本轮全保持（逐案对照）。navigation-api Timeout 3→38——
navigation 对象在场后 async_test 得以 await（navigate 事件机器缺席 → promise 挂起
60s 超时），同 replace-before-load helper 补齐现象：真缺口显形，非回归。

## 运行时注记

全量批首次触发 test-guard 1800s 墙钟（语料增长 + 38 案 timeout × 60s）；本轮经
Makefile 既有 `TIME_LIMIT` 参数提到 2700s（批级墙钟，非单案超时——单案 60s 不变）。
**避免与 make test 并发跑**（cargo 目标目录锁 + CPU 争用会把 30min 的批顶过墙钟，
并发轮实证）。

## 质量门禁

- `make test`：全绿——20,189 P / 0 F（首轮与语料批并发跑时
  send_keys_dispatches 键盘用例超时一例，solo 复跑通过——并发负载敏感性，与 S2 轮
  同现象，非本切片回归）
- `cargo fmt --all -- --check`：零 diff（本切片仅 .js 变更）
