# M2-s8 — 运行时 img src-set 检查点（全绿 72→74 零丢失）

**日期**: 2026-09-25
**corpus**: `make testharness-csp`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-25-m2-s8-csp-final.json](2026-09-25-m2-s8-csp-final.json)
**前序**: [M2-s7](2026-09-25-m2-s7-csp-extensions.md)（72/445，28.5%）

## 结果

| 指标 | M1 基线 | M2-s7 | M2-s8 | Δ |
|---|---|---|---|---|
| 全绿用例 | 30/415 | 72/445 | **74/445** | **+2 零丢失** |
| subtests Pass | 91/557 = 16.3% | 172/604 = 28.5% | **174/604 = 28.8%** | +2 |

新增全绿：meta/meta-img-src、meta/meta-modified（meta 政策面经 img 门禁闭合）。
securitypolicyviolation img 族（block-image / from-script）状态：violation 事件 +
blockedURI/指令面已到位——**剩余红点 = 上游模板占位符未替换**（`{{location[scheme]}}`
等 .sub.js 服务端模板面，GET 模板替换不覆盖 location 族——记账）与 from-script 案
的 line/column 断言（eval 同款调用点定位面）。

## 本切片落地（kill-switch 延续，default off）

1. **运行时 img src-set 检查点**（shim 双路径 + webview 回调）：
   - **IDL setter**（part05 R56h `img.src=` 分支）：fetch 链前 `__zwCspImgCheck`
     判定——阻止 → 跳过 fetch + `_defer` 派 error（onerror 语义）；
   - **setAttribute 路径**（part04 setAttribute('src') on IMG）：同判定 + 同步
     `__zw_dispatch_img_event(abs,'error')`；
   - webview `__zwCspImgCheck` 原生回调（resolve abs → check_image → violation 入
     connect 共享队列 + '1'）；注册点迁移至**无条件区**（__zw_get_image_size 注册后
     ——首版误落 `if let Some(fetch_handler)` 条件块，无 fetch_handler 页面回调
     undefined，探针修复）。
2. **targetSelector 元素站泛化**（shim `__zw_dispatch_securitypolicyviolation` +
   PendingCspStyleViolation.target_selector）：querySelector 解析目标元素——本切片
   violation 实走 document 站（corpus img 族不断言 target；元素站能力就位供后续）。

## 剩余缺口（M5 收口面）

| 项 | 归宿 |
|---|---|
| img corpus `{{location[scheme]}}` 模板族 | 记账（.sub.js location 模板替换未实现——GET 之外的面） |
| img 案 line/column 断言 | 与 eval 调用点定位同域（M5 评估是否值得） |
| **M5 收口主战场** | kill-switch default-on 决策（A/B 零回归门禁）+ DC-1~4 逐项判定 + 挂账定稿 |
