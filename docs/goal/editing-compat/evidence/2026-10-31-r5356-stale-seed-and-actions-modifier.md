# inline-host 710F 主簇归因收口：序列化 walk 过时 R5273 单文本种子 + Actions 修饰键字面插入

**日期**: 2026-10-31
**前置**: [ghost text 宿主镜像补发 + R383a 域收缩](2026-10-31-inline-host-ghost-text-host-mirror.md)
（inline-host 80P→170P/710F）；本轮收口其挂账「inline-host 710F 主簇」

## 0. 终账

**inline-host 语料 170P/710F → 720P/160F**（+550P），对原 710F 失败集 `comm -13`
= **0 新败**。两笔修复分属两层：

| # | 层 | 修复 | 段账 |
|---|----|------|------|
| 1 | engine shim（e9fce9466） | 序列化 walk 放行过时 R5273 单文本种子改读基座 | 170P→410P/470F |
| 2 | wpt-runner（f93f9484b） | Actions 键盘链修饰键步名映射+字符默认动作抑制 | 410P→720P/160F |

修复 1 揭开视图层毒源后，修复 2 的 runner 层缺陷（shift-enter 全部带 U+E008
前缀）才在账面上可见——两簇此前互相叠加在同一批失败行上。

## 1. 缺陷 1：序列化 walk 吐过时 R5273 单文本种子（engine shim）

### 归因（z 系探针 + ZW_DBG_LOG trace）

探针 `tests/wpt-runner/wpt-data/editing/other/zz-probe-ih-lb.html`（gitignored
工作件，z 系）复现语料主簇流程：`setAttribute(style)` → `setupEditingHost("[]abcd")`
→ `sendEnterKey` → `removeAttribute(style)` → dump container。

机制链四步：

1. `setupEditingHost("abcd")` 纯文本 innerHTML 走 part04 `_zwRegisterTextEl`
   登记 span→T:"abcd" 种子条目（gen=G0）。
2. CE enter 文本拆分经 R48 CharacterData 写路径**原地**改注册节点 data 为 ""，
   引擎树 apply 后宿主已是 `[br, T:abcd]`——引擎快照读本身正确（此前多轮
   「行断语义缺失/基底缓存 stale」假设均被 probe9 引擎快照直读证伪）。
3. `editingHost.removeAttribute("style")` 触发 `_zwPendingAttrDirty` 祖先臂
   （`_zwFusedInnerHtmlSel` 第三门）开 container 融合门。
4. 序列化 walk 内 R383a 跳并拦掉桶修正，`_zwOverlayPendingChildNodes` 的
   R5273 种子替换 `if (_r5273e && _r5273e.node) out = [_r5273e.node];` 在桶
   检查前**无条件**执行——吐出 data 已空的过时单节点，container 视图丢子。

字节对比定性：语料 got = 头段文本 = 过时种子内容，非行断语义问题。

### 修复（part05 + part06，e9fce9466）

- part06 `_zwRegisterTextEl` 条目加 `gen` 代际戳（登记时 `_zwApplyGeneration()`）。
- part05 `_zwOverlayPendingChildNodes` 种子块加过时判定：`_zwSerSkipCrossGen`
  （序列化 walk 域内）且 `entry.gen` 非空且 ≠ 当前代且处于配对环境
  （`_zwApplyPairedEnv()`）→ 放行过时种子，落回基座子节点。live 视图消费
  （种子+桶修正即 live 真值）与 engine 纯测试环境（无配对 apply，代际恒等）
  均不受影响。

### 验证

- 修后 z1：`ser="<br>abcd"`、`container="<span contenteditable=\"\"><br>abcd</span>"`（probe12，剥离插桩树复验）。
- inline-host 170P/710F→410P/470F；join 三案 7,518P/0F、chd 420P/0F 零位移。

## 2. 缺陷 2：Actions 键盘链修饰键 PUA 字面插入（wpt-runner）

### 归因（字节对比）

