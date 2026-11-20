# d44/d45：B4-S2 残余形收尾——join 缝尾随 br 消费 + gtr 端点克隆/持活分流

日期：2026-11-16。前置：d43 B4-S1 select-delete 主臂落树（0f1a8d410）后，master.md
下一步①= B4-S2 残余形收尾（左块尾随 `<br>` 消费 + delete-without-unwrapping
四变体跨 child-block gtr 端点规范化期望 +22 名义红）。上一 session 在本刀编码与
对拍中途被 429 使用上限中断（d44 runner 14:25、d45 runner 15:26、d45b 全语料
对拍 15:30 已完成；本 session 接续：复核遗留 diff 与三份运行产物、补跑双哨兵、
落账）。探针/运行产物均在 /tmp（gitignored）。

## 落码（part01.js，+65/−4，唯一产品变更）

1. **join 缝尾随 br 消费**（`_zwCeJoinApply` 壳落位段）：块界 join 后原行尾 br
   沦为行中换行——Blink 删一个多余 br（`<li>list-item1<br></li>` + join 期望
   `<li>list-item1list-item2</li>` 精确形；双 br `<li>list-item1<br><br></li>`
   accept 数组保留一个）。仅跨块尾 append 路径触发（dst=左块、行壳落 dst 尾；
   同容器行间 join 不动）。对应 d43 挂账 FIXME「左块尾随 `<br>` 消费」。
2. **`_zwCeJoinStartOf` 尾随 br 边界**：前块尾 br 不入界——界落**首个尾随 br**
   前的父偏移（`<li>list-item1<br></li><li>[]…` (li,1)、`<p>abc<br></p>` (p,1)
   语料形；双 br 同落首 br 位），纯 br 块无可见内容 → 块首（br 即占位）；
   无尾随 br 保持旧口径（`<dt>list-item1</dt>` (text,10) 文本端形）。
3. **`_zwCeTargetRange` 端点克隆/持活分流**（新增 `liveEndpoints` 参数）：
   input-events 语料 getRangeDescription 断言在变更应用后读取 node.data（harness
   在传播期把 getTargetRanges() 冻结进 cachedRanges 再断言），报告值 = 构建期
   所引节点在**断言期**的 data。对拍实证两族并存，一刀切不可行：
   - **替换承载族**（join 块界/界选区臂、select-delete 摘除核）：变更机制把树内
     节点替换/摘离，测试捕获的变更前节点值不变——克隆冻结同一 pre-value 给出
     等值观测。d44 全克隆 +385/−8 对拍实证此族为主。
   - **原位截断族**（backspace/forwarddelete 文内**部分**纯字符切片）：编辑臂对
     文本节点原位截断，测试捕获节点即被截节点、断言期读 post-value——端点必须
     持 live 引用（克隆冻结 pre-value 反泄旧值，8 案 P2F 实证）。
   - **d45 收窄判据**（对拍定谳）：`liveEndpoints` = 切片不含 `\n` 且非整节点
     全覆盖。范围含 `\n`（pre-line 硬换行/块分隔删除 4 案）与整节点删除
     （`[list-item1]` 全选 4 案）期望读原节点 pre-value，属替换承载族——
     Chrome 摘除/替换节点而非原位截断，保持克隆。
   - Backspace/ForwardDelete 文内臂传入该判据；其余调用点（join/select-delete
     承载）保持克隆。

## A/B 轨迹（input-events 全语料，zz-probe 排除口径；基线 = d43 终态二进制）

