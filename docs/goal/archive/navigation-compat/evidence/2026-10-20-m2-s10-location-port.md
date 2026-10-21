# M2-S10 location_port origin 端口化 + history 008 定性改判（2026-10-20）

复核轮对 6 案 runner 形态/任务模型挂账重评，一案可收口、一案定性改判。

## location_port（Fail→Pass，+1P）

**案义**：断言 `location.port` 与 `location.href` 权威段 ":" 后子串全等——案自洽
要求页面 URL **带显式端口**（无端口时权威段整体 ≠ ""，如 `"wpt.test" ≠ ""`，
恒 Fail）。上游 wptserve 恒带端口（http :8800 / https :9443）；runner 缺省
origin 无端口即案设环境缺失——S1 起挂账「runner 无端口 URL 形态缺口」，本轮
单案收口。

**修**（runner 单案 origin 端口化，+18/−2）：

- 谓词 `navigation_case_uses_ported_origin`（跟随 net-api M5-S5 / webcrypto
  页面形态谓词先例），https 案锚 `wpt.test:9443`；`prepare_document_state` 与
  `page_url` 同锚。fetch handler 按纯路径解析（`wpt_url_path` 剥 scheme+host，
  端口安全），子资源/helper 解析不受影响。
- 引擎 URL 权威端口解析与 `location.port` 语义仍是真实断言目标（单案探针：
  修复前 `assert_equals: port expected "wpt.test" but got ""`，修复后 Pass）。

## history 008 定性改判（维持 Timeout 挂账）

原账「`www.` 跨主机脚本 URL」不完整：探查定谳 **document.write 引擎全缺**
（js_dom_shim 与原生绑定均无 write/writeln 注册面；语料在账用户零——
reload_document_write / traverse_the_history_write_onload 等案均不在账本）。
008 靠 `document.write` 注入跨主机脚本，注入链根本不成立（solo 实测
`tests:0`——无任何测试注册）；即便注入成立，`http://www.` 混合内容与主机
形态仍是第二缺口。挂账改判：**parser 域 document.write**（html-syntax/engine
parser 域——本 goal Support Envelope 明示 dom「不改 parser 核心」）+ runner
主机形态双缺口，非 M3、非纯 runner 形态。

## 验证

- 隔离：location_port 单案 Pass；`cargo test -p zero-wpt-runner` 224P/0F
  （test-guard 包裹）。
- 全量：`make testharness-navigation` **410P/34F/26T/14NR**（410/476 = **86.1%**），
  对 [final 账本](2026-10-20-composition-recheck-final.txt) per-subtest 语义
  diff **恰一行**（location_port Fail→Pass），零回归。
- `cargo fmt --all -- --check` 干净；`cargo clippy --workspace --all-targets
  -D warnings` 零 warning。
- 域计数：location-interface 43/43（**100%**，S1 41.9% → 收满）；
  history-traversal 目录 44/45（唯一 Fail = order-in-bfcache-restore，bfcache
  挂账维持）。

## 组合态追认（a621721d4 合入后）

push 时 rebase 撞兄弟流 R4004 label 激活归一（引擎 shim part03 共享面）——组合态
HEAD 全量复跑对 S10 账本**逐行恒等**（410P/34F/26T/14NR 零漂移），label 转发面
对导航语料零波及。

原始输出：[2026-10-20-m2-s10-location-port.txt](2026-10-20-m2-s10-location-port.txt)。
