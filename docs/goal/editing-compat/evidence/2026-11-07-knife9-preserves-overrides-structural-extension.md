# 第九刀——preservesOverrides 快照门扩展至结构命令族（2026-11-07，part06 cmdPresOv）

fbq 轮收口（7ab68347f）后的续刀：mt 残 Fail 键普查归因 → fbq4 快照包裹门（part06 cmdIsFbq4）
扩展至全部 spec preservesOverrides:true 的结构命令。mt 终判 **VERDICT OK：F2P=180 / P2F=0 /
lost 0 / rows 10092 键恒一**（F 416→236），较 mt2（F2P=93）再翻绿 87 键，零回归。

## 普查归因（mt2 残 330 raw Fail 键，judge 键级 323，同键跨窗 last-wins 口径差 7）

by last-command：inserttext 249 / insertText 29 / insertparagraph 17 / 其余各 5-10。
fbq4 相关链（has-fbq4）仅 26 compare + 1 qV + 1 qS——四命令面 mt2 已收干净。主簇：

- **40 键 `qV(style)-after` 结构命令族**（census 按 last-command 口径；按链首结构命令归族实为
  80 键，见终判分解）：`[值命令(style), 结构命令(, inserttext)]` 链的 qV(style) after——
  与 fbq4 包裹病同根：结构命令 spec 均 preservesOverrides: true，apply 重锚选区后 override
  惰性 staleness 判定误清 → inserttext 裸打 → qV 读 ""（Chrome 录音保留值）。
- 172 键 inserttext compare 族（typing 壳 caret 落位挂账，难啃，未动）。
- 29 键 insertText（styleWithCSS+forwarddelete styled 壳重建族，F6 域，未动）。

**spec 侧坐实**（vendored implementation.js 全图）：preservesOverrides:true = formatblock /
indent / inserthorizontalrule / insertimage（fbq4 已裹）+ insertorderedlist /
insertunorderedlist / insertparagraph / insertlinebreak / justifycenter / justifyleft /
justifyright / justifyfull / inserthtml / forwarddelete / outdent；值命令/style 命令/
inserttext/stylewithcss 等 false（缺省）。

**shim 侧五臂审计**（选区移动 → staleness 红源）：ilb `__zw_ce_line_break_mutate`（part01:10775）、
justify `__zw_ce_justify_apply`（part01c:1351）、iol/iul `__zw_ce_list_toggle`（part01e:1549）、
inserthtml `_zwExecCmdApplyInsertHtml`（part06）全裸奔；insertparagraph 双分支——ipara_apply
自管（part01f:1530 TakeRaw + finally Reanchor/PutRaw），fallback `__zw_ce_paragraph_split_at`
裸奔（census ip qV 5 键红源）。

## 刀体（part06 三笔，快照包裹机制零新增原语）

1. **快照上提**：fbqOvSnap/fbqTt/fbqTtoff 三变量组（原在 inserttext 臂与 fbq4 臂之间）上提至
   delete/forwarddelete 臂之后、insertparagraph 臂之前——包裹区覆盖 ip/ilb/justify/iol/iul/
   value/inserthtml/inserttext/fbq4 全部臂位（每臂自有 cmd 门，单 exec 仅一臂运行）。
2. **门扩展**：`cmdIsFbq4` → `cmdPresOv` = fbq4 四命令 + iol/iul/ip/ilb/justify 四向/inserthtml
   共 13 命令。排除面成文化（final2 回归刀教训全数落注释）：delete（spec false 本应清）、
   forwarddelete（spec true 但 F6 apply 自管，外层双重处理 = final2 43 行翻红同病）、
   inserttext/值命令/style 命令（spec false 自记自还）、outdent（spec true 但无实应用臂无重锚）。
3. **回写重命名**：presOvSnap/presTt/presTtoff（PutRaw + ReanchorOverrides + typing 双数组，
   语义与 fbq4 版逐字同）。insertparagraph ipara 分支外层+内层快照同态，PutRaw/Reanchor 幂等。

## 验证链（三批 + 复核）

- **探针批**（batch-fbq-knife9-probe.sh，05:27 二进制，shim md5=48eb0a3c，marker cmdPresOv×4/
  presTtoff×2）：formatblock 4894P/152F 与 legs5 逐字恒等（fbq4 零位移门）；iol 1759P/2F 持平
  nq7（无新红）；ilb 1641P/50F / jc 6126P/77F / ihtml 1446P/119F 全 rc=2 完跑（三页无历史
  基线，P2F 由 mt 键级 judge 权威判）。
- **mt 终判**（batch-fbq-knife9-mt.sh，05:48-06:18，judge vs batch-nq7-mt-post.log）：
  **rows 10092→10092 / P 9676→9856 / F 416→236 / F2P=180 / P2F=0 / post-only 0 / lost 0 →
  VERDICT OK**（raw 19,696P/476F）。
- **mt2→knife9 直比**（双红线第二证）：F2P=87 / P2F=0。87 分解全落结构命令族：
  **qV 80 = 8 命令 × 10 键**（ihtml/ilb/iol/ip/iul/jc/jf/jr 各 10，census 40 键族全清 + 同族
  另 40 census 口径遮蔽键）+ **compare 5**（ilb typed-carry）+ **qS 2**（ip）。fbq4 面 93 键
  逐键无位移（fb 零位移探针 + 直比 P2F=0 双证）。
- **enginelib**：九红名册恒一（ROSTER OK，knife9 树复核）。

## 残账

mt 残 236 F 键 = 172 键 inserttext compare 族（typing 壳 caret 落位挂账主体）+ insertText
styled 壳重建 ~29 + insertparagraph compare 12 + 零散。挂账四族中「qV/qI 读面 Chrome delta
12 行（fb 面）」与「indent 27F/img 23F 零散边界族」维持待刀；typing 链 caret 偏尾与 inserttext
compare 族同源合并观察。
