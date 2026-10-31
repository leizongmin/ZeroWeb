# s175（2026-10-31）：detached 节点选择器身份盗用根修——unique_selector_for_node 可达门 + 枚举镜像撤下归因

## 入场定刀位

候选①「声明式根内容枚举」（s173 LOST 20 承接）：shadowRoot 的
childNodes/children/firstChild 族读通道——findElements 类顶层枚举
（`wrapper.children` + `child.shadowRoot` 递归，dialog-popover-closedby-complex
line 78/85）读未物化声明式根恒空 → line 107 `assert_array_equals` 中断 →
shadow-slots 段 20 promise_test 不注册。

## 施工过程（含撤下归因——最终落盘 3 文件 +66 行）

### 第一刀：枚举镜像钩（已撤下）

part04 两钩 + part05 `_zwDsEnsureEnumerated`（shadowRoot 枚举读前惰性触发
`_zwDeclarativeShadowEnsureMirrors`）+ webview 钉测 4 案。钉测 red→green 4/4、
webview lib 834P/0F。**WPT 域门回归**：interactive-elements 423P/60F，
对 s172 基线 join **P2F=16** 全落 dialog-popover-closedby-complex normal DOM
nesting 段。

### 第二刀：管线等值轻量臂（已撤下）

勘根初判「镜像 wire 批 → 探测拍 flush → 全量臂换代 → normal 段代际偏斜」
（沿用 s173 首刀结论）。施工：`render_with_dom_mutations_persistent` apply 前
取 `outer_html` 基线 + 等值轻量臂（等值 ⇒ 跳过 re-parse 换代，返回 `(None, None)`)
+ webview 两处 apply 消费端 bump 门禁（等值批跳过 `__zw_apply_generation_bump`）。
实证：light arm 确实触发（12 mutation 镜像批等值），**P2F 不动**（16→18 噪声内）
——换代理论证伪。

### 定谳勘根：选择器身份盗用

带 `{:?}` 插桩重跑 solo，mutation 日志暴露真机制：normal 段测试的
`showModal`/`close()` 走 `SetAttrOnHandle { handle: "__n13" }` /
`RemoveAttrOnHandle { handle: "__n1" }`——`__n13`/`__n1` 是**镜像批**
CreateElement 的句柄。失败值吻合（`dialogB should be closed expected false but
got true` = close 打在镜像上，真 dialogB 永不关）。

引擎探针钉死（最小 4 mutation 复现）：

```text
ZZDBG handle __n13 -> selector dialog.dialogA
```

根因链：detached 镜像 `<dialog class=dialogA>` 过
`unique_selector_for_node` → `stable_selector_for_node` 返 `tag.class`
→ uniqueness 门在 **doc** 上查询（镜像不在 doc）→ 在树真元素 `dialog.dialogA`
顶替唯一匹配 → 同串选择器进 apply 的 handle_selectors → webview
`merge_handle_selectors` 的 sel→handle 表**被镜像句柄顶替** → normal 段后续
handle 升级写全部打在镜像上。s173 首刀 18 键 P2F 的「代际偏斜」归因不完整——
真机制同此（镜像批一旦存在即触发，与换代无关）。

### 最终落盘（3 文件 +66 行，零 webview/pipeline 改动）

1. **engine `js_dom_bridge.rs`**：`unique_selector_for_node` 顶部加可达门——
   parent 链不达 `doc.root()`（detached fragment 镜像等）返 `None`。物化
   shadow 树 parent 链达 root（attach_shadow 挂 host 子列表），不受影响；
   shadow 内容选择器本就因封装查询门返 None（先证：parent 链探针 chain
   `7v1→9v1→5v1→4v1→2v1→1v1` 达 root，None 来自查询门，非本门）。
2. **engine `dom_bridge_tests.rs`**：钉案
   `test_detached_node_unique_selector_is_none_s175`——镜像句柄不得进
   handle→selector 表 + 真元素 `dialog.dialogA` 寻址不受扰。
3. **shim `part05.js`**：`_handleChildNodes` 前 s176 deferral 注（枚举通道
   撤下原因 + 重开前置：引擎侧 shadow 寻址）。枚举钩三处调用 +
   `_zwDsEnsureEnumerated` 撤除（装配序拼接 node --check SYNTAX_OK）。

## 验证

- engine lib **3617P/10F**（基线 3626 + 1 钉测；10F 基线同键 fetch 族 9 +
  ua_display_li）。
- webview lib **830P/0F**（= s174 合并树基线；枚举钉测 4 案随通道撤下删除）。
- interactive-elements **437P/26F/463** 对 s172 基线 (key,state) join：
  **P2F=0**、F2P=2（dialog-focus-previous-outside / form-submit-dialog-shadow，
  s173 双靶既有）、LOST=20（shadow-slots 段，s173 账面既有归因——其中基线 4P
  为摊平 DOM 凑巧 Pass，s173 已记「假 Pass 清除归因非回归」）。**与 s173 终态
  437P/26F 逐键同态**（枚举通道撤下后回归 s173 shipped 行为 + 根修硬化）。
- forms 家族 **96P/12F** 持平（12F 全既有已归因）。
- fmt/guarded-clippy 见轮次账 commit。

## 教训与新记

- **选择器唯一性门不得跨 detached/in-tree 语义混用**：uniqueness 在 doc 上
  查询，detached 节点的身份串天然「盗用」在树同身份元素——身份函数必须先证
  可达。
- **枚举镜像零 Pass 增益实证**：hooks-on 态 shadow 段 20 行恢复注册但
  0P/20F（基线 4 假 Pass 未幸存）——detached 镜像对引擎 light-dismiss/hit-test
  不可见，注册恢复在渲染/交互语义（R2926）前无翻绿价值。
- FIXME(s176)：枚举通道重开须走引擎侧 shadow 寻址（物化 shadow 根 handle 化），
  非 detached 镜像。

## 事故记录

施工中 `dom_bridge_tests.rs` 被探针 Write 全文覆写（1755 行 → 41 行，
`Write` 工具对既有文件误报 created）——发现于 diff 审计（+1749 deletions 异常），
`git checkout HEAD` 恢复全文后重附钉测，engine 测试总数复证 3627（3626+1）。
无内容丢失。

## 并行合并复证（rebase 并入 zero-web 流 4 commit）

push 前 `git pull --rebase` 并入并行流 editing 簇（c81777413 insertlinebreak /
93c02bd3c R5356b / 345140946 inline-host 镜像补发等，同触 part04/part05 不同
区域，自动合并无冲突）。合并树复证：shim 装配序 node --check MERGED_SYNTAX_OK；
engine lib **3617P/10F**、webview lib **830P/0F**（同键）；interactive-elements
**437P/26F** join 仅 F2P=2——三项与本刀验证逐键同态，零跨流扰动。
