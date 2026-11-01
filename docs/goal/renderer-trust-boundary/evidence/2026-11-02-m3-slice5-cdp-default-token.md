# M3 切片 5:CDP 默认强制 token(决策包 ①a)——交付证据

日期:2026-11-02
拍板:[2026-11-01 决策包](../2026-11-01-decision-package-1234.md) ①:「选 a:CDP 默认强制 token + 迁移说明(仓内消费者已核对可控,同切片更新)」「拍板即一行级变更+回归调整」。
代码:commit `d5b305045`(rebase 后;原 `9f6ec39e7`)。

## 设计:Chromium 同款能力模型

- 默认(未显式指定 `--remote-debugging-token`):getrandom 生成 128-bit hex 令牌;stderr 打 `CDP auth token: <t>` 交付操作员;发现端点 `webSocketDebuggerUrl` 携带 `?auth_token=<t>` 交付标准客户端。
- 认证双通路:WS 升级 URL query `?auth_token=`(playwright connectOverCDP 等标准客户端,无法注入首消息)或首消息 `params.token`(raw 客户端);无令牌连接一律 `-32001`。
- 显式 token 语义保留:操作员自带令牌时 stderr 不回显,认证行为相同。

### 排除项(设计期否决)

| 方案 | 否决原因 |
| --- | --- |
| refuse-start(无 token 配置时拒启) | 破坏 HTTP-only 消费者(browser-launch-smoke 只 curl /json) |
| 仅首消息认证 | playwright connectOverCDP 无法注入首消息 token |
| 前置放宽(默认关、flag 开) | 与拍板「默认强制」相反 |

### DevTools frontend 限制(fail-closed,记档)

ZW_DEVTOOLS_FRONTEND_DIR provision 下,Chrome stock frontend 的 page-direct 连接不带 token,在强制 token 下失败。限制记入 master 待决;后续选项:per-page ws 路径携带 token 的 frontend 注入,或 frontend 场景文档化「显式 token + 手工拼 URL」。

## 实现

- `apps/browser/src/headless/security.rs`:`generate_auth_token()`(CSPRNG 失败 panic,fail-closed)、`verify_upgrade_query()`(键名精确匹配 `auth_token`)、`discovery_ws_suffix()`。
- `apps/browser/src/headless/discovery.rs`:`extract_request_query()`(请求行 query,剥 `?`/`#`);version/targets 的 `webSocketDebuggerUrl` 拼 `ws_suffix`。
- `apps/browser/src/headless/mod.rs`:WS 升级分支 `authenticated = 无 token 配置 || query 校验通过`。
- `apps/browser/src/main.rs`:`run_headless` 装配——显式 token 直用;否则生成 + stderr 交付。
- 分层:库默认无 token 不变(单元测试直构不受影响),强制语义在 bin 装配层。

## 消费者迁移(同切片完成)

| 消费者 | 迁移 |
| --- | --- |
| `apps/browser/tests/cdp_{fetch_cache,focus_governance,history_traverse,input_events,navigate_evaluate,t27p35_click_latency}.rs` | 显式 `--remote-debugging-token=` + ws URL `?auth_token=` |
| `scripts/compositor-animation-comp0{1,4,4a}-probe.{sh,mjs}` | sh 从 stderr sed 取 token 拼 URL;mjs 转发 `u.search` |
| `tests/playwright-matrix/scripts/cdp-capture-proxy.mjs` | 上游 URL 保留 `${wsUrl.search}`(丢弃即 -32001);downstream 重写正则天然保留 query |
| browser-launch-smoke、background-resource-probe | 零迁移(只走 HTTP 发现面 / fetch /json 后连发布 URL) |
| playwright connectOverCDP(cdp-e2e) | 零迁移(消费发现 URL 携带的 query) |

## 语义决策:token 均发布于发现端点 URL

自动生成与显式 token 一致发布于 `/json/version`、`/json` 的 `webSocketDebuggerUrl`。理由:Chromium 能力模型即「发现端点发布不可猜能力」,loopback 读端点即得令牌与 Chrome 一致;统一路径少分支。纯带外交付(发现 URL 不携带)留作后续 flag——运维场景有此需求时再立项。设计期「显式=带外」的草案与此冲突,以实现为准记档。

## 验证矩阵

| 验证 | 结果 |
| --- | --- |
| CDP 单元(bin target,filter `headless::tests::cdp`,含新增 4 测试:默认 token query 认证 e2e、令牌随机性、query/后缀对偶、发现 URL 后缀断言) | 19P/0F;stash pop 后复跑 5P/0F |
| 6 个 cdp_* 集成二进制(test-guard) | 25P/0F(首跑红为 renderer staleness 守卫,`cargo build -p zero-renderer` 后全绿) |
| 真二进制默认路径 smoke | stderr 令牌行 ✓;`/json/version` webSocketDebuggerUrl 带 `?auth_token=` ✓;无 token WS → -32001 ✓;query token `Target.getTargets` 成功 ✓ |
| browser-launch-smoke.sh | PASS(HTTP-only 消费者不受影响) |
| 全量 bin 测试 | 736P/2F(两红均为既有,与本切片无关) |
| fmt / clippy | `cargo fmt --all --check` clean;workspace clippy -D warnings 清零 |
| cdp-e2e(playwright 链) | 见下节归因 |

## cdp-e2e 归因(clean HEAD 对照)

`make cdp-e2e` 当前红:test-guard 帽下 run 1 大面积超时(connected 成功后 emulation/screenshots/setContent/Target.getTargets 等 watchdog FAIL)。归因流程:

1. 手工跑(无帽):本切片树 26 ok / 8 failed;**clean HEAD(stash 对照)26 ok / 8 failed,失败步骤逐一相同**(click.button、click.dblclick、click.withPosition、network.events、dialog.accept、dialog.confirm+prompt、frames.click+evaluate、screenshot.element;全为 locator click 的 element-stability 等待超时)。
2. 结论:**token 链零回归**——默认 token 生成 → 发现 URL 交付 → playwright query 认证 → 命令流(connect/browser.version/26 步)与 HEAD 行为一致。
3. 8 失败为 HEAD 既有问题:1 个是 cdp-protocol goal 挂账(frames.click+evaluate,子帧能力解冻后回填);其余 7 个 click-stability 在 S1114 基线(2026-09-18,33 ok/1 挂账)后现形,期间隔 devtools 与 renderer-trust-boundary 两 goal 多切片,回归点未定位。**非本切片范围,转入待决清单另案归因**;归因时先隔离负载因素(两次手工跑均稳定复现同集合)。

另:cdp-e2e 的 CDP_ENDPOINT_URL 直连模式不经 capture proxy;proxy search 保留改动为 token 化发现 URL 的前瞻对齐(Chromium 路径无 search,行为不变),单测面由 proxy 消费方覆盖。

## 残余与待决

1. cdp-e2e HEAD 既有 7 个 click-stability 失败——回归候选另案归因(本切片已证与之无关)。
2. DevTools frontend page-direct token 下 fail-closed——限制记档,后续选项见上。
3. 显式 token 纯带外交付 flag——按需立项。
4. M4 收口项(决策面增量复测、剩余风险报告)转入下一步。
