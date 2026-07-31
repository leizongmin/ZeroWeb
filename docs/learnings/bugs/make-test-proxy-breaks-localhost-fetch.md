# make test 在 http_proxy 设定时假失败（localhost fetch 被代理路由）

**日期**：2026-07-31
**相关模块**：`apps/browser/src/tab_js_worker.rs`（`tab_js_worker_default_fetch_handler_real_http`）、`make test` / rally 无人值守执行流程
**触发轮**：R2298（首轮 make test 假 fail）

## 问题描述

R2298 提交门禁跑 `make test`（前置 `source ~/use-proxy.sh` 以拉 github wpt-data）后，唯一失败：

```
thread 'tab_js_worker::tests::tab_js_worker_default_fetch_handler_real_http' panicked
  assertion `left == right` failed
  left: ""
  right: "hello-from-server"
```

该测试本应在本地起一个 `127.0.0.1:0` TCP server，生产 `default_fetch_handler` 经 net pool 真实
HTTP GET 拿 `"hello-from-server"`。但 fetch 返回空字符串。此失败与当轮代码改动（outline-offset）
完全无关——纯环境问题。

## 根因分析

`~/use-proxy.sh` 设置 `http_proxy=192.168.1.212:7078` / `https_proxy=...`（仅路由 github，
用于 `make fetch-wpt-data`）。`make test` 继承该环境变量后，net pool 的 HTTP 客户端把对
`http://127.0.0.1:<port>/data` 的请求**经代理路由**——代理无法回到本机的动态端口 → 连接失败 /
空响应 → `r.text()` 解析为 `""` → 断言失败。

即：**代理环境变量污染了 localhost fetch**。这是个真测试（不依赖外部网络，本地 server +
本地 fetch），但代理让它假失败。

## 解决方案 / 如何避免

**跑 `make test` / `make reftest` / `make product-smoke` 前_unset_ 代理环境变量**
（这些命令本身不需要 github——wpt-data 已 fetch、oracle PNG 已在位）：

```bash
env -u http_proxy -u https_proxy -u HTTP_PROXY -u HTTPS_PROXY make test
```

或仅在 fetch wpt-data 时带代理，跑测试时切回无代理 shell。

**判定 make test 单个 failure 是否为此假失败的信号**：

1. 失败的测试名含 `_real_http` / `fetch` / `_http` 且断言 `left == ""`（空响应）。
2. 失败仅在 `http_proxy` 设定时出现；`env -u *_proxy make test` 重跑该 case 立即 pass。
3. 失败与当轮代码改动语义无关（改动在 CSS / paint，却挂在 HTTP fetch 测试上）。

命中上述三点 → 是代理污染，**不是真回归**，unset 代理重跑确认即可，勿当成代码 bug 深查。

## 验证（R2298 收尾）

`env -u http_proxy -u https_proxy -u HTTP_PROXY -u HTTPS_PROXY make test` 重跑：
所有 binary `test result: ok ... 0 failed`，`tab_js_worker_default_fetch_handler_real_http`
pass，exit 0。R2298 / R2299 均按此方式跑门禁。

## 备注

- 代理仅路由 github（`make fetch-wpt-data` 需要）；npmjs registry 走另一条网络路径（直连可达，
  见 R2290b product-smoke oracle 抓取流程）。故「github 走代理、npmjs 直连、localhost 须无代理」
  是本环境的三条网络路径，跑测试前务必 unset 代理。
- 若未来有更多 `_real_http` / localhost 测试加入，同样受此影响。
