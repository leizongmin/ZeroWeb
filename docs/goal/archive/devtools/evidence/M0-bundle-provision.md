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

### 1.4 正式 pin 路径（本 goal 采用）——✅ 已完成（2026-09-16）

- **pin commit**: `9bd6a496c3394422674c62a19e9faa627817c56e`（devtools-frontend main，
  "Roll browser-protocol and CfT"，2026-09-16）
- **front_end 基底**: 官方 npm 源码包 `chrome-devtools-frontend@1.0.1697595`
  （tarball sha256 `e134e6c67c5ef0779c6910529e1f2b6774d41455e9150a1cf502e39fcb0db7ff`），
  与 pin commit 的漂移文件经 by-SHA raw 重取并用 `git hash-object` 对 `git ls-tree`
  闭环校验（238 文件对齐 + 1047 测试文件按需）
- **工具链**: 自建 gn v2465（GitHub 镜像 JiauZhang/gn 源码 + 系统 g++14，googlesource/
  CIPD 本机不可达）+ 系统 ninja 1.12 + 系统 node v24.21.0 + npm devDeps（esbuild 0.28.2）；
  depot_tools / gclient / vpython3 / siso 全部绕开
- **构建**: `gn gen out/Default --args='use_siso=false'`（5551 targets）+
  `ninja -C out/Default devtools_frontend_resources`（1711 targets 全绿）
- **产物**: `out/Default/gen/front_end/`（`inspector.html` 根入口 + entrypoints/ + Images/
  + locales/ + devtools_resources.grd 链），关键文件 sha256：
  - inspector.html: `e5ae9e7c4a5b1c29dd0576c42cbb17eebd6a2dab7a6e08438f8c424a71b9752a`
  - entrypoints/inspector/inspector.js: `465e86cf8c6df52d6753c2ec73a1d6c6f84c696ac24cc4bf0c88534fbe2ebdfe`
- **可重放脚本**: `fetch-devtools-frontend.sh`（本目录），机器本地落
  `$HOME/.cache/zeroweb/devtools/devtools-frontend-<pin>/`（不入库）
- **本地构建补丁清单**（全部为构建胶水，不影响编译产物语义；升级 gn 至上游同版后可撤）：
  1. `.gn` gn_version 断言字面量化 + `script_executable` 改绝对路径
  2. newer-gn builtin `public_inputs`/`path_exists` 降级（copy.gni / typescript.gni /
     devtools_pre_built.gni / skills/BUILD.gn 用 `not_needed` 替代）
  3. `use_siso=false`（siso 走 CIPD，不可达）
  4. `//build` 子模块（chromium build.git pin）不可达 → 最小 stub：toolchain x64
     （copy/stamp）+ timestamp.gni + rbe.gni/siso.gni 空实现；kythe.gni/devtools.gni/
     chrome_build.gni 取自 chromium/src GitHub 镜像
  5. typescript@7.0.2 原生 tsc 二进制 + typescript@6.0.3 标准库 d.ts（CIPD 包不可达）

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

## 3. M0-P3 面板可用度基线——✅ 已完成（2026-09-16）

- **探测脚本**: `panel-baseline-probe.mjs`（可重放；零 mock：ZeroWeb headless serve
  frontend，Playwright pin Chromium 为被调试方，另一 Chromium 实例作 driver 打开
  frontend 页面）。
- **基线判定（全绿，`probe-final-pass.txt`）**：
  | 判据 | 结果 |
  |------|------|
  | ZeroWeb `/json` 发现端点 + devtoolsFrontendUrl 注入 | ✅ |
  | `/devtools/` serve 官方 bundle 入口（200, text/html） | ✅ |
  | 路径穿越拒绝（`%2e%2e` → 404） | ✅ |
  | frontend 完整 boot（773 shadow-DOM 节点，tab 条/侧栏全渲染） | ✅ |
  | Elements 面板渲染被调试页**活 DOM 树**（DOM 域活）+ 样式侧栏（CSS 域活） | ✅ |
  | Console 面板经 `&panel=console` 直开实例化 | ✅ |
  | Network 面板经 `&panel=network` 直开实例化（录制条就位） | ✅ |
  | console 零致命错误 | ✅ |
- **截图存证**: `official-elements-panel-live-dom.png`（Elements+Styles+盒模型，被调试页
  即 ZeroWeb 的 `/json/version` 输出——ZeroWeb 发现端点 JSON 被 Chromium 当页面渲染、
  再被 ZeroWeb serve 的 frontend 检查）、`official-network-panel-live.png`（Network 全 UI）。
- **判据现实化结论（供 M1/M2 复用）**：
  1. DevTools UI 全在 shadow root 内，外部判定必须递归 shadow root；面板**惰性实例化**，
     窄窗口下 tab 折叠进溢出菜单——`&panel=<name>` URL 参数是可靠的直开入口。
  2. Chromium 111+ 拒绝带非白名单 Origin 的 WS 附接（`--remote-allow-origins` 加固），
     空跑调试方需加该 flag；ZeroWeb 侧 WS（loopback + 默认放行 origin）无此问题——
     M1 附接 ZeroWeb 时不受影响。
  3. frontend 对缺失域优雅降级实证：指向 ZeroWeb 浏览器端点时 UI 正常装载，
     逐方法 -32601 灰置（`Page.getResourceTree` 等）——「面板可用」以逐面板演示流为准、
     不以零报错为准（goal 入口文档口径成立）。
  4. frontend 版本远新于被调试 Chromium（playwright pin chromium-1243 ≈ Chrome 151 时代
     差距）仍可附接工作——M1 附接 ZeroWeb 的协议面以 ZeroWeb 自身 CDP 账本为准。

## 4. 遗留注记

- 本轮未入库 bundle（几十 MB 不进 git）；`ZW_DEVTOOLS_FRONTEND_DIR` 未设置时 serve 面
  503 + provision 提示，`/json` 维持 `devtools://` 占位形态（cdp-protocol 守成面零漂移）。
- 2026-09-16 当日 googlesource.com / chrome-infra-packages.appspot.com（CIPD）本机直连
  不可达，github.com 主站时通时断；codeload / raw.githubusercontent（限速）/ npm registry
  稳定可用——后续重放本脚本若遇 raw 突发限流（404 形态假阴性），降低并发即可。
