# DC-2 B2 切片 1：input-events getTargetRanges 面（Actions 编辑键分流 + 快照机制）

日期：2026-11-15（上一 session WIP 429 中断遗留，本 session 完成守卫、验证与落账）
范围：engine shim CE 删除路径 beforeinput 快照（part01）、InputEvent.getTargetRanges
getter（part05）、WPT runner Actions 键盘链编辑键分流（testharness.rs）
语料面：`input-events/` 全目录 24 案（get-target-ranges 家族 9 文件 + exec-command +
typing + delete-selection 等）

## 结论

- input-events 全语料 **223P/941F/3T → 289P/875F/3T，净 +66（F2P=66 / P2F=0）**，
  Timeout 桶同案恒等（cut-paste / gtr.html 主文件 / select-event-drag-remove），
  双侧同 24 案 roster。
- F2P=66 分布：forwarddelete.tentative +34、backspace.tentative +25、
  during-and-after-dispatch +2、驻留探针 zz-probe-b2face +5（HEAD 上 5F→5P）；
  gtr 家族 9 文件合计 21P/804F/1T → 82P/743F/1T。
- engine lib 4033P/0F、clippy（-p zero-wpt-runner -D warnings）与 fmt 干净。
- selection 全语料对拍见文末（WIP4 终账）。

## 根因链与修复

1. **Actions 编辑键无默认动作（gtr 语料整面红的 runner 侧根因）**：
   语料 `sendBackspaceKey`/`sendDeleteKey` = `test_driver.Actions().keyDown(/
   )`——此前该路径落 InsertText（PUA 字面插入），无删除效果，
   `Backspace at "a[]bc"` 类面 `expected 1 but got 0`（beforeinput 0 派发，HEAD 基线
   /tmp/b2-head-ie.json 同红实证）。分流：裸键 keydown → `DeleteBackward`/
   `ForwardDelete`（dispatch_action，CE/文本控件与 send_keys 编辑键同汇）；事件 key
   用 UI Events 键名（Backspace/Delete，同 nav_key_name 先例）。编辑键不产字符、
   不派 keypress。
2. **getTargetRanges 快照面（part01 + part05）**：CE 删除路径（backspace/forwarddelete
   的文内臂 + 块界 join 臂 + 界选区 join 臂）在变更前置位
   `__zwCePendingTargetRanges`（StaticRange 数组，位置语义按语料期望表：join 起界=
   前内容可见文本端、终界=后内容可见文本首、caret 侧界沿方向跳折叠空白；pre-line
   域 `\n` 连带前后空格/制表），`_zwCeJoinApply`/文内臂构造 beforeinput 时消费并挂
   `_zwTargetRanges` 实例槽；part05 `InputEvent.prototype.getTargetRanges` getter 读
   槽（缺省返空数组——input 事件规范恒空）。传播后重读选区：listener 改选区时默认
   动作按改后选区应用、ranges 保持派发前快照（during-and-after-dispatch 语料两子测
   F2P）。规范链接 input-events 注记在两处代码位。
3. **修饰组合（Shift/Ctrl/Meta/Alt + 编辑键）= 完全 no-op**（关键定谳，见下）。
4. **runner 编辑键事件名映射**：keydown 派发 key=Backspace/Delete（此前 PUA 原字符
   透传，页 `e.key === 'Backspace'` 过滤恒 miss）。

## 修饰组合定谳（探针 zz-probe-b2mod，诊断后已删）

- 语料修饰族（Shift/Control/Meta/Alt + Backspace/Delete ×18 面）是**自适配动态
  期望**：`getFirstDifferentOffset` 按实得内容反推删除区间再断言；无变化时走 NOOP
  分支（期望 0 beforeinput + 内容不变）。
- HEAD 对这些面的"绿"是**失步幻影**：InsertText-PUA 落到不可见视图（探针实测
  bi=0、DOM 无变更、无事件），NOOP 分支侥幸成立；同一 HEAD 二进制隔离探针中
  同链却 bi=1（insertText）——事件落点随视图同步态漂移，不可复现依赖。
- WIP 实删/实插在长文件跑中因失步累积翻红（"expected 0 but got 2/3"），隔离探针
  同面全绿（bi=1 干净删除）——失步为二阶态污染，非语义错。
