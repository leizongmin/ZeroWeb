# s428：select listbox 形态 carve-out（四面施工 + 归属修正）

日期：2026-11-14 ｜ 车道：engine/dom（我的车道） ｜ 前序：s425 归属保守 → 本轮修正

## 触发与归属修正

s425 曾把 select listbox carve-out 保守归属渲染流。复核 s387（7a90cb732，11-10）
确认：该提交即我车道所建——"fix(engine): select option display:inherit→contents +
closed 盒面抑制三面"，触 style-system + layout-engine，单一 kill-switch
`ZW_SELECT_SUPPRESS_OPTIONS`。面归属跟 driving corpus 走，不跟 crate 走；
listbox carve-out 是 s387 面的直接延续，本轮施工归位。

## 驱动判例（s388 判决 + 本轮勘根）

- `select-listbox-touch.optional`（1F，簇 D 关联）：`<select multiple>` 两 option，
  touch 切换语义，断言只看 state——需引擎 act:option 臂（下轮第二刀，s425 已定设计）。
- `select-multiple-resize`（1F）：CSS height:100px + overflow:auto，80 注入 option，
  106 = 100 + UA padding 6——listbox 面只翻断言②，③④（drag resize）是浏览器
  chrome 级交互面，维持 s388 不施工挂账。
- 上游语义（whatwg/html#3748 / bugzil.la/2028130）：closed select 在 picker
  display:none 子树包 slot（option 永不生成盒）；multiple/size>1 行内渲染 listbox
  option 行；真浏览器 gCS(option).display = 'contents'（slot 继承）。

## 判据（HTML 渲染规范 §the-select-element-2）

listbox ⟺ `multiple` 属性在场，或 `size` 属性解析值 > 1（trim + 全量 u32 解析；
非法/缺席按默认 1 计）。gate `ZW_SELECT_LISTBOX_FORM` default-on，`=0` 回退
形态盲抑制（= s387 原行为）。

## 四面施工（+92/-1，4 文件）

1. **layout-engine tree.rs**：`is_suppressed_select_control_child` 在 select 祖先
   命中处接 `select_listbox_render_enabled(doc, pid)`——listbox 形态放行子树。
   新增 pub fn `select_is_listbox_form` + `select_listbox_render_enabled`
   （OnceLock 进程级单读，R5346 纪律；gate 合入版供跨 crate 复用——runtime_flags
   为 pub(super) 不可导出）。
2. **layout-engine collect_items.rs**：`is_select_control_descendant` 同点接同一
   gate 合入版谓词（Path A 臂；Path B `ua_hidden_without_styles` 域不含
   option/optgroup，无需动）。
3. **style-system lib.rs**：UA 注入点（`ua_default_display` → option/optgroup
   → None 分支）listbox 形态时 display 改推 "block"。style-system 不能反向依赖
   layout-engine——谓词同判据本地副本 `select_ancestor_is_listbox_form`（父链
   行走，gate 由调用方承担，LazyLock 进程级单读）。
4. **engine painter/controls.rs**：`paint_select_value` 开头 listbox 形态提前
   return——multiple listbox 无单值标签面（engine 可正向依赖 layout-engine，
   `crate::tree::select_listbox_render_enabled` 直接复用）。

**刻意不动**：s387 style hook 9.2（作者显式 display:inherit → Contents）——
上游真浏览器行为本就 form-agnostic（closed/multiple 均 contents），其
multiple-select 子测为本判据的现成回归面。

## 验证链

| 门 | 结果 |
|---|---|
| cargo check（workspace） | 清（29.8s） |
| cargo fmt --all | 清 |
| guarded clippy -D warnings（3 crate） | 清（2m10s；style-system 文档注释 `> 1` 顶行首误判 blockquote 修复后过） |
| guarded release build | exit 0（1m32s） |
| solo: display-inherit（s387 回归） | 3/3 Pass，exit 0——closed-form 判据不回归 |
| solo: select-multiple-resize | 1F（红仍② `expected >106 got 106`，如预判不变） |
| solo: select-listbox-touch | 1F（同形 `option one should be selected...`，待引擎臂） |
| 探针（临时 zw-s428-probe，验后即删） | gCS display=block ✓；option gBCR 非零 ✓；scrollHeight==clientHeight==42——**scroll overflow 记账是独立缺失面**（零账面影响：multiple-resize 需③④才能绿） |
| the-select-element 目录回归 | stable=328 diff=0 new=8 gone=2（8/2 = 簇 D flake 键 file-Timeout↔子测完成表示漂移，已知家族非本轮引入） |
| product-smoke | exit 0，struct-check PASS 0 issues |
| product-smoke-legacy | exit 0，struct-check FAIL 0 |
| 全域 sweep ×2 + stable-set join vs s427 | 双轮收敛沾染 0；stable-P **8241=8241 零动**；stable-F 202 vs 201（+1）/ stable-T 13 vs 14（−1），唯一 delta 见下判读（T→F 表示漂移非本轮引入） |

## 判读

carve-out 单独落刀翻转零 corpus 键（seltouch 同形红待引擎臂；multiple-resize
③④ 不可达维持 1F）——与施工前预判逐条吻合。

**复核**：join 结论由两次独立 r2 全程跑逐字确认（原跑 02:28–02:32 与
误判死亡后的重跑 02:32–02:35，stable-set 与唯一 delta 完全一致）——
重跑系运维误判（21 字节首行缓冲 log 被读作启动即死），非数据问题。

**唯一 delta 归因（T→F 表示漂移，非本轮引入，三证）**：
①该文件 5 个裸 `<select>`（无 `multiple`/`size`）→ carve-out 谓词对其中
每一个 select 均为 false（gate-on 但休眠），本轮改面不可能触达该键行为；
②s424 sweep1 旧二进制已出现**同一 F 表示**（`page script threw … Execution
timeout: 90000ms`）——s424=F 表示、s427=T 表示、s428=F 表示，表示层在本就
跨二进制翻转，病根恒定：4 个 pending 测试永不上报的挂起（closed picker
面，簇 D 挂用户门 GB-20261031 不施工）；③先触发哪层超时决定表示——s427
双轮 harness 级「completed without reporting」先到 → Timeout，s428 双轮
script 级 90s Execution timeout 先到 → page script error 转 Fail；同二进制
双轮一致排除纯时序 flake，跨二进制 + s424 反例证实表示漂移先在。账面口径：
该键红位不变（F 桶 ↔ T 桶），总病根数 215 不变。reftest 面核实：select reftest 对
（select-multiple-size-{1,4} 等）不在两套件任一（reftest-manifest 481 条零
the-select-element 条目）；testharness sweep 域 ref-linked 文件零命中。

## 残留面与下一步

- **引擎 act:option 臂**（下轮第二刀）：`_zwHasOwnActivationBehavior` OPTION
  stop + act:option post-activation 分支（gate `_zwUaDispatch===true`，镜像
  act:button；toggle 语义镜像 plan_option——multiple 不清他项；input/change
  派发到 SELECT 镜像 `__zw_select_key_action`）。双刀合龙后 seltouch 应翻绿。
- **scroll overflow 记账**（可选优化面）：定高 listbox select 的溢出记账缺失
  （探针实锤）；multiple-resize 断言②即使补上仍被③④挡绿，零账面影响。
- 用户门禁待决不变：GB-20261114-VIDEO、GB-20261031（簇 D）、s137。
