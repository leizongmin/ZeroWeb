# M2-s7 — script-src-attr 检查点（event-handler 门禁；M2-s6 −1 回退案修复）

**日期**: 2026-09-25
**corpus**: `make testharness-csp`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-25-m2-s7-csp-final.json](2026-09-25-m2-s7-csp-final.json)
**前序**: [M2-s6](2026-09-25-m2-s6-csp-extensions.md)（71/445，28.0%）

## 结果

| 指标 | M1 基线 | M2-s6 | M2-s7 | Δ |
|---|---|---|---|---|
| 执行用例 | 415 | 445 | 445 | — |
| 全绿用例 | 30/415 | 71/445 | **72/445** | **+1（M2-s6 −1 回退案修复，零丢失）** |
| subtests Pass | 91/557 = 16.3% | 169/604 = 28.0% | **172/604 = 28.5%** | +3 |

回退案修复：`script-src-event-handler-on-inline-script`（M2-s6 −1）4/4 全绿——
unsafe-hashes + hash 匹配的 onclick 处理器放行执行、不匹配处理器阻止
（script-src-attr violation）。**M2-s6 引入的 Function 代理误拦已闭合。**

## 本切片落地（kill-switch 延续，default off）

1. **script-src-attr 检查点**（zero-security `check_script_attr` +
   `effective_script_attr_directive` + csp.rs `has_directive`）：处理器内容**原文**
   hash（spec 逐字节，不 trim）→ `is_script_attr_allowed`（unsafe-hashes /
   unsafe-inline / nonce）。指令面：显式 script-src-attr / script-src 上报
   "script-src-attr"，default-src 回退如实。
2. **两个 shim 求值点接门禁**（handler → 页面全生命周期，非 per-script 窗口）：
   - `_ensureInlineHandler` 编译点（dispatchEvent/click 统一经缓存 handler 触发）：
     编译前 `__zwCspAttrAllowed(code)` 判定，阻止 → 缓存 false（视为无 handler）；
   - R155 activation 路径 inline onclick：同判定，阻止静默跳过。
3. **`__zwAttrClear` 标记通道**：编译窗置位 → Function Proxy 对标记窗口内的构造走
   真实 Function（预检已在**未包装属性原文**上做——代理复查包装壳 hash 必然失配，
   首版双查缺陷探针修复）；页面自发 new Function 不受标记影响（eval 面归
   __zwCspEvalBlocked，violation 单记不重计）。

### 探针闭环（防复踩）

- 首版代理用**包装后代码**复查 hash（`with(document){with(this){…}}` 壳）必失配——
  attr 判定基准 = 未包装属性原文，代理只信任标记。
- shim 处理器求值有两个独立路径（_ensureInlineHandler 编译缓存 + R155 activation
  直eval）——门禁须两点同盖。
- 页面自发 new Function 与 handler 编译共用 Function Proxy——attr 放行通道必须
  flag 限定，否则 unsafe-inline 误覆盖 eval 面（violation 重计）。

## 剩余缺口（M2-s8 / M5）

| 簇 | 形态 | 归宿 |
|---|---|---|
| 运行时 img | createElement('img') src-set 钩子（securitypolicyviolation img 4 案） | M2-s8 |
| eval 调用点定位 | blockeduri-eval 15:13（v8 栈面） | M2-s8（shim per-eval 传参评估） |
| fetch blocked 契约 | shim resolve(ok:false) vs 上游 reject | 记账（shim 面） |
| 外链 css ID 选择器应用 | 渲染流域域 | 记账（跨域） |
| report-uri / Reporting API | report 端点族 | 记账（infra） |
| inheritance/sandbox 簇 | 跨文档/iframe 面 | 挂账（深多进程） |
| **M5 收口** | A/B 零回归 + default-on 决策 + DC 逐项判定 | 下一阶段主战场 |