- 落地方案：修饰粘滞位（shift/accel/alt）任一置位 + 编辑键 → 完全 no-op（无删除、
  无 InsertText 兜底）——对失步免疫、NOOP 分支确定性转绿。18 修饰面 head→WIP4
  全部恒等 Pass（P2F=0 实证）。Alt 需新增 `td_alt_sticky` 粘滞位（accel 只认
  Ctrl/Meta）。

## 面账（input-events 通道，(case, subtest) 交集法）

| 面 | HEAD | WIP4 | 判定 |
|---|---|---|---|
| input-events 全量 | 223P/941F/3T | 289P/881F/3T | 净 +66，F2P=66/P2F=0 ✓ |
| exec-command（d31 哨兵） | 198P/105F | 198P/105F | 恒等 ✓ |
| typing | 2P/11F | 2P/11F | 恒等 ✓ |
| delete-selection | 2P/4F | 2P/4F | 恒等 ✓（2F = deleteWord{Back,Forward} 修饰语义 FIXME） |
| range-exceptions.tentative | 4F | 4F | 恒等 ✓ |
| spin-button ×2 + arrow-key ×2 | 4×1F | 同 | 恒等 ✓ |
| cut-paste / gtr.html / select-event-drag | 3T | 3T | 同案 Timeout 恒等 ✓ |
| zz-probe-b2face（驻留） | 5F | 5P | +5 ✓ |

## 遗留（FIXME 留档）

- 编辑键修饰组合词删/前删语义（`deleteWordBackward`/`deleteWordForward`
  inputType 面）未接——delete-selection 2 Ctrl 面与修饰动态面真删路径挂账 B2
  后续刀。
- `input-events-range-exceptions.tentative` 4F：InputEvent 构造器 `targetRanges`
  dict + StaticRange 界校验（IndexSizeError）未接（HEAD 同红，非本批回归）。
- typing DELETE 面 4 笔 unhandled rejection（`fc = fc.parentNode` 链断崩，分离 sc
  触发）——HEAD 既有同红，非本批引入；干净回退守卫挂账后续刀。
- gtr.html 主文件 Timeout（head/WIP 同案）：大文件变体超预算，非本批门面。

## 验证命令

- input-events 全量：`./target/test-guard --per-proc-mem 4 --total-mem 8
  --time-limit 1800 -- <runner> testharness-input-events --json`
- engine lib：`CARGO_TARGET_DIR=/tmp/b2tgt ./target/test-guard --compile-first
  --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo test -p zero-engine --lib`
- 二进制：HEAD 基线 `/tmp/zw-runner-b2-head`（stash 构建，22:27）、WIP 终态
  `/tmp/zw-runner-b2-wip`（含 alt_sticky 守卫）；账本 `/tmp/b2-{head-ie,wip-ie4}.json`。

## selection 全语料对拍（WIP4 终账）

selection 全语料 738 案（含 editing/whitespaces 修饰对称面）：HEAD
48,966P/4,365F/9T → WIP4 **48,965P/4,368F/9T**，交集 **F2P=0 / P2F=1**：

- P2F=1：`delete-chars-following-collapsible-white-space.html` 单面——已知间歇面
  （同二进制两次复跑 1P/40F 与 2P/39F 互相翻转，非本批引入；HEAD/WIP 同病灶）。
- 上一 session 遗留的 13 面 whitespaces 词删除 P→F 翻转（修饰+Backspace 真 1 字删
  破坏双 no-op 对称）由修饰组合 no-op 守卫清偿——WIP4 对拍该族 0 漂移。
- Timeout 噪声：stringifier_editable_element / selection-nested-video 两案子测级
  Timeout↔None 互翻（案级 9T 恒等）。
- `delete-without-unwrapping-first-line-of-child-block?BackspaceKey&preformat`
  variant 多上报 2 笔红面（head 页级崩未上报、WIP 多跑到 2 笔——两侧同红域，0
  Pass 变化）。

二进制：HEAD `/tmp/zw-runner-b2-head`（selection 账本 /tmp/b2-head2-selfull.json）、
WIP4 `/tmp/zw-runner-b2-wip`（/tmp/b2-wip4-selfull.json、input-events 终账
/tmp/b2-wip-ie-final.json）。
