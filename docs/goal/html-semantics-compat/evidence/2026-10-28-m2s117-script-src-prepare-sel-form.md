# M2 s117 · 连接态 sel 形 SCRIPT src 后设 prepare 触发面（026/027 双面）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s116（referrer 族 JS checker 面）

## 刀位与判据链

execution-timing/026「scheduler: doc write added script, .src set later」1F：doc.write
写空 `<script></script>` → 后设 `.src` → 期望外链执行（assert_any 双序）。入场 026 F、
048/051/052 等同组 Pass。判据链：corpus 页 7 轮探针插桩（断言面 Fail 行带出运行时值，
备份 /tmp、验后手动还原）定谳元素链与触发链断点。

探针定谳链（026）：
1. doc.write 空脚本落位正确——body 第 4 子、sel 形（`__zwSelector` =
   `html:nth-child(1) > body:nth-child(2) > script:nth-child(4)`、`__zwHandle` null）、
   `_zwScriptStarted[sel]` false（解析期空源 bail 不盖章）、src 属性 `.src=`/setAttribute
   两形态均落位（`srcAttr:scripts/include-1.js`）。
2. `_zwPrepareSrcScript` 从未被调（spy `__probePrep=0`）——prepare 触发面缺口。
3. MutationObserver 记录管道本身通（ctl=1 对照组 Pass；异步 microtask 投递，同步读恒 0
   为探针设计坑）。

静态定谳三缺口：
- **connected 假阴性（真拦点）**：`_zwScriptPrepConnected`（part03）直信
  `el.isConnected === false` 快速返回；sel 形 isConnected getter 走宿主树快照查询
  （`__zw_contains('html', sel)`，part04:2665），执行期 doc.write 插入只在 shim 侧
  mut-tree、宿主快照无此元素 → 假阴性（探针 `conn:false` 而 `pnode:1` 父链达文档）。
- **IDL setter 门形排除**（历史误判）：part05 s96 SCRIPT.src 臂触发条件 `handle &&`
  ——注解理由「parser-inserted post-connection 首步即 return」对 doc.write 空脚本
  不成立（parser prepare 因空源 bail 未置 already-started，post-connection 无章可查）。
- **mut-tree setAttribute 绕流**（gap 2，挂账）：part02 `_zwParseEl.prototype.
  setAttribute` 走 `_ensureMutTree().setAttribute`，不投递 `_mo_notify` script 臂——
  setAttribute 形 src 后设仍不触发（探针实测 `__probePrep=0`）；026 原页用 `.src=`
  IDL 形不受此缺口影响。

## 027 对偶定谳（P→F 回归的根因）

首刀直扩 s96 sel 形臂调 `_zwScriptMaybeStartDynamic`（s109 全门链）→ 026 F→P 但
**027 真翻红**（P→F，多出 `external script #1`）。027 = 姊妹页：doc.write **带内容**
script（写流面已执行内联体）→ 后设 `.src` → already-started 门应拦未拦。探针二轮：

- `scripts[4]` 即 doc.write 脚本（`txt:log('doc.write script')` 实证；预处理后索引
  与上游对齐，非错位）。
- 写流面（part06 `__zwDocWriteRunFace`）执行内联体的对象是**回放重建 plain
  wrapper**：`__zwSelector`/`__zwHandle` 全空（body 子元素探针全 `nosel`）——
  already-started 章落不了（`_zwScriptStarted[sel]` 键不存在）。
- 换代重注册后 src-set 读到的是**新 proxy**（sel `nth-child(4)`），与执行时 wrapper
  身份断链 → sel 盖章路线不可行（补盖章后 027 仍红实证，回滚）。
- 架构约束口径：**非空体 sel 形脚本必已随投递（delivery 盖章）/写流（回放执行）
  跑过**，src/href 属性后写按 already-started 语义跳过；空体脚本从未执行，照常触发。
- 附带定谳：IDL setter 写属性 → `__zw_set_attr` 记录 → part01 `_mo_notify` s109
  属性臂是 sel 形 IDL set 的实际触发点（part05 直呼臂冗余）；baseline 里 027 不翻红
  是因旧 connected 假阴性同时挡住了属性臂——s117 修 connected 后须在属性臂补体空性门，
  否则 027 沿属性臂绕过 s96 门直达 prepare。

## 施工（净变更三文件）

