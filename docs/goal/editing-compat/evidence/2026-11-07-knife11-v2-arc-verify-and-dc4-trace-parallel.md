# knife11 v2 探针弧线独立核验与 DC-4 静态追踪（并行轮，2026-11-07 15:4x）

继 2026-11-07-knife11-v2-ab-probe-and-census-yield.md（让位记录）与本目录
typed-fate 弧线笔记：本会话按核验方职责独立复核四相探针日志逐案 verdict，
并落 DC-4（496/499 fallback 拆分病形）静态追踪与相 F 探针增强。

## 探针弧线独立核验（read-only，四日志逐案对账）

对 post-a/post-b/post-c/post-c2 四日志 `IP2-CASE CASEnnn ok=` 全量提取与
typed-fate 笔记弧线表逐案比对：

- post-a（131P/29F）：绿 = {514,516,519,520} + 哨兵 {531,533,535} = 7 ✓
- post-b（149P/11F）：+{515,517,518,529,539,545,547,553,555} = 16 ✓
- post-c（149P/11F）：与 post-b 逐案恒一 ✓（(c-v1) 前导不翻案，dump 归因）
- post-c2（158P/4F）：+{495,497,498} = 19；残红恰 {496,499} ✓
- 全弧线 P2F=0（无任何案绿→回翻），哨兵三案逐相恒绿 ✓
- Fail 行四条 = 496/499 各两行（faithful compare + IP2-CASE），无隐性第三案 ✓

**弧线表 VERDICT OK，typed-fate 笔记数字面逐项吻合。**

## build 锚与门禁批

post-c2 build-anchored md5 d74e396efa32e4086b051cea0266a09f；verdict 批
（15:23 起飞，batch-knife11-v2-verdict.sh）SHIM-DRIFT 门同锚恒一。本会话
自 post-c2 锚立后冻结 part0*.js 字节（批链静默前不触碰）。

## DC-4 静态追踪（post-c2 TRACE + part01f/part01 字节面）

CASE496 实测链（probe post-c2 TRACE 行）：ip@pre caret `#text@3`（extractRange
起点 (abc,3)）；route `wants=false`（caret 宿主本体裸文本）→ FALLBACK；
ip@post `ghi<p>ghi</p>` caret P@0，`tt=["b"]`（(a) 捕获面已生效）。

delete 前导（`__zw_ce_delete_apply` → `_zwDelDeleteSelection`）对
`abc[<b>def]</b>ghi` 的字节面推演（part01f :331-724）：

1. 链外边界（origChainHasInline=false，上行链无 inline 元素）。
2. start 等价点归一下钻入壳：(abc,3)→(host,1)→(b,0)→(def,0)，提升
   (text,0)→(b,0)；end (def,3) 提升为 (b,1)。活动 range = (b,0)..(b,1)。
3. nodeList=[b] 整含摘除（delKeepShell=null 链外无留壳）→ host=[abc,ghi]。
4. startBlock==endBlock（host）→ :535 出口臂：**链外面 caret 落原边界**
   （:536-546 Chrome delta）：collapse(origStart=(abc),3)——实测 `#text@3`
   与推演一致，**delete 面正确**（abc/ghi 保全、caret 居中）。

边界归一推演给 `_zwCeParagraphMutate` ①boundaryText 分支
（part01 :10633-10671）输入 host=[abc,ghi]、caret (abc,3)：
head1="abc"、tail1=""、headBlock1=<p>abc</p>、sep1 收 ghi → 期望
`<p>abc</p><p>ghi</p>`——**实测 `ghi<p>ghi</p>`（abc 消失 + ghi 重复）与
推演不符** → 病灶在 delete→split 之间的 live 重读面（caret_range 重锚
①登记表/②sel-path 补偿域）或 D3 世界同步回灌（el.innerHTML 整树换代，
mutation op 流零记录通道）。ops=[mk:p|mk:p] 与 ①分支双 p 创建吻合；
mutation 包装对引擎内部节点家族可能漏记（inserttext 有 mk:b 而 ip 无
i:/r: 移动记录，包装覆盖面不全已知）。

## 相 F 探针增强（gitignored zz-probe-knife11-v2.html，非刀体字节）

