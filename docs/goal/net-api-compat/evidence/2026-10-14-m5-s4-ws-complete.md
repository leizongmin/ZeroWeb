# M5-S4 — websockets corpus 收口：017 工件 + basic-auth（2026-10-14）

**背景**：M5-S3 后 websockets 残余 2 案（basic-auth + constructor/017）。本切片
双收——**websockets corpus 376/376 子测试 = 100%，case 106/106 = 100%**（M5-S1
基线 334/375 = 89.1% 起步，四切片收口）。

## ② constructor/017 —— `__SCHEME is not defined`（runner 预处理工件 → 产品缺口）

**归因**（探针页三级定位）：constants.sub.js 的 `const`（__SERVER__NAME/
SCHEME_DOMAIN_PORT）跨脚本可见、`let`（__SCHEME/__PORT）不可见。最小复现二分：
`let x = 'init'`（带初始化器）跨脚本存活，`let x; x = …`（无初始化器）蒸发——与
`let` 语义无关，是**声明形态扫描面**缺口。

**根因**：classic 页面脚本经间接 eval 执行（R147 机制），跨脚本可见性由
[`scan_top_level_decl_exports`]（script_gen.rs，bing-t2 单遍状态机）把顶层声明
导出到 `globalThis`——`const|let` 分支的声明符接受集只有 `=` 或源尾，**`let x;`
（无初始化器、后续赋值）形态不导出**（注释声称「或行尾」但实现未含）。WPT
websockets/constants.sub.js 恰用此形态：`let __SCHEME; let __PORT;` + if/else
赋值，017 顶层 `__SCHEME + ':'` 直读 → ReferenceError。属产品面缺口（非 M5-S1
所记「预处理工件」——工件现象根因在声明导出扫描面）。

**修复**（script_gen.rs）：声明符后接受集 `=` → `= | ; | ,`。`;` 形态导出为
accessor（let 语义——后缀在源体后执行，getter 闭包持活 eval 词法绑定，读到赋值
后终值且跨脚本可读写）；`,` 收链首声明符（`let a, b = 2` 链后续名保守漏导出，
与现状一致）；无分号 ASI 形态（`let x\n`）仍不导出（保守，未语料化）。

**回归测试**：js_dom_bridge_tests/part50.rs `m5s4_uninit_let_decl_exports_cross_script`
（sloppy/strict 双态 × 017 形态 × `,` 链首）。

## ① basic-auth —— WS URL userinfo 预置凭据

**spec 语义**：WHATWG HTML「establish a WebSocket connection」——URL 含 credentials
（userinfo）→ **预置** `Authorization: Basic base64(user:pass)`（一步直发，无 401
挑战往返）。语料：`ws://foo:bar@wpt.test:<port>/basic_auth`。

**修齐三面**：
1. **net**（websocket.rs）：userinfo 凭据提取（手工解析 authority——url crate
   `username()/password()` panic 史，M3 catch_unwind 教训同源规避）+ RFC 4648
   base64 编码（零新依赖，data-url 仅解码面）+ 握手预置 `Authorization` 头。
   userinfo 百分号编码面（%XX 凭据）保守不解码（语料无此形态，记账）。
2. **runner fixture**（ws_fixture.rs）：`/basic_auth` 端点——凭据正确 101，缺失/
   错误 401 + `WWW-Authenticate: Basic realm="wpt"`；KNOWN_PATHS 白名单同步。
3. **fixture 宿主改写**（rewrite_wpt_authority）：userinfo 感知——权威段按最后
   一个 `@` 切分（旧 `split('@').next()` 把 userinfo 当 host → `*.wpt.test` 判定
   失败 → 不改写 → 真连外网失败）。

**回归测试**：net `websocket::basic_auth_tests::url_credentials_and_base64`
（userinfo 提取矩阵 + base64 known-answer ×4）。

## 门禁

- cargo fmt 干净；clippy（engine/net/wpt-runner，-D warnings）零告警。
- make test 全量（test-guard 包裹）——结果见 master.md 本切片条目。
- websockets 全量 376/376（含本切片两案 + M5-S3 全部五案回归）。

## P6 二期收口注记

websockets corpus **106/106 case 全绿**（不测面维持 M5-S1 记账：wss/h2 变体、
`.https.*` 5 案 TLS、multi-globals/unload-a-document/bfcache 结构面）。剩余记账
跨域项（与本 goal 通道无关）：userinfo 百分号编码凭据解码、url crate panic 防护
调用点审计（低优先观察项）。
