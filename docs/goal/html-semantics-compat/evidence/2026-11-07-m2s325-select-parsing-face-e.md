# M2 s325 — Face E select 解析定制族收口：real 64P/133F→72P/125F，select-parsing/button-behaviors/option-form-ancestor 三页全绿，select-options-id 3/3 全绿；select-input-parsing 2F=html5ever 上游 #12050 缺口定性

日期：2026-11-07。切片号 s325（承 s324 Face E 候选：select 解析定制族施工）。

## 面与基线

Face E（s323 五面分派候选）：`html/semantics/forms/the-select-element/customizable-select/`
下 select **解析/结构定制** testharness 页，`make testharness-html-semantics
FILTER=customizable-select` 通道全域（75 testharness 页，含 s324 已收口的 selectedcontent
族——本切片账记全域 real 数，selectedcontent 子域数字不变）。基线（r1，s324 升级
html5ever 0.38 后）**64P/133F**。

四页钉面：
- `select-parsing.html` 17 子测（div/button/datalist 于 select 内保留、嵌套 select 闭外层等 0.38 新解析面）
- `select-button-behaviors.html` 2 子测（select 内 button 提交候选排除/点击面）
- `option-form-ancestor-select.html` 2 子测（option 内 form 祖先判定）
- `select-options-id.html` 3 子测（wrapper div 内 option 的 options 集合/移动/重解析全链）
- `select-input-parsing.tentative.html` 4 子测（input 于 select 内解析，见不可达定性）

语料页为 wpt-data 本地 gitignored 数据（s323 勘根批带入，无 imported-tests.txt 登记）。
探针页 s325-probe-options-move.html / s325-probe2-corpus-replay.html 为本切片调试用
gitignored 本地文件（不入库）。

## 轮次进展（日志 evidence/logs/s325-cs-r{1,2,3,4,5,7}.log；r6=⑩单独复验零位移不入表）

| 轮 | real 子测 | 变化 |
|----|--------|------|
| r1 | 64P/133F | 基线（s324 0.38 升级后全域） |
| r2 | 67P/130F | ①select-button-behaviors（form_state.rs :default select 父门）→ 该页 2P/0F |
| r3 | 70P/127F | ②option-form-ancestor（JS shim 祖先甄别）+ ③`_zwFragmentAdded` SELECT 上下文深解析（sel 形传 hostSel 第三参）→ 两页各 2P/0F、select-parsing 17P/0F |
| r4 | 70P/127F | ④⑤⑥移动/removal 面修复入树后回归复验（探针 r6 同轮，零位移） |
| r5 | 71P/126F | ⑨overlay 快路径门补桶态 → options-id test2（multiple remove）绿 |
| r7 | 72P/125F | ⑩物化选中态键迁移 + ⑪value setter 写键 `_zwOptStateKeyOf` → options-id test3 绿 |

终态 **72P/125F**：select-parsing / select-button-behaviors / option-form-ancestor-select
三页全绿，select-options-id 3/3 全绿。余 2F=select-input-parsing（不可达，见下）；
余 123F 分布与 s323 勘根分派一致（selectedcontent 9F=解析期记录天花板、Face A 协付点、
Face F filterable-select 等，非本切片新增）。

## 施工面（Rust ① + JS ②-⑪）

### ① `:default` submit 候选的 select 父门（crates/dom/src/document/form_state.rs）

`is_submit_button_candidate` button 分支 Auto 态（缺省/无效 type）加 `parent_is_select`
门——select 首子 button 为 inert、非提交面（whatwg/html #9799 谱系，Chromium CL 6004443 /
issue 498095295）。submit 显式 type 恒候选不受门约束。select-button-behaviors 2P/0F。

### ② option 祖先甄别 + ③ SELECT 上下文片段深解析（part03）

- option-form-ancestor：option 的 form 祖先判定走 `_realTag` 身份态（sel/handle 双形），
  不再被包裹层误导。
- `_zwFragmentAdded(html, hostHandle, hostSel)` 签名扩展第三参：SELECT 容器上下文
  （`_realTag` 判定，sel/handle 双形）改走 `__zw_parse_fragment_children(.., 'select', ..)`
  深解析（0.38 新 select 规则的 JS 侧镜像），替代 body 上下文通用解析——select 内
  div/button/datalist 保留不剥壳、嵌套 select 闭外层。innerHTML setter 调用点
  （part04:14246）传 sel。select-parsing 17P/0F。

### ④⑤⑥ 移动/removal 面三修（part04 R334 臂 + part05 对冲门 + parentNode getter）

select-options-id 的 option 在 wrapper div 间移动/跨测试 remove 的 fused 视图账本面：

- **④ R334 appendChild reparent 桶撤账**（part04:7885 区）：sel 子移动时旧父桶
  splice added + removedSet 记账（移动语义闭环）。
- **⑤ 对冲前提修正**（part05 `_zwHCLiveInvalidate` 对冲分支）：R51c 消零假设
  「added ⇒ host 快照从未见过」对 R334 移动补偿条目不成立（它们是基底快照成员，
  engine 纯测试环境无配对 apply、基底恒原始位）。判据改**本父基底缓存 identity 成员**
  （`_zwChildBaseCache.get(mutSel)` 扫描）：基底成员 remove 时不对冲——照常记 removed
  （overlay 剔除基底残影）+ splice added（补偿撤销）；真 pending 新子（createElement/
  解析产物）不在基底 → 对冲保持，R51c 热路径零变化。
