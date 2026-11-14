---
date: 2026-11-14
modules: engine
---

# shim 双族 DOM 的 JS 视图/宿主注册表分裂：一个缺口六张脸，判读先分面再动手

## 问题

掘金 feed（Vue 2 + Nuxt SSR 标准挂载）在 zw 永不呈现：框架把 handle 族
（createElement 产物，`__zwHandle` 键）子树挂进 sel-only 解析容器
（querySelector/innerHTML 解析产物，`__zwSelector` 键）后，JS 指针视图
（childNodes/innerHTML/parentNode）已挂，而宿主注册表（渲染引擎文档级
QSA/TreeWalker 的镜像）不同步。自建最小复现页（t67-minrepro 七案例）把
同一缺口暴露为四张互不相同的脸：

1. **wire 缺失面（fragment 视图双臂）**：无 handle 的 fragment 视图
   （createContextualFragment 产物、template.content 派生视图）插入分支
   （insertBefore 的 R97/t8i 臂）只做 JS 侧 registry splice/overlay，
   handle 族子不发宿主 wire——宿主树永久缺席。
2. **序列化空串折叠面**：removeChild 唯一子后桶门激活、融合序列化正确
   产出空串，但生产端 `out !== '' ? out : null` 与消费端
   `fused() || hostRead()` 双重 `||` 折叠把空视图交回宿主 stale 快照。
3. **drain 后 identity 分裂面**：宿主 apply + merge handle→selector 后，
   childNodes 基底重建走裸 `_wrapSelector` 产新 sel proxy，与页面持有的
   handle proxy 身份分裂（indexOf 按对象恒等判 miss）。
4. **查询归并缺形态面**：文档级 QSA 的 pending 归并只有 tag/attr/id
   形态，class 形态（`.cls`）无归并臂——同步窗口恒 miss。

四脸修复后活体掘金仍不落地，取证轮挖出**第五脸（跨批 ref 失效 skip 面）**：
s165 臂（sel 父 + handle-only ref）wire 把 handle 串填进 ref_selector，
apply 侧 InsertBefore arm 的 ref 三查（selector / 批内预解析 / handle 表）
全 miss 后 warn + skip 无降级——该 arm 的同批钉（meta color-scheme）从未
覆盖跨批常态（`refresh_if_html_changed` 全量重建使 handle 表换代清空），
而兄弟变体 `InsertBeforeByHandleHandle` 的约定本就是「miss 降级 append」。
活体铁证：feed 挂载窗口 22 次 `insert_before ref no match, skipped: __n69`，
vue entry-list/卡片挂接整批静默丢弃；JS 视图 20 子稳定是因为 pending 桶
的 handle 子豁免 K3 清除——视图/宿主分裂的最终一环。

修法 5 落地后活体复跑仍不落地（降级 append 40 次全执行但落点为
`global-component-box` 登录组件循环，非卡片挂接），诊断取证挖出**第六脸
（handle miss 硬错毒批面）**：apply 侧 OnHandle 族的 parent/child/attr
miss 全部 `ok_or_else(...)?` 硬错中止整批——跨批 refresh 重建后 handle
表换代清空是活体常态（`unknown parent handle __n232` x5、`unknown handle
__n2224` x25 每次毒化一批），批内后续 sel 域 mutation 全部丢弃（webview
apply Err → 快照不更新 → 渲染丢批）。同族 `SetTextOnHandle` 早有 R100
lenient 口径（warn+skip）与 sel 域 P19 口径（parent/ref 失配 warn+跳过），
handle 族其余 13 站点是口径未补齐而非设计分叉。MO 审计同时铁证 JS 视图
侧另一面：entry-list 挂接当时三视图一致（added realPar=wrap、skeleton 同
毫秒摘除），此后 childNodes 数组静默重置回 [skeleton]（无 MO 记录）而
元素指针链完整（contains=true、122KB 卡片全在）——视图数组重建机制是
第七面候选，本轮未动。

## 根因

- 双族 shim 的可见性补偿分散在四处独立机制（宿主 wire、pending 桶
  overlay、融合序列化、wrapper 反查缓存），每处各有自己的形态白名单和
  门条件；新增一种「族组合 × 插入路径 × 读视图」组合时，四处未必同步
  覆盖。判读时四张脸各自可独立复现、独立修，但根因是同一个：
  「handle 族进入 sel-only 容器」这条路径在四处机制里都不是一等公民。
- 空串折叠是 JS 惯用法 `x || fallback` 在「空串是合法有效值」语义下
  的系统性坑：生产端折叠一次、每个消费端再折叠一次，任何一端漏改都
  复活 stale 面。
- 迭代方向与保序的隐含耦合：同位 splice（固定 index 插入）倒序迭代
  保序，但 push（尾附）倒序迭代**反转**；同 ref 前插的宿主逐子 wire
  必须**正序**。同分支内 appendChild 孪生臂（正序 push）正确、
  insertBefore 带位臂（倒序 push）反转的不对称，说明倒序循环不是
  有意设计而是复写走样。
- 第五脸的根因是「lenient 约定不对称」：同一套 bridge 里 InsertBeforeByHandle
  与 InsertBeforeByHandleHandle 对 ref/parent miss 各自选择了 skip 与降级
  append 两种语义，s185（apply 注释「跨批引用同 handle 视图边界 miss 跳过」）
  把 skip 当设计写进了注释——skip 只在「ref 从未存在」时等价于 append 语义，
  「ref 曾在树中、跨批后失效」的场景语义是丢节点而非丢锚序。
