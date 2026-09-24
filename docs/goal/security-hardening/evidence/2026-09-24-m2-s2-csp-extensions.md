# M2-s2 — CSP 检查点扩面（源位置定位 + 元素站 target + markup img 检查点）

**日期**: 2026-09-24
**corpus**: `make testharness-csp`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-24-m2-s2-csp-final.json](2026-09-24-m2-s2-csp-final.json)
**前序**: [M2-s1](2026-09-24-m2-s1-csp-wiring.md)（33/415，19.6%）

## 结果

| 指标 | M1 基线 | M2-s1 | M2-s2 | Δ（s1→s2） |
|---|---|---|---|---|
| 全绿用例 | 30/415 | 33/415 | **37/415** | **+4 零丢失** |
| subtests Pass | 91/557 = 16.3% | 109/557 = 19.6% | **115/559 = 20.6%** | +6 |

新全绿案（s1→s2）：
- `securitypolicyviolation/blockeduri-inline`——**位置定位**（lineNumber 15 /
  columnNumber 9 精确命中）
- `img-src/img-src-none-blocks` / `img-src-none-blocks-data-uri` /
  `img-src-full-host-wildcard-blocked.sub`——**markup img 检查点**（不 fetch + onerror
  派发；data: URI 同口径受检）

## 本切片落地（kill-switch 延续，default off）

1. **violation 源位置定位**（engine extract.rs `extract_script_source_positions`）：
   原文扫描每 `<script` 内容起点（开标签 `>` 后首字符）→ 1-based 行/列，全量 script
   序号口径。**runner 扭曲修正**：harness 内联扭曲装配后文档行号——位置表从**原始
   case 源**预计算，经 `WebViewConfig::csp_script_positions`（runner 专用覆盖面，
   默认 None = 生产自算）传入；**prepared ordinal 空间重映射**（testharnessreport/
   testdriver-vendor/testdriver-actions 标签被整标签移除，其后脚本序号前移——剔除
   对应条目对齐）。
2. **元素站 target 派发**（shim `__zw_dispatch_securitypolicyviolation` 扩展）：被
   阻止 script 元素经 `__zwSelector`/`__zwHandle` 持有 listener-store key 时按元素
   站 `_dispatchWithBubble`（receiverProxy = 页面持有 proxy——R52 identity 保持）
   派发，bubble 上行 doc/win；解析失败回落 document 站。violation `e.target` =
   被阻止元素（targeting corpus 语义；该案全绿仍需 script-attr/style 门禁——M2-s3）。
3. **markup img 检查点**（zero-security `check_image` + webview）：
   - `fetch_page_images` 入口**装配前移**（img 检查点在 fetch 侧需要政策集；
     run_page_scripts 侧幂等重装）+ 阻止队列按轮重建；
   - `fetch_image_subresources` 逐 src 检查（img-src/default-src 源匹配，data: 原样
     受检），被阻止 src 不 fetch/不入缓存；
   - error 事件在 **run_page_scripts 页面脚本之后**派发（`__zw_dispatch_img_event`）——
     上游 error 于文档序内触发，页面此前注册的测试句柄（t1.step）须已就位。落点
     在 `page_scripts_initialized` 置位后、pending 早退分支之前（无变更页面同达）。

### 中途探针闭环（防复踩）

- **runner 行号扭曲**：位置首跑得 5267（harness 内联 5000+ 行）→ 原始源预计算 →
  5（ordinal 错位：report 标签整标签移除）→ prepared 空间重映射 → **15:9 精确命中**。
- **markup img error 派发时序**：脚本前派发使 `t1.step` ReferenceError（img-src
  corpus 形态先注册测试句柄后触发 img）→ 移至脚本后；`page_scripts_initialized`
  置位后存在无变更早退分支，落点须在其前（探针记档）。
- **运行时 img src-set 钩子**（`document.createElement('img')` 族——
  securitypolicyviolation img 4 案）需 shim 属性写管道手术，**挂账 M2-s3**。

## 剩余缺口（M2-s3+）

| 簇 | 形态 | 切片 |
|---|---|---|
| style 检查点 | style-src inline/外链阻止族（style-src 33 红案） | M2-s3 |
| script-src-attr | onclick 等内联事件处理器（targeting block2/4） | M2-s3 |
| 运行时 img | createElement('img') src-set 钩子（securitypolicyviolation img 4 案） | M2-s3 |
| eval 检查点 | blockeduri-eval | M2-s3/s4 |
| report-uri / Reporting API | .sub.html report 族 | 记账（infra） |
