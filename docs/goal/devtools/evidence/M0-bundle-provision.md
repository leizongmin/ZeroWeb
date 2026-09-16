# M0-P1/P3 — devtools-frontend bundle 供给与基线探测记录

**日期**: 2026-09-16（M0 首轮）
**执行流**: devtools goal（本 goal），门控状态：cdp-protocol goal 已 Done（2026-09-16 M5 定稿收口），
M3 DOM/CSS 域在位 → 启动门控解锁，M0 自主面落地。

---

## 1. bundle 供给调研（2026-09-16 实测）

### 1.1 候选渠道盘点

| 渠道 | 实测结果 | 结论 |
|------|----------|------|
| GCS `chrome-devtools-frontend/<sha>/devtools_frontend.zip`（旧预构建桶） | 403（对象不存在） | 不可用 |
| npm `devtools-frontend`（2.0.2） | 2016 年代古董包 | 不可用 |
| npm `chrome-devtools-frontend`（**官方**，README 点名） | 最新 1.0.1697595（2026-09-16T06:29 发布，每日一更），20MB tgz / 4789 files | **官方源码包**（无 `.gn`/`scripts/` 构建胶水，不能直接构建）——作为源码 pin 基准之一 |
| npm `@chrome-devtools/inspector` 等（第三方 iam-medvedev/chrome-devtools 仓库 CI 产出的预构建包） | 1.20260913.0，16.4MB tgz，flat 目录 inspector.html + chunk-*.js | 可 serve 但**实测有致命缺陷**（见 1.2），仅作对照样本 |
| 官方仓库 `main` tarball（codeload.github.com） | 下载中（本机到 codeload 约 70-110KB/s，仓库树大） | **本 goal 采用的正式路径**：pin commit + 从源构建 |
| googlesource.com / cipd（chrome-infra-packages.appspot.com） | git ls-remote 挂起 / http 000 | 不可达 → depot_tools 不可用，gn 需自建 |

### 1.2 第三方预构建包缺陷实测（@chrome-devtools/inspector@1.20260913.0）

- tarball sha256: `2c8b086a1c282304e5df07b6713cf9a476c0124133d3f97e6ef72e278fd2e9fd`
- 现象：经 ZeroWeb `/devtools/` serve 后在真实 Chromium（playwright pin chromium-1243）打开，
  页面白屏零渲染；console 唯一报错：
  `Loading the script 'node:worker_threads' violates ... CSP directive: "script-src 'self' ..."`。
- 根因：共享 chunk 内含**静态** `import * as zF from "node:worker_threads"`（Node 宿主
  runtime 被打进 inspector 入口的模块图）。浏览器按 CSP 拒载该 scheme → 整个模块图解析
  失败 → 静默白屏。截图存证：`third-party-bundle-blank-boot.png`。
- 结论：该包对浏览器 serve 场景**不可用**（其宿主形态疑为 Node 进程内嵌）。第三方重打包
  另有供应链/许可证声明不符问题（package.json 标 MIT，payload 实为 upstream BSD-3）——
  双重原因，不采用。

### 1.3 许可核查（官方源实测）

- **devtools-frontend LICENSE = BSD-3-Clause**（"Copyright 2014 The Chromium Authors"
  标准再分发条款；npm 官方包 `chrome-devtools-frontend` license 字段同为 BSD-3-Clause）。
- **入口文档预期修正**：devtools.md 基线事实写的是「Apache-2.0 系」——实测为 BSD-3-Clause。
  两者同为宽松许可、非 copyleft；对父目标排除条款的实质关切（非 MPL/GPL 系传染）不成立
  风险。goal 入口文档按此记账，不改判 Mission。
- front_end 内 third_party 组件（codemirror/lit/lighthouse 等）各自带许可证文件，
  构建产物再分发时随附 LICENSE（构建输出含 LICENSE 文件，入 evidence 复核项）。

### 1.4 正式 pin 路径（本 goal 采用）

- 源：官方仓库 main tarball（codeload）+ commit pin；工具链：自建 gn（GitHub 镜像源码 +
  系统 g++ 构建，v2465）+ 系统 ninja 1.12 + 系统 node v24.21.0 + npm devDeps；
  depot_tools / gclient / vpython3 因 googlesource+cipd 不可达而绕开（直接 gn gen + ninja）。
- 构建产物：`out/Default/gen/front_end/`（bundle），sha256 记账（见下节补录）。
- 供给方式：**不入库**（产物几十 MB 不进 git）；仓库携带 fetch/build 脚本 + 本 evidence
  记账（pin commit、tarball/产物 sha256、许可、构建命令），机器本地落在
  `$HOME/.cache/zeroweb/devtools/<name>/`（不占 target/，免 cargo clean 误删）。

## 2. M0-P2 serve 骨架（同轮落地）

- `apps/browser/src/headless/devtools_serve.rs`（新增）：`/devtools/<path>` 静态 serve，
  词法 `..` 检查 + canonicalize 前缀复核（穿越实测拒绝，见 probe `zw-serve-traversal-blocked`），
  MIME 表覆盖 bundle 资源类型；bundle 未配置 → 503 带 provision 提示。
- 配置入口：`ZW_DEVTOOLS_FRONTEND_DIR`（runtime-config 集中注册 + docs/runtime-environment.md 同步）。
- `/json` 的 `devtoolsFrontendUrl`：bundle 已配置 → `/devtools/inspector.html?ws=<addr>`
  （Chrome 同款相对 URL 形态）；未配置 → 维持既有 `devtools://devtools/bundled/...` 占位
  （cdp-protocol 矩阵面零漂移）。
- 实测（probe，对真实 Chromium 空跑）：discovery → entry HTML(200) → 14.9MB chunk(200)
  → 穿越拒绝(404) 全绿——serve 骨架本体可用。

## 3. M0-P3 面板可用度基线（进行中）

- 探测脚本：`panel-baseline-probe.mjs`（可重放；ZeroWeb serve + 真实 Chromium 双端）。
- 已实证：第三方 bundle 白屏缺陷（1.2）。官方 from-source bundle 构建落地后重跑本 probe
  作为基线判定（判据：frontend boot 渲染 + Elements/Console/Network 面板 DOM 就位 +
  evaluate 生效 + console 零致命错误）。
- 已知判据约束：DevTools UI 全在 shadow root 内，外部 querySelector 不可见——probe 用
  shadow root 递归遍历判定；面板「可用」以最小演示流生效为准（goal 入口文档口径），
  不以 frontend 零报错为准（frontend 对缺失域优雅降级是预期行为）。
