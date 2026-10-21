# M3-S6 子窗局部 history 遍历 + parser 期 iframe 自主载入（2026-10-21）

M3 第六切片：两个独立根因同轮收口（上轮 429 断点 WIP 经核对语义后补齐
落账）。零既有测试改写；runner 仅解锁 skip 规则三案。

## 根因一：子窗 history 遍历 = 父窗账本回退

子窗 stub 的 history `back/forward/go(delta)` 此前一律 `iframeReload({history:
true})`（跨文档重载路径）——子文档 pushState 后子窗 back 落不到同文档条目
（state 不还原、无 popstate），越界 back/forward 也不 no-op 而是触发重载。
iframe_sandbox_navigate_history 三案（上游 skip 记账池）全 Timeout 不可达。

## 根因二：parser 期 iframe 无人启航

`_zwStartConnectedIframe` 仅由连接变更路径（动态插入）调用；解析期带的
`<iframe src/srcdoc>` 若无人读 contentWindow，子文档 fetch 恒不发起、元素
load 恒不派——`<iframe onload="doNavigation()">` 内联处理器与 onload
property 双断言面全断，go_back 族三案载体不可达。

## 实现

- **子窗局部会话遍历**（part05 子窗 stub history）：
  - `_zwDocBase` 文档代基线（合成时游标位）——该位及之后条目与当前 Document
    共享；之前的条目属先前文档。跨文档重载同步（part01
    `_zwSyncIframeWindowHistory`）后重置基线 = 新代从当前顶开始。
  - `go(delta)`：越界（target < 0 或 > 顶）= no-op；`target < _zwDocBase`
    （跨文档代）= 维持 `iframeReload({history: true})` 旧路径；当代内 =
    游标迁移 + URL 槽同步 + `_defer` 异步 trusted popstate（state 取目标
    条目）。`back/forward` 转 go(±1)。
  - onpopstate 绑定面：子窗 stub 预置 `onpopstate: null` 槽（S5 onmessage
    同款 with 捕获前提）+ part01 子文档脚本 with(window) 包合正则扩为
    `on(message|popstate)`（未命中子文档逐字节不变）。
- **parser 期 iframe 自主载入**（part04 `__zwStartParserIframes` + part06
  window load 分支调用）：扫描 connected iframe（R139 同款扫描位），有
  src/srcdoc 且 `loading` 非 `lazy` 者补启 `_zwStartConnectedIframe(f, true)`
  （幂等——`_zwFrameElementLoadQueued` 门；window load 派发前一次性）。
- **runInlineScripts allow-scripts 门**（part05，首跑回归收口）：defer 通道
  `__zwRunInlineScripts` 此前无 sandbox 门（`_zwRunIframeScripts` fetch 通道
  有门）——补启使子文档先于父窗 sandbox 改写执行脚本，`iframe_sandbox_
  allow_script`（首消息时序断言）翻红。修 = runInlineScripts 顶部按
  frameKey 现读 `_zwIframeSandboxFlags`，`sandboxed && !allowScripts` 直接
  返回（不置 `_zwScriptsRan`——属性改写后 reload 的新指令随新值生效）。共享
  路径修复连带收口 `sandbox-ascii-case-insensitive`（S5 账本既有 Fail——
  「no scripts should run in the iframe」子测试，同洞）。
- runner（testharness.rs）：`iframe_sandbox_navigate_history_*` 三案解锁；
  ancestor/descendants/itself/other_frame_popup 维持记账 skip。
- 集成测试 part63 两案：
  - `m3s6_child_local_history_traversal_pops_child_only`——子 pushState 后
    back = 子窗 trusted popstate（state 还原 + isTrusted）+ 父窗零 popstate
    + 子窗 state/length 随游标 + forward 重入 + 越界 no-op。
  - `m3s6_parser_src_iframe_autoload_fires_element_load`——零 contentWindow
    预读时 parser iframe 自主载入：内联 onload 属性与 onload property 各
    恰 1 次（trusted）+ 子文档 fetch 结算 + loading=lazy 不补启（fetch 日志
    断言）+ sandbox 无 allow-scripts 子脚本禁跑（message 日志断言）。

## FIXME 记账（最小面边界）

- 子窗跨文档代遍历维持 `iframeReload({history: true})` 重载近似（非真联合
  遍历；S3 joint history.length 顶层长度挂账既立）。
- 子窗局部遍历不触父窗 popstate/联合序（spec 子窗自有 session history——
  go_back/go_forward 断言面正是父窗零 popstate）。
- with(window) 包合为文本启发式：`on(message|popstate)`；其余 on* 裸赋值
  （onclick 等）子文档面未触（corpus 零消费）。
- `iframe-loading-lazy-in-script-disabled-iframe` Timeout→Fail 位移暴露双
  缺口记账：① 子文档（合成树）元素 proxy 的 contentDocument 面缺（读得
  undefined）；② script-disabled（sandbox 无 allow-scripts）文档不豁免
  loading=lazy 的 spec 例外未实现。两缺口均后续切片候选。

## 验证

- 域内：the-iframe-element 32P/32F/23T→**36P/32F/22T**。
- 全量 corpus：`make testharness-navigation` **472P/69F/43T**（584 案行）
  对 S5 全量（468P/69F/44T，581 案行）逐路径 diff **恰 5 路径变化**：
  +3 解锁案 Pass（navigate_history go_back/go_back-2/go_forward）+
  sandbox-ascii-case-insensitive Fail→Pass（runInlineScripts 门连带收口）+
  iframe-loading-lazy-in-script-disabled Timeout→Fail（位移，见 FIXME）；
  **其余 576 路径逐行恒等零回归**。
- 引擎库：`cargo test -p zero-engine --lib` **3360/3360**（S5 态 3358 +
  m3s6 两案）。注：全量套件与 corpus 批并发时
  test_t8g_npo_iframe_scan_no_collection_leak（墙钟型）一次忙窗假红
  4.09s，solo 复跑 1.13s 恒过——归因 CPU 竞争非本切片（扫描位为 window
  load 一次性，非逐节点循环）。
- 跨套件回归臂：`make testharness-webmessaging` **132P/3F** 对 S5 账本
  135 行逐行恒等（差项仍为 user-activation + /common/gc.js 资产洞既有
  挂账）。
- fmt 干净；clippy workspace --all-targets -D warnings 零 warning；
  `make test`（test-guard，DISPLAY=:20）76 suites 全绿（见 master.md
  本轮记录）。
- reftest 未跑：变更面（子窗 history 遍历 + iframe 启航时机 + 脚本门）
  零布局/绘制触达（S1~S5 同理由）。

原始输出：[2026-10-21-m3-s6-child-history-parser-iframe-family.txt](2026-10-21-m3-s6-child-history-parser-iframe-family.txt)
（关键七行）、[2026-10-21-m3-s6-child-history-parser-iframe-full.txt](2026-10-21-m3-s6-child-history-parser-iframe-full.txt)
（全量 584 案行）、[2026-10-21-m3-s6-webmessaging-crosscheck.txt](2026-10-21-m3-s6-webmessaging-crosscheck.txt)
（跨套件回归臂）。
