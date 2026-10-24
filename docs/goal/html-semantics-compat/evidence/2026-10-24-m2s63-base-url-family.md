# M2 切片 63（s63）——document-metadata 域：base 族 URL 面收口

日期：2026-10-24。document-metadata 43P/63F/18T → **52P/54F/18T**（GAIN 9 / REG 0）：
the-base-element 13/13 全绿（base_href_empty/unspecified/specified 5F +
base-data/base-javascript 6F 收口，base_href_invalid 原绿守住）。tabular-data
130P/4F、interactive-elements 439P/44F 逐值不变；zero-engine lib **3456P/0F**
（+2 为 oracle 流新增测试，0F 达标）。

## 修复面（R5270，三刀）

1. **`_zwDocBaseURL` scheme 排除**（part03）——spec 文档基 URL 算法末步：解析结果
   scheme 为 data/javascript 时**立即返 fallback**（非继续找下一 base——base-data
   案 1 首 base data: + 二 base https://example.com/ 期望解析落 document.URL 非
   example.com 实证）。空 href 解析即文档地址不受此门。
2. **`_zwDocBaseURL` pending 增补 base 先扫**——querySelectorAll 的 R161 归并把
   pending 追加快照尾，树序不保（head.prepend 的新 base 应先于快照 base 评估；
   案 3「动态插入 data: base → 回落」实证走到快照二 base 返 example.com）。
   corpus 未钉「pending valid base 在快照 base 之后插入」的树序案——先扫近似记
   已知限制。
3. **BASE href IDL**（part03 R5008 块内新分支）——spec dom-base-href 四态：href
   缺失或空串 → **文档地址**；非空解析（基 = 文档地址）成功 → 绝对 URL；解析
   失败 → **原属性串**（base_href_invalid corpus 钉定 "//test:test"——首版
   「失败 → 文档地址」翻红回退，见验证）。sel 形走 lw 读（setAttribute 后同步
   读，M2-S9 A href 同款）。LINK 保持 R5008 口径不变（harness resolveUrl 消费面）。

## 验证

- the-base-element 13/13 全绿（base_multiple/base_srcdoc/base_target 族走 iframe
  排除不入账）；document-metadata 域 **52P/54F/18T**（GAIN 9 / REG 0）
- base_href_invalid 首版「解析失败 → 文档地址」翻红——corpus 期望原属性串，回退
  为「失败 → 原串」后守住（spec 口径以 corpus 权威为准记录差异）
- tabular-data 130P/4F、interactive-elements 439P/44F 逐值不变；zero-engine lib
  **3456P/0F**（`_zwDocBaseURL` 共享消费面：a/form action、baseURI、img src 零回归）
- 拼接 shim node --check 通过；fmt/clippy 不适用（diff 仅 JS shim）

## 未解决项（document-metadata 余 54F+18T 挂账序）

- **meta color-scheme 10F**：applied-color 语义，归属核对（疑 style-system/渲染面）
- **link/style 事件族 30F+18T**：load/error 事件与外部资源管道，runner file 通道
  能力上限待分诊
- style type/media 族、CSP tentative 族待分诊
