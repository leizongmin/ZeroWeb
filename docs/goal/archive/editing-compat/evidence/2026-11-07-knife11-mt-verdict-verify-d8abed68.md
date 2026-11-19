# knife11 sep 刀 mt 终判核验方独立复核（d8abed68，2026-11-07 13:25）

## 终判绑定

census 轮终判批（batch-knife11-verdict.sh，12:51:39 起跑 head=98b1e5296，
13:22:10 ALL-DONE）：release 重建 29.9s（含 sep 翻转四文件字节）→ build-anchored
md5 `ad212b5f8ed9422bb835190abb43487d` 断言过 → 双面制四 marker 全中（paren
old=0/new=3、noparen old=1/new=1）→ mt 全分母（FILTER=multitest，rc=2=Fail>0
常态，raw Pass 19900/Fail 284）→ judge-dps.py 红线 + 扩展红线门。

**核验方独立复算**（judge-dps.py 二跑 + 对 knife10c 基线预测集预核，13:24）：

- rows 10092→10092 / P 9926→9949 / F 166→143 / F2P **23** / **P2F 0** /
  post-only(P/F) **0** / lost **0** → **VERDICT OK**（批内判读与独立复算逐位同）。
- 树态绑定：批后四文件组合 md5 复测仍 `ad212b5f`（判读域=提交域，无漂移）。

## F2P 键级账：实翻 23 vs census 预测 24，单键偏差全归因

- **B 簇 2-cmd 12/12 全翻**：backcolor/bold/createlink/fontname/fontsize/
  forecolor/hilitecolor/italic/strikethrough/subscript/superscript/underline
  × `["insertparagraph",""] "foo[]bar" compare innerHTML`——纯 separator 因子，
  预测精确命中。
- **rich 3-cmd 11/12 翻**：同上十二值命令 × `[..., ["insertparagraph",""],
  ["inserttext","a"]] "foo[]bar"`——唯 **createlink rich 键未翻**（仍 F）。
- **零 UNEXPECTED**：styled 22 键全数留 F（探针已证 port 容器复制翻置免疫 +
  typed-fate 病），qS-after 2 键留 F（caret 入壳修复权属 v2 刀）——预测集
  外零位移。

**createlink rich 偏差归因**：期望形 `<p>foo</p><p><a href="…">a[]</a>bar</p>`
要求 typed `a` 续入链接壳——该语义不在 separator 层，而在 **typing 链 `<a>`
排除面**（part01f `__zw_ce_capture_typing_chain` :1121/:1126 `hasA → return []`，
knife10 期 Chrome delta 决策：链接不随 typing 续入）。此键与 styled/qS 残账同
属后续刀域，非本刀回归或漏翻。

## 后续门禁（gates 批，13:22:43 起飞，进行中）

batch-knife11-gates.sh：enginelib 名册（期望 6 红 = knife10c 漂移档）+
insertparagraph/iol/iul 三腿 pre/post 双采 judge（红线同口径）+ ipara qV 值面
硬门（`queryCommandValue("defaultparagraphseparator")` Fail 须 0）。注：enginelib
腿编译现树 = 翻转 shim + **part06.rs r3254 期望预改（div→p 双断言块）同树**——
r3254 在翻转+期望改组合态保持绿，名册恰落 6 红；该期望改与刀体同窗提交（分工
笔记 d8abed68 已约定）。

## 判读结论

红线四项全零 + F2P 键级账 23/24 命中且偏差单键归因闭合 → **sep 双面制刀体
mt 终判通过**，待 gates 批三腿 + 名册收口后落地提交。VERDICT 绑定
knife11-mt-post.log @ ad212b5f（组合字节态），落地提交须核 md5 再绑（push 前
pull --rebase 若并入新提交，绑定句按 mtrerun-anchor-vs-pushed-bytes-drift
纪律以 md5 断言而非 hash 表述）。
