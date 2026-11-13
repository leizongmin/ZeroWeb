# ihtml 轴二行内壳选区刀——inserthtml 壳域解析臂（孤轮收养尾链）

- 日期：2026-11-13（刀体施工 01:59-02:26 原认领轮；22:0x 收养轮尾链）
- 刀体：`crates/engine/src/js_dom_shim/part06.js` `_zwExecCmdApplyInsertHtml` 宿主直子文本门 bail 修复 + 新增 `_zwInsertHtmlShellResolve` 壳域解析臂；钉测 `crates/engine/src/js_dom_bridge_tests/part112_ihtml_shell_sel_probe.rs`（四形双向）。
- 收养背景：01:59 认领轮施工至 02:26（刀体 234 行+探针 286 行+注册行全落树未 commit），02:27 构建测试二进制后机器 reboot，进程死亡无日志。22:0x rollover 轮按孤儿认领协议接手：刀体 commit（pre-commit-guard PASS 零发现）→ pull --rebase 并入上游 da93989e4→cf1d63b（他流 d12-d14/s415-s422 九笔 engine 域，与刀面 part06.js ~3214-3500 无 hunk 交叠；js_dom_bridge_tests.rs include 尾部冲突=两流 part111/112 编号撞号，按 part99 双文件先例拼接共存解决）→ 门禁批。

## 修复面（普查档轴二族落点）

旧守卫 `sc.parentNode === host` 对选区锚在行内壳/rawtext 容器内文本节点的场景整体静默 bail（NOOP 101 键主面的轴二子族）。新增臂：

- 端点归一：sc/ec 五形（同文本子/同壳内文本/host 直子文本/另一层壳内文本/host 子边界）统一到宿主子文本空间 [ws,we)。
- 拼接壳（xmp/script/plaintext/listing/style/textarea/pre）：选区全在壳内时壳内拼接保壳，round-trip 门（重建串=现串）不合形即旁路；pre 内块载荷单块解包。
- 其余壳（styled 行内壳）：壳内容全含于选区 = 整壳溶解、载荷落 ws；部分选区元素子/非文本兄弟不合形 → null 旁路（主路径零改动）。

## 验证

### 桥级探针（part112，22:05 验）

四形 4/4 绿：① styled 壳全含选区溶解（foo<span style>[bar]</span>baz + ab<b>c</b>d → fooab<b>c</b>dbaz + 跨壳起止/双壳同判）② xmp/script 壳内拼接保壳 ③ pre 块载荷解包（<pre>1[23]4</pre> + <pre>abc</pre> → <pre>1abc4</pre>）④ xmp markup 载荷 mutation 发射面。全 lib test_inserthtml 过滤跑 `4 passed; 0 failed`。

### 语料门禁批 batch-ihtml-shell-gate-v1（ALL-DONE 22:38:26，~33min 五腿全绿零 ABORT）

- 锚：head 20f3df096、bin 92485383b8c191b58e2ce8a9c1d0ace5（特征串函数+调用点双验）；基线 E1 r2 post（e1-*.log @ffdcd28af bin 788d4b10），上游 d12-d14/s415-s422 已并入树，flip 归因按命令族签名先分家族再定刀体因果。
- L1 ihtml 腿：**F 119→106，F2P=13 全落轴二族**（styledspan 8：全含/跨壳起止/双壳溶解形；rawtext 2：xmp/script 纯文本拼接；pre 3：块载荷解包含 collapsed/全选同臂），P2F=0，lost=0，rows 1565 全对账（rc=2 为失败计数退出非截断），judge OK。
- L2 ab 腿：10092/0/0 严格 wall=236s judge OK。
- L3 mt 腿：rc=2 P=10090 F=2 T=0 wall=1626s，judge **全零漂移**——rows 10092→10092、F2P=0（ihtml=0/other=0）、P2F=0、po_fail=0、lost=0、残红恰 2（X37 维持）；468 个 inserthtml 键全 P 面零翻越，跨命令面零回归定谳。
- L4 enginelib：3918 passed / **7 failed 名册逐名恒一**（rc=101=失败计数退出；passed 3911→3918 = +4 本刀探针 + 3 上游新钉测参与跑）。

## 已知非门红点（不挂本刀账）

1. **rawtext markup 载荷语料 4 键残 F**（xmp/script × swcss 双态 `<b>abc</b>` 载荷）：本刀 rebuild 串过 `host.innerHTML` 解析环时被 rawtext 语义转义（got `<xmp>f&lt;b&gt;abc&lt;/b&gt;o</xmp>`），Chrome 录音为 DOM 节点插入产物（`<xmp>f<b>abc</b>o</xmp>` 含真元素）。probe ④ 钉的是 mutation 发射面（镜像 parser apply_mutations_to_html 保 markup，与发射串恒等）；live shim 序列化面与镜像 parser 对 xmp 内容语义两宇宙分歧——同 2026-11-12 d10 桥层宇宙面定性，parity 收口需 Rust parser 侧事实刀，不在本刀范围。
2. **styled span 残 18 键**：局部选区（壳内部分文本选中不合「全含溶解」形）、更深嵌套、壳带属性等不合形旁路维持 F——后续形扩刀候选。
3. **caret 落壳内精度**：shellMode 下 caret 文本锚走 host 子级 kTextLen 累计定位，壳内落点经 element offset 语义（子索引≠字符偏移）可能失准，try/catch 静默；语料 compare innerHTML 不覆盖 caret 面，后续 caret 精度刀候选。

## 门禁批日志

- 批脚本 `.acceptance/dps/batch-ihtml-shell-gate-v1.sh`；批日志 `.acceptance/dps/batch-ihtml-shell-gate-v1.log`；腿日志 `ihtml-shell-{ihtml,ab,mt,enginelib}.log`。
