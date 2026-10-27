# M2 s120 · fragment 解析 script 克隆 already-started 继承（章传播 + 插入臂章门）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s119（FORM 索引命名属性臂）

## 刀位与判据链

s118 的 FIXME(spec-gap)「cloneNode 重解析产物同属 fragment parsing
already-started 面待语料需求补章」在语料 `scripting-1/the-script-element/
svg-script-fragment-parsed-not-executed-on-clone.html`（3F）坐实需求：innerHTML
产物 script 浅克隆（test2/4）与含 script 子树深克隆（test3）再插入 body 均执行
（spec cloning steps：already started 与源一致）。同轮勘察：lazy-loading 族 26F
（viewport 可见性/滚动派发/延期取数基建，挂大基建）、dialog 43F 多根、meta
color-scheme 21F 刀位在 style-system（渲染流，rule 9 让位）——本页为唯一单根
常规刀位。

## 施工（净变更两 shim 文件 + pin）

探针页四臂（断言消息带运行时值，新建 gitignored corpus 文件验后删除）定谳两
处断链，修正原三臂计划：

1. **part03 `_wcRebuildAsHandle` 章随物化传递**（R5197 序列化印章传递同款）——
   t8r 物化把 s118 打章的 `_zwMEl` 重建为**新建 wrapper** 存 registry，章丢
   （探针实证查询产物 `src.flag=undefined`）。
2. **part04 R387 直插臂 + s97 insertBefore 镜像臂章门**——两同步执行臂先于
   part01 notify s111 臂的章门消费，带章 handle 形脚本插入照执行（探针实证
   `p2=true`）；门读 `child.__zwFragNoExec === true`。
3. **part04 s95 克隆块源章检查**（章继承 run-once 表 `_zwRanScripts[nh]`）+ **deep
   分支重解析产物按源章 gate 整体打章**（`_handleChildren[handle]` 源子树含章才
   打，合法待执行脚本不扩 already-started 面）。
4. **part03 `_zwDeepCloneEl` 章传播**（plain 元素克隆臂，递归子层继承）。

**首刀教训（两轮探针）**：①s95 块初版读 `n.__zwFragNoExec`——`n` 是 proxy
内部 target，不持章（章经 proxy set/get 存取）；改 `_makeProxy(sel,
handle)` 重读后两面全通。②detached 树 deep 克隆走 `_cclone` 递归臂（host
innerHTML 空 → 重解析分支 `_cnd` 不触发），逐子 cloneNode 复用 s95 块——
s95 一处修正同时覆盖浅克隆与深克隆两面。③`tbl=true` 歧义：执行器跑完也盖
run-once 表，须 append 前采样（`tblBefore`）才能定谳章继承。

## 验证

- 探针四臂语义全对：源章在场、插入臂章门消费（p2 undefined）、克隆 tblBefore=
  true 章继承（p3/p4 undefined 未执行）。
- 语料页 **3F→P**（4/4 全绿，含 sanity）。
- bin pin `script_fragment_clone_not_executed_s120`（浅克隆 + 深克隆 + 合法动态
  control 三面）：新 shim **绿**；旧 shim stash 往返**红**（行为失败：
  `浅克隆 fragment SVG script 不得执行: clone runs`，非编译失败）。
- 定向 `FILTER=execution-timing/0`：78 行全行归一 diff vs s119 **零变化**
  （031/028/033/079 维持 P）。
- **全量 A/B**（/tmp/s119-full-suite.log → /tmp/s120-full.log，全行归一 diff）：
  6612P+1186F+105T → 6602P+1184F+107T（原始计数）；diff 目标面 =
  **svg-script-fragment-parsed-not-executed-on-clone 3F→P 精确三行对**；另三处
  漂移逐一归因非翻转：①focus-dynamic-type-change-on-blur 12P+1T（负载型页级
  超时，过滤重跑 **80P 零超时**）②src-then-lazy-load 1P→1T（同型，重跑 **8P**
  ）③modulepreload-cross-origin-referrerpolicy 基线页挂起（2F+页级 T）本轮跑
  完（3F）——同 modulepreload referrerpolicy 挂账族行形漂移 → **真翻红 = 0**
  （8-job 并发负载 flake 判据：过滤单页重跑即复绿）。
- engine lib **3570/0**（全量套件后串行跑）；guard 内 clippy（runner
  --all-targets + engine）零告警；fmt 过；shim 拼接语法过（真实装配序
  part01+01b+02..07，`node --check`）。

## 挂账（本切片新增/更新）

- **FIXME(spec-gap) 收窄**：cloneNode 克隆件传播面已补（本切片）；余
  insertAdjacentHTML 产物与 handle/sel 形源 deep 重解析臂（part04 trap deep
  分支 `_zwFragmentAdded` 产物）待语料需求——本切片已顺手按源章 gate 补了
  deep 分支打章，剩余面仅 iah 产物。
- 承袭：写流执行身份盖章断链 FIXME、part02 setAttribute 绕 notify 臂（gap 2）、
  execution-timing 构成面 056/063+051/052 单根（增量解析大基建）、041 nested
  write（写流队列重构）、lazy-loading 族 26F（viewport 基建大项）、dialog 43F
  多根、meta color-scheme 21F（跨流 style-system）、css-module 族 corpus 门
  （待决须用户拍板）、www2 可达性、modulepreload referrerpolicy 族、
  module-import-referrer TLA、script-crossorigin-network cookies、
  redirect-relative-urls、export-from with FIXME、script-charset-02/03、
  143 write、defer 多写完成序、dynamic import() with 透传、classic 外链 SRI、
  接口对象基建面、FIXME 组、/tmp/zw-hb*.txt；**待决（须用户拍板）**：css/text
  数据模块语料资源缺失、parser form element pointer 4F 缓行（2026-10-23）。

## 遗留近似

- deep 分支重解析打章是**源子树含章 → 全部重解析 SCRIPT 打章**的存在量词
  gate：源子树同时含带章与合法待执行脚本时后者克隆被误惰（语料无此形态；
  精确按位对齐须并行走源/产物双树，复杂度高不作）。
- 章传播不落 host：克隆件已 started 语义全在 JS 侧（run-once 表 + 对象属性），
  host 侧无对应状态——移动换代链路已在 R368/s95/t8r 三盖章点覆盖，跨代
  （`__zw_reset_pending_state`）不清表（`_zwRanScripts` 惰性全局，同 s95 语义）。
