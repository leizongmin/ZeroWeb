# d10 定谳：bold/italic 数值壳 toggle-off 残余毒根在 wrapper/handle 桥层

**2026-11-13 · 诊断批（无行为修复）**。

## 结论

bold 700/900 toggle-off 的语料残余（bold 12F 中的数值壳部分选区族）不是
execCommand 语义层的 bug。毒根在引擎 wrapper/handle 桥层：用
`document.querySelector` 取宿主时 execCommand 流程在 peel 切分之后集空转，
摘壳永不发生；改用 `tc.querySelector`（元素作用域）取宿主，同一页、同一段
代码全部变绿。宿主的获取方式就是这个宇宙的开关。

corpus tests.js:5560 恒用 document 作用域查询取宿主，所以语料恒红；d9/d10
探针页用元素作用域查询，所以单 case 隔离页恒绿——这也推翻了 d9 批
isolation 分诊"bold 残余大半为跨 case 状态面"的判断：单 case 纯净页
（document 作用域）同样红，红与绿的分界不是 case 间状态，而是宿主获取方式。

## 定谳链（按实验时序）

全部实验在 instrumented 构建（CARGO_TARGET_DIR 独立目录 + systemd 守卫）下
用探针页跑，引擎侧临时探针已全部摘除，本节保留结论与关键数据。

1. **四层现场全对**（承接前窗）：fq 读面（rng tx@3..DIV@1、nodes=[tx]=900、
   state=true）、format_apply 传值（nv='normal' 正确）、setSelectionValue
   入口、cv pass 插入点现场（SPAN/900/edit:true/simple:true）逐层正确，
   全流程零异常。毒不在这些位置。
2. **forceValue 激活门实验（负结果，防重蹈）**：假设"移除面不该包"在
   forceValue 入口加 bold/italic 非激活值早退门，重建后 face 全量对跑
   base face-d9c2.log → **P2F=84、F2P=0**（bold?2001-3000 h3 族 32 断言、
   italic?1-1000 address/cite/dfn/var 族 52 断言整腿红）。gate trace 显示
   ADD 面调用的是 forceValue(node,'italic','normal')：vendored 的包裹面由
   command 决定，'normal' 只是并值/等值门的参数，ADD 面（address 内
   italic ADD 产出 `<i>`）依赖这次调用。**"newValue='normal' 即移除面"
   的假设不成立，该门方向错误，已摘除**。plain 单腿对照：italic ADD 走
   forceValue(tx,'italic','italic') 正常产出 `<i>`。
3. **forceValue 等值门 trace**（单腿纯净页）：b900r 腿（900 壳部分选区
   toggle-off）fv=[]——forceValue 连等值门都没到达过一次。
4. **clearValue entry trace**（守卫之前）：cv=[]——`_zwVaClearValue` 一次
   都没被调，即 setSelectionValue 的 contained-element clearValue pass
   元素集为空。
5. **setSelectionValue 终态 DOM 快照**：返回之前宿主 innerHTML 已经是
   `<span 900>foo</span><span 900>barbaz</span>`。毒发生在 setSelectionValue
   内部、且在 cv pass（未执行）之前。