| 二进制 | 构成 | 全语料 | 判决 |
|---|---|---|---|
| d43b（0f1a8d410） | BASE | 715P/2613F/3T | 基线 |
| zw-runner-d44 | 全克隆（无 live 分流） | 1092P/2236F/3T | F2P=385/**P2F=8**（backspace 4 + forwarddelete 4 = 原位截断族反泄 pre-value） |
| zw-runner-d45（中途态） | live-all | 1048P/2214F/5T | 替换承载族大面积回红（-44P），不可落 |
| **zw-runner-d45（终态）** | 判据收窄分流 | **1099P/2229F/3T** | **净 +384，F2P=384/P2F=0** ✅ |

d44→d45 差异 = 8 案原位截断族（live 判据收回）F2P=8 + during-and-after-dispatch
1 案 P2F：传播中 listener 改选区触发替换承载时，快照引用随机制被分离的节点，
期望读分离原节点 pre-value；live 判据按切片文本形态无法区分该形——**接受 1F**
（与克隆门互斥，不可能两全；d43 基线该案同为 Fail，net 口径无影响）。

逐文件 F2P（+384 全落 join/gtr 目标簇，零面回退）：

| 文件 | P |
|---|---|
| joining-dl-elements?Backspace / ?Delete | 72→208 / 132→268（+136×2） |
| joining-dl-element-and-another-list?Backspace / ?Delete | 12→48 / 44→80（+36×2） |
| deleting-in-list-items?Backspace{ol,ul} | 23→33（+10×2） |
| deleting-in-list-items?Delete{ol,ul} | 36→42（+6×2） |
| non-collapsed-selection?{Backspace,Delete} | 20→22（+2×2） |
| backspace / forwarddelete 主文件 | 41→43 / 51→53（+2/+2） |

## 哨兵（d45 终态二进制）

- html-semantics 全语料（950 文件）：`/tmp/d45-sem.json` **P 恒等 5378**，
  F2P=0/P2F=0（F/T 仅 select-home-end-pagedown 已知 flaky 带互翻 2 例）✅
- selection 全语料（738 文件）：见下节（d43 基线 48,979P/4,352F/9T）。

## selection 对拍（d45 终态二进制）

`/tmp/d45-sel.json` vs d43 基线（`/tmp/d43b-sel.json`，738 文件）：
**48,979P/4,352F/9T → 48,979P/4,352F/9T（+4 NotRun）**——按 (page,name) 多重集
对拍 **P 集恒等、F 集恒等，F2P=0/P2F=0** ✅。唯一差异 =
`stringifier_editable_element.tentative.html` Timeout 边界子测级↔页级互翻产生
4 笔 NotRun（d43 已记档同形 flaky，案级超时预算非行为漂移）。记档：子测名
去重投影会把 selection 总账 4,352F 投成 4,301F（multitest 同名断言），对拍以
原始多重集为准。

**dwuf 清偿判定：未清偿**。delete-without-unwrapping-first-line-of-child-block
八变体（BackspaceKey/DeleteKey × br/preformat + deleteCommand/forwardDeleteCommand
× br/preformat）两侧全红恒等（29/33/29/37/25/25/25/25）——本刀克隆/持活分流只改
端点快照语义，dwuf 期望的跨 child-block gtr **端点规范化**（d43 主臂
`_zwCeCanonGtrStart/End` 同族，但 dwuf 走 caret join/deleteCommand 臂未接）仍在
队列。master.md 下一步①中该子项持位。

## 守卫与门禁

- 语料：`./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit
  {1200,2700} -- /tmp/zw-runner-d45 testharness-{html-semantics,selection} --json`；
  input-events 三份对拍沿用上一 session 同守卫产物（d44-ie.json 14:29 /
  d45-ie.json 15:23 / d45b-ie.json 15:30）。
- runner 构建晚于 part01.js 末次编辑（15:25:12 → 15:26:52），d45b-ie.json
  15:30 即当前树产物；本 session 未再改码。
- 本轮零 Rust 变更（仅 part01.js，fmt/clippy 不消费）；`git diff --check`
  干净；runner 构建已覆盖 engine 编译面。

## 遗留（持位，B4-S2 后续标的）

input-events 残余 2229F 簇结构：

- deleting-in-list-items ×4 = 1114F：join 壳吸收深化（`<li><p>list-item1</p></li>`
  + join 期望 `list-item1list-item2` 入内层 p 壳，got 落壳外）+ 同块摘空 br 落壳
  （`<ul><li>[list-item1]</li></ul>` 期望 `<ul><li><br></li></ul>`）+ 深锚 gtr 形。
- joining-dl-elements ×2 = 404F：跨 dl join 期望右 dt 并入左 dl 的 dt 壳
  （`<dl><dt>list-item1</dt>list-item2</dl>` not in accept）——同壳吸收族。
- backspace/forwarddelete 主文件 = 243F：跨元素边界 caret join
  （`<p>a<span>b</span>[]c</p>` bi=0）+ 文首空删 beforeinput 形。
- non-collapsed TypingA variant = 69F：insertText 非塌缩选区**替换**臂缺失
  （`<p>{abc}</p>` + 'a' 期望 `<p>a</p>`，got `<p>aabc</p>`）。
- exec-command 105F / typing 11F / delete-selection 4F / range-exceptions 4F 等
  小面持位（master.md 修复队列 B3/B5）。

dwuf（delete-without-unwrapping 四变体）+22 名义红是否由本刀 gtr 分流清偿——
见 selection 对拍节。
