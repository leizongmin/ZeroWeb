---
date: 2026-11-06
modules: engine
---

# fetch cors check 按 spec 步序重排——`*` 通配步骤先于 origin 比对，opaque 分支不可先行特判

## 问题

net-api-compat-v3 M2 修 authentication-redirection 跨源腿（跨源重定向链
`Failed to fetch`）时，第一版修复在 `_zwFetchApplyFilteredResponse`
（crates/engine/src/js_dom_shim/part01.js）的 cors check 里加 opaque origin
先行分支（`allowOrigin !== 'null'` 即拒），把 fetch/api/cors
cors-redirect + cors-redirect-preflight "cors to another cors" 十个子测试
Pass→Fail——与修复目标同族（都在跨源重定向链上）的既有绿面回退。

## 根因

fetch spec 的 CORS check（https://fetch.spec.whatwg.org/#cors-check）是
**有序算法**，不是若干并列条件的合取：

1. ACAO 缺席 → failure；
2. credentials 非 include 且 ACAO `*` → **success（先行短路）**；
3. byte-serializing a request's origin ≠ ACAO → failure；
4. credentials 非 include → success；
5. ACAC true → success，否则 failure。

两个分支性来源在此交汇：request's origin 经跨源重定向链被置 opaque
（HTTP-redirect fetch），序列化为字面 `null`；而服务端（如
preflight.py fixture）缺省回 ACAO `*`。为个案加的「opaque 先判 `null`」
特判遮蔽了步骤 2 的 `*` 短路，`*` 被拖进步骤 3 与 `null` 比对恒失配。
分支顺序 = 算法步序，特判前置就是改写算法。

## 解决方案

按 spec 步序写，opaque 仅替换**比对基准串**（request's origin 的序列化值），
不新增先行分支：

```js
if (credentials !== 'include' && allowOrigin === '*') {
  // 步骤 2 恒过（opaque 同权）
} else if (allowOrigin !== (opaqueOrigin ? 'null' : requestOrigin) ||
    (credentials === 'include' && String(allowCreds).toLowerCase() !== 'true')) {
  throw new TypeError('Failed to fetch');
}
```

opaque 标志来自重定向链（`_zwHopOriginOpaque`）——逐跳重定向响应的门控传
**先序跳**的值（spec 里 request's origin 随跳演进），终局响应传全链终值。

## 避免方式

为 spec 算法补分支前先把算法定义抄成有序清单再落码；新条件找不到步序
位置时，优先怀疑它是既有步骤的**参数化**（如比对基准换源）而非新步骤。
回归面不只跑修复目标的用例——同算法族的既有绿面（本例 cors-redirect 全族
115 子测试）要一并 FILTER 复跑逐对比对。
