# editing-compat — join 语料行内壳变体收敛：ws 摘离快照 + per-run 兄弟壳 + 行内 run 升位 + ended 出参（2026-10-31）

承接 [join 语料第二动作 no-op 根因（双 wrapper 家族视图滞后）+ plain 寻址补全](2026-10-31-join-corpus-family-resync-and-wired-addressing.md)
（fix7=3,108P/4,410F 基准）。本轮交付 join-different-white-space-style 三案
**行内壳变体（b/i/span 嵌套）join 形态收敛**：fix8 = +1,784P 零回归；
fix9 追加块间分隔空白消费与 `ended` 出参（数字见 §6）。

## 1. 失败形状定量分类（fix7 4,410F word-level delta，2026-10-31）

对 /tmp/join-merge-final.log 逐测试 ID 断言 delta 分类：

- **Family A（~1,300F，llrp 面主导）**：host 级 join 提取壳嵌进 caret 侧
  inline 容器（`abc<span style=pre>def</span></span>` 嵌套），期望**兄弟位**。
  根因：forwarddelete/select 臂 `fdst=(fn===sc)?fc:fn` 传 caret 侧 inline run
  作 dst → joinApply `dst.appendChild(shell)` 嵌套。
- **Family B（~1,585F，paragraphs 面主导）**：join 壳 `white-space:normal`
  错印（丢右块继承值）+ 混合行单一外包 span 双层嵌套。根因：
  `_zwCeTakeLineFrom` 先 `removeChild` 后记 srcEl → `_zwCeWsOf(r.ws)` 走
  detached 空祖先链恒回落 normal。
- **Family C（1,027F，`\n` 变体）**：`</div>\n<div>` 源码换行残留——块界
  join 后分隔空白文本未消费。
- **Family D（156F + 旧账）**：`<b>[]def<br></b>ghi` br 在壳内——行界在
  inline 壳内被消费后循环继续提取余行（ghi 误并、右块清空）。

## 2. 修复链（part01.js）

1. **ws 摘离前快照（B 族）**：`_zwCeTakeLineFrom` 返回值新增 `wsVal`——
   srcEl 记名点（整壳臂/块臂/克隆臂/递归传播）在节点仍挂树时
   `_zwCeWsOf(n)` 快照随返回值带出；joinApply 两处壳印统一消费
   `_wsVal = r.wsVal ?? _zwCeWsOf(r.ws || srcParent)`。
2. **混合行 per-run 兄弟壳（B 族）**：多节点行不再单一外包 span 序列化
   （双层嵌套语料拒收）——元素节点保壳补印、文本节点新建带印 span 壳
   （innerHTML t8r 物化路径同 D1 对偶面），同锚序插入。
3. **行内 run 升位（A 族）**：joinApply 落位段新增升位分支——dst 为
   caret 侧行内 run（非块）且行容器=srcParent 时，壳落行容器内 dst 之后
   （`abc</span><span style=ws>def</span>` sibling accept 形态）；块 dst
   不触发（既有 363 修复路径不变）。
4. **落位与序列化逐壳交替（per-run 序）**：混合 plain/handle 壳序列统一
   尾批序列化会逆序——plain 壳 beforeend 落引擎尾、handle 壳落位即达
   （right=normal 混合行 24 案 `<span>ghi</span><b>def</b>` 引擎序错位
   实证）——落位后即时序列化，引擎序=JS 序。
5. **块间分隔空白消费（C 族）**：joinApply 提取前置位收集 startNode 前的
   纯空白文本 run，提取非空后按块界判定移除——判定=startNode 本身为块，
   或跳过的空白串前邻是块（host 级行捐赠 `</div>\n<span>[]def</span>`）；
   行内 run 间有效空白（`abc <span>[]def</span>` 同容器 join）不动。
