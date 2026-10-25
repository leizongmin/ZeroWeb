# M2 切片 80（s80）——跨批 id 改名 journal：apply 面 selector 失配追链（hash-name-dynamic 5F 收口）

日期：2026-10-25。对象：s79 遗留的 runner 缺陷 C
（image-map-processing-model/hash-name-reference-dynamic.html
3P/5F）。s79 账本 §4 的假说（「疑序列化/apply 管线只经客户端缓存
属性」）本轮仪器化证伪并定位真根因——**apply 面 selector 跨批失配**。

## 1. 分诊链（Rust 侧 attr-trace + resolve 时点探针）

临时仪器（ZW_TD_DEBUG 门，`__zw_set_attr`/`__zw_get_attr_lw`/
`__zw_attr_names` 三回调 eprintln + resolve 脚本身份 dump；跑完即还原）：

- mutations 队列**从不 drain**（vec 1→9 单调递增，8 测试全程可查）；
- 全部写以**创建期 sel** 入队（proxy 闭包 sel 建立时冻结：
  `set_attr sel=#testmap` 8 条贯穿 id 改名前后）；
- apply 后快照 id=other 已落地（批 2 的 `SetAttr{#testmap,id,other}`
  在管线 doc 仍持 `#testmap` 时直查命中），**批 3 起的旧键属性写全部
  失配 skip**——管线 doc 元素已改名 `#other`，旧 `#testmap` 恒 miss。

即 s79 假说反向：写**在**队列里、读链（lw consult 同键）也通，丢在
**apply 解析**——`apply_dom_mutations_full` 的 `find_by_selector` 对
改名后元素按创建期 sel 恒 miss，走 siteopt t4 P19 的 lenient
warn+skip 静默丢写。序列化无 name、视图查询 miss、gEBI 旧键落空均
为该丢写的下游表现。

## 2. 修复：跨批 selector 迁移 journal（R45 的跨批半边）

R45（js-dom M4）已有**批内** id 改名追链（`rewrite_pending_id_selectors`
重写本批剩余队列）。跨批面补 journal：

- `apply_dom_mutations_full` 新参 `sel_renames: Option<&mut HashMap>`：
  SetAttr(id) 落地时记 (改名前 sel → `#新id`)（改名回原 id 不记）；
  SetAttr/RemoveAttr 失配臂 `journal_find_sel_rename` 查表追链（封顶
  4 跳防环）后重试解析。直查命中优先——新元素抢占旧 id 时不劫持
  （journal 只补失配面，不改写命中语义）。
- 持有方 `RenderPipeline::sel_renames`（镜像 `persistent_handle_nodes`
  /`detached_sel_nodes` 的跨 apply 模式）；导航边界清（webview 三处
  nav reset 调 `clear_sel_renames`）。**同页 `render_html` 重建不清**：
  journal 是 selector→selector 映射（非 NodeId），不受 slotmap 换代
  失效约束；runner 的 refresh 重建来自 shim 序列化、改名后 id 已在其
  中，selector 空间延续（首版在重建点清空——语料复跑无效后定位此误，
  修正后单文件 5F→0F）。
- 契约：dom_bridge_tests `test_apply_sel_rename_journal_cross_batch_attr_s80`
  （跨批 name 写落地 / 改名回原 id 直查命中 / 新元素抢占旧 id 直查
  优先不劫持）。注：同批改名后同键 op 落同元素是 R45 既有语义
  （批内重写），不属 journal 面。

## 3. 验证

- zero-engine lib **3515P/0F**（含 s80 新增契约）。
- 域（make 目标 + FILTER=embedded-content）：
  **976P/252F/30T**（s78→s80 基线 971P/257F/30T——**GAIN 5 / REG 0**）；
  hash-name-reference-dynamic **3P/5F → 8P/0F**。
- interactive pin（FILTER=interactive-elements）：**439P/44F/0T 逐位
  持平**（命中测试面未触碰，实证）。
- fmt clean；clippy 无新增告警（video_registry/part90 既有 2 条不在
  本 diff 内）。
- 附带记录：一轮全树跑（无 FILTER，7429 行）因启动漏带过滤参数产出，
  不作域对比基线；其 hash-name 行与修复前一致（5F，journal 重建清空
  版本），与 §2 的首版误判互证。

## 4. 范围与未解决项

- journal 只挂 SetAttr/RemoveAttr 失配臂——SetText/SetStyle 等
  selector 型变体的跨批改名失配仍走 lenient skip（无语料实证面，
  按需扩展）。
- shim 读侧的创建期 sel 冻结（改名后新 proxy 持新 sel 读、旧 proxy
  持旧 sel 写）在语料面已被 apply 修复闭合（快照烘干后双键均可读）；
  同 turn 极端读写序的残余分歧未观测、未处理。
- 控制面既列遗留不变：`querySelector('[id=…]')` 同 turn 改名可见性
  （共享查询视图面）、720-stretch 22F（归 layout 集成面）、
  sizes-auto→F5（用户门）、empty-picture 请求对象模型（issue 10531）、
  sizes 数学函数 FIXME、parser form element pointer 4F（用户决策）。
