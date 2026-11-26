# kc5r137 verdict——`:has-slotted` bare 形端到端＋刀域重定谳（2026-11-26）

## 结论

- **刀域重定谳（wpt.fyi 上游定谳闭环）**：k136 挂账的 `:has-slotted` 22 键
  实测分解——
  - **functional-changing-001..004（4 键）踢出刀域**：wpt.fyi 定谳三浏览器
    全红（chrome/firefox/safari 0/1×3，POST /api/search run_ids
    5277405514825728 / 5194221192216576 / 5093879951654912 @ run
    c197984917）＝speculative 测试期望自相矛盾，不实现反规范语义，FIXME
    注记挂账。
  - **可赢域＝18 键**：manual-assignment 4＋query-selector 8＋changing-001
    4＋changing-002 2。本刀兑现 manual-assignment 4 键；其余 14 键依赖更
    深基建挂 k138+（见「残 F 挂账」）。
- **manual-assignment 4 键根因收窄**：k136 b1 JSON 四键失败消息全是
  `' :has-slotted' is not a valid selector.`（test 3 首断言
  assignedNodes().length===1 已过）＝slot.assign()/assignedNodes() 通道端
  到端已工作，唯一缺口＝选择器引擎的 `:has-slotted` 识别＋匹配。
- **实现四面**：
  ① **dom/query.rs**：`PseudoClass::HasSlotted` 枚举变体＋parse bare 放行
  （ASCII 大小写不敏感；函数形 `:has-slotted(<compound>)` query 域拒臂——
  shipped 引擎默认态走未知函数伪类拒）＋matches_full 延后返 true
  （deferred-pseudo 模式，Has/Disabled 同型）＋**Not 内层同型延后**
  （s326 形——matches_full 的 Not 臂对含 HasSlotted 内层延后返 true，否则
  `:not(:has-slotted)` 恒否决；钉测首跑实证）。
  ② **dom/document/mod.rs**：element_matches_selector 求值臂
  `!self.get_assigned_nodes(node).is_empty()`（manual 手动表/auto 自动表两
  源；文本节点计入＝bare 语义非 `:has-slotted(*)` 别名）；Not 守卫臂扩
  HasSlotted 内层经 element_matches_selector 递归复评。
  ③ **shim part05.js**：`_parseCompoundOf` 白名单 bare 形＋`_matchPseudosOf`
  求值臂（tag 门→assignedNodes().length>0，try 包裹）——R2928 registry
  qSA 路径消费（探针实证 P4 qSA live 绿）；`_zwHasSlottedLive` 新帮手＋
  `_zwHandleMatchesComplex` 保守门扩 **单 compound 组伪类面 ⊆ bare
  has-slotted** 例外（函数形/组合器携带形仍拒——祖先 compound 静默放行会
  over-match）＋右 compound 活体求值。
  ④ **shim part03.js**：`_r134MatchTypeSelector` 对含 `:has-slotted` 形返
  null 分流——本函数尾部 `low === local` 标签比对对「无 `\s>+~[.#` 特殊字
  符的伪类形」早退 false，截走 s83 活体臂（探针分诊定谳：P1–P5 全绿仅
  matches has-slotted 假→R134 截胡实证；唯一调用点 part04 matches trap，
  仅此伪形分流零行为外溢）。
- **为什么 matches 须活体（快照链定谳）**：`matches`/host 查询走 dom_html
  快照（parse_html 现解析），SlotAssign 变异**批末才落地**（R57 视图文档
  structural-only 重放不含 SlotAssign；handle ref 亦不可宿主侧解析）——同
  turn `assign()` 后立即 `matches()` 恒 stale。qSA（ShadowRoot 域）走 R2928
  JS 端 registry 树搜＋客户端匹配＝天然活体。marker 通道（
  data-zw-manual-assigned）只承载跨 turn 持久化且要求全 id 引用（无 id 降
  级）——判据页 test 3 分配无 id div，marker 不发射；单独修 marker 反毒化
  test 4（快照带旧分配态＝stale matching）＝两者都不修，活体求值是唯一
  正确面。FIXME(45-M3+) 树位路径编码（无 id 稳定引用）保持挂账。
- 指纹 `ZW-KC5R137-HAS-SLOTTED`（css_wire.rs `[u8;22]`，strings=1）。
- **电池**（r3 终账；b0=k136 归档 bin r136-other-slots-runner md5
  150390c2／b1=新代 r137-has-slotted-runner md5 94582c53；锁键
  zw-k137-battery.lock；2 腿全 RC=0）：
  **css-shadow F2P=4 P2F=0 NEW=0 GONE=0＝净 +4 红线成立**（271→275）。
  范围声明：改动面＝`:has-slotted` 形选择器全新接受（此前 invalid/
  unsupported 恒败），非该形页面零行为差（parse 白名单/求值臂/分流均以
  伪类名门），故 8 suite 扩面不发射，红线以 css-shadow（has-slotted 22 键
  唯一驻留 suite）承载。
- **F2P 分解**（4 键全数兑现；键级差分 /tmp/zw-k137/diff-k137.py
  Counter）：
  - has-slotted-manual-assignment：does not match with no manual assigned
    nodes／- 2／does match when a child is manually assigned／no longer
    matches when no children become manually assigned。
- **r1→r3 电池考古**：r1（+3）缺 test 3＝marker 无 id 降级＋R57 不重放
  SlotAssign 双因；r2（+3）`_zwHandleMatchesComplex` 活体臂被 R134 截胡
  （分诊探针 7 test 定位）；r3（+4）R134 分流后四键全数。
- **钉测**：dom kc5r137 5 钉（selector_validity 双态＝bare/compound 合法＋
  函数形拒／auto_assignment 两态含 qSA 同判／manual_assignment＋非 slot 恒
  不匹配／text_slottable_counts＝文本节点计入 bare 语义／not_combination
  ＝`:not(:has-slotted)` 两态——后者首跑红抓出 Not 恒否决缺陷即修）。
- **门禁**：clippy zero-dom＋zero-engine -D warnings 绿；zero-dom 全量
  923P 0F（+5 新钉）；engine lib 4347P 3F＝红名单三键 s330/r3046/p99s 恒
  对账零新红（k136 账面同键）；shim 9.2MB 拼接体 node --check 绿；
  has-slotted-k137-probe 探针页按 trap-rm 协议发射前删除。
- **残 F 挂账 k138+**：
  - query-selector 8 键＝嵌套声明式 shadow root 物化缺口（
    collect_declarative_template_candidates children 边跳 shadow root 树
    ＝nested 域 `querySelector('slot')` 返 null）＋changing 页 gCS 返空串
    ＝s402 handle 身份类（declarative 域未解）。
  - changing-001/002 6 键＝reactive invalidation（赋值变化后样式失效重算
    ＋transition 断言）——依赖样式引擎失效通道。
  - `:is(:has-slotted)`/`:where(:has-slotted)` 内层 matches_full 延后真值
    恒命中（over-match）——需 Document 侧 Is 复评臂（同 Not s326 形），
    query.rs FIXME(k138+) 注记。
  - k136 既有挂账不动：::part 盒布局 7 键（渲染流域）＋exportparts＋shim
    gCS length/item＋computed_value inherit/unset/revert 父链。
- 电池工件：`.acceptance/wptfyi-css/k137/results-{b0,b1}-css-shadow`＋
  transcript；差分脚本 /tmp/zw-k137/diff-k137.py；分诊探针
  /tmp/zw-k137/probe-run{1..4}.log。