6. **`ended` 出参（D 族）**：`_zwCeTakeLineFrom` 返回值补 `ended` 字段
   （**自实现起即缺失**——克隆臂 `r2.ended` 恒 undefined，行界在壳内被
   消费后外层循环继续提取余行）。补后克隆臂正确停界、提空原壳移除。

## 3. 验证账

- shim 全量拼接 `node --check` ✓。
- walk 形状 Node 最小 DOM 复算（/tmp/zw-walk-sim.js）：`<b>def<br></b>ghi`
  案 fix8 形状 `abc<b>def</b><span>ghi</span>`（错）、补 `ended` 后
  `abc<b>def</b></div><div>ghi</div>`（对）——§2-6 因果锚定。
- **fix8（§2-1..4）全套三案 4,892P/2,626F，对拍 fix7 逐测试 ID：
  +1,784P / 零回归**（P→F=0、ID 集合 7,518/7,518 无增减）。
- fix9（§2-5..6 追加）数字见 §6。

## 4. 剩余长尾（本轮定位不改）

- **lrlp host 级 join 引擎收敛簇（三方法各 396 案 = 1,188，fix7 起既有）**：
  两种实际形态同域——①`abcabc` 双计（`abc<div style=ws><b>[]def<br>ghi</b></div>`
  backspace 实际 `abcabc<div style=ws><b>ghi</b></div>`，caret 侧文本双计 +
  壳 insert 丢失，56 案/方法）；②行内 run 后继文本壳落引擎丢失
  （`<b>[]def</b>ghi<br>jkl` → ghi 消失，期望 `<span style=ws>ghi</span>`
  兄弟壳）。JS 侧形状均已正确（Node 复算 + 落位路径推演）。
- **paragraphs forwarddelete/select-boundary（各 316 案 = 632）**：dst=兄弟块
  几何引擎侧反相——`<div normal>abc[]</div>\n<div nowrap>def</div>` fd 实际
  `"\n<div nowrap>def</div>"`：**左块整体消失、`\n` 分隔残留、壳丢失**（期望
  `<div normal>abc<span nowrap>def</span></div>`）——引擎侧移除命中错位节点
  的疑似 stale-path 形态，与 lrlp 簇同属 D1/apply 引擎收敛域。
- lprl 面已清零（§6 分面账）。

## 5. 回归门

（fmt/clippy/make test 结果见 §6。）

## 6. fix9 终账

- **join 语料全套（3 文件，test-guard + `ZW_SCRIPT_TIMEOUT_MS=300000`）**：
  **5,698P/1,820F（75.8%，fix7 41.3%）**——对拍 fix7 逐测试 ID：
  **+2,590P / 零回归**（P→F=0、F→P=2,590、ID 集合 7,518/7,518 无增减）。
- 分面账（F 计）：

| 文件 | 方法 | fix7 F | fix9 F |
|---|---|---|---|
| paragraphs | backspace | 676 | **0** |
| paragraphs | forwarddelete | 676 | 316 |
| paragraphs | select-boundary | 0 | 316 |
| lprl | backspace / forwarddelete | 85 / 85 | **0 / 0** |
| lprl | select-boundary | 0 | 0 |
| lrlp | backspace | 517 | 396 |
| lrlp | forwarddelete | 805 | 396 |
| lrlp | select-boundary | 0 | 396 |

（初版表 select-boundary 行因统计脚本 `method=(\w+)` 截断连字符误记 0，
已更正——总量 1,820 恒等。）

- **合并终验（rebase 拉入并行流 a897d536b（part03 计算样式锚定）+
  84811327c（webview form 面）后 union 树复跑）**：**5,698P/1,820F 逐位
  复现，7,518/7,518 对拍 drift=0**——本轮账与 union 树无交互漂移。
- `cargo fmt --all --check` ✓（本轮零 .rs 变更）；make test（guard
  首轮 10.07GB 超限被杀，`CARGO_BUILD_JOBS=2` 复跑）engine lib
  **3,611P/11F = 已知红集**（9 fetch 域 + 2 渲染流债 r113/s24）零新增。

