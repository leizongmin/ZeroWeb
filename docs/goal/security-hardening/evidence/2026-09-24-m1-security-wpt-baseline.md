# M1 / DC-1 — 三 corpus window 子集通过率基线（security-hardening）

**日期**: 2026-09-24
**套件**: `make testharness-csp` / `make testharness-mixed-content` / `make testharness-secure-contexts`
**WPT pin**: 315976933870b34d6ea30e3f6643403edae678ba（与 observers/clipboard-apis/fullscreen 同 pin）
**原始数据**: [csp](2026-09-24-m1-security-wpt-baseline-csp.json) /
[mixed-content](2026-09-24-m1-security-wpt-baseline-mixed-content.json) /
[secure-contexts](2026-09-24-m1-security-wpt-baseline-secure-contexts.json)

## 结果

| corpus | 执行用例 | 全绿用例 | subtests Pass | Pass 率 |
|---|---|---|---|---|
| content-security-policy | 415 | 30 | 91/557 | **16.3%** |
| mixed-content | 2 | 0 | 0/2 | 0% |
| secure-contexts | 4 | 0 | 0/4 | 0% |

CSP 非 Pass 形态：Fail × 332、Timeout × 134。Timeout 大半为「期望被阻止的加载/事件
永不发生」案（运行时不强制 CSP → 资源照常加载/违规事件不派发 → watcher 挂起）——
这正是 M2 接线（enforcement + `SecurityPolicyViolationEvent` 派发）的直接验收面。

## CSP 分目录全绿率

| 目录 | 全绿/执行 | 说明 |
|---|---|---|
| style-src | 9/42 | hash/nonce 允许路径案（运行时无强制 → 「allowed」案天然可绿） |
| script-src | 12/73 | 同上 + eval 允许案 |
| frame-ancestors | 3/31 | runner 无嵌入方语义，仅恒允许案绿 |
| media-src | 2/10 | 允许案 |
| generic | 2/21 | 矩阵散点 |
| form-action | 1/5 |  |
| frame-src | 1/6 |  |
| gen/top.meta | 0/130 | 生成面（script-src-self/wildcard + worker-src-*） |
| securitypolicyviolation | 0/19 | 违规事件面全红（引擎零实现）——M2 主验收面 |
| meta | 0/5 | meta 解析面（响应头/meta 都未接线） |
| img-src/connect-src/object-src/base-uri/blob/child-src/default-src/font-src | 0/各 | 资源阻止案全部待接线 |

**口径注记**：现段「allowed」案绿是**无强制下的天然绿**（资源不被阻止 = 加载成功），
不表征 CSP 语义正确；「blocked」案全红才是真实缺口。M2 接线后分母不变、两侧同时
受检——通过率提升才来自真实语义。evidence 记档以 blocked/allowed 双向对照为准。

## mixed-content corpus 导入面

上游 corpus 的 window 可执行面天然很小：主体在 `gen/`（window.js/iframe 包装形态，
runner 无 wrapper，不拉取）。已拉：顶层 blob.https.sub.html + imageset.https.sub.html
+ resources/。两案现状：blob Timeout（iframe 语义）、imageset Fail。

## secure-contexts corpus 导入面

顶层 15 案中 4 案过筛执行（其余 iframe/worker 内容规则筛减；postMessage-helper 两
helper 页 name 规则排除——独立执行必 Fail 的 iframe 载荷页）。4 案现状 0/4
（shared-worker 案 Timeout——worker 面挂账）。

## M2 修齐方向（失败聚类 → 接线切片）

1. **CSP 装配面**：meta 解析（`meta/` + gen/top.meta 130 案 + 全 corpus 的 meta 政策
   载体）+ 响应头（生产路径）→ kill-switch 后接入文档。
2. **`SecurityPolicyViolationEvent`**：`securitypolicyviolation/` 19 案 + Timeout 大
   半——事件派发 + blockedURI/lineNumber/columnNumber/sourceFile 语义。
3. **子资源检查点**：img/script/style/connect/frame/media/font/object 各 src 面——
   webview 已有 `check_subresource_url`（零调用方）待接。
4. **runner infra 记账**：`/common/` 绝对路径 helper 不服务（与 web-api-batch2 M1
   同款）；worker 面（inside-worker/child-src/worker-src-*）重入条件 = worker 管道。
