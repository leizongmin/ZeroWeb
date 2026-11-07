# M3 清单 1——attachment deep JSON 格式修复 A/B 收口（2026-11-07）

## 根因与编辑面

机制一真根因 = Batch 2 引入的 `deep_entry_json_unsafe` sr 字段拼装错位：format
串把带前导逗号的 `,"sr":{...}` 拼在条目对象**闭括号外**（`...]}{}`），产出非
法 JSON → shim `JSON.parse` 抛错静默回退 legacy 未物化臂 → **视图侧物化自
Batch 2 起从未生效**（gethtml 通道通过属 triage「模板滞留 + attachShadow 重建
+ 序列化遮蔽」虚构链）。修复 = sr_field 移入闭括号内。

实际编辑面（认领预告的 dom/ 零改动修正——contents 枚举盲区属机制面必需）：

1. `crates/engine/src/js_dom_bridge.rs`：format 串修复 +
   `parse_html_subtree_deep_json_unsafe` 加 `context_local` 参数（HTML ns 有
   context 时走真实 context 物化片段解析，消「解析丢弃父 → 模板重挂 body ∈
   名单误物化」机制二视图臂）。
2. `crates/engine/src/js_dom_bridge/callbacks.rs`：`__zw_parse_html_subtree_unsafe`
   注册加 arg[4] ctx_tag 透传。
3. `crates/engine/src/js_dom_shim/part03.js`：`_zwMBuildBodyTree` 第 5 参
   ctxTag 进深臂调用；ShadowRoot.setHTMLUnsafe 保持缺省（引擎 ShadowRoot 写
   body 兜底同态）。
4. `crates/engine/src/js_dom_shim/part05.js`：`_zwFragmentAddedUnsafe` 取宿主
   local name（shadow 容器 handle 除外）透传。
5. `crates/dom/src/document/shadow.rs`：物化候选枚举自
   `get_elements_by_tag_name`（只走 children 边）改私有遍历
   `collect_declarative_template_candidates`——下探 template contents（嵌套
   声明族 + 外层 template 包裹声明式模板案的枚举盲区）；shadow 树内层不枚举
   （声明式 attach 仅解析期生效，spec 行为）。跳过路径 s173 值还原随之覆盖
   contents 内层。
6. 单测：engine 3（sr 字段对象内嵌 + 合法 JSON / 无根无键 / context 片段路
   径同态）；dom 3（contents 内物化 / 嵌套穿过 contents / contents 内跳过模
   板值还原小写）。

## A/B 门（WC ×1 全语料逐子测试身份比对，同机同参守卫）

- 基线 `.acceptance/shadow-m3-residual/wc-m3c2-before.json`（365 文件
  8210P/1581F/8T，与 r5433-final 共享文件逐子测试零漂移）。
- after `wc-after-fmtfix.json`：365 文件 **8530P/1460F**/8T。
- **P→F = 0、T→F = 0、vanished = 0；F→P = 121；after_only = 199 全 Pass**
  （gethtml.html——修复前 shim JSON 异常使该文件 harness 提前中断截断，现完
  整跑通，非口径变化）。认领门全过。
- solo：attachment 484P/170F → **597P/57F**（+113P）；basic.html 0P/11F →
  **16P/6F**（delegates focus / clonable / 空链案转绿）。

## 剩余 57F 分族（本刀修复暴露的下层，前后同红）

| 族 | 案数 | 断言面 | 机制 |
|---|---|---|---|
| A | 34 | `attachShadow` 同模式重挂返回同根但**未清空** children（line 53） | attach 恢复语义缺「重挂清空」步 |
| B | 17 | attach 恢复后第三次 attach **未抛** NotSupportedError（line 57/61） | attach 状态机缺一次性恢复计数 |
| C | 6 | canvas disallowed 无描述 assert_true（line 68/71/75 之一） | 前后同消息，待复现甄别 |
| D | 2 | basic 两案 `assignedNodes()` 视图面（静态=[]、fragment=裸对象；assignedSlot 双双已过） | shim slot 视图 |
| E | 4 | basic contents 族 `template.content.cloneNode(true)` 后 innerDiv.shadowRoot null | shim contents 视图 unmaterialized（`parse_template_contents_deep_json` opt-in）+ 克隆携根面 |

- 45 号 adopted 留池 9 文件 23F+5P 仍为 adopted 机制面（rendering-compat 归
  口，清单 2 paperwork，非本刀）。

## 门禁

`git diff --check` / `cargo fmt`（-p zero-dom -p zero-engine）干净；
clippy `-p zero-dom`/`-p zero-engine` `-D warnings` 干净；
`cargo test -p zero-dom --lib` 892P/0F（全绿，含新增 3 案）；
`cargo test -p zero-engine --lib` 3790P/**9F**——**零新增红**（stash 往返
对拍：9F 全部先在；契约清单 3 的「既有 5F」口径过期，现树实际先在集合 =
R2809/R2810/R3276/p56/p99s + test_fetch_filtered_response_type_matrix +
test_iframe_service_worker_messages_dispatch_to_iframe_container +
test_t8k_websocket_error_nonterminal + test_t8k_websocket_event_flow_open_message_close）。

## MEl 两刀回装评估输入

Batch 3 撤销判据「flavor 走 part04 trap 重建路径不经 MEl 克隆」在本刀修复后
需重估：族 E 案面 `template.content.cloneNode(true)` 的 contents 子树视图由
`parse_template_contents_deep_json` → `_zwMBuildDeepTreeNode` 建树（MEl
flavor），撤销的 `_zwDeepCloneEl` 克隆臂（A2）正是该面候选回装对象；回装与
否随族 E 实施刀定，不悬置。