- **⑥ 槽让位移除标记**（part04:2480 parentNode getter）：R334 移动留下的
  `_zwSelPendingParent` 槽在后续 remove()（R125 元素路径不清槽）后仍指旧挂点，槽优先
  分支先于 `_parentNodeFor` 的 `_zwIsRemoved` 短路 → remove 后 parentNode 恒返旧父。
  标记中节点先置空槽走下游（spec remove() 后 parentNode null）。

### ⑦ qS stale 命中穿落 R322（part04 querySelector R310 臂）

同 turn innerHTML 整体替换（test3 自往返重解析）时 host 快照恒旧（纯测试环境无配对
apply），stale 命中原直返 `null` 使 R322 pending 回落不可达——qS 对重解析新子恒 null
而 qsa 走 R309 重建即见（qS/qsa 分叉）。改为穿落 R322 桶归并：added 含同选择器新子返
新子（live 视图语义），纯 remove 落尾部 `return null` 语义不变。

### ⑧ overlay 快路径门补桶态（part05 `_zwOverlayPendingChildNodes`，s325 深根因）

原快路径门只看全局 pending 表（`!_zwPendingAdded.length && !_zwPendingRemoved.length`
直返未滤基底）；桶级 removed（R51c 消零条目不入全局表 + R334 移挂/对冲仅桶级记账）
在全局表排空时被跳过 → 基底残影泄漏。select-options-id test2（跨测试复跑：首轮 remove
的全局条目被重添加对冲/消零悉数排空，桶 removed=[four] 独存，remove 后 select.length
恒 6）钉此。门改全局空**且**桶 miss/空才快出；全局空+桶空热路径零变化。
探针2 r1→r4 实证（evidence/logs/s325-probe2-r{1,4}.log：x2-post-remove len 6→5）。

### ⑨⑩ 选中态键迁移 + value setter 写键（part04 R322 物化臂 + value setter）

select-options-id test3（reparse + multiple + value=）双根因：

- **⑨ value setter 写键**（part04:15421 select value 分支）：`_zwOptKey(sel, handle)`
  对解析代理（innerHTML= 产物，无 sel 无 handle，state key = `plainOpt:pN` 实例键）
  双 null → 编程选中写进垃圾键，`_zwOptSelOf`（读侧 `_zwOptStateKeyOf`）恒 miss。
  改写 `_zwOptStateKeyOf(option)`（三形统一，`_zwSetSelectSelectedIndex` 同款）。
- **⑩ 物化态键迁移**（R322 qS 物化臂）：`_wcRebuildAsHandle` 换 identity，
  plainOpt:pN 键下 dirty/clean 选中态随迁 @handle 键（runTest 首读 qS 物化顶层
  option.one 恰是 value= 已选中条；multiple 无 pristine 回落 → select.value 读空串）。

## 不可达面定性（select-input-parsing 2F）

`<input>` 于 select 内解析：2F=「no options first」/「nested in another tag」两案
（input 应滞留 select 内成下一个 select 兄弟……实际为 `<button>button</button>` 壳），
2P=option 邻接两案（input 关 select）。

定性**不可达（上游缺口）**：html5ever 0.38 rules.rs 的 input 规则在 select 上下文
无条件 pop-until-select（#814 区），whatwg/html #12050 的 refine（input 于 select 内
滞留，以「其后是否出现 option」为界）未实现——语料 `.tentative` 即挂该 issue
（select-input-parsing.tentative.html line 3）。下沉侧不可修：input-before-option 与
input-after-option 两案的 sink 产物形状不可区分（input 均为 select 下一兄弟），分叉
判据只在上游解析器状态机。不 fork 上游，挂账 upstream issue（若上游落地随升级复验）。

## 验证

- `make testharness-html-semantics FILTER=customizable-select` r7 终态 72P/125F（real，
  全域；探针页 24 子测 assert_false 设计不入账）——options-id 3/3、select-parsing 17P/
  button-behaviors 2P/option-form-ancestor 2P 全绿，selectedcontent 子域 12P/9F 不变。
- `make test` 全量 **3787P/9F**（9F=对端 js_dom_bridge 在册红腿——stylesheet 规则文本
  序列化/fetch response type/websocket/service worker 对端面，与 s324 账本逐项一致，
  零新增）；`make reftest` **725/725**（Layout 521/521 + Text 204/204，exit 0）；
  `cargo clippy -p zero-dom --all-targets -D warnings` PASS；`cargo fmt --check` PASS。
  （make test 全量 ~40 分钟，setsid nohup 后台跑，日志 /tmp 落盘后核对。）
- 探针资产：s325-probe-options-move.html（13 子测 move/removal/reparse 序列）、
  s325-probe2-corpus-replay.html（corpus 1:1 复刻 + dump 注入 + x3 value 面定位）——
  gitignored 本地文件，调试日志 evidence/logs/s325-probe2-r{1,4}.log。

## 下一步

- s326 候选：Face F filterable-select（s323 分派末位，低优先）或转 mt 残 236F 普查
  （v1.66 控制面既定方向）。
- 挂账：html5ever 上游 #12050 input 规则（随上游升级复验）；Face A 协付点
  （appearance:base-select 渲染流）不属本流。
