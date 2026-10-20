# Cookie M2 — document.cookie 语义重写（shim in-JS jar，2026-10-12）

## 结论

可执行面 **533/533 = 100%**（M1 基线 21/533 = 3.9% → 100%，单轮收敛，无下行轮次）。
不可执行面 304 与基线一致（标记 Fail 303 + 传输腿 Timeout 1），总面 837 记账完整。
JSON 全量：[2026-10-12-m2-semantics.json](2026-10-12-m2-semantics.json)。复现：
`make testharness-cookies`。

## 实现面（js_dom_shim/part06.js `_zwCookieJar`，jar 本体不动）

RFC 6265bis 的 window 面语义（数组序 = 创建序；域/路径/安全上下文对
document.location 即时求值）：

| 语义 | 规范锚点 |
|---|---|
| 整串 CTL 拒收（%x00-08/%x0A-1F/%x7F，HTAB 豁免） | bis §5.6.1 |
| name=value 解析（无 `=` /空 name 忽略；两侧 WSP 裁剪） | bis §5.7 |
| 属性顺序解析（同名末值生效；非法值仅忽略属性） | bis §5.7 |
| cookie-date 宽松解析（token 式四要素 + 2 位年窗） | bis §5.1.1 |
| expires / max-age 过期判定（Max-Age 优先；≤0 即刻过期） | bis §5.7 |
| domain-match / path-match / default-path | bis §5.1.3/§5.1.4/§5.1.5 |
| Secure 属性安全上下文门（非安全上下文拒设） | draft-west-leave-secure-cookies-alone |
| `__Secure-` / `__Host-` 前缀拒收（大小写不敏感） | bis §4.1.3 |
| HttpOnly 属性经 document.cookie **整条忽略** | RFC 6265 §5.3 步骤 6 |
| `__Http-` / `__Host-Http-` 前缀恒拒（经 DOM 不可满足 HttpOnly） | Chromium 前缀扩展（WPT corpus 口径） |
| 覆盖**原位更新**（继承 creation-time → 检索序保持首设位置） | bis §5.6 步骤 20 |
| getter 可见性过滤（httpOnly/secure/domain/path）+ `"; "` 拼接 | bis §5.6 检索 |

## 本轮单测抓到的真 bug（修复后入回归）

- **覆盖序**：首版 splice+push 把覆盖项挪到末尾（last-update 序），bis §5.6 步骤 20
  要求原位（creation-time 继承）。corpus 的 ordering 案因 window.open 被 skip（M1
  甄别），通道未暴露——`test_document_cookie_semantics_c2k` 单测锁住（part02.rs）。
- **单测期望修正**：cookie name 大小写原样保留（`__SeCuRe-` 原样读出），仅前缀
  **匹配**不敏感。

## 混合案口径（M2 起）

`set_prefixed_cookie_via_dom_test`（prefix 系 31 子测试）= DOM 腿（document.cookie
写 + DOM 可见性断言）+ list.py 服务端回执腿。回执腿静态语料不可执行（credFetch
404 → Unhandled rejection），且「请求附带/响应入 jar」属 Mission 排除的传输域
——runner 侧替换为**仅 DOM 面计分**版本（`assert_dom_cookie` 语义逐字保留，
pinned 老版 testharness 的 promise_test 强制 thenable → 断言后补
`Promise.resolve()`）。回执面不可执行性在此记账 + wpt-suites.csv note 同步，不
静默。

## face 分账（2026-10-12）

| face | subtests | Pass（可执行） | 标记不可执行 | Timeout |
|---|---|---|---|---|
| cookies/attributes | 522 | 434 | 88 | 0 |
| cookies/encoding | 6 | 0（面全头驱动） | 6 | 0 |
| cookies（顶层 meta-blocked） | 1 | 1 | 0 | 0 |
| cookies/name | 111 | 33 | 78 | 0 |
| cookies/prefix | 76 | 31 | 45 | 0 |
| cookies/secure | 2 | 1 | 0 | 1（echo-json.py 传输腿） |
| cookies/size | 25 | 0（面全头驱动） | 25 | 0 |
| cookies/value | 94 | 33 | 61 | 0 |
| **合计** | **837** | **533** | **303** | **1** |

## 回归覆盖（DC-3 账本纪律）

- 常驻断言集：`make testharness-cookies` 通道（corpus 经
  `scripts/goals/18-cookies-compat.sh` 按需拉取，pin `315976933`）。
- 单测：`zero-engine` `test_document_cookie_semantics_c2k`（CTL 拒收/HTAB 豁免/
  expires/max-age/覆盖原位/path 可见性/domain 整废/前缀族/HttpOnly 忽略）。
- 账本注记：`imported-tests.txt` 为 **reftest** 专用账本（文件头契约），testharness
  通道 goal（webstorage/webmessaging 先例）以通道 + 证据目录为账，不入该账本。

## 门禁

- `make testharness-cookies`：533 Pass / 303 标记 Fail / 1 Timeout（837 记账完整）。
- `zero-engine` lib：3000 passed / 0 failed（test-guard 包裹 +
  `RUST_MIN_STACK=33554432`，与 `make test` 入口同环境）。
- `cargo fmt --all -- --check` 零 diff；`cargo clippy -p zero-engine
  -p zero-wpt-runner --all-targets` 零告警。

## 遗留 / 边界

- jar 仍不接 zero-net 真 jar（网络传输/响应入 jar 属 host-layer defer，goal 契约
  「jar 本体不动」）；`fetch()` 不携带 cookie。
- SameSite/CHIPS/3PC、domain/origin-bound 服务端预置、第二 window、frame 树、
  WebSocket 握手、非安全上下文前提案——M1 甄别的排除面不变（重入条件见 M1 证据）。
- CookieStore API（`cookie-store/` corpus）挂账不变。
