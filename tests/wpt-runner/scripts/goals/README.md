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
| 20 | [net-api-compat](../../../../docs/goal/archive/net-api-compat.md) | fetch; xhr; url; mimesniff; streams; eventsource | **二期立项**（2026-10-27，coverage 对账驱动：url/mimesniff/fetch 失败簇收敛 + WebSocket 重估；[一期归档](../../../../docs/goal/archive/net-api-compat.md)） |
| 21 | [console-compat](../../../../docs/goal/archive/console-compat.md) | console | 无（极小快赢；console 捕获通道 M1 甄别，触面先与 devtools 流协调） |
| 22 | [touch-events-compat](../../../../docs/goal/archive/touch-events-compat.md) | touch-events | **已收口**（2026-10-14）；注入面挂宿主触摸注入管线重入 |
| 23 | [selection-compat](../../../../docs/goal/selection-compat.md) | selection | **二期立项**（2026-10-27，coverage 对账驱动：语料 20→101 文件扩面；[一期归档](../../../../docs/goal/archive/selection-compat.md)） |
| 24 | [compression-compat](../../../../docs/goal/archive/compression-compat.md) | compression | 无（实现型小套件，M2 实现轮） |
| 25 | [html-semantics-compat](../../../../docs/goal/html-semantics-compat.md) | html/semantics 十子域 | **主攻大域**（分域切片收口制） |
| 26 | [webcrypto-compat](../../../../docs/goal/webcrypto-compat.md) | WebCryptoAPI | **二期立项**（2026-10-27，coverage 对账驱动：扩面 derive_bits_keys + tentative 转正甄别，一期 77.89% 零回归底线；[一期归档](../../../../docs/goal/archive/webcrypto-compat.md)） |
| 27 | [page-state-compat](../../../../docs/goal/archive/page-state-compat.md) | beacon; page-visibility; online | **已收口**（2026-10-18）；偏差记账 beacon 混合内容阻断与 keepalive in-flight 配额池（FIXME） |
| 28 | [dnd-compat](../../../../docs/goal/archive/dnd-compat.md) | dnd |  **已收口**（2026-10-18，437/446 = 98.0%，已归档） |
| 29 | [pointerlock-compat](../../../../docs/goal/archive/pointerlock-compat.md) | pointerlock | **已收口**（2026-10-19 M3 完成 22/22，已归档） |
| 30 | [encoding-compat](../../../../docs/goal/archive/encoding-compat.md) | encoding | **已收口**（2026-10-04） |
| 31 | [encoding-legacy-mb](../../../../docs/goal/archive/encoding-legacy-mb.md) | encoding/legacy-mb 文档族 | **已收口**（2026-10-29 DC-1~4：decode 面 360,431 = 100%，全量头条 24.19%；encode 面甄别归口 36 号） |
| 32 | [wasm-compat](../../../../docs/goal/archive/wasm-compat.md) | wasm/core; wasm/webapi | **已收口**（2026-10-29 DC-1~4：core 188,280/188,289 = 99.995% / webapi 272/272 = 100%（残差 2F 由 net-api M2-S18 清偿）/ jsapi 底线 878/878，已归档） |
| 33 | [editing-compat](../../../../docs/goal/editing-compat.md) | editing/run; editing/other | 无（2026-10-27 coverage 立项；长线先语料后引擎，按簇收敛） |
| 34 | [dom-expansion](../../../../docs/goal/dom-expansion.md) | dom/ranges; html/dom 扩面 | 无（2026-10-27 coverage 立项；40 号划界承接，parser 通道零回归底线） |
| 35 | [css-testharness-compat](../../../../docs/goal/archive/css-testharness-compat.md) | css/css-values; css/selectors（M0 试点）；css/* 89 目录面 | **已收口**（2026-10-30 DC-1~4：css testharness 面 0.12%→9.08%（快照 coverage-2026-10-30），十六目录基线 65,501/17,781 = 27.1%，已归档；失败簇挂账见 goal M3 各批 evidence；渲染面归 rendering-compat） |
| 36 | [form-url-encoding-compat](../../../../docs/goal/archive/form-url-encoding-compat.md) | encoding/legacy-mb encode 面（form 46 + href 22） | **已收口**（2026-10-29 DC-1~4：encode 面 766,958/766,959 = 99.9999%，全量头条 24.19% → 58.02%，已归档；唯一非 P = data: iframe Timeout 契约挂案） |
| 37 | [encoding-compat-v2](../../../../docs/goal/encoding-compat-v2.md) | encoding 根面残余；encoding/legacy-mb-{japanese,korean,tchinese} 未导入族；encoding/streams 残余 | **coverage 驱动立项**（2026-10-31，coverage-2026-10-30 对账：encoding 缺口 573,911 全 WPT 最大且无活跃 owner；一期 30/31/36 通道复用，零回归门在册；M0 族甄别后落 DC-4 数值门） |
| 40 | [html-syntax-compat](../../../../docs/goal/archive/html-syntax-compat.md) | html/syntax; html/dom | **已收口**（2026-10-04） |
| 50 | [uievents-compat](../../../../docs/goal/archive/uievents-compat.md) | uievents; pointerevents |  **已收口**（2026-10-07 M4；M5 巡检 2026-10-20 收口 1805P/166F/95TO，已归档）；IME 挂账 |
| 60 | [navigation-compat](../../../../docs/goal/archive/navigation-compat.md) | history; navigation-api; html/browsers | **已收口**（2026-10-21 M3 收口，M2 437/476 = 91.8%、全量 473P/584 案行，已归档）；残余超口径记账池挂账 |
| 70 | [workers-compat](../../../../docs/goal/archive/workers-compat.md) | workers; dedicated-workers | **已收口**（2026-10-21 M4，693/714 = 97.1%，基线 45.6%，已归档）；残余 21 挂账 runner 基建/isolate 化/功能缺口 |
| 80 | [svg-compat](../../../../docs/goal/archive/svg-compat.md) | svg | **已收口**（2026-10-22 M4，corpus 70.3%，基线 8.8%，已归档）；SMIL 时钟等七项挂账归口他域；渲染面归 rendering-compat |
| 90 | [security-hardening](../../../../docs/goal/archive/security-hardening.md) | content-security-policy; mixed-content; secure-contexts | **已收口**（2026-09-27） |
| 92 | [web-api-batch2](../../../../docs/goal/archive/web-api-batch2.md) | clipboard-apis; fullscreen | **已收口**（2026-10-04） |
| 99 | [webgl-compat](../../../../docs/goal/archive/webgl-compat.md) | webgl | **已收口**（2026-10-23 M4，用户签收 §5 三项后归档）；基线 2112P/3F（3F 为上游 IDL 重载判据）、reftest 716/716；WebGL2/WebGPU 挂账定稿见 M4 evidence |

**预留与协调面（2026-10-27 更新）**：31-35 已占位（coverage-2026-10-27 对账立项：
encoding-legacy-mb / wasm / editing / dom-expansion / css-testharness），新候选从
36 号起顺延。原候选 cssom-view / geometry-1 已纳入 35 号 M3+ 扩面（与 rendering-compat
工作面重叠，落地前先按 run-rules §9 协调）。未拆候选：webrtc / getUserMedia /
webaudio / geolocation / sensors / speech 类（host 设备与音频图缺失，远期门控池；
websockets 随 20 号二期 DC-3 重估）。

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
