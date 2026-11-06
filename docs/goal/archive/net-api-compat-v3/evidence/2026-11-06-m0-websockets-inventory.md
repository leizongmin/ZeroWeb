# M0 甄别二：websockets 全量文件清单 + fixture 端点需求 + wss 面交接

日期：2026-11-06。方法同 fetch 档（pin rev `315976933870b34d6ea30e3f6643403edae678ba`
Git Trees 清单 websockets/ 267 blob + 133 缺失 case 全量下载内容甄别；分母 =
coverage-2026-11-05 websockets 2,389 子测试）。

## 一、通道既有账（M0 实测）

`make testharness-net-api FILTER=websockets/`（test-guard 包裹）：**107 文件，
377P / 0F / 0T**。分布：constructor 185P、top-level（Close-*/Open-*/Send-* .any.js）
主体、binary 4、closing-handshake 3、cookies 5、opening-handshake 6、security 2、
keeping-connection-open 1。面内零失败——现状是「窄而绿」：上游 den 2,389，通道实跑
377，缺口主要是**未导入目录 + coverage 变体膨胀**，不是引擎失败。

wpt.fyi den 口径与通道口径差异记账：websockets/constructor den_files 71 vs 本地
case 文件 20（上游 ?default/?wss/legacy code 变体膨胀；ZW 单变体枚举）；top-level
den 769 同理。

## 二、逐目录清单与批次归属

| 目录 | case 数 | py 引用 | wss 引用 | den | 结论 | 批次 |
|---|---:|---:|---:|---:|---|---|
| websockets/interfaces | 74 | **0** | 72（常数替换面） | 291 | **纯客户端 WebSocket API 语义面**（close/events/url/bufferedAmount/binaryType/readyState/send/protocol）：连接全走 `SCHEME_DOMAIN_PORT+'/'`（35×）、`/echo`（19×）、`/empty-message`（8×）、`/handshake_sleep_2`（1×），全部命中既有 fixture KNOWN_PATHS | **M2 批次（DC-3 主增量）** |
| websockets/stream | 11 | 1（stash-take.py） | 8 | 652 | **WebSocketStream 新 API（tentative）**——引擎未实现 | **排除**（新 API 面，如实挂账） |
| websockets/handlers | 34 .py | — | — | — | wptserve WebSocket handler **服务端行为面**（无 client case） | **排除**（非 window 可执行；语义已由 ws_fixture 等价承接部分） |
| websockets/multi-globals | 5 | 0 | 0 | 3 | 多全局面解析 | **排除**（runner 无多文档管道，M5-S1 既定口径） |
| websockets/unload-a-document | 10 | 0 | 0 | 11 | 多页 unload 编排 | **排除**（同上） |
| top-level back-forward-cache-*.window.js | 6 | 0 | 0 | — | bfcache 生命周期面 | **排除**（runner 无 bfcache 管道） |
| Send-data.worker.js | 1 | 0 | 0 | — | worker 执行面 | **排除**（worker 面既定口径） |
| top-level Close-*/Open-*/Send-* .any.js（67） | 已本地已跑 | 0 | — | — | 通道在账 | — |
| constructor（20）/cookies（11）/opening-handshake（8）/binary（4）/closing-handshake（3）/security（3）/keeping-connection-open（3） | 已本地已跑 | — | — | 671 | 通道在账 | — |

## 三、interfaces 批次的 fixture 端点需求（DC-3 前置盘点）

连接 path 清单 × 既有 KNOWN_PATHS（ws_fixture.rs）比对：

- **已覆盖**：`/`（35）、`/echo`（19）、`/empty-message`（8）、`/handshake_sleep_2`（1）
- **需新增**（合计 2 条，改动面极小）：
  1. `/echo_raw`（3 案）——上游 `echo_raw_wsh.py` 等价：raw 帧回显（不做文本解码）
  2. `/sleep_10_v13`（1 案，close-connecting.html）——握手前 sleep 10s（v13 旧服务代
     端点形态；close-during-CONNECTING 时序面，`/handshake_sleep_2` 同构放大）
- 误报排除：`'/test'`（url/002.html）仅作 `ws.url` 只读断言字面量，**不发起连接**；
  `/echo?foo%20bar baz` 查询串形态走 `/echo` path（query 不参与 path 匹配，既有
  `/echo-query` 语义不受影响）

## 四、wss 依赖面清单（46 号 runner-net-infra 交接）

- **范围**：全部 `.https.*` 变体案（net_api_case_skipped 既有筛减记账）+ 上述 wss 引用
  常数面在 `?wss` 变体下的执行形态。上游 TLS wss 端点 = wptserve `--wsstart` 的 TLS
  监听（{{ports[wss][0]}}）。
- **ZW 现状**：页锚 http origin + fixture 无 TLS（ws_fixture.rs 模块注释 M5-S1 两臂
  均不可测，筛减记账）。constants.sub.js 的 `{{ports[wss][0]}}` token 现映射到明文
  fixture 端口（token 落在未走分支），wss 变体执行需 46 号交付 TLS 端点 + 页锚
  secure-context 装配后重估。
- **量化**：`.https.` 命名案在 websockets 面共 0 个（interfaces 74 案全部非 .https
  命名，wss 只在 variant/query 形态）——**非 wss 依赖面导入不被 wss 阻塞**，DC-3
  只依赖第三节 2 条端点新增。wss 变体的量级（den 里 ?wss 变体约半数）随 46 号
  TLS 端点落地后在 coverage 复测口径兑现。
