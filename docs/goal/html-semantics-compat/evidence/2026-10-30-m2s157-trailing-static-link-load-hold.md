# M2 s157 · 尾随静态 link 触发收口——脚本流末尾补扫 + stylesheet 取回 load 持有（尾扫门控）

日期：2026-10-30 · 切片：html-semantics M2 · 前序：s156（LinkStyle sheet 创建门）

## 入场定刀位

s156 下一步候选②：link-load-event document load 阻塞面 1F（the-link-element/
link-load-event.html「Check if the stylesheet's load event blocks the document
load event」——`saw_link_onload` 恒 false）。

## 勘根（根因升级定谳）

1. **根因不是「stylesheet 未 delay load」而是尾随静态 link 零触发**：案面形态
   = 3 scripts + 尾随 `<link rel=stylesheet onload>` + window load 断言
   `saw_link_onload`。s152 静态扫描按 script ordinal 触发「script 之前的
   link」——最后一个 script 之后的尾随 link 从未被任何臂触发，link.onload 恒
   不 fire。先修「delay load」无对象可 delay。
2. **spec 时序不变式**：script-blocking stylesheet 只阻塞其后 pending script；
   尾随 stylesheet 不阻塞任何已执行 script，只 delay document load event
   （the-link-element styling——加载完成前 window load 不派发）。
3. **trickle 滞留假设被证据否定**：runner 对 stylesheet 忽略 `?pipe=trickle`
   MIME 面（testharness.rs:5251）+ 扫描窗折叠 trickle（`__zwScanCollapseTrickle`
   ——尾扫 fetch 瞬时完成），3s 滞留算术不成立。

## 施工（3 文件净 +61：两件套 + 归因轮门控修订）

1. **webview.rs 尾扫**：脚本流末尾补扫 `_zwStylesheetStaticScanUpto(2147483647)`
   （page_scripts_initialized 前）——shim 侧序号越界收敛为全部 link，游标幂等
   （已触发段零成本返回）。
2. **part03 在飞计数**：`_ssVisit` 每层恰一次 settle（同步门/then 链全路径）
   → 包装计数归零（`_zwSsLoadPending`）；@import 内层递归逐层自计；fetch 同步
   抛按取回拒绝语义归零派 error（旧路径外层 catch 静默无事件——计数平衡优先）。
   preload 取回不计数（resource hint 不 delay load）。
3. **loadStep5275 第四谓词（testharness.rs bootstrap）**：
   `__zwStylesheetFetchPending()` 轮询在飞取回。
4. **归因轮门控修订（见下）**：谓词按尾扫触发印记 `_zwSsTailFired` 门控——仅
   越界补扫真实推进游标的页面置位；常规交错扫描（序号有界）不置位，无尾随
   link 的页面谓词恒 false 与基线全同。

## 归因轮（全量 link-multiple Timeout → A/B 定谳 → 门控修订）

1. **回退面**：全量跑 link-multiple-load-events 2 Pass 键失 → Timeout 键得
   （pending:2/tests:6 unreported——testharness 全局 ~10s 预算收文件）；定向
   6/6 Pass、目录级不复现、document-metadata 级复现。
2. **语料探针定谳（wpt-data 是 gitignore 本地数据——stash 不收，探针还原须
   手动）**：案 1-4 幂等属性写各烧 `t.step_timeout(2000)` 竞速窗 = 名义 8s；
   探针 A（失败断言消息携带 elapsed——日志无 console 透传）实测 **9757ms**
   （溢出 ~440ms/案），案 5/6 仅 38ms。
3. **基线 A/B（stash 全部源码改动、同探针语料）**：基线 ZZ-A **9334ms**——
   基线自身溢出 ~1.33s，距预算仅 0.67s 余量；本刀边际 +423ms（n=1，同源
   f4/f5 间噪声带 ≥200ms）。**定谳：贴边是固有属性**（案 1-4 竞速窗 ×4 +
   计数 fetch 往返延迟），非尾随面按页边际须归零。
4. **门控修订**：`_zwStylesheetStaticScanUpto` 越界调用且游标推进时置位
   `_zwSsTailFired`；谓词 `return _zwSsTailFired && _zwSsLoadPending > 0`。
   修订后全量 link-multiple **6/6 全 Pass**（贴边案在基线行为域内）。

## 验证

- **语法**：规范序拼接 node --check SYNTAX_OK **110462 行**（s156 110429，
  净 +61）。cargo fmt --check PASS；**guarded-clippy workspace 全绿**
  （CLIPPY_EXIT=0）。
- **门禁顺带修复（78754deb3 引入面，本流域 lane）**：quickjs_dom_bindings.rs
  s151 compatmode 孪生 `document_compat_mode_string` 无 cfg、唯一调用臂
  cfg 在 `all(quickjs, not(v8))`——默认 v8 构建 dead-code 断 clippy 门禁。
  修法 cfg 镜像调用臂（函数存在则调用者必存在）；与 s157 主刀无机制关联。
- **靶面定向**：link-load-event **Fail→Pass**（2/2 Pass）、MAKE_EXIT=0。
- **全量案级 join（对 s156 /tmp/zw-s156-full.log，(key,state) 对集直接
  去重）**：**8082 对 = 8082 对**（7199P/796F/87T 保持）——**F2P 1（靶案）+
  P2F 0 + LOST 1（即靶案旧 Fail 键）+ GAINED 0** 全域零位移（the-style-element
  族、LinkStyle 面、s150-156 修复面全保持；门控修订后 link-multiple 6/6 Pass
  在基线行为域内）。
- 探针语料验后已删（`__zzT0`/probe A/probe B 三块，工作树零残留）。

## 挂账（本切片更新）

- **收口：link-load-event document load 阻塞面 1F**（尾随 link 触发 +
  load 持有，挂账销案）。
- **FIXME 新记**：link-multiple-load-events 全量贴边——案 1-4 竞速窗 ×4
  名义 8s 固有溢出 ~1.3s（基线标度实测 9.3s/~10s 预算），余量 0.7s 噪声
  敏感；s155/s156 全量通过属余量侥幸，任何全局边际开销都可能翻键。上游
  语料无 meta timeout=long，语料是 gitignore 本地数据（修改不持久），runner
  侧根治面（meta long 通道放宽/计时器溢出收敛）另立。
- **承袭不变**：link-rel-attribute 2F（sheet 应用/removal 渲染面——跨域边界
  待归因）；link-rel-attribute-tokenization 1F；stylesheet-media-change-no-fetch
  1F；stylesheet-not-removed-until-next-stylesheet-loads 1F；meta long ×
  corpus 通道放宽失效；details 族 lib flake；promise_test 串行启动面；
  innerHTML 身份分裂表示面；sel 重解析同槽同 href 印记失配边角；待决三项
  须用户拍板。

## 下一步（s158 候选）

①stylesheet-media-change-no-fetch 1F（media 变更不重取门——helper 同型扩展
面）；②link-rel-attribute 2F 归因轮（sheet 应用/removal 渲染面——渲染流域
边界候选，先归因再定刀法）；③link-rel-attribute-tokenization 1F；④stylesheet-
not-removed-until-next-stylesheet-loads 1F。