在并行轮相 D 修复（assert_true(false) 数据通道）+ 相 E 全行 dump 基础上：

- `_rawR()`：engine `_getSelection()._ranges[0]`（caret_range 重锚后真值）
  dump + testDiv/document contains 三面分家检查——页面 getSelection 与
  engine 选区分家即 D3 域伪锚实锚。
- split_at 包装扩展：入口 `hostIn=<innerHTML> raw=<engine range>` + 出口
  `FALLBACK-out hostOut sel raw`——拆分前后宿主树与引擎选区同框。

批脚本 `.acceptance/dps/batch-knife11-v2-probe-e.sh`（post-e 日志名，
SHIM-DRIFT 门锚 d74e396e 恒一），待门禁批+电池静默后运行。

## DC-4 修复设计（相 F 取证后施工，本会话施工域 per 让位记录）

1. **容器标签**（独立于病形，可直接确定）：Chrome collapsed→separator /
   range→container-copy 规则的 fallback 面——delete 前导分支
   （part06 :4769 else 臂）向 `__zw_ce_paragraph_split_at` 传 blockTag
   （宿主标签 copy，`_zwIpWrapTag` 同构：`_zwJusTag(host2)`）；part01
   `split_at`/`_zwCeParagraphMutate` 加第四参，①臂 boundaryText
   headBlock1/sep1 与元素边界分支 sep1 用 `blockTag || sepTag`。
   collapsed 中支不传参（sepTag='p' 不动，rich 12 键已绿面保持）。
2. **拆分病形**：待相 F 数据定位（live 重读错锚 vs 世界同步回灌），
   根因落 `_zw_ce_caret_range` 重锚域或 fallback 传参面，独立定谳后
   最小修。

门禁批 + battery2 出数后本会话另做：judge 红线/F2P 账独立复核、
goal doc 控制面勘误句（knife11 落地提交正文方向反写，per
2026-11-07-knife11-landing-verify-399090bd.md）。

## 相 F 取证回执（第二核验方读数，16:3x）

probe-e（knife11-v2-probe-post-e.log，16:27 完成，锚 d74e396e）Fail 通道
数据完整落地，§2 两嫌疑可裁定：

- **STAGE-D 496/499**：delete 面**正确**——`delete rc=true html=abcghi
  sel=#text@3..#text@3 tt=["b"]`（abc/ghi 保全、空 b 摘净、caret 居中、
  (a) 捕获链存活）。与 §DC-4 静态追踪推演逐位吻合。
- **ip 单步病形确认**：TRACE `hostIn=abcghi → hostOut=ghi<p>ghi</p>`
  ——split_at **入口树正确**，破坏发生在 `_zwCeParagraphMutate` ①臂
  单步内（非 live 重读错锚、非 D3 异步回灌）；ops=[mk:p|mk:p] 双 p
  创建而 hostOut 仅存一 p——headBlock1（p[abc]）消失 + ghi 复制重影，
  最终 sel=P@0 与 nr1.setStart(sep1,0) 一致。
- **头号嫌疑 = plain pending 节点跨域语义**（同 ⓪ void 臂注释 R54xx
  先例：innerHTML 解析 plain 子 appendChild 序列化拷贝不摘原父，
  zz-probe-void2 M 臂 sepKids=1/contKids=1 双现实证）：探针 testDiv
  系 innerHTML 建树全 plain pending，①臂 mv1h/mv1t/removeChild 直用
  宿主桥原语命中拷贝面——**与 multitest 语料同构**（browserTest 同
  innerHTML 建树），496/499 mt 红同根。
- **对照线索**：同树 delete 步自身移动/摘挂全部正确（stripWrappers
  路径）——bisect 入口 = 逐原语对照 delete 用与 ①臂用的宿主桥差异
  （_zwJus 包装 vs raw appendChild）。
- `raw=noraw`：`_rawR()` engine 真值采样未捕获（访问路径失效），
  若 §2 定谳需引擎侧 range 真值须先修探针。

容器标签（§1 blockTag 穿参）独立于病形可直接施工；病形修复落宿主桥
plain-node 摘挂共享路径面则与 editing 域解耦（fix-gate 分批验收）。
v2 落地（dps-unset 门）与此完全正交，按 p2f 笔记 §4 推进不受阻。
