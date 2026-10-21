# Goal 推进脚本（按序号执行）

每个 goal 一个带序号脚本：**fetch 上游 WPT 语料子集 + 本地盘点 + M1 下一步检查单**。
序号即建议推进顺序（快赢 → 主攻 → 深结构/门控收尾），留有间隙便于插入。

## 用法

```bash
bash tests/wpt-runner/scripts/goals/10-timing-animation-compat.sh
# FORCE=1 强制重列目录重拉（默认幂等：已拉过的目录自动跳过，防 GitHub 未认证 API 60 req/h 限流）
```

脚本只做 M1 的**资产预置**（语料落入 gitignored 的 `tests/wpt-runner/wpt-data/`）。
基线运行需要 runner 通道（`zero-wpt-runner` 子命令 + per-dir skip 规则 + Makefile
target）——那是各 goal M1 rally 轮的落地内容，检查单见脚本输出尾部，契约见各
goal 入口文档 `docs/goal/<goal>.md` DC-1。

## 推进顺序

| 序号 | Goal | WPT 测试集 | 门控/挂账 |
|------|------|-----------|----------|
| 10 | [timing-animation-compat](../../../../docs/goal/archive/timing-animation-compat.md) | hr-time; performance-timeline; user-timing; web-animations | **已收口**（2026-10-04） |
| 15 | [webstorage-compat](../../../../docs/goal/archive/webstorage-compat.md) | webstorage | 无（快赢；2026-10-08 立项） |
| 16 | [fileapi-compat](../../../../docs/goal/archive/fileapi-compat.md) | FileAPI | 无（2026-10-08 立项） |
| 17 | [webmessaging-compat](../../../../docs/goal/archive/webmessaging-compat.md) | webmessaging | 无（2026-10-08 立项；跨 frame 面挂 navigation-compat M3 门槛） |
| 18 | [cookies-compat](../../../../docs/goal/archive/cookies-compat.md) | cookies |  **已收口**（2026-10-12，533/533 = 100%，已归档） |
| 19 | [performance-api-compat](../../../../docs/goal/archive/performance-api-compat.md) | performance-api（resource/navigation/event/longtask） | 缺面甄别在 M1（host 数据管道） |
| 20 | [net-api-compat](../../../../docs/goal/archive/net-api-compat.md) | fetch; xhr; url; mimesniff; streams; eventsource | **已收口**（2026-10-04）；WebSocket 二期挂账 |
| 21 | [console-compat](../../../../docs/goal/archive/console-compat.md) | console | 无（极小快赢；console 捕获通道 M1 甄别，触面先与 devtools 流协调） |
| 22 | [touch-events-compat](../../../../docs/goal/archive/touch-events-compat.md) | touch-events | **已收口**（2026-10-14）；注入面挂宿主触摸注入管线重入 |
| 23 | [selection-compat](../../../../docs/goal/archive/selection-compat.md) | selection | 无（2026-10-08 立项） |
| 24 | [compression-compat](../../../../docs/goal/archive/compression-compat.md) | compression | 无（实现型小套件，M2 实现轮） |
| 25 | [html-semantics-compat](../../../../docs/goal/html-semantics-compat.md) | html/semantics 十子域 | **主攻大域**（分域切片收口制） |
| 26 | [webcrypto-compat](../../../../docs/goal/archive/webcrypto-compat.md) | WebCryptoAPI | 无（2026-10-08 立项；随机源面已有，subtle 实现轮分片自主） |
| 27 | [page-state-compat](../../../../docs/goal/archive/page-state-compat.md) | beacon; page-visibility; online | **已收口**（2026-10-18）；偏差记账 beacon 混合内容阻断与 keepalive in-flight 配额池（FIXME） |
| 28 | [dnd-compat](../../../../docs/goal/archive/dnd-compat.md) | dnd |  **已收口**（2026-10-18，437/446 = 98.0%，已归档） |
| 29 | [pointerlock-compat](../../../../docs/goal/archive/pointerlock-compat.md) | pointerlock | **门控 goal**（M1 盘点 ✅ 2026-10-18，1/4=25.0%；指针捕获管线须用户点名——M2 RFC BLOCK 中） |
| 30 | [encoding-compat](../../../../docs/goal/archive/encoding-compat.md) | encoding | **已收口**（2026-10-04） |
| 40 | [html-syntax-compat](../../../../docs/goal/archive/html-syntax-compat.md) | html/syntax; html/dom | **已收口**（2026-10-04） |
| 50 | [uievents-compat](../../../../docs/goal/archive/uievents-compat.md) | uievents; pointerevents |  **已收口**（2026-10-07 M4；M5 巡检 2026-10-20 收口 1805P/166F/95TO，已归档）；pointerlock/IME 挂账 |
| 60 | [navigation-compat](../../../../docs/goal/navigation-compat.md) | history; navigation-api; html/browsers | M2 轻面推进中；**M3 iframe 切片用户门控** |
| 70 | [workers-compat](../../../../docs/goal/archive/workers-compat.md) | workers; dedicated-workers | **已收口**（2026-10-21 M4，693/714 = 97.1%，基线 45.6%，已归档）；残余 21 挂账 runner 基建/isolate 化/功能缺口 |
| 80 | [svg-compat](../../../../docs/goal/svg-compat.md) | svg | 推进中；渲染面归 rendering-compat |
| 90 | [security-hardening](../../../../docs/goal/archive/security-hardening.md) | content-security-policy; mixed-content; secure-contexts | **已收口**（2026-09-27） |
| 92 | [web-api-batch2](../../../../docs/goal/archive/web-api-batch2.md) | clipboard-apis; fullscreen | **已收口**（2026-10-04） |
| 99 | [webgl-compat](../../../../docs/goal/webgl-compat.md) | webgl | **远期门控**（M2+ 全门控，M1 盘点为唯一自主切片） |

