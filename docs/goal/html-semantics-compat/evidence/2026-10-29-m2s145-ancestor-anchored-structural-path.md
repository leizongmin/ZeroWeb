# M2 s145 · 刀 D 祖先锚定结构路径——selector 键跨代碰撞根除（requestsubmit 20 键全翻 + 零 P→F）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s144（刀 C re-entrant submission hang 收口）

## 入场定刀位

s144 下一步候选①：刀 D selector 键元素状态跨代泄漏面勘根（16+4 键，
s144 探针链 T1/T5/T6 已单根定谳：parse 产物 sel-identity 按碰撞性
selector 字符串键存状态，跨 populateForm 代际泄漏）。

## 勘根（本轮下钻的三层事实）

1. **碰撞不止脏值 store，是元素身份本身**：`_elKey`（part03.js:10995）
   对无 handle 元素落裸 selector 串；`_proxyCache`（part01.js:63，P1b
   S2）按 `_elKey` 缓存 proxy（`querySelector('#t') === querySelector('#t')`
   语义钉）→ 同串两代元素**共享同一 proxy 对象**——listener store、
   expando、`===` 全部跨代串。s144 账本的「双通道」实为三重：`_inputValues`
   /`_inputValuesSet`（T1）、宿主 lw pending 属性队列（T5，`__zw_get_attr_lw`
   扫 selector-keyed 共享 mutation 队列）、listener store（runTest
   submitter click listener 跨代累积，旧代闭包经共享 proxy 写共享键）。
2. **碰撞形态收窄**：s142 已给 form[target] 发属性锚（populateForm 的
   frameName 计数器逐代唯一）→ form/iframe 不碰撞。碰撞面 = form 内
   歧义控件的结构路径：`html > body > form:nth-child(2) > input:nth-child(1)`
   ——afterbegin 连环插入使每代新 form 恰落 `:nth-child(2)`，各代
   issue 时刻算出的路径串**字节相同**。绝对位置路径只在单一时刻唯一。
3. **修复方案取舍**：①视图戳 epoch 失效（`__zw_dom_view_stamp` 现成）
   ——过度失效：read-after-unrelated-insert 断 `.value=` read-back，
   位置键写路径不可用读宿主兜底（`SetFormValue` 在 applied view no-op，
   宿主树不存值）；②R100 `persistent_nodes` 持久 NodeId 见证——需新
   宿主绑定 + slotmap 复用 removal 失效（apply 核心手术），注册制架构
   面整项；③**祖先锚定结构路径**——锚（id/class/name/target）随节点
   移动，锚内 `:nth-child` 段只在锚子树内定位，锚子树无插入则路径恒
   解析回原节点；零新增基建，单函数收口。取③。
4. **裸 tag 锚排除**：`body` 唯一但携带节点身份为零——对 afterbegin
   位移与 `html > body` 同层暴露，排除后无锚链保持旧 root 形态，形态
   变化面收敛到真锚子树（本目录全部门靶元素有 form[target] 锚）。

## 施工（js_dom_bridge.rs 3 处 + part39.rs 2 处）

- `unique_attr_anchor` 提取（s142 name/target 锚块成函数，self/祖先
  两用）；`unique_selector_for_node` 重排为纯重构（语义不变）。
- `ancestor_anchor_selector` 新增：`#id`/`tag.class`（唯一匹配门，裸
  tag 排除）+ name/target 锚。
- `structural_path_selector` 祖先锚定：自 node 上行最近身份锚祖先落
  根（`anchor > tag:nth-child(pos) > …`），无锚链回落旧 root 形态；
  fastpath 不认领锚定首段（`split_once(":nth-child(")` 失败）安全回落
  通用引擎（锚定路径天然短，解析税可忽略，fastpath 扩展暂缓——
  OPTIMIZATION 注释在位）。
- part39.rs：t8b 主钉断言域收窄（fastpath 仅对其认领形态断言，引擎
  命中恒断言 + 认领覆盖 ≥6 下限）；新增刀 D 回归钉
  `structural_path_anchored_distinct_across_afterbegin_generations`
  （两代 populateForm 形态：逐代互异 + 锚定落根 + 各自恒解析回本代
  节点，InsertAdjacentHtml 单条保序）。

