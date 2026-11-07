# knife11 终判核验方复核（d8abed68，2026-11-07 13:22 VERDICT OK 出数后）

## 终判结果（batch-knife11-verdict.log，12:51:39 起飞 / 13:22:10 ALL-DONE）

- md5 锚 ad212b5f ✓ + 双面制四 marker ✓（paren old=0/new=3，noparen old=1/new=1）。
- mt 全分母：rows 10092→10092 恒一，P 9926→**9949**，F 166→**143**。
- 红线：**F2P 23 / P2F 0 / post-only 0 / lost 0 → judge-dps.py VERDICT OK**
  （本核验方独立复算同数）+ 扩展红线门 KNIFE11-REDLINE OK（批内联脚本）。
- mt VERDICT 绑定：flip 字节（四文件 ad212b5f@c9b7a2416 系基座 + census 轮 12:28
  落树翻转）。新基线 = knife11-mt-post.log（9949P/143F）。

## F2P 键级账：23/24，缺 1 键定谳（非 separator 病）

预测集（census）= rich 12 + B 簇 12 = 24；实翻 23 = **B 簇 12 全翻 + rich 11/12**。

- **未翻键**：`[["createlink","http://www.google.com/"],["insertparagraph",""],
  ["inserttext","a"]] "foo[]bar" compare innerHTML`（multitest-ab 与
  multitest.html?4001-5000 双窗均仍 F）。
- got vs exp：exp `<p>foo</p><p><a href="http://www.google.com/">a</a>bar</p>` /
  got `<p>foo</p><p>abar</p>`——**separator p 形已正确**，残缺面唯 typed 'a' 的
  锚携带：createlink 在 collapsed caret 不落 typing 态（bold 族 toggle 命令落
  `__zwTypingTags` 故 typed 'a' 带壳；createlink 为值命令无 typing 记账），且
  `__zw_ce_capture_typing_chain` 见 `<a>` 自返空（part01f:1121-1126，
  :2660 img/锚语料强制语义）。
- **定性**：新增残账 = createlink collapsed-caret 锚 typing intent 缺口
  （Chrome 录音语义：collapsed createlink 后 typed 文本承锚），独立刀域
  （typing 态需扩展锚形载荷或专用记账面），与 separator 刀无关、与 styled
  22+2 v2 刀正交。12 键的 createlink B 簇（无尾 inserttext）已全翻。

## styled 组零扰动确认（v2 刀前提成立）

- styled ip-compare 22 键仍 F（正则域实测，无 P2F/无意外 F2P）。
- qS-after 2 键仍 F（`<div><b>[abc]</b></div>` / `<div>abc<b>[def]</b></div>`
  两形，期望 caret 入壳后 bold=true）。
- 531/533/535 形（翻后仍绿的哨兵）无回归——P2F=0 全局佐证。

## 落地与配对

- shim 翻转四文件提交权 = census 轮（分工记档 2026-11-07-knife11-landing-
  division-d8abed68.md）；r3254 期望预改（js_dom_bridge_tests/part06.rs 双断言
  块 div→p）已在其工作区待同窗配对（svg 核验方预警承接）。
- 落地后门禁：census 轮 gates 批（batch-knife11-gates.sh，13:22:43 起飞
  head=fdfa770f5）enginelib 名册腿已出数——rc=101，**3793P/6F = ROSTER OK
  (6-red)**：r3254 在期望预改（div→p）编译入腿下保持绿，svg 核验方 12:41 预警
  被预改吸收，6 红外债档维持；ipara 单文件腿（insertparagraph/insertorderedlist/
  insertunorderedlist）13:24 起在飞，结果以 batch-knife11-gates.log 为准。

## 下一步（本核验方刀单）

v2 styled 刀（22+2）：port typing ④/⑤ 同构修（part01f:1734 finally 回写点
已锚）+ caret 入壳下行（:1731 collapse 目标）+ fallback 臂病（探针 op 流定谳）。
探针 zz-probe-knife11-v2.html 就绪，flip 后二进制（现 release 产物）直接可跑。