修复 1 落地后残余 470F 全量字节对比：**全部** shift-enter 失败行 got =
可接受数组项 + 前缀 `U+E008`。editor-test-utils.js `sendKey(key, "shift")`
走 `Actions().keyDown(kShift).keyDown(kEnter)...send()`，plan op 逐条落
`apply_testdriver_command` 的 keydown/keyup 臂——该臂只有 send_keys 循环的
修饰键表所没有的盲区：PUA 修饰字符（U+E008 Shift / E009 Control / E00A Alt
/ E03D Meta）落单字符 InsertText 分支被字面插入。

### 修复（testharness.rs，f93f9484b）

keydown/keyup 臂补修饰键映射：①事件 key/code 用 WebDriver 键名（Shift 等）
而非 PUA 码点；②修饰键步在 prevented 检查后早退（返回 None），不落字符默认
动作。cargo fmt 已过。

**FIXME 留置**（源内注记）：Actions 步骤仍无修饰键**状态**——kShift+Enter
链在块级 CE 宿主与裸 Enter 同汇 Submit（拆段而非换行）；inline-host 语料
只压行内宿主，本修复不触碰该面。

### 验证

- inline-host 410P/470F→720P/160F，对原 710F 集 comm -13 = 0 新败。

## 3. 剩余 160F 单一机制（下轮首位）

全部 160 行同一形态：可见性补断 `<br>` 落在宿主 span **之后**（container
子级）而非内部——EXP `<span ...><br><br></span>` vs GOT
`<span ...><br><br></span><br>`；构成 30 enter（`{}<br>`→`\n\n` vs
`\n<br>`）+ 130 shift-enter。即行断插入的落位锚点取 container 而非宿主
span，属 insertLineBreak 落位语义，与上述两缺陷无涉。

## 4. 门禁账

| 门 | 账 | 说明 |
|----|----|------|
| inline-host | **720P/160F** | 终树复验（剥离插桩） |
| join 三案 | 7,518P/0F | 全扫保持 |
| white-space-style | 616P/488F | 零位移 |
| cannot-have-div | 420P/0F | ×2 复验 |
| engine lib | 3612P/11F | 11 红与已知集逐条 identical |
| webview lib | **824P/0F** | 新基线（旧 r3259 红已被并行流 s170 b100c742a 修掉） |

**记账说明**：join/wss 账出自带惰性日志+变量改名插桩的二进制（`_r380Arm1/2/3`
等），probe/chd/inline-host/engine lib/webview lib 均已在剥离插桩的干净树复验；
插桩剥离后探针旗标引用 grep 清零、part04 无 diff。

## 4a. rebase 撞车合并组合态复验（远端 7721142c4 碰头仲裁轮后）

push 前 `git pull --rebase` 撞车并行流 ghost text ①簇碰头仲裁轮（7721142c4：
part92 钉测入库 + r113 期望同步 eabdd5efa + s172 根修），shim 面零碰撞
（远端对 js_dom_shim/ 零改动）。合并树全量复验：

| 门 | 账 | 对比 |
|----|----|------|
| join 三案 | 7,518P/0F | 恒等 |
| white-space-style | 616P/488F | 零位移 |
| cannot-have-div | 420P/0F | 恒等 |
| inline-host | 720P/160F | comm 对原 710F 集 = 0 新败 |
| engine lib | 3614P/10F | = 远端新基线（r113 已销账，红集恰为已知 10 条） |
| webview lib | 825P/0F | = 远端账 |

上文 §4 的 engine lib 3612P/11F / webview lib 824P/0F 为变基前分支树账，保留存证。

## 5. 提交

- e9fce9466 `fix(engine): R5356 序列化 walk 放行过时 R5273 单文本种子改读基座——inline-host 语料 170P→410P`
- f93f9484b `fix(runner): R5356 Actions 键盘链修饰键步名映射+字符默认动作抑制——U+E008 字面插入清零，410P→720P`
