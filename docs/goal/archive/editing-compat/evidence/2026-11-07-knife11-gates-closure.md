# knife11 门禁收口——gates 批 ABORT 修复补跑 + 单文件腿三腿 VERDICT OK（2026-11-07）

## gates 批原跑 ABORT 归因（脚本 bug，非刀体）

census 轮 `batch-knife11-gates.sh`（13:22:43 起飞）完成 post 半区后 ABORT：
`ABORT pre md5 mismatch got=7cda92fe`。根因 = **md5 口径 bug**——`md5snap()` =
`md5sum $SHIM4 | md5sum`（清单含文件名），`PRE_EXPECT` = HEAD blob 内容直拼
md5（无文件名），两布局恒不等，pre 态逐文件核验必然失败。post 半区已出数：

- enginelib 名册：3793P/6F = **ROSTER OK(6-red)**（6 红外债档
  knife10c 漂移档恒一；r3254 在期望预改编译入腿下保持绿）；
- post 腿：insertparagraph 6987P/208F/7195 rows、insertorderedlist
  1759P/2F/1761、insertunorderedlist 2061P/4F/2065。

pre 腿、三腿 judge 与 ipara qV 硬门细查未跑，stash pop 复原干净
（ABORT 分支自带 pop；tree md5 复测 ad212b5f）。

## 修复补跑（本会话 batch-knife11-pre.sh，14:09:59-14:27:25）

- 口径修复：pre 核验改**逐文件 `git show HEAD:$f | cmp`**（语义直接），二进制
  marker 双检保留（pre 期望 paren old=3 new=0 / noparen old=2 new=0，
  实测逐字命中；复原后 rebuilt old=0 new=3 / noparen old=1 new=1）。
- stash 纪律：slots before=4 / after=4（无 WARN），临时 stash
  （406e160d）pop 后即弃，常驻 4 槽（typing-style-round-1102 等三笔历史
  WIP + ghost-br）未动。
- **三腿 judge（pre → post，judge-dps.py 元组口径）全 VERDICT OK**：

| 腿 | rows | P | F | F2P | P2F | post-only | lost |
|---|---|---|---|---|---|---|---|
| insertparagraph | 7195→7195 | 6987→6987 | 208→208 | 0 | 0 | 0 | 0 |
| insertorderedlist | 1761→1761 | 1759→1759 | 2→2 | 0 | 0 | 0 | 0 |
| insertunorderedlist | 2065→2065 | 2061→2061 | 4→4 | 0 | 0 | 0 | 0 |

单文件腿**零翻转移恒等**——与 census「显式 setter 链翻置免疫」预测精确吻合
（insertparagraph 语料 464 显式链 + ambient 双收域，行级逐键恒等）。

## ipara qV 值面硬门归因（8 Fail = 既有共享 F，非本刀）

post log `queryCommandValue("defaultparagraphseparator")` Fail 行 8 条，全部落
**显式 setter 链**（`["defaultparagraphseparator","div"|"p"]` 前置）；值面代码
（part06:5015 getter + :4893 setter）本刀**零字节改动**（diff 仅注释），
`__zwDefaultParagraphSeparator` 全局态唯 setter 可写。pre/post judge rows/P/F
全恒等 + F2P/P2F/post-only/lost 全零 ⇒ 8 键属 **shared-F**（pre 同败），
非本刀引入。病形 = 特异宿主 qV 断言（`<script>` 宿主 / blockquote 嵌套 p /
ul>li>div 造型），归 insertparagraph 语料 208F 既有桶，DC-4 残账普查域。
explicit-setter compare 196 Fail 同判 shared-F（表造型/br 造型等既有病害族）。

## census 验证面补全：ambient 敏感腿电池（batch-knife11-battery2.sh）

census 验证面要求 join/wss/ilb 腿零位移，gates 批未含 → 本会话补 7 腿电池
（post 态现役二进制，marker 0/3 断言先核）。v1 轮（batch-knife11-battery.sh）
误走 `testharness-editing-run-one`（run 面子命令）致 7 腿全 0 行——other 面
文件 run 通道不识别的**通道错配假象**（learning：零 Pass 行=工具页/通道坏
先核），v2 改 `testharness-editing-other-one` 复跑，14:32:48-14:45:58
**七腿全 OK 零位移**：

| 腿 | rows | P | F | T | 判定 |
|---|---|---|---|---|---|
| insertparagraph-in-inline-editing-host | 880 | 880 | 0 | 0 | OK |
| insertparagraph-in-editing-host-cannot-have-div | 420 | 420 | 0 | 0 | OK |
| insertparagraph-with-white-space-style | 1104 | 1104 | 0 | 0 | OK |
| insertlinebreak-with-white-space-style | 552 | 552 | 0 | 0 | OK |
| join-different-white-space-style-paragraphs | 3456 | 3456 | 0 | 0 | OK |
| join-different-white-space-style-left-line-and-right-paragraph | 2607 | 2607 | 0 | 0 | OK |
| join-different-white-space-style-left-paragraph-and-right-line | 1455 | 1455 | 0 | 0 | OK |

rows 与历史稳定账逐腿恒一、F=0 不变量保持——ambient 产线块面翻 p 对
other 面 insertParagraph/join/linebreak 语料**零扰动**（join 族 delete/join
路径不产 separator 元素；wss/ilb/inline-host/chd 宿主臂不落 sepTag 缺省面或
期望形态与翻置无交集），census「单文件腿零位移」验证面闭合。

## 判读结论

knife11 sep 双面制刀体门禁全绿：mt VERDICT OK（三方法独立复核一致）+
enginelib 6-red 名册恒一 + 单文件腿三腿零翻转 + qV 硬门 Fail 全数
shared-F 归因闭合 + ambient 敏感腿七腿电池零位移。落地提交（五文件同窗：
四 shim + part06.rs r3254 期望预改）具备条件。
