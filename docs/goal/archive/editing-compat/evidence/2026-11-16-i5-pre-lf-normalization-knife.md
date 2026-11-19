# I5 pre 换行归一刀（2026-11-16）：劈裂空右块垫 br 臂 white-space 感知

## 输入与目标

C5 轮 census 真红 3 键中的 I5 单键：insertparagraph.js :367 `<pre>foo&#10;[]</pre>` dPS div，exp **三选一**（`<pre>foo<br>\n</pre>` / `<pre>foo\n<br></pre>` / `<pre>foo<br><br></pre>`），got `<pre>foo\n<br><br></pre>`。语料注释原话：trailing linefeed is invisible——尾字面 `\n` 在 pre 面**已持行**（隐行），单 br 已足垫新空行；现役垫底臂在「nxt 出容器」时无条件再插第二 br，pre 形产三行。相邻形 `foo[]&#10;`（:345）现役绿仅内嵌形红，故收窄为白空间感知跳垫。

## 修复（part01f 一处，免视守卫）

`__zw_ce_paragraph_split_at` pre/address/listing 支路垫底臂（:2289 区）：插第二 br 前加 `heldI5` 守卫——`_zwJusWhiteSpaceOf(container)` ∈ {pre, pre-wrap, pre-line} **且** `br.previousSibling` 为 Text 且 nodeValue 以 `\n` 结尾 ⇒ 左块尾字面 `\n` 已持行，跳第二 br（caret 维持 (node, offset+1) 不动）。address 等折叠 white-space 面尾 `\n` 不持行，双 br 维持原判——守卫是合取门，非 white-space 全跳。

## 钉测（part123 五形）

- `i5-pre-lf`：:367 I5 键——exp **取定形** `<pre>foo\n<br></pre>`（三选一合法集内取一，innerHTML-only：语料 exp 三形均无 selection marker，不锁 caret，见 C5 轮同款教训）。
- 控制四键恒一：:340 `foo[]bar` 拆形、:375/:380 相邻 pre 形现役绿（`foo[]&#10;`/`<pre>foo[]\n</pre>` 族）、:345 `foo[]&#10;` 尾形。caret 实测捕获锁定（这些面语料锁 caret）。
- 断言回路复用 part122 `pin123` 包装（err/ok/out 失配 throw；expSel 非 null 才核 caret）。

## 验证

- enginelib 全量 **4053P/0F**（4052 旧 + part123 新；漂移含 gtr 流 part02 diag 测试同期入树，逐名非本刀域）。fmt 本刀文件净（workspace 既有红 dom/tests_9 属 gtr 流域自修）；clippy -p zero-engine -D warnings 绿。
- **语料门禁批 i5p1**（01:14:11 发射→02:23:53 ALL-DONE，锚 `76918f601`，bin `30201a904465cc2b9456e9ca24dd9254` ksig 2/2 全程恒一；发射后 CC 会话 429 断链，批经 systemd 单元独立完走，02:3x 接管轮判读）：
  - **ip A/B 双腿**：7195 行 **7183P/12F/0T**（wall 994s/1001s）——F2P=1 恰 I5 键（`?1-1000 :: [["insertparagraph",""]] "<pre>foo&#10;[]</pre>" compare innerHTML`），comm 对 c5p1 基线（7182P/13F）**零新增红**，A/B 红集逐行恒一（接管轮独立 fail-set diff 复核，非仅 tail 脚本自述）。rc=2 系 make 层退出码（editing-gate 纪律：门=计数＋judge 双证，均已过）。
  - ab 10092/0/0（250s）／mt 10090/2 X37 双键恒一（1585s，残 Fail 双键逐行核=`queryCommandState("subscript")`+`queryCommandValue("formatblock")`）／ilb 1658/33/0（36s 零暴露）／enginelib strict 4053P/0F（15.9s）。
  - 批后 `pull --rebase` 并入他流 column-rule 刀（88b6ab519）重放为 `cfd1a06af`：两树 crates/ 域 diff 仅 style-system apply/registry + engine part02 diag（他流域），编辑域字节恒一——批判读对现行树传递闭合。
  - census 真红 **3→2**（I5 清偿，残 I4 CE=false 列表项块化 2 键维持域敏感缓）。批脚本 `.acceptance/dps/i5p1-tail.sh`（md5 `ad68f2af` 冻结），log `i5p1-{ip-a,ip-b,ab,mt,ilb,lib}.log`（ephemeral 不入库）。
