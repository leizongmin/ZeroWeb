# M2 s436 — window.open 真弹窗链评估轮（零源码改动，判决：立项挂账归并 runner 多文档能力域）

日期：2026-11-14。前情：s431/s432 施工 target=_blank aux 上下文臂时，
window.open 弹窗链（含 rel 特性串）defer 转评估轮候选（s431 残留记账；
s432/s433 下一步延续）。本切片勘现状、语料依赖面与施工依赖链后判决。

## 现状勘定

- **stub 非零实现**：part01b.js R2979 `globalThis.open = function open() {
  return null; }`——headless popup-blocked dismiss 语义（alert/confirm/
  prompt 同族；`if (win)` 守卫自然跳过，站点脚本不炸）。契约有桥测试
  钉面：part10.rs:2022-2043 `window.open() → null` 断言。
- **评估对象 = stub → 真 window opening steps 升级**（返回 WindowProxy /
  null(noopener) + features 串解析），非从零实现。
- **s431/s432 aux 通道**：form/anchor target=_blank 提交臂已合成 aux
  entry（`_zwLoadIframeEntry('__zwAux*')` + opener/close/referrer 预置）
  ——但 aux 上下文无可返回 WindowProxy（提交臂不回传新 window）。

## 语料依赖面（html/semantics 域，grep window.open 字面）

21 文件 = 14 断言页 + 7 helper。**全部被 runner 内容级/目录级 skip 规则
挡在可执行集外，稳定红账零键**（s432 pairs-5/6 基线逐文件核对 total=0）：

- the-iframe-element/ **11 文件**（iframe_sandbox_* 族、
  iframe-loading-lazy-nav-window-open、sandbox-inherit-*、2 helper）——
  目录级排除域（navigation M3 多 frame 门控）先中；
- links/ 3（htmlanchorelement_noopener、
  rel-opener-prevents-browsing-context-group-change.tentative、
  target_blank_useractivation_multi_globals）；
- forms/form-submission-target/ 2（form-target-blank-useractivation-
  multi-globals + endpoint helper——s431 记账 skip 先例）；
- forms/the-input-element/ 4（range-restore .https + 3 helper）；
- the-img-element/ 1（image-loading-lazy-move-document）。

skip 判据：testharness_html_semantics.rs `html_semantics_case_skipped`
`source.contains("window.open")`（「多窗口/popup——runner 单文档无窗口
管理」）+ `/the-iframe-element/` 目录排除 + `.https`/helper 目录规则；
webmessaging/cookies/workers/fileapi/page_state 五域同款内容镜像在案。

## 施工依赖链评估（若解禁）

1. engine arm 升级：features 串解析 + rel 语义矩阵（可复用 s432 anchor
   臂）+ 返回新 WindowProxy 或 null——engine 域内可施工；
2. **WindowProxy 物化 = runner 架构级门**：aux 文档第二 global 物化需
   runner 多文档/多窗口管道——与 the-iframe-element 目录排除、
   navigation M3 门控、s86 moving-between-documents 挂案同门（runner
   单文档通道上限面）；
3. runner 侧：window.open 内容 skip 解除 + popup URL 端点仿真 + aux
   文档加载通道。

## 判决

**立项挂账，不流内施工**：依赖案零账面暴露 → 施工零直接翻绿收益；前置
= runner 多文档管道（架构级，跨 goal 门控共享）；真弹窗的站点仿真收益
（OAuth 弹窗/离开守卫真开窗）属 site-opt 流域与本 goal 无账面交集。
归并记账：与「navigation M3 多 frame 门控/the-iframe-element 目录排
除/moving-between-documents 挂案」同族并入 **runner 多文档/多窗口管道
能力域**单一挂账项（能力上限记账，同 listbox-touch 通道上限先例；非深
结构改动不新开用户门——若用户点名做 runner 多窗口通道，届时按三面分期
立项：runner 管道 → engine arm+物化 → skip 解除+端点仿真）。

## 收口验证

全桥套件绿窗轮末收口跑（s433 常态化挂账兑现）：**1570P/0F** exit 0
（基线维持；s435/s436 两轮零源码改动，窗口树恒等）。工件
/tmp/s436/bridge.log。