1. **part03 `_zwScriptPrepConnected`**：`isConnected === true` 捷径保留；`false` 不再
   直信——先 parentNode 爬升复核（可达宿主连接祖先即连接；nodeType 9/11 终判与原
   while 体一致，抽 `_walkSc` 复用两遍），爬升不可得再落身份反链
   （`_zwMutationInDoc`，undefined 路径原序不变）；detached（remove 后 parentNode
   null、反链失含）仍 false。单调用点（`_zwScriptMaybeStartDynamic` connected 门）。
2. **part01 `_mo_notify` s109 属性臂**：src/href/xlink:href SCRIPT 臂加 sel 形非空体
   跳过门（`el.text` trim 非空 → 不触发；handle 形 run-once 表完整不设门；
   childList 臂不动——145 形 textContent= 面依赖非空体照常执行）。
   FIXME(architecture)：写流执行身份盖章断链，增量解析落地后应回归 already-started
   表判。
3. **part05 s96 SCRIPT.src 臂**：触发代码回原形（handle 形直呼 `_zwPrepareSrcScript`
   不变；sel 形实走属性臂），注解修正——记录 s117 双面机制与旧注解的错误理由。

## 验证

- 新增 bin pin ×2（testharness.rs，temp root 外链 + `__log` 序断言）：
  `script_src_set_after_docwrite_empty_s117`（026 面：空脚本 src 后设须执行外链）、
  `script_src_set_after_docwrite_content_no_refire_s117`（027 对偶：带内容 src 后设
  不重执行）。旧 shim stash 往返：empty pin **红**（修复前）→ **绿**（修复后），
  content pin 两态绿（旧行为本就不重触发）。
- 定向 `FILTER=execution-timing/0`（69 行面）：**026 F→P**；027/048/051/052 维持 P；
  归一 diff 唯一变化 = 026 翻绿，零其他翻转。
- 全量 A/B（/tmp/s116-full-suite.log → /tmp/s117-full-suite.log，全行归一 diff）：
  **6606P+1192F+105T → 6607P+1191F+105T（+1P −1F ±0T）**；diff 唯一变化行 =
  026 F→P，其余状态行零变化 → **真翻红 = 0**（同页混合状态子测试的名字级交集
  会伪报大批翻红，全行 diff 为权威判据——工具坑记录）。
- shim 拼接语法 OK（真实装配序 part01+01b+02..07，`node --check`）；fmt 过
  （testharness.rs）；guard 内 `cargo test -p zero-wpt-runner --bin zero-wpt-runner`
  pin 腿 2/2。

## 挂账（本切片新增/更新）

- **写流执行身份盖章断链**（FIXME(architecture)）：写流面执行的内联脚本无法落
  already-started 章，靠体空性门近似；增量解析落地后回归表判。
- **part02 mut-tree setAttribute 绕过 `_mo_notify` script 臂**（gap 2 承袭明确化）：
  setAttribute 形 src 后设（连接态 sel 形）不触发 prepare——026 探针实证；修法须
  mut-tree 属性写路径补 script 臂投递，涉共享路径单列。
- **execution-timing 构成面**（承袭 s117 探针轮发现，单根）：056/063 F+T 与
  051/052 的 P 同根——eager-parse 全树可见性 vs report 元素移除的构成差
  （未执行 body 脚本 +1 入列），两页组无法同修，须增量解析。
- **css-module 族全脸 corpus 门**（待决须用户拍板，承袭 s116 确认）：resources/
  无 .css fixtures，redirect/charset/dynamic/repeated-import 全族受阻。
- 承袭：www2 可达性、modulepreload referrerpolicy 族、module-import-referrer TLA、
  script-crossorigin-network cookies、redirect-relative-urls 动态重定向基、export-from
  with FIXME、script-charset-02/03、143 write、defer 多写完成序、063、dynamic
  import() with 透传、classic 外链 SRI。

## 遗留近似

- 体空性门的边界：脚本执行后**清空体**再设 src 的形态（spec 不重执行、本架构会
  触发）语料无此形；type/nomodule 阻挡解除后再设 src 的形态由 helper 门承接。
- connected 爬升复核对 mut-tree 深嵌套（写流产物套写流产物）中间层仍可能假阴性
  （中间层 isConnected false 直返 false）——语料深度 ≤1（doc.write 直写 body）。
