# playwright-matrix — Playwright (pin) CDP 测试链

CDP 协议 goal（`docs/goal/cdp-protocol.md`）的 Playwright 测试工程。Node 为 **测试-only
依赖**，不进入任何产品构建。

## Pin

- `playwright-core` **1.63.0**（lockfile 固定；对应 Chromium 缓存 revision 1243）
- `ws` 8.18.3（捕获代理用）

安装：`npm install`（缓存命中时不下载浏览器；勿提交 `node_modules/`，已 gitignore）。

## 命令

```sh
npm run capture:chromium   # 启动 Chromium → 经捕获代理 connectOverCDP → 全核心流空跑
npm run matrix             # 从 out/capture.jsonl 生成捕获明细 + 汇总 JSON
make cdp-e2e               # （仓库根）ZeroWeb 收口门：双跑全核心流 @ ZeroWeb headless
                           #   → deterministic 判定 + expected-green 回归门（test-guard 包裹）
```

产物（`out/`，gitignore，可重新生成）：

- `capture.jsonl` — 代理记录的全部 CDP 流量（HTTP 发现 + WS 逐条）
- `steps-report.json` — 逐步 ok/error 报告；**指向 ZeroWeb 时此文件即差距清单**
- `chromium-capture.md` / `chromium-capture-summary.json` — 按域分组的命令矩阵事实源

账本落点：`docs/goal/cdp-protocol/evidence/cdp-command-matrix.md`（三态登记为人工判定件；
生成的明细拷贝入 `evidence/` 时按日期命名）。

## 指向 ZeroWeb（M1+ 验收同款入口）

```sh
# 终端 1：ZeroWeb headless（CDP 端点 :9222）
cargo run --bin zero-browser -- --headless
# 终端 2：
CDP_ENDPOINT_URL=http://127.0.0.1:9222 npm run capture:chromium
```

直连模式（`CDP_ENDPOINT_URL`）跳过代理，不记录流量，只产 steps-report —— 即 M1「首连」
验收判据：哪些步骤 ok / 哪些 FAIL。

## 结构

- `scripts/cdp-capture-proxy.mjs` — 纯观测代理：透传 HTTP 发现 + WS 流量并记 JSONL。
  会改写发现响应里的 `webSocketDebuggerUrl` 指向自身（客户端按响应体地址建 WS 连接，
  不改写则流量绕过代理）。
- `scripts/capture-core-flow.mjs` — 全核心流：navigate / evaluate(5 形态) / fill /
  keyboard / click(3 形态) / boundingBox / network 事件 / cookies / dialog(3 种) /
  frames / emulateMedia / screenshot(3 形态) / setContent / viewport 断言 / 多页生命周期。
  每步独立 try/catch，失败不阻断后续步骤。
- `scripts/build-matrix.mjs` — JSONL → 按域明细 + 机器可读汇总（命令计数、参数键、
  结果样例、错误样例、事件计数）。
