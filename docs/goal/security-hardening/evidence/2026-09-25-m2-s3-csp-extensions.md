# M2-s3 — style 检查点（inline `<style>` + 外链 stylesheet + 多算法 hash 匹配）

**日期**: 2026-09-25
**corpus**: `make testharness-csp`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-25-m2-s3-csp-final.json](2026-09-25-m2-s3-csp-final.json)
**前序**: [M2-s2](2026-09-24-m2-s2-csp-extensions.md)（37/415，20.6%）

## 结果

| 指标 | M1 基线 | M2-s2 | M2-s3 | Δ |
|---|---|---|---|---|
| 全绿用例 | 30/415 | 37/415 | **37/415** | **±0 零丢失** |
| subtests Pass | 91/557 = 16.3% | 115/559 = 20.6% | **118/552 = 21.4%** | +3 |

全绿持平口径注记：首跑曾 −3（style-src-hash-allowed / -case-insensitive /
-default-src-allowed 回退）——两轮探针闭环归因为 **hash 匹配器两处缺口**（见下），
修复后全数恢复；style 检查点新增的阻止面（style-blocked / stylenonce-blocked 族）
待 violation 断言面（error 事件 + 字段断言）齐备后翻绿，M2-s4。

## 本切片落地（kill-switch 延续，default off）

1. **inline `<style>` 检查点**（engine `extract_style_elements_csp` + webview
   `gate_inline_styles`）：原文扫描（大小写不敏感标签 + nonce 属性 + 内容 span）→
   `check_style_hashes`（nonce / sha256-sha384-sha512 hash / unsafe-inline）→ 被阻止
   元素**等长空白原地清空内容**（元素保留——targeting 断言 target 可达；偏移面零扰动）。
   装配在 load_html 入口幂等重装。
2. **外链 stylesheet 检查点**（`resolve_external_css` &mut 化 + 逐 link 检查）：被阻止
   href 不 fetch/不并入 combined；violation 入队（target = link 元素）+ link error
   事件（abs href）随 run 尾统一派发。
3. **run 尾 style violation 派发**：`pending_csp_style_violations` 队列——页面脚本后
   统一派发（元素站 targetTag/targetOrdinal 泛化形态，shim `__zw_dispatch_
   securitypolicyviolation` 扩展）。
4. **hash 匹配器补全**（csp.rs，`is_inline_script_allowed` / `is_inline_style_allowed`
   对称）：①**三算法并立**（sha256/sha384/sha512——style-src-hash-allowed 一政策各
   算法一枚对应不同元素）；②**算法前缀 ASCII 大小写不敏感**（'SHA256-'/'sHa256-'
   面——style-src-hash-case-insensitive corpus）。`style_hash_base64(alg, content)`
   新增 + `SecurityContext::check_style_hashes` 多 hash 并立判定。

### 探针闭环（防复踩）

- −3 假回退归因链：hash-allowed 案 0px → isoA 隔离到 strip 循环 → 逐 style 打印 →
  content2/3 的 hash 本就不同（sha384/sha512 各一枚）→ 匹配器 sha256-only 缺口；
  第一轮修了 script 侧匹配器（锚点错位），style 侧复测仍红后定位——**修 csp.rs 同型
  代码块时须逐一核对其宿主函数**（is_inline_script_allowed vs is_inline_style_allowed）。
- inline style 等长空白替换保偏移不变量（原文 span 消费方零扰动）。

## 剩余缺口（M2-s4+）

| 簇 | 形态 | 切片 |
|---|---|---|
| style 阻止族翻绿 | style-blocked/stylenonce-blocked/error-event 族——需 style 阻止的 error 事件语义 + violation 字段断言对齐 | M2-s4 |
| style-src-attr | inline style 属性（style attribute 面——style system 侧） | M2-s4 |
| script-src-attr | onclick 内联事件处理器（targeting block2/4） | M2-s4 |
| 运行时 img | createElement('img') src-set 钩子（securitypolicyviolation img 4 案） | M2-s4/s5 |
| eval 检查点 | blockeduri-eval | M2-s5 |
| @import 面 | import-style-blocked/allowed | 记账（CSSOM import 管道） |
