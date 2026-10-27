# s111：原子插入语义——script prepare 连接门 + 记账后触发 + 子树下降

日期：2026-10-27。目标域：`scripting-1` execution-timing 147/148（fragment 内多 inline
script，首脚本读/移/删同批后继兄弟）。

## 根因（探针定谳）

语料 147/148 形态：`container.appendChild(frag)`，fragment 内 3 个工厂 inline script，
#1 执行时 `getElementById` 同批后继兄弟（147 移动 / 148 删除）。修前报
`Cannot read properties of null (reading 'parentNode')`。

四轮探针（临时插桩 `_mo_notify` 记账段 / `_zwPAIdAdd` / getElementById 单候选分支）逐层定位：

1. 执行序正确（s1 先于 s2），s2 同 turn 已连接（`conn2=true`），`querySelector('#id')`
   同 turn 可命中—— getElementById/[id=] 形 miss 的差异在 in-doc 判定的数据源。
2. `_zwNodeParent` 反链在 s1 执行时刻仍是 fragment 内旧链（`par=nolink` → `inDoc=false`），
   turn 结束后才指向 `#container`（`inDoc=true`）。
3. npReg trace 实锤：**s1 在 `frag.appendChild(s1)` 时就执行了**（gEBI trace 夹在
   fragment 内部追加记录与 s2 追加记录之间），早于容器 flatten 记录——R387 直插臂无
   连接门（s96 注释明言「连接态判定归调用方」，调用方从未补），detached fragment 里
   eval + 置 run-once，容器插入时被记账挡掉。

## spec 模型（/tmp/spec-scripting.html 提取）

- post-connection steps 只在插入元素**仍连接**时运行——早插入脚本移除/删除后继脚本时，
  后继的 post-connection 不再跑、不 prepare（HTML spec 自带示例与 148 同构）。
- 「atomically inserting nodes」：批插入先完成全部 generic insert（无可见副作用），
  再按文档序逐 script 跑 post-connection steps。insert 步骤遍历**inclusive descendants**
  （子树里的 script 也 prepare，129 形）。

## 修复（三处 + 一处连带）

1. **part01**：s109 addedNodes script 臂自 notify 头部移至 childList 链接记账之后
   （`_zwNodeParent` 登记完再触发 prepare）——批内前置脚本执行时后继兄弟反链已指向
   真实插入目标，`_zwMutationInDoc` 上行成立、getElementById 命中。同处新增非 SCRIPT
   元素的**子树下降**（文档序递归，inclusive descendants；SCRIPT 子是文本不再下降）。
2. **part03**：新增 `_zwScriptPrepConnected` 统一连接判定 helper（属性布尔直用；
   plain/盖章 plain 子属性读 undefined 时回落身份反链 + parentNode 链爬升）；
   `_zwScriptMaybeStartDynamic` connected 门改走 helper。外链分支补
   `__zwHostDynamicScripts` 让位（renderer bootstrap 宿主通路单执行者契约，r387b2
   同语义；让位属 prepare 拒绝不置记账）。
3. **part04**：R387/R387b 三执行臂前置 `_r387conn` 门（`_zwScriptPrepConnected`）——
   fragment/detached 追加期不提前 prepare；未连接不置 run-once（重插入可重走门）。

### 中间态教训（两轮自摆乌龙）

- **链式合并改变分支语义**：R387 原为「独立 if（R377 回落臂）+ if/else 对（内联/外链）」
  三个平铺语句，我把 `!_r387conn` 作为首臂接成单链——A 臂命中后 C 臂（外链）被 else
  吞掉。平铺语句合并成 else-if 链前必须核对各条件是否互斥（A 臂空体与 C 臂 src 在场
  可同时成立）。
- **臂移位的隐能化**：旧位臂先于链接记账，maybe-start 的 connected 门对刚插入节点恒
  bail，外链 prepare 实际全靠 R387b；移位后 notify 臂首次触达插入态外链，而 helper 只
  认 `__zwHostOwnsDynamicScripts` 不认 R387b 的 `__zwHostDynamicScripts` → engine lib
  `r387b2_host_dynamic_scripts_bootstrap_flag_skips_shim_fetch` 抓到双执行（fetchCalls
  "2"≠"1"）。移位使原本「意外失活」的路径生效时，要排查该路径上所有让位契约。

## 验证

- execution-timing 页集合 vs s110 基线（`/tmp/s110-execution-timing2.log`）：147/148/049
  新转绿，117-119/129 首轮回归（子树下降 + helper 修复后）全部回绿，零 P→非P。
- 全 scripting-1：s110 基线 395P/155F/37T → s111 **398P/152F/37T**（+3 行/3 页零
  P→非P；归一 diff 恰为 049/147/148 三页 Fail→Pass，`/tmp/s111-scripting-1b.log` 存档）。
- runner bin 236/0（s109/s110 pin 保持）；engine lib 3548/0（含 r387b2 回归钉）；
  webview lib 814/0。本轮零 .rs 变更（shim 全 JS + 文档），rustfmt/clippy 按实际
  影响跳过。
- 附带修复：049（`Fail → Pass`，fragment flatten + 记账后触发面的顺带收益）。
  033 仅在链合并错误中间态转绿（C 臂被吞的副产物），最终形态与基线一致（仍 Fail）。
