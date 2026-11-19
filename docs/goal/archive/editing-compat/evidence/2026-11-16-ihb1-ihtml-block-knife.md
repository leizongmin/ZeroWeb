# ihtml 轴一载荷块级拆分插入刀门禁批收口（ihb1→ihb3）

**日期**: 2026-11-16
**刀体**: 4f3469026（917183e38 经三次 pull --rebase 重放：917183e38→f9cec9dde→4f3469026，crates/engine + tests 域 diff 恒空，字节恒等传递闭合）
**批链**: ihb1（停批）→ ihb2（ABORT 常量修正）→ ihb3（判读完成）
**census**: inserthtml 全页 1459P/**106F** → 1509P…→ **1508P/57F**（F2P=49，零新增红）

## 刀体内容

`crates/engine/src/js_dom_shim/part06.js` 新增 `_zwExecCmdApplyInsertHtmlBlock`（host 级 innerHTML 字符串手术臂），insertion dispatch 块臂优先、不合形返 false 落回既有内联拼接臂（零改动）；`crates/engine/src/js_dom_bridge_tests/part124_ihtml_block_pins.rs` 三十钉锁定语料实录形。

臂面构成（对应 [2026-11-13 普查档](2026-11-13-ihtml-residual-mechanism-census.md) 缺陷主面）：

- **顶层跨度扫描** `_zwIhtmlTopSpans`：深度 1 记号器；void 元素不入深度；自闭合判据 `charAt(gt-1)==='/' && charAt(gt-2)!=='='`（排除未引号属性值 `href=/>` 形）；未闭合尾 span 端点=innerEnd。
- **载荷补全** `_zwIhtmlCompleteFrag`：首标签未闭合补 close（void 表例外）。
- **文本→html 偏移映射双版**：`_zwIhtmlTextPosToHtml`（START 语义，tag 跳走＋实体计数 1，want=0 落首文本字面）＋`_zwIhtmlTextPosToHtmlEnd`（END 语义，返回第 want 个文本字符之后——list 右区 `</li><li>` 边界不吃字）。既有内联臂 `textToHtmlOffset` 把 tag 字符计入文本，仅无 tag 前缀成立——本刀换 tag 跳走版。
- **结构 T 定位**：宿主直子元素游走＋祖先包含＋非元素子跳过（空 li 整体换载形无文本锚时仍可定位）；文本锚 hStart/hEnd＋tSpan 一致性门。
- **路由面**：裸载荷（li 降级 dPS div/p、dt·dd dl 包裹、td·th 溶解）；list 上下文（文本锚 li 拆段 `</li>+fragC+<li>`、嵌套 li 内、右区劈裂**重开 list**——corpus :169 实录 `</ol><p>…</p><ol>`）；dl 上下文 ddTag 拆段；td 上下文溶解；T 级劈裂（同名 SPLIT_T a/nobr 三段；BLOCK_T 载荷 hr/h2 等；携带族 font/span color 旁路维持红——后续携带刀）；空 li 整体换载（载荷子 li 替换 li0，旧 list 壳与 id 保留）。
- **旁路门**：单根载荷门（multi-root 旁路）、CE=false 链门、携带族门。

## 门禁批链记录

| 批 | 锚 | 发射 | 终态 | 死因 |
|---|---|---|---|---|
| ihb1 | 917183e38 | 04:17:06 | 04:26:53 停 | 外部 `systemctl stop`（journal Stopping/Stopped 实证）——同窗 patrol 轮 pull --rebase 把刀提交换锚（917183e38→f9cec9dde），硬 HEAD 锚已失效，停批合规 |
| ihb1b | ab5f6c45b | 04:34 | 04:42 停 | 04:32 单批契约仲裁让位（ihb2 在飞） |
| ihb2 | f9cec9dde | 04:30:55 | 05:13:03 ABORT | L1 计数门 1509P/56F vs 实测 1508P/57F——**50 预测含 multi-root div 载荷键（`<div class="d1"></div><div class="d2">…</div>`＋`{}` 选区），刀体单根门正确旁路；49 即真值**（writer/核验方双源 F2P 集合恒等、newred 双源皆空） |
| ihb3 | 108817951 | 05:21:54 | 判读完成 | 见下 |

锚定 v2（ihb2 起）：硬 HEAD 锚被 docs-only 并行轮常态击穿——发射时冻结 LAUNCH_HEAD＋每腿 bin 字节复查＋engine/Makefile 域漂移（未提交脏＋已提交漂移）熔断；docs 轮提交不误伤。三次 rebase 实证抗性。

## ihb3 判读（终批）

- **L1/L2 iht A/B 双腿**（1565 行，乘数 2，wall 11s×2）：**1508P/57F/0T，F2P=49 恰块级族＋零新增红**（comm 对 ihf1 基线 1459P/106F，raw 行格式恒一直比）。
- **L3 mt**（wall 1935s）：**10090P/2F/0T**，残红恰 subscript+formatblock X37 双键，对 i5p1-mt 零新增红——multitest 语料 inserthtml 仅 `ab<b>c</b>d` 行内载荷 24 键，块臂全旁路，零暴露核验成立。
- **L4 enginelib ABORT-DOMAIN-DRIFT（非刀体失败）**：mt 腿 26min 窗内他流 shape() 文法刀 07fcc9e0e（css-parser/computed_style/border_shape/transition，与本刀 js_dom_shim 域不相交）经 pull 落树，域漂移熔断正确拒绝绑定过期判读域。
- **lib 证据链（携带）**：刀提交态 enginelib strict 4054P/0F（917183e38 自验，与 4f3469026 字节恒等传递）；他流 s504 归因轮（8fc297fc1）在刀含树实测全桥绿窗 **1660P/0F（1659→1660，+1 恰 test_ihtml_block_pins）**「零语义位移」第三方实证。

## 残 57 红族谱（106−49 对账闭合）

| 族 | 键数 | 处置 |
|---|---|---|
| carry `<p>abc` 载荷入 font color 壳 | 4 | 维持红——override 携带后续刀 |
| carry `<p>abc` 载荷入 span color 壳 | 4 | 同上 |
| span color 壳＋行内 `ab<b>c</b>d` 载荷 | 8 | 同上（内联臂携带域） |
| CE=false pre 链 | 1 | 轴二 pre 壳臂现役 got 锁定（part124 ⑲ 钉） |
| multi-root div 载荷＋`{}` 选区 | 1 | 单根门旁路——multi-root 扩展挂账 |
| inline 载荷其他（text/空/ws/comment/b/i/script/xmp 等） | 39 | 轴二/assert/ws 族维持 |

## 下一步

- **ihtml 携带族刀**（8 键：font color ×4＋span color ×4 override 携带——载荷/既有壳样式继承语义）；
- multi-root 载荷扩展（1 键，可搭车后续块臂轮）；
- assert_in_array 9 键收尾批量判读、exec1 474 域决策挂账维持。

## 批脚本

`.acceptance/dps/ihb3-tail.sh`（md5 8d8c5a9a4363338878600bfa4e1f936f，chmod 400 冻结；ihb1/ihb2 同目录留档）。writer 死亡与多执行体协调全录 `.acceptance/dps/COORD.md` 2026-11-16 04:32–05:22 条目。
