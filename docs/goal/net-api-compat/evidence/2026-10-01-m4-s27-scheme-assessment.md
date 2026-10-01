# M4-S27 — credentials/cookie 腿评估 + 页面 scheme 对齐评估（双记账，零代码实施）

**日期**: 2026-10-01
**基线**: M4-S26（2026-10-01，9290/11658 = 79.7%）
**结果**: 双评估记账，零代码实施（全量门禁沿用 M4-S26 状态：make test 19547P /
clippy / fmt 干净）。

## 评估一：cors 域 credentials/cookie 管道腿（~13 腿）→ 记账回流

- cors-cookies* / cors-redirect-credentials / credentials/authentication-* 需要
  **真实 cookie jar**（Set-Cookie 响应头持久化 + credentials: include 时 Cookie 头
  回发）与 **Authorization 重定向保留流**（Authentication.py + userinfo URL）——
  属浏览器存储/HTTP 栈状态面，非 net-api goal 范围（goal 排除「HTTP 栈本身」）。
- **回流去向**：cookie jar → storage goal（zero-storage 侧）或 zero-web P1（宿主
  网络状态）；userinfo/Authorization 重定向流 → security-hardening 邻域评估。
  本 goal 缺口清单记 P2 残余不实施。

## 评估二：runner 页面 scheme 对齐（https → http）→ 实施前需两项前置审计

**试验**（全量 testharness-net-api 实测，后回退）：

- 翻转面：testharness.rs 34 处（页面 URL/资源 fetcher strip/SW scheme 门/单测）+
  shim `_zwResolveIframeSrc` 1 处。
- **解锁 +29**：access-control-and-redirects-async「same-origin resource file」、
  access-control-basic-allow-access-control-origin-header（两腿长挂账）、
  cors-origin 12、cors-basic 3（port/protocol 变体——http 页锚下语义归位）、
  integrity/response-url 2、url a-element/a-element-origin ~10（http: 形态
  href/origin 期望面）。
- **耦合回归 −41**：
  1. url a-element/a-element-origin ×20——`Parsing origin: <http:...> against
     <about:blank>` 面：a.href 相对解析的文档基址随页面 scheme 变化，url shim
     about:blank/相对回退链需审计；
  2. fetch/api/credentials ×21——Authentication.py Authorization 重定向保留流 +
     userinfo URL 形态在 http 页锚下的 CORS check 链需审计。
- **净 −12**——翻转本身非一行改动，需前置修复两项再翻。**结论：记账不定稿，
  重入条件 = (1) url shim 文档基址回退审计 + (2) credentials/Authorization 流程
  审计完成**；两脚长挂账腿随翻转入袋。
