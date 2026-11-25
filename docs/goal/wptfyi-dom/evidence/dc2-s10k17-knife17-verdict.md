# dc2-s10k17 刀⑰ 判定：激活链 MEl 区域查询并查面 + R517b template 下钻守卫（2026-11-25）

刀⑰＝双提交合账：k17 码刀（`44ba2a03d`，R517+R518+part163 七段钉）＋ k17b 修复刀
（rebase 前 d6fb96621／后 `d132ff630`，R517b + part163 ⑧段钉）。v1 电池发现 WC 36F 回归 → probe 链定根因 →
k17b 修复 → v2 电池五腿零回归收账。

## 刀面（根因链）

**k17 头① R517**（part04）：`_zwMEl` 装配块产物（纯 JS 私有树，无 host wire）对
handle 域 createElement 作用域的 GECN/qsa 静态结果不可见 → WPT
`dom/events/Event-dispatch-single-activation-behavior.html` 残 121F（k16 后）的
stamp/href 相位漏嵌套 target → classList 缺 `test{i}` → filter 全弃。修复＝
`_zwMElRegionMerge`：engine 静态结果与 fusion 子树 walk 并查
（identity + `__zwHandle`/`__zwSelector` 键双去重）；匹配真值源
`getAttribute('class')` ASCII 分词（probe17 实证 MEl classList indexed 读返空串
缺口挂账）。

**k17 头② R518**（part03）：A/AREA href IDL getter 返回解析绝对 URL（HTML spec
dom-hyperlink-href；R130 升级：`__zw_parse_url` base=页面 URL，空 attr 早退保
R130 口径）——WPT expected 相位 `e.href` 对 hashchange `e.newURL` 绝对形同形比对。

**v1 电池回归（36 P2F，全限定 shadow-dom）**：`shadow-dom/slots.html` 14 ＋
`slots-fallback.html` 7 ＋ `event-composed-path.html` 6 ＋
`event-composed-path-with-related-target.html` 5 ＋ 单发 4。A/B bisect 定责：
R518 无关（noR518 腿 F 集与 solo 逐字节恒等）、R517 是唯一根因。

**机制（probe c 复刻 corpus createTestTree 嵌套形实证）**：corpus
`walk(node.cloneNode(true))` 的 clone root 是 handle 代理（`H=__n0`）、子树为纯
MEl（`H=undefined`）→ clone 的 `querySelectorAll('template')` 走 part04 get-trap
→ R517 并查触发：engine 侧 R175 正确排除 template content，但 **fusion walk 经
MEl template 的 `childNodes` 下钻进 content fragment**，嵌套 template 误入顶层
结果（probe c 实测 `tpl n=2`）→ corpus `attachShadowFromTemplate` 对同一嵌套
template 二次装配 → ids 表污染 → slots/composed-path 断言全断。即 part03
WC-M3 序列化面注释（11865 行）记载的同一旧坑在并查脸上复活。

**k17b 修复 R517b**：`_zwMElRegionMerge.walk517` 对 TEMPLATE 子元素不下钻
（template 本体仍参与匹配）。spec：template 内容属独立 DocumentFragment、不在
查询树（引擎 R175 同口径、part03 WC-M3 序列化面同规则先例）。
https://html.spec.whatwg.org/multipage/semantics.html#the-template-element

**作废记录（v1 shadow-scope 域门）**：429 死亡前一轮曾试「shadow 根容器不并查」
域门（`_shadowHandles[handle]` 早退）——gated-v2 腿 F 集与 solo 逐字节恒等实证
无效：回归作用域是 detached light clone（handle 非 shadow handle），门永不触发。
k17b 提交已撤除该门，保留净修复面。

## 验证链

- probe b（已删，备份 `.acceptance/dps/s10k17b-probe-tmp-backup.rs`）：单层 clone
  戳记（子树纯 MEl 无 handle、fusion 与查询同身份）——单层形不复现，排除身份面。
- probe c（已删，备份 `.acceptance/dps/s10k17b-probe-c-nested-backup.rs`）：嵌套
  createTestTree 同构复现（修复前 `tpl n=2` → 修复后 `tpl n=1`，装配链端到端断言
  c1→s1→s2 全过）。
- 钉测 part163 ⑧段常驻（嵌套 template 不外漏 + createTestTree 同构装配链
  assignedNodes 断言）；s10k* 全系 8 pins 1 passed 0 failed。
- **v1 电池**（二进制 `e6048692b222c3259c27043b48c5d509`，k17 码刀）：WC
  1118→1153（36 P2F／1 GONE 噪声）、DP/SAN/TT SET-IDENTICAL、**dom 485→430
  ＝GONE 55 全限 Event-dispatch-single-activation**。bisect-noR518 腿（F 集与
  solo 恒等）＋gated-v2 腿（F 集与 solo 恒等＝域门无效实证）。
- **v2 电池**（二进制 `ca63598434038b48c7e1da8daf0c7397`，k17+k17b，test-guard
  包裹，发射器 `.acceptance/dps/s10k17b-launch-v2.sh`，main log EXIT 五腿齐）：
  WC **1118→1118＝GONE 0／P2F 0 双向零差**（36 P2F 全消除）；DP/SAN 日志
  **md5 与 v1 逐字节恒等**（`f70f8f97…`/`6e53a750…`）；TT 逐行恒等（F=362 对齐
  锚）；**dom 430F＝GONE 55／P2F 0**（F 集与 v1 dom 逐键恒等，diff 0；P 漂移
  ＝Range-mutations 截断带非回归）。
- fmt 零 diff；clippy `-p zero-engine --lib --tests -D warnings` 0 warning。

## 判读结论

- 刀⑰ 合并净增益＝**dom +55**（Event-dispatch-single-activation 121F→66F），
  WC/DP/SAN/TT 四腿对 k16 锚 SET-IDENTICAL。
- 累计 83,462 → **83,517**（距 ≥99,947 差 ≈16,430）。
- 残 66F＝激活 containment 组合面（child=FORM 嵌套双层提交链／parent=INPUT void
  父嵌套／a-area hashchange 异步／label-details 跨边界——k16 census 继承）。
- 挂账：part03 R516 GECN walker（`_r516Walk`）同型 template content 下钻风险
  （corpus 未触，spec 同规则待同一守卫）；MEl classList indexed 读返空串缺口
  （probe17，k17 承接）。
