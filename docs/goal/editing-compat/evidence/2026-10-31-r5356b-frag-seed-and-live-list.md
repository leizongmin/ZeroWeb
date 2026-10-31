# R5356b：frag 元素种子与 live-list 同步——inline-host B 簇 `{}`/`[]` setup 60F 清零

**日期**: 2026-10-31
**前置**: [R5356 双缺陷收口](2026-10-31-r5356-stale-seed-and-actions-modifier.md)
（inline-host 720P/160F）＋本流前半段「无 id pending 元素跨批 remove 宿主镜像」
（820P/60F，同提交序列 345140946）

## 0. 终账

**inline-host 语料 820P/60F → 880P/0F 全扫**（提交序列 345140946 + 93c02bd3c，
rebase 后远端 HEAD）。zz-probe-b 四案（b1 send_keys 管线 / b2 直驱 /
b3 分步 / b4 视图检视）全 Pass，b3 ser=`\n<br>` 恰合。

| 门 | 结果 | 对照 |
|---|---|---|
| inline-host | **880P/0F** | 60F 清零，820P 无翻转 |
| join 三案 | 7,518P/0F | 零位移（见 §3 撞车复归） |
| insertparagraph-with-white-space-style | 616P/488F | 零位移 |
| cannot-have-div | 420P/0F | 零位移 |
| engine lib | 3,614P/10F | 已知红集逐条 identical（fetch/Request 族 8 + iframe SW + s24） |
| webview lib（rebase 合并树） | 830P/0F | 与 s174 并行流合并复证一致 |

`insertlinebreak-with-white-space-style` 0P/552F 为 M1 基线已知 deferred 簇
（行断未插入签名，见 [M1 baseline §editing/other](2026-10-30-m1-corpus-baseline.md)），
非本轮回归——语料账引用的 wss 门恒指 insertparagraph 单文件 1104 子测。

## 1. 缺陷机制：盲基座 + 桶清空（B 簇）

`{}<br>` setup（`{}` 为 editor-test-utils 标记，剥离后宿主内容 = 单 `<br>`）：

1. setup 经 innerHTML= journal SetInnerHtml + 解析代理 `_ihAdded`；runner
   引擎读路径对该内容**永久盲**——`with_query_view_doc` 视图基座钉定
   （uievents-compat 尾簇 13 + s143 游标，SetInnerHtml 在 applied cursor 后
   不重放），b3 分相仪表 `eng-after-setup=0 … eng-final=0`（gen 6→7）钉死。
   **引擎读不能作为该 setup 形的真值依据**。
2. CE enter 的 LBA（`_zwCeLineBreakAt`）按融合视图（= 种子）落位：
   `insertBefore(LF, childNodes[0])`，`brk3.nextSibling` 在场无补断——
   视图正确时产物恰为 `\n<br>`。
3. **b1（send_keys 管线路径）独有**：管线每脚本 `execute_dom_script` 的
   apply+gen bump（pa2a `__zw_reset_pending_state`）清空 pending 桶 →
   LF 只存在于桶 → 命令与断言之间桶丢失 → 序列化仅剩种子 `[br]` →
   got `<br>`。b2/b3 直驱同回合桶存活故过。

## 2. 修复：frag 种子三层（345140946 后半 + 93c02bd3c）

- **注册**（part04 setter）：sel 形 markup innerHTML= 登记
  `_zwRegisterFragEl(el, handle, sel, _ihAdded.slice())`——条目只进
  `_zwTextElsBySel/ByEl/ByHandle` Map，不进 `_zwTextEls` 数组（数组消费者
  全是文本几何扫描）。**叶内容门**：片段顶层全为文本/无子元素才登记。
- **消费**（part05 overlay 种子块 + part06 `_zwLocalChildNodes`）：frag 条目
  按注册序整体替换视图基；**已摘离成员 `parentNode != null` 过滤**——解析
  代理（无 sel/handle）的本地 remove() 只 splice+置空 parentNode（标记剥离
  即此形），不落 pending/removed 标记面，不过滤则 detached 节点滞留视图
  （二轮扫描命中已剥离 `"{}"` 文本 → `indexOfContainer` 读 null.parentElement
  抛）。frag 条目豁免 R5356 过时门（`&& !_r5273e.nodes`）。
- **live-list 同步**（part04 insertBefore 尾部）：种子宿主上的插入把新节点
  splice 进 `entry.nodes`（ref identity indexOf 定位，miss/null 尾追）——
  gen bump 清桶后序列化仍从种子吐完整子列表。桶活路无双计：overlay 并桶
  的 seen identity 跳过（R182 handle 臂）先于插入定位生效；`_zwNodeParent`
  反链不在 pa2a 清单，splice 成员 parentNode 判定跨代际存活。

### 探针定位方法论

- 引擎侧单测复刻（V8Sandbox + generate_js_dom_shim）区分 shim 层与环境层——
  单测通过证明 shim 正确、抛错来自页面环境。
- runner unhandled rejection 不带栈；testdriver 页面 JS 内 try/catch +
  `__zw_dbg_log(e.stack)` stepwise 抓栈最快定位（editor-test-utils 标记剥离器
  即此法锁定）。
- 分相 eng 读打点证伪「apply 时序」假设，确立「永久盲」结论。

## 3. 撞车复归归因：注册面过宽 × 序列化权威化（本轮最大反复）

首轮实现取「全面注册（凡 markup innerHTML=）+ 序列化无条件吐 frag 种子 +
过时门豁免」，zz-probe-b 全绿，但 **join 三案 7,518 全 F**——got 恰为 setup
原文（`abc<div style="white-space:nowrap">def<br>ghi</div>`），join 动作
生效而序列化吐过时种子。根因：join/wss/chd 语料的 setup 含容器元素
（`<div>`/`<b>` 嵌套），其引擎基座健全且随 wire 推进；种子权威化把健全
基座整体遮蔽。管线流下「out 空门」也不成立（管线把 LF apply 进基座，
base 部分 populate，b1 got `\n` 实证）。

终案 = **注册面收窄**（叶内容门，含容器元素的片段根本不建种子，门失败
回退 unregister 保持 pre-change 链语义）+ 消费面恢复无条件。盲基座域
（B 簇 setup 形）种子即唯一真相；健全基座域零种子零影响。

## 4. 插桩剥离

part01（4×R5356T breadcrumb + PSA-THROW wrapper）、editor-test-utils.js
（2×B3DBG 块）、part06.rs 临时测试 `r5356b_tmp_throw_probe` 全部还原/删除；
part04 reg-throw 包装随叶内容门改写一并剥离。grep
`R5356T|PSA-THROW|_zwCePsaInner|B3DBG|R5356B reg-throw|r5356b_tmp|R5356B-TMP`
清零。§0 门禁全表在剥离后净树复验。

## 5. 残余与下一步

- inline-host 全扫后残余面归零；控制面顺位接 ② white-space-style 488F
  （italic/bold 组）→ ③ void-element 70F → ④ renderer 侧 R383a 对账 →
  ⑤ 簇 A glue 残余、execCommand 簇序。
- 叶内容门外的种子遮蔽风险面：凡「markup innerHTML= 后引擎基座健全但仍
  走种子消费」的形态已被门排除；纯叶内容 + 健全基座 + 后续 wire 推进的
  组合未见现网语料，挂账观察。
