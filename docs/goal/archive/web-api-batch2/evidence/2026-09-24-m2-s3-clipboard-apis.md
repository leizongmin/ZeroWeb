# M2 切片 3 — 权限 denied 拒绝语义 + read(options) 校验 + 图片魔数校验 + DataTransfer 'Files'（clipboard-apis 修齐轮 3）

**日期**: 2026-09-24
**套件**: `make testharness-clipboard-apis`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-24-m2-s3-clipboard-apis.json](2026-09-24-m2-s3-clipboard-apis.json)
**前值**: [2026-09-23-m2-s2-clipboard-apis.md](2026-09-23-m2-s2-clipboard-apis.md)（70.8%）

## 结果

| 指标 | M2-s2 | M2-s3 | Δ |
|---|---|---|---|
| subtests Pass | 51/72 = 70.8% | **60/72 = 83.3%** | +12.5pp |
| 全绿用例 | 19/33 | **25/33** | +6 |

新增全绿（6）：read-unsanitized-null、unsanitized-standard-html-read-fail（×4）、
dataTransfer-clearData、write-image-read-image（malformed 案转绿后整案全绿）、
permissions 四案全绿（denied ×2 转绿 + granted ×2 保持）。

**通过集零丢失**（与 M2-s2 pass set diff：lost 0 / gained 9，+9 精确命中本轮计划面）。

## 落地面（三处）

1. **权限 denied 拒绝语义**（engine part02.js + runner testharness.rs）：
   - clipboard IIFE 内权限状态注册表（`_permStates`，name → prompt/granted/denied，默认
     prompt）+ 内部钩子 `__zwSetPermission`（注入）/ `__zwPermissionState`（查询）——
     `__zw` 前缀约定同 `__zwClipboardStoreWrite`。
   - read/readText 门 `clipboard-read`、write/writeText 门 `clipboard-write`：denied →
     NotAllowedError（promise 拒绝）；**'prompt' 不拦截**（headless 无权限提示 UI，WebKit
     风格——user-activation.js 同款现状注释），既保既有 25 个免 tryGrant 案零回归又让
     denied ×2 成立。
   - runner 侧：`unsupported_testdriver_dependencies` 白名单加 `set_permission`（此前
     svg 案整案 Unsupported）；TESTDRIVER_STUB 增 `set_permission`（纯页面侧转发钩子，
     非法 state 拒绝，不经宿主命令队列）；`unsupported_testdriver_command_is_explicit`
     单测改用 set_context（白名单语义断言保持）。
   - navigator.permissions.query 消费 `__zwPermissionState` 如实返回（R2817 固定
     'prompt' 升级为活状态）。完整权限语义层仍归 security-hardening DC-4 对齐。
2. **read(options) 字典校验**（part02.js `_validateUnsanitized`）：WebIDL 转换先于算法体
   （promise-returning → 异常走 rejected promise）。unsanitized null/非序列 → TypeError；
   多格式或非 text/html 单项 → NotAllowedError（上游 unsanitized read-fail 案口径）；
   空序列/缺省 → 常规读。unsanitized-null ×1 + unsanitized-standard ×4 转绿。
3. **write() image/* 魔数校验**（part02.js `_validateImageMagic`）：spec write 步骤
   "parse the image" 失败 → DataError。headless 无图片解码器 → 按类型魔数甄别
   （PNG/JPEG/GIF/WEBP/BMP/SVG `<` 前缀），未识别子类型不拦截；校验在 promise 链内
   throw → 整 write 拒绝。malformed 案转绿；真 PNG（write-image 案 1）零影响。
4. **DataTransfer types 'Files'**（part05.js）：items 含 file 项时 types 追加 'Files'
   （spec DnD types getter）。dataTransfer-clearData 转绿（首断言 `types.length 1`）；
   items.clear() 清 files 已有实现（R2948）无需改。

## 已甄别未解（记档，不硬解）

- custom formats 校验簇 ×6（>100 项拒绝、无 web 前缀拒绝、web 前缀 MIME/Blob type
  失配拒绝等）：write() 自定义格式面，tentative，后续切片。
- svg read/write ×1（本轮 Unsupported→Fail 解锁，svg 净化期望 Chrome 化）：记账。
- unsanitized html 内容规范化 ×1（期望 Chrome 序列化形态 `<head> </head> <body>`）：
  内容规范化面，记账。
- write-html-read-html `headElement.remove` ×1：js-dom 面（DOMParser 产物缺 remove），
  非 clipboard 域，记账。
- basics 停滞 ×1（Timeout，~11-12 激活周期）：runner 探针循环长程行为，M2-s2 记账保持。
- /common fetch ×1 + copy-event isTrusted ×1：runner infra，记账。

## 附带影响（fullscreen corpus，如实记账）

- set_permission 白名单解锁 3 案（permission.tentative / without-user-activation /
  element-request-fullscreen-options）：Unsupported 伪子测 2 → 真子测 5，全数仍非绿
  （PermissionStatus 类缺失、navigator.userActivation 面、fullscreen options 面未落）。
  Pass 绝对数 92 持平（通过集与 M1 基线零丢失），分母 146→149，比率 63.0%→61.7%
  ——**解锁效应非回归**；PermissionStatus/userActivation 列入 M3 修齐候选。
