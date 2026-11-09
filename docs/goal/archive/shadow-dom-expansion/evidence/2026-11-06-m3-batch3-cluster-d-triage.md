# M3 Batch 3 — 簇 D（adoptedStylesheets/module 链 + clone/IDL）甄别收口

**日期**: 2026-11-06
**前置**: M2 baseline 簇归因表 D 行（≈54F：module 载入链 + adoptedStyleSheets IDL +
shadowrootclonable clone + template IDL 反射）。本轮实跑
`shadowrootadoptedstylesheets` 组（tentative/，22 文件）：52F/2P/1T，与账面吻合。
测量口径同 attachment 甄别（05:56 build6 二进制）。

## 四面拆分（52F）

1. **module/fetch CSS module 载入链（环境受限）~17F**——modulepreload-*（7 文件）
   与 async-fetch-*（8 文件）+ importmap duplicate/ordering。实证：
   modulepreload-basic 的 `<link rel=modulepreload as=style>` onerror 拒约
   （`Unhandled rejection "[object Event]"`）→ module map 无从同步解析 → 下游
   全空。载入链属 script-sandbox/loader 面，M2 预判「环境受限」坐实——按实
   剔池或挂账 script-sandbox（shadow-dom 范围内不可修）。
2. **adoptedStyleSheets 机制面 ~14F**——basic/multiple/invalid-specifier 的
   `shadowrootadoptedstylesheets` 属性 → importmap 命名解析 → constructable
   stylesheet 进 `ShadowRoot.adoptedStyleSheets` → **计算样式着色**
   （`getComputedStyle(...).color == rgb(0,0,255)`）+ idl-reflection ×2
   （IDL↔content attribute 反射）+ serialization ×3（round-trip）。样式应用面
   与 **rendering-compat 边界耦合**（constructable stylesheet + 计算样式接线），
   按依赖约束先归口不双改。
3. **clone 面 ~3F**——`shadowrootclonable` 克隆物化（`Clone has a shadow root`
   ×2 + 克隆序列化回读 ×1）：cloneNode(true) 对声明式根宿主的重建语义，engine
   clone 路径，shadow-dom 范围内可修（M3 尾段候选）。
4. **null 链面 ~18F（与 attachment 机制一同根）**——clonenode.html（11F）等
   **静态声明式模板**页 `host.shadowRoot` 为 null（`Cannot read properties of
   null (reading 'getElementById')`）→ sanity/克隆断言连锁失败。静态物化在该
   形不生效，与 attachment 甄别机制一（setHTMLUnsafe 不物化）同族；精确边界
   （静态 vs API 路径）随 m3c2 物化修复落地后复测定格。

## 依赖与归属

- 面 1/4 = dom 物化路径 = 簇 C 流（m3c2）活跃面，已按 §9 在
  [attachment triage](2026-11-06-m3-batch3-attachment-170f-triage.md) 交付，
  复测时同批量化。
- 面 2 = script-sandbox/loader，**剔池候选**（不可修项按实剔，M2 收敛路径预案）；
  若后续 script-sandbox 支持模块载入可回池。
- 面 3 = rendering-compat 边界（样式应用），归口挂账，不双改。
- 面 3 的 clone 部分与面 4 的 IDL 反射可在 M3 尾段作为 shadow-dom 范围内
  小刀独立排期（均不与 m3c2 现行编辑面文件重叠：clone 走 engine clone 路径，
  IDL 反射走 shim 接口面——落地窗口待其提交后核 §9）。

## 门值影响（按 M2 修订规则预演）

簇 D 54F 中可翻转上限 = 面 3+4（~21F，假设物化修复后）+ 面 2 剔池（~17F）；
面 3 样式应用部分依赖 rendering-compat 归口，不入本轮门值口径。
