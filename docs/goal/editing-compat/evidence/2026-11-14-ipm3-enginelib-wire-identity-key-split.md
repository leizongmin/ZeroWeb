# ip M3-A 门禁批 enginelib 五新红归因 ＋ 线章/身份章分离修复（2026-11-14）

## 背景与批链谱系

M3-A 刀体 266b45602（part01 `_zwCeParagraphMutate` host 边界臂 A 族分支 + part04 R125
child 摘挂 fresh 章发射 + appendChild move-in 臂同类修 + part114 五形钉测）13:25:33 落树后，
门禁批链 ipm3-gate-v1→v6 于 13:54–15:47 运行（写者会话 15:25 rate-limit 终止，批脱离会话
存活完走）：

| 批 | 终态 |
|---|---|
| v1 | L1 ip 绿（F2P=22 全 br）→ ABORT DOMAIN-DRIFT（在飞窗他流 pull 拉入非本域提交） |
| v2 | 脚本 heredoc 引号缺陷 bash 解析崩（脚本缺陷非行为面） |
| v4 | L0+L1 绿 → 发射会话死亡批 bash 被杀；ab 腿孤儿完走 10092/0/0 |
| v5 | L0+L1 salvage 绿 → 运行中脚本被改写风险主动 TERM 止损（quarantined-hijack-editshift） |
| tail-1/2/3 | ab 三次完走（两次 10092/0/0、一次预算沿 1T 伪影）、mt 1559-1711s 完走 judge 四零 |
| v6 | L1 salvage / L2 ab / L3 mt 全绿 → **L4 enginelib ABORT：3944P/12F vs 基线 7** |

v6 终态：ip salvage F 99→77 F2P=22（br=22，A 族恰值预期）；ab 10092/0/0；mt rows 10092
F2P=0/P2F=0/po_fail=0/lost=0 残红恰 2（X37）；enginelib 五新红：

- `named_access_subtree_descendant_unregister_s40`（part38）
- `show_modal_fully_active_gate_r5250`（part79）
- `t68_remove_child_fused_empty_serialization_case_c`（part114_t68，PR #302 新钉）
- `test_media_pause_on_removal_m3b7`（part09）
- `test_media_play_pending_and_removal_abort_m3x14`（part09）

## 归因：A/B bisect 定谳 M3-A 刀域

- 父树 `36de32a4d`（= e3b19fc44 代码态，R5530 仅 docs）全 lib：**3944P/7F，名册恰基线**
  （e1-enginelib-failures-now.txt 七红逐名恒一）。
- HEAD `266b45602`（v6 批域）：3944P/12F（+5）。
- 6c8457a36..266b45602 引窗内 engine 域提交含 t68 PR #302 链、Selection 三笔、B1 切片 1、
  M3-A；父树已含前四者 → **五新红全部归 M3-A**。

统一签名：**移除发生后，既有 JS 代理仍判「在树」**——contains=true（t68）、named access
stale 可解析（s40）、detached showModal 不抛（r5250）、media pause-on-removal tick 不落
（m3b7/m3x14，`_rmStop` 的 `_zwIsRemovedNode(child)` 守卫 bail）。

## 根因：TEMP 探针取证（part115_tmp_m3a_probe，判读后删除）

t68c 场景（`<div id=t68c><p class=oldc>x</p></div>`，`w.removeChild(c)`）修复前现场：

```
__pre_c_sel  = "p.oldc"                     # 缓章是查询原串复合形，非结构路径！
__removed_sels = "#t68c > p:nth-child(1)"   # M3-A 标记落 fresh 章键
__post_c_sel = "p.oldc"                     # 写回失败：accessor 章赋值+defineProperty 双静默 no-op
__contains   = "true"                       # 旧键不在 removed 表 → parentNode 不短路 → 全断
Remove { selector: "#t68c > p:nth-child(1)" }  # engine wire 已携 fresh 章
```

三点定谳：

1. `child.__zwSelector` 是**该代理被获取时的查询原串**（getElementById→`#w40`、
   querySelector→`p.oldc`、物化→结构路径章），R125 旧实现从不改写它；
2. M3-A 首版的 fresh 章写回（赋值 + defineProperty 兜底）对 parse 物化 accessor 章
   **双静默 no-op**（「R191 同款教训」兜底自身同样失效——章属性 non-configurable）；
3. 于是 `_zwMarkRemoved` 记 fresh 键，而全部既有代理（含预存变量、闭包捕获）持旧键——
   `_zwIsRemovedNode`/`_zwIsRemoved`/parentNode 短路/showModal fully-active 门/media
   pause tick 的 removed 判定**全数失配**。engine 侧 fresh 线章不受影响（corpus F2P=22
   的真修复面是 wire 键，不是写回）。

## 修复：线章/身份章分离（part04.js 两臂）

- **engine wire 键 = fresh 章**（保持 M3-A corpus 语义）：R125 摘挂 `__zw_remove(_r125Wire)`、
  move-in 臂 `__zw_remove(_mvWire)`；
- **身份键 = 旧章不改写**：删除 `child.__zwSelector` 写回与 defineProperty 兜底，
  `_zwMarkRemoved(_r125Sel)` 保持旧键（M3-A 前口径；旧位置章同批占位毒化由 pa2 apply
  代际清表兜底——M3-A 前既有风险面，语料长期绿）；
- **move-in append 配对**：新函数顶槽 `_mvWireSel` 传递 fresh 章到
  `__zw_append_child_handle_sel(handle, _mvWireSel || String(child.__zwSelector))`——
  Remove/AppendChildByHandleSel 同键才可复用 detached_stash NodeId（双现缺陷防线），
  函数顶声明保证每调用重置防跨调用残留串键。

## 验证

- 五新红单测全绿（--exact 五名 5 passed）；
- part114 M3-A 五形钉测 5/5 绿（corpus 增益面不回退）；
- 全 lib `cargo test -p zero-engine --lib` **3949P/7F** 名册恰基线；
- fmt --all 零 diff；
- corpus 四腿（ip/ab/mt/enginelib）重跑 = 门禁批 batch-ipm3-fix-gate-v1（本 evidence 提交
  后锚新 head 发射，ip 腿须 fresh 重跑——shim 已变，v6 的 ip salvage 证据失效）。