**预留与协调面（2026-10-08 更新）**：上表 10-29 槽位已全部立项。未拆候选：
cssom-view / geometry-1（渲染 API 面，与 rendering-compat 工作面重叠，立项前先按
run-rules §9 协调）、websockets（等 net 二期）、webrtc / getUserMedia / webaudio /
geolocation / sensors / speech 类（host 设备与音频图缺失，远期门控池；30 已被
encoding 占用，从 31 号起顺延占位）。

## 与 rally 启动脚本的关系

- `scripts/rally-NN-<goal>.sh`（仓库根 scripts/）：**启动无人值守推进循环**
  （`rally run docs/goal/<goal>.md`）——这是 goal 的执行入口。**编号与本目录
  槽位一一对应**（rally-10 ↔ 10-timing、rally-15 ↔ 15-webstorage……），
  按编号序执行即按优先级序执行。2026-10-20 整体归档的 18 个 goal 已删除
  对应启动器；本目录语料预置脚本保留用于回归验证。cookies、dnd、uievents
  保留启动器，质量门禁例外待复核。2026-10-08 前旧编号为立项流水号，已全部重对号。
- 无数字的 rally 脚本（rally-zero-web / rally-rendering-compat /
  rally-devtools / rally-desktop-browser / rally-cron）：
  非 WPT 语料目标与总控入口，不占本目录槽位。
- 本目录 `NN-<goal>.sh`：**M1 语料预置**（fetch + 盘点 + 检查单），rally 轮内或
  手动执行均可；两族脚本按 goal 名互指。

## 约定

- **同一 WPT_REV pin**（lib.sh，与既有 fetch-*-subset.sh 一致）——保证语料可比、可复现
- 每个 goal 基线落 `docs/goal/<goal>/evidence/` 后，按
  `docs/compat/trends/wpt-suites.csv` 头注释约定把 planned 行转数据行
  （官网测试集总表自动展示）
- 新增 goal：复制任一脚本改 `GOAL`/`DIRS`（或按序号表插空），并在本表登记