6. **时序收敛**：`<span 900>foobarbaz</span>` 选区 foo[barbaz + bold
   toggle-off，peel start 切分把壳切成两段是正常行为（第一段 foo 克隆壳
   正是期望形的前半）；第二段（原壳，fully contained）应该由 cv pass 摘除，
   但 cv 元素集为空——contained 集 walker 从选区边界向上爬宿主失败。peel
   切分搬移/拆分文本节点后，边界所指 wrapper 在 document 作用域宇宙下
   已经爬不到宿主链。

## 桥层同族证据（支线 bug 清单）

三个独立观到的症状与主毒同族，指向 wrapper 缓存/handle 链失效：

- tc.querySelector 在 tc.innerHTML 重建子树后返回 children 为空的包装
  （探针 qdump `{"h1":"DIV:0"}`）。
- tc 作用域宇宙下 addRange 后 public `getRangeAt(0)` 抛 index-out-of-range
  （rangeCount=0，内部 `sel._ranges[0]` 却有值——双存储分裂）。
- `_zwJusMovePreservingRanges` 的 B2 臂（part01c）：bare 文本节点入
  handle 壳时经 host 重建新文本节点 + JS 侧旧 node 再 insertBefore，两棵树
  的落地形态分裂只在 `newParent.__zwHandle` 存在时触发——handle 链的有无
  正是两个宇宙 wrapper 的差异点。

## 修复方向（移交独立批）

动 wrapper/handle 桥层（part01c 的查询包装一致性、子树替换后缓存失效、
B2 臂双落地）。影响面是语料全域而非 editing 单面，不适合在 editing 批内
顺手修；建议开 DC-2 桥层专项批，落点核对 face 六文件 + multitest/misc +
selection 片三面。

## d11 补充：混合树机制定谳（2026-11-13 第二轮）

诊断探针 `tests/wpt-runner/wpt-data/editing/run/zz-probe-d11tree.html`
（保留为桥层批验证页）对同一用例按宿主获取方式分腿，execCommand 前后
双时点 dump wrapper 形态（`__zwHandle` 有无）与选区边界爬升链：

- **pre-exec 树形态由宿主获取方式决定整棵解析子树**：tc-scope 宿主为
  handle 形（`DIV.H:__n3`），innerHTML 解析产物全链 handle 形；doc-scope
  宿主为 plain 形（`DIV.plain`，`_zwMEl` 物化是既定设计，part03.js:1667
  注释），解析产物全链 plain 形。
- **peel 切分的克隆壳恒为 handle 形**（两宇宙一致——peelOne 的
  `document.createElement` 工厂，part01d.js:2014/:2041）。tc-scope 全
  handle 树吸收 handle 克隆壳形态一致，链路自洽（POST got = 期望形）；
  doc-scope plain 树混入 handle 壳即成混合树（POST climb：
  `tx.plain < SPAN.H:__n1 < DIV.plain`）。
- 混合树上 handle 形壳的 childNodes 走 handle 侧记账，叠加 B2 臂双账本
  形态（bare 文本入 handle 壳：host 重建 handle 文本 + JS 侧旧 plain 节点
  再 insertBefore），"选区边界在壳内"判定失配，contained 集 walker
  空转——与 d10 定谳的 cv 集空、clearValue 零调用完全吻合。
- tc-scope 全 handle 链上 peel+cv+摘壳组合全程工作正常：摘壳逻辑本身
  成立，分歧只在形态一致性。

修复实施的两个候选（桥层批评估择一）：

1. **克隆壳形态对齐**：peelOne 克隆工厂按被切壳形态分流——被切壳为
   plain 形时克隆壳走 plain 工厂，保持全 plain 链。改动面小（两处
   createElement），但 plain 链上的摘壳组合从未跑通，须先验证 plain 树
   的 `_zwJusMovePreservingRanges`/removeChild 摘壳面。
2. **混合树对账**：修 handle 壳 childNodes 对 plain 子的可见性或
   contained walker 的混合形态判定。动 walker 共享面，风险更大。

两案都须过 face 六文件 + multitest/misc + selection 三面对跑。

## 哨兵与基线

- 哨兵页 `tests/wpt-runner/wpt-data/editing/run/zz-probe-d10diag.html`：
  document 作用域宿主 + 900 壳部分选区最小复现，当前锁定 re-wrap 形
  （Pass）。桥层修复落地后把断言改为期望形 `<span 900>foo</span>barbaz`，
  届时本页 Fail 即修复信号。
- 基线 face（d9c2，2026-11-13 07:08）：六 format 文件 11868P/61F
  （bold 3036P/12F、italic 2070P/3F、u 2143P/4F、strike 2131P/16F、
  sub 1228P/13F、sup 1260P/13F）。本轮无行为修复：摘净探针后重建，抽查
  bold.html 全页 3036P/12F 与基线恒等。
- 本轮错误修复版 face 对跑（P2F=84）数据仅作负结果记录，不入账面。

## 待办移交

- 桥层专项批：混合树机制修复（d11 补充节两候选择一）+ 支线 bug 清单
  + `zz-probe-d11tree.html` 验证页（doc-scope 腿转绿为修复信号）。
- bold/italic 数值壳残余在桥层修复后预期大幅转绿；届时重对 face 并更新
  本档与哨兵断言。