## 验证

- 宿主 structural-path 单测 **6/6**（含新钉；t8b 其余 3 钉不动全绿
  ——无锚链 root 形态保持）。
- 定向（form-submission-0 全目录，`make testharness-html-semantics
  FILTER=form-submission-0`）：**277P/63F/1T** vs s144 目录
  256P/83F/1T——requestsubmit **20F→0F（36P 全 Pass）**，preventdefault
  5P 保持，其余文件零位移（form-submit-action 40F/urlencoded2 6F/
  text-plain 6F/multipart 6F/submission-checks 3F/input-type-change 2F
  = 已知基线面），唯一 Timeout = 已知 t10（algorithm，iframe.src 异
  步装载挂账）。
- **全量 join**（对 s144 最终树 /tmp/zw-s144-full4.log）：s145
  **7069P/852F/98T**（8019 行）vs 基线 7010P/872F/99T（7981 对）。
  (key,state) 状态集 join：**状态位移恰 20 键全 Fail→Pass、零 P→F、
  全落 form-double-submit-requestsubmit.html**（runTest 4 + runTest2 16
  ——刀 D 靶面精确闭合）。key-diff 40 = select-event 39（s144 基线
  文件级 Timeout flake 负偏差恢复，−38P/+39P + Timeout −1 归因闭合）
  + algorithm 1（文件级 Timeout 下键集报法漂移，t10 同面）。行数守
  恒：7981 + 39 − 1 = 8019 精确闭合。
- 门禁：clippy zero-engine --all-targets -D warnings 零告警；fmt 本轮
  编辑区（js_dom_bridge.rs 618-720/part39）零波及，7 处 diff stash 对
  照 HEAD 同现（4248-4559 net-api 区预存漂移，归口记录）；engine lib
  **3592/10F**（9 名单 = 8 已知 + R3045 归口 net-api 不变；第 10 =
  details 在册间歇）；runner bin **258/1F**（trend_csv_check 行 196，
  并行流 f62bf89aa 归口不变）。shim 未动（无 node --check 项）；验
  证集对齐 s142 同面先例（定向 + 全量 join + lib + runner bin，无
  reftest 义务）。

## details 族 flake 强化观察（非本修引入，实验定谳）

test_details_name_exclusivity_template_deep_entry 单测隔离跑翻转
（`2:1,0` 期望得 `2:0,0`，树序第一 open 被误关）。归因实验：祖先锚
定**短路（全关）与开启各 10 次采样，失败率相同（4F/6P 两臂）**——与
本轮改动零相关；单测隔离跑仍翻转 = 沙箱内异步调度非确定性（R5254
resolver 时序邻域，刀 B 在册）。s144 曾同测现「pre-merge 现合并树
绿」间歇；本轮观测到更高翻转率（≈40%），环境负载敏感面，复现归口
刀 B 勘根时一并收口。

## 挂账（本切片更新）

- **刀 D 收口**（本轮主刀，20 键全翻）。残余理论面：**无身份锚祖先
  链**（多层匿名 div 包裹 + afterbegin 连环插入）仍有跨代碰撞可能
  ——本目录语料已无此形态；真节点身份注册制（持久 handle 化，R100
  persistent_nodes + removal 失效）保留为架构远期项，影响面评估挂
  下步候选⑤。
- 承袭不变：algorithm t10 iframe.src 异步装载（候选③）；accept-charset
  w1252 + `&#NNNN;` 实体回退 + lone surrogate USV 18 案（候选②）；
  刀 B runner 多跑 + 首遍挂（含本轮 details flake 强化观察）；C2
  input-type-change 2F；engine lib 9F 环境面；runner trend_csv_check
  行 196（归口 encoding 流）；「真实再物化」观察项；待决（css/text
  语料资源、green-2x2.png、parser form pointer 缓行）。

## 下一步（s146 候选）

①accept-charset w1252 + 实体回退 + lone surrogate USV 18 案（面单在
案，urlencoded2 6F 同根）；②algorithm t10 iframe.src 异步装载；③刀 B
runner 多跑机制勘根（含 details flake 强化观察）；④无锚链碰撞理论面
影响面盘点（刀 D 残余）。
