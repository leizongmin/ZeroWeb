# M1 / DC-1 — fullscreen window 子集通过率基线

**日期**: 2026-09-23
**套件**: `make testharness-fullscreen`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-23-m1-fullscreen-baseline.json](2026-09-23-m1-fullscreen-baseline.json)

## 结果

| 指标 | 值 |
|---|---|
| 导入用例 | 80 案 fetch / 55 案执行（25 案内容级 skip：iframe 依赖为主） |
| subtests | **92/146 = 63.0% Pass** |
| 全绿用例 | 5/55 |
| 非 Pass 状态 | Fail × 35、Timeout × 8、Unsupported × 11 |

**立项基线事实修正**：goal 立项时（2026-09-12）记「无 Element.requestFullscreen 面、
无 fullscreenchange 事件」——本基线实测引擎已有**部分** fullscreen 语义（继承自 viewport
桥遗产面）：model 案 `trusted_request` + `fullscreenchange` 事件已触发、
`document.fullscreenElement` 状态机部分在、crashtests 两案绿。63.0% 起点显著高于预期，
M3 收敛有实底。

分目录：api/ 89/132、model/ 1/9、crashtests/ 2/5。

## 失败聚类（M3 修齐方向）

| 簇 | 案数 | 形态 | 归属 |
|---|---|---|---|
| `:fullscreen` 选择器/样式联动 | ~4 | `matches(':fullscreen')` false（伪类面） | **跨域记账** rendering-compat 流域 |
| fullscreenchange target/null | ~6 | 事件 target 语义、栈空同步置 null 时序 | M3 |
| exitFullscreen 后状态 | ~4 | twice/active-document/timing 面 | M3 |
| fullscreenElement 时序（请求后仍 null / 旧值残留） | ~5 | 栈更新时序 | M3 |
| promise resolve/reject 语义 | ~4 | promises-reject/resolve、not-allowed 拒绝形态 | M3 |
| testdriver 越白名单（bless/set_context）Unsupported | 11 | crashtests ×2 + api ×6 + model 间接 | 记账（runner infra：bless 用户手势面） |
| Timeout（completion 未回） | 8 | timing 类 + iframe 类漏 skip + frameset-crash | 逐案甄别（M3 首轮） |
| `:fullscreen`/UA 渲染样式面（rendering/ 不导入） | — | 伪类渲染、backdrop、UA 样式 | **跨域记账** rendering-compat |

## 导入面与排除（fetch 脚本显式清单）

- 拉：api/ 60 + model/ 11 + crashtests/ 5 + trusted-click.js + api/resources/ ×6（helper）
- 不拉（记账）：`rendering/` ×9（`:fullscreen` 伪类与全屏 UA 渲染样式面——rendering-compat
  流域，goal 排除 + 跨域记账回流）、api/*.window.js ×4（keyboard-lock——wrapper 形态 +
  Keyboard API 依赖）
- 运行面双保险 skip：`fullscreen_case_skipped`（rendering/api-resources/manual/iframe 内容）
  ——iframe 规则砍掉 25 案（fullscreen 的 nested/allowfullscreen/cross-origin 深依赖
  iframe 语义，重入 = iframe 文档管道就位后回流的账）
- 全屏状态 viewport 语义联动（DC-3 第二条）：viewport 桥消费验证挂 M3，与
  fullscreenchange 联动面同轮做

## 下一步（M3）

1. fullscreenchange/error 事件 target 与栈时序簇（最大 F 簇）
2. promises resolve/reject 拒绝形态
3. Timeout 8 案逐案甄别
4. viewport 联动验证（消费 ④ viewport 桥）
