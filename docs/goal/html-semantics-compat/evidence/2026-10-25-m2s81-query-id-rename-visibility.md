# M2 切片 81（s81）——查询面 id 改名同 turn 可见性：双臂补偿 + IDL 反射读 lw 化（R125 读侧查询半边）

日期：2026-10-25。对象：s80 收口时控制面挂账的
`querySelector('[id=…]')` 同 turn 改名可见性（共享查询视图面）。
R125（s79）读侧只接 getElementById；查询面（querySelector /
querySelectorAll / matches / closest）同 turn `setAttribute('id',…)` /
`.id=` 改名后 `#new` miss、`#old` stale hit。

## 1. 根因补刀：IDL `el.id` 读与 getAttribute 同 turn 分歧

仪器化 probe（临时文件，跑完即删）分轮收敛：`[id="new1"]` 属性形态
拉真通过、`#new1` token 形态失败——分歧在 `_matchCompoundOf` 的 ids
校验读 `_hIdOf(p)`＝`p.id`（IDL 反射）。读源追踪：

- `getAttribute`（part04 get trap）走 `__zw_get_attr_lw`（R2995
  latest-wins 链，consult 变更列表）——同 turn 改名可见；
- `el.id`（part03 get trap id 臂）走 `__zw_get_attr` 纯快照——恒
  stale。

spec 上 IDL 反射读内容属性，`setAttribute` 同步生效（DOM standard
「set an attribute value」即改 attr）——快照读是 R2993/R2995 同类
stale gap。共享路径修一处（part03 id 臂 lw 化，`_zwRawIds` 孤立代理
原值优先保持、handle 形不走 sel 变更列表保持），查询匹配读
（`_hIdOf` → `_matchCompoundOf`）与属性读同源对齐，全部消费面
（R3902b/R309/R322 既有匹配臂在内）自动修正。

## 2. 查询面双臂补偿（part05 helpers + part06/part04 六挂点）

host 查询视图 `with_query_view_doc` 只应用结构级 mutation（R3190
「查询反映 pending 前」属性语义锚定）——查询面补偿只动**选择器 id
约束**（`#id` token 与 `[id]`/`[id=v]`），其余属性约束语义不变：

- part05 新增：`_zwSelIdConstraint`（最终复合段 id 约束提取；轻量
  预门无 `#`/`[` 直返 null；`[id~=…]` 等无 lw 单值近似形态返 null 保
  host 行为）、`_zwIdCurOfSel`/`_zwIdConViolated`（lw 现值复验）、
  `_zwIdOverrideCandidates`（R125 表现持目标 id → `_proxyCache`
  反查）、`_zwElInSubtreeSel`（子树门，48 跳封顶同 R125 口径）、
  `_zwDocPosCmp`（文档序比较，异根/不可得返 0 保插入序）。
- 双臂语义（gEBI R125 ①②臂同构）：
  - **stale 剔除**（全形态，含 complex 最终段）：host 命中元素 lw
    现 id 不满足约束 → 未命中；
  - **单 compound 拉真**：R125 表候选 + `_zwMutationInDoc` 门 +
    full compound 复验（tag/class 共面）+ 子树门（元素级）；qSA 归
    并按文档序插入、身份/elKey 双重去重。
- 挂点：part06 文档 qS（主命中 + pendingRemoved 复扫两处）、qSA；
  part04 元素 qS、qSA（三 return 路径同一 `_s81mrg` 归并闭包）、
  matches（host true 被约束否决 + 单 compound 拉真）、closest（单
  compound 时 `__zw_parent` 链上行逐祖先复验，spec 祖先序不变；
  非约束形态回落 host `__zw_closest` 不变）。

## 3. 验证

- 细粒度 probe 12 面（D0/A1-A5/B/C/D/E/F/H，临时文件已删）：全
  Pass。probe 轮次记录：轮 4 仅属性形态通过 → 定位 `el.id` 读源分
  歧（§1）→ 共享路径修复后轮 6 全绿。
- zero-engine lib **3521P/0F**（新增契约
  `query_id_rename_same_turn_visibility_s81` 17 断言：qS/qSA 文档
  面、IDL 改名对称、子树 + tag 复合段、matches 双向、closest
  self、改名回原 id 毒化面；s79 测试处指针注释同步）。
- 域（make 目标 + FILTER=embedded-content）：**976P/252F/30T——
  与 s80 基线逐位持平，REG 0 / GAIN 0**（s80 账本预判无直接语料
  F 面依赖此切片，实证）。
- interactive pin（FILTER=interactive-elements）：**439P/44F/0T 逐位
  持平**。
- fmt clean。

## 4. 范围与未解决项

- **complex 祖先段约束 pull-in 不接**（`_zwSelIdConstraint.single`
  门）：`#pa div#new8` 拉真未接（祖先段约束需 host 复验）；stale
  剔除全形态生效（含 complex 最终段）。
- **同 turn insert 元素上行盲区**（与 id 改名无关的结构性身份缺
  口）：同 turn `createElement`+`appendChild` 的元素 `__zwSelector`
  为 null（批末 apply 才有身份），从其上行的 closest/query 全盲——
  无 id 对照面 `inner.closest('body')` 同样 null 实证。后续独立切片。
- handle 形元素不进 R125 表（写侧 `!handle` 门）——handle attr 写
  host 即时域，无此改名面，读侧不涉及。
- 控制面既列遗留不变：720-stretch 22F（归 layout 集成面）、
  sizes-auto→F5（用户门）、empty-picture 请求对象模型（issue
  10531）、sizes 数学函数 FIXME、parser form element pointer 4F
  （用户决策缓行）。
