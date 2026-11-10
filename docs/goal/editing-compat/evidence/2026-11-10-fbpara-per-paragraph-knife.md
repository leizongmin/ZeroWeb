# formatBlock per-paragraph 不合并刀 — evidence（2026-11-10）

刀体：`crates/engine/src/js_dom_shim/part01g.js`，提交 `963c791d3`（v1
`ede1126fc` 被 v1 门禁批 fb 主门打回后 amend）。目标：fb 残 21F 中 Chrome
per-paragraph 不合并族 4 行（blockquote compare ×2 + 同键 qV ×1 +
section-hgroup compare ×1，与 div>p 摆位刀同根不同形）。施工推演与核验方
独立对账见 `.acceptance/dps/fbpara-knife-notes.md`（会话协调信道）。

## 三机制（Chrome delta）

1. **同 tag 剔除臂收窄**：nodeList 成员为非 wrap-only SLC
   （p/h1-6/div/pre/address，`_zwFbqIsNonListSingleLineContainer &&
   !_zwFbqIsWrapOnlyBlock`）时不再剔除——放行尾环 SLC 溶解臂原位重 wrap，
   产出 Chrome 录音的嵌套换块形（`<blockquote><p>[foo]<p>bar</blockquote>`
   + `<blockquote>` → `<blockquote><blockquote>[foo]</blockquote>
   <p>bar</p></blockquote>`，:698）。裸 text 成员（:1784 列表爬升）与
   wrap-only xmp/listing 剔除保持（探针 K5/K8 锚全绿）。
2. **wrap sibCrit 自产邻居合并（v2）**：sibCrit 只对本次 apply 已产出的
   邻居块（fbqCreated 登记表）放行合并，div/p 维持恒 false。Chrome delta
   实为「不并入命令前已存在的邻块」：`[foo]<blockquote>bar</blockquote>` +
   `<blockquote>` 分开两块（:703）；而跨段部分选择的分段 wrap 合并出单块
   （`<p>[foo</p>bar]` + `<h1>` → `<h1>[foo<br>bar]</h1>`，:1734/:1739 ×6
   绿面依赖）。
3. **hgroup 直亲跳 fixDisallowed**：新块直父为 hgroup 时跳过
   `_zwListFixDisallowedAncestors`——保留 blockquote 落 hgroup 的内容模型
   非法嵌套不外提（`<section><hgroup><h1>[foo]</h1>...` + `<blockquote>`
   → `<hgroup><blockquote>[foo]</blockquote><h2>bar</h2>...`，:726）。

## v1 门禁批 ABORT 归因（教训入档）

v1（锚 `ede1126fc`，18:40:33 发射）fb 主门 ABORT：5022/24 ≠ 期望
5029/17，judge vs rb-fb **F2P=4（四目标翻键全兑现）+ P2F=7**。根因：v1
sibCrit 恒 false 的安全域断言「聚合走单块臂不经 sibCrit」错误——SLC 分组
条件（next 非 single-line container 才入 sublist）使各段自成 sublist 分段
wrap，聚合本由 sibCrit 合并实现；七面回归（h1 跨段合并形 ×6 + :708
qV-after ×1）。v2 登记表设计恢复七面、保持四翻键。判读留痕
`batch-fbpara-gate-v1.log` / `fbp-fb-abort.txt`。

## 门禁批 v2 全绿（20:01:11 FBPARA-ALL-DONE，锚 head 963c791d3 + part01g md5 3d3bc63d）

兼任新树红线重立（复绑批混树定谳后 probe/fb/mt 在 3c622f21a 域未复立）：

- 探针 zz-probe-fbpara **131P/0F**（三刀面行 + K1-K10 控制行，含 :1734
  合并面与 :708 qV-after 面；K5/K6 qV(dps) before='div' 为
  part06.js:5490 三元式初值 quirk，探针页首触无录音链携带，非刀回归）。
- fb 主门 **5029P/17F 精确** + judge vs rb-fb **F2P=4 恰族签名
  P2F=0**（:698 compare+qV-after、:703 compare、:726 compare）。
- mt **20182P/2F** + judge vs rb-mt F2P 0/P2F 0 VERDICT OK + 归因门零
  非 formatblock 签名翻键——ShadowRealm/s385/s386/DC-encoding 对 editing
  域零漂移实证，新树红线重立完成。
- singles iol 1759/iul 2061/ip 6989/ilb 1658/ihtml 1446 + exec×3 judge
  零漂移；enginelib 3828P/**7F** 名册双门恒一；battery 七腿
  420/880/1104/552/3456/2607/1455 全同数 F=0 T=0；binary sig 全在产物
  （div>p 摆位刀 + per-paragraph 不合并刀 + 漂移层标记）。

## 挂账更新

- **残 21F → 17F**：per-paragraph 族 4 行收口。残 17F 族级（fbp-fb 实测）：
  compare 照树红 ×2（cE=false span 容器行、h1 合并形 dps-p）+ qV·qI 读面
  ×15（table ×4 / h1 合并形 ×2 / dl qI ×2 / del · quasit · xmp · listing ·
  pre-div · h5 · h1 qV-before 各 ×1）。
- **part06.js:5490 qV(dps) 初值 quirk**（本轮探针定谳）：三元式
  `__zwDefaultParagraphSeparator === 'p' ? 'p' : 'div'` 初始 undefined 读
  'div'（spec 默认 'p'）；语料同面 before 值是录音链携带伪影。修复动
  part06 九锚之一，跨刀成本，挂账待立项。