- 第六脸的根因是同族口径未补齐：SetTextOnHandle 的 R100 lenient（warn+skip）
  证明「跨批换代失效不是 bug」已被接受过一次，但同族其余 OnHandle 站点
  （attr/innerHTML/style/append/insert 的 parent/child miss）保留硬错。
  硬错的破坏力不在 miss 本身（该条丢了就丢了）而在**毒批**：整批 Err 使
  批内后续 sel 域 mutation 全部不落宿主、不渲染——一条失效条目的代价被
  放大为整批视图/宿主分裂。

## 解决

- 修法按面拆：fragment 双臂补 wire（ref 有 selector/handle 走对应
  insert 变体，否则 append 变体；`_fragmentHandles` 印记子仍走整体
  平展分支）；融合序列化改 null-check（`!= null ? fused : host`），
  生产端空串原样返回；`_wrapNodeEntry` 'E' 臂接 R331 反查
  （`_zwQueryWrapIdentity`，pending 窗口反查 miss 零变化）；QSA
  R3902b 预门扩「无组合器含 '.' 形态」。每处带 spec 锚
  （concept-node-pre-insert/remove、concept-tree-child）。
- 修法 5（第五脸）：apply 侧 InsertBefore arm ref 三查全 miss 改降级
  append（warn 措辞 `degraded to append` 保可观测），对齐 ByHandleHandle
  lenient 约定；锚序损失接受——ref 不在树中本无锚序可保。
- 修法 6（第六脸）：apply 侧 13 处 handle/fragment miss 站点统一 R100
  SetTextOnHandle 口径（warn+skip 不中止批）——OnHandle 属性/样式/
  innerHTML 臂、append/insert 变体的 parent/child miss、fragment 变体
  parent miss 与 helper 的 fragment miss。判据从「child 悬垂=真 bug」
  改为「同批 wire 失败才是真 bug；跨批换代失效由 JS 视图权威 + warn
  留证兜住」。常驻钉双面：miss 条目 skip（不产出效果）+ 同批后续 sel
  域 mutation 照常落地（不毒批）。
- 判读先分面：最小复现页按视图布尔字段分组（qsaFinds/htmlGone/
  inChildNodes 分属四张脸），逐字段归因后再决定共享路径改法；禁止
  一把抓的「在插入点补一次全量 wire」式改法——会双计 overlay。
- 活体取证教训（第五脸挖出过程）：复现页全绿后活体仍不落地时，①对照
  探针的查询形态要先对齐归并门覆盖面（QSA `*` 通配不归并，inQSA=false
  是探针形态假象）；②zw DOM 对象是 Proxy，包装 `Node.prototype.*`
  采集调用在 zw 臂恒采空（Proxy handler 拦截不走原型链），改走宿主
  bridge 日志（renderer warn）+ 行为面对照。
- 槽标记语义跟随事实：noWire 槽标记只在「节点确实不进宿主快照」时
  保留（K3 apply 代际清除豁免、R322 物化豁免按代理返回）；wire 补齐
  后 handle 子是否保留标记按下游消费面判据逐个核（K3 清除判据本就
  不含 handle 节点；R322 豁免臂按代理返回恰为 identity 保持面）。

## 避免方式

- 给双族 shim 加插入路径时，四张脸各过一遍：宿主 wire 发了吗？pending
  桶/overlay 记账了吗？融合序列化会读到吗？drain 后 wrapper 重建 identity
  保持吗？缺一即是本轮四脸之一；再加第五问：wire 落到 apply 后 ref/parent
  寻址 miss 的行为是什么？同一 bridge 内兄弟变体的 miss 语义要对齐。
  第六问：miss 站点的错误传播是本条 skip 还是整批 `?` 中止？「该条丢失」
  与「毒批放大」要分开定级——活体面高频跨批失效场景一律本条 skip。
- bridge 变体的 lenient 约定（skip vs 降级 append）写代码时按「ref 曾在
  树中」的语义判：降级 append；「ref 从未存在/结构性非法」才 skip。注释
  把 skip 写成设计前先跨批场景推演一遍。
- 「空串是合法结果」的读取面一律 `!= null` 判，不用 `||`；生产端不要
  把空串折叠成 null 交回调用方推断。
- 写批量插入循环先定迭代方向再定移位：宿主 wire / push 型记账正序天然
  保序；固定下标 splice 型倒序虽保序但同分支 push 臂会反转（T68 初版
  缺口），两臂混用只能正序＋插入计数移位（splice 位=锚位+真实插入数，
  假子不计数——DF-1 返修订正：旧文「倒序正序皆可、正序可同时满足」
  前提错误，正序下固定下标 splice 同样反转多子序）。

## 关联

- T67 诊断定谳（视图分裂四证据链）与本轮修复同源：bench 运行目录
  evidence/t67-diagnosis.md、evidence/t68-op-result.md。
- 常驻钉：`crates/engine/src/js_dom_bridge_tests/part114_t68_selhandle_registry_sync.rs`
  （七测：caseA 归并 / caseC 空串 / caseDAsync identity / fragment 双臂 wire /
  s165 跨批 ref 失效降级 append / handle miss 不毒批）。
