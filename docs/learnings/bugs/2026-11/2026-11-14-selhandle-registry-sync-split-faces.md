---
date: 2026-11-14
modules: engine
---

# shim 双族 DOM 的 JS 视图/宿主注册表分裂：一个缺口四张脸，判读先分面再动手

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

## 解决

- 修法按面拆：fragment 双臂补 wire（ref 有 selector/handle 走对应
  insert 变体，否则 append 变体；`_fragmentHandles` 印记子仍走整体
  平展分支）；融合序列化改 null-check（`!= null ? fused : host`），
  生产端空串原样返回；`_wrapNodeEntry` 'E' 臂接 R331 反查
  （`_zwQueryWrapIdentity`，pending 窗口反查 miss 零变化）；QSA
  R3902b 预门扩「无组合器含 '.' 形态」。每处带 spec 锚
  （concept-node-pre-insert/remove、concept-tree-child）。
- 判读先分面：最小复现页按视图布尔字段分组（qsaFinds/htmlGone/
  inChildNodes 分属四张脸），逐字段归因后再决定共享路径改法；禁止
  一把抓的「在插入点补一次全量 wire」式改法——会双计 overlay。
- 槽标记语义跟随事实：noWire 槽标记只在「节点确实不进宿主快照」时
  保留（K3 apply 代际清除豁免、R322 物化豁免按代理返回）；wire 补齐
  后 handle 子是否保留标记按下游消费面判据逐个核（K3 清除判据本就
  不含 handle 节点；R322 豁免臂按代理返回恰为 identity 保持面）。

## 避免方式

- 给双族 shim 加插入路径时，四张脸各过一遍：宿主 wire 发了吗？pending
  桶/overlay 记账了吗？融合序列化会读到吗？drain 后 wrapper 重建 identity
  保持吗？缺一即是本轮四脸之一。
- 「空串是合法结果」的读取面一律 `!= null` 判，不用 `||`；生产端不要
  把空串折叠成 null 交回调用方推断。
- 写批量插入循环先定迭代方向：宿主 wire / push 型记账正序，固定下标
  splice 型倒序正序皆可——同分支两种型混用时以正序为准，可同时满足。

## 关联

- T67 诊断定谳（视图分裂四证据链）与本轮修复同源：bench 运行目录
  evidence/t67-diagnosis.md、evidence/t68-op-result.md。
- 常驻钉：`crates/engine/src/js_dom_bridge_tests/part114_t68_selhandle_registry_sync.rs`
  （五测：caseA 归并 / caseC 空串 / caseDAsync identity / fragment 双臂 wire）。
