# C5 根层裸 br 刀（2026-11-15）：宿主唯一子 div 选区删净解包

## 输入与目标

C4 垫刀轮 census 重述真红 5 键中的 C5 两键（[控制面](../../editing-compat.md) Active 行）：insertparagraph.js :2102/:2107 `<div>[abc]</div>` dPS div/p，exp `<br><div><br></div>`/`<br><p><br></p>`。语料注释（:2098-2101）原话：div 应解包、`<br>` 置于其位、新段建于其后。spec 参考实现（implementation.js insertParagraph）对该面产双空块就地拆形——语料期望系 Chrome 录制 delta，忠实移植路无法覆盖，需 break-out。

## 根因链（四层，逐层探针定谳）

C5 面路由：`__zw_ce_ipara_wants()`=true（startContainer 文本归一提升到 div 块，元素 container 接管）→ part06 ip 分支进 `__zw_ce_ipara_apply`（part01f 忠实移植）→ 非 collapsed 走 deleteSelection 前导 → `__zw_ce_dissolve_emptied_div_tail`（⑦ A1 空壳消解臂）。四层 miss：

1. **消解子检查 bail**：初版假设「delete 后 div 全空」——探针 C5PROBE 实证**非零子**：deleteSelection 全删空后归一化垫占位 br（`<div><br></div>`，blkKids=1），原稿 `!kids.length` return false 直接 bail（mt 483/485/487 面有 `<b></b>` 幸存壳故能命中，纯文本面命中不了）。
2. **持行垫 br 误判可见**：撤零子 bail 后仍 bail——`_zwJusIsVisible` 对块级父唯一子 br 按「非冗余 br」判可见（`_zwJusIsExtraneousLineBreak` 块级父唯一子臂＝持行语义，C4 刀同形状），kids 循环把它当幸存内容。
3. **壳爬升活区重锚必败**：`_zwVaGetActiveRange` 返回 `sel._ranges[0]` 活区（part01d :47），deleteContents 重锚后 ar0.startContainer 不再是原文本节点——全删净面无幸存壳接锚，a1Shell 爬升得 null 即 bail。mt 483 能命中靠 `<b></b>` 壳接住重锚；K3 `{abc}` 元素界面同形捕获。C5/K3 的区分信号只剩 delete 前的边界类型（双 CharData vs 元素界）。
4. **① 臂把垫 br 包块**：消解命中后 ipara 续走 `paragraph_split_at(host, null)` ① 宿主级臂——探针 psLog 实证 (host,[br]) 状态下 br 被视行内容包块（`htmlFinal=<div><br></div><div><br></div>`），空宿主 caret 才走 A2 裸 br 垫＋sepTag 新块。

## 修复（四件套）

1. **part01h 消解 kids 循环持行垫 br 免视**：唯一子 br 形跳过可见检查（直父唯一子收窄，与 `_zwJusIsExtraneousLineBreak` 块级父唯一子臂同形状；多子 br 与壳幸存面维持原判）。
2. **dissolve 签名扩 `ar0Txt`**：两调用点（delete_apply 尾 :1426、ipara_apply 前导 :1933）在 delete 前捕获双 CharData 边界布尔，dissolve 用它放行壳爬升/尾包含两守卫。K3 元素界面捕获 false 维持原判；部分覆盖面由 kids 可见幸存/宿主级 sibs 守卫挡。
3. **ipara 续走尾随归一**：消解命中且宿主唯一子恰为垫 br 时移除并落 (host,0)，让 ① 臂 A2 空宿主径接管（垫 br 身份换成 ① 臂重垫标记 br，语料仅核 innerHTML 无观察面）。delete 调用点不经此（宿主留 `<br>` 即 Chrome 全删空形，delete/fd 语料无此面记录）。
4. **钉测 part122**：C5 双面（innerHTML-only，见下）＋双控制键（`<p>[abc]</p>` 不解包 :2117 现役就地拆形 caret P@0；宿主多块 :2132 现役绿形 caret DIV@0）。

## 关键教训：钉测断言必须逐字段核对语料契约

初版钉测给 C5 双面加了 `sel=DIV@0`/`P@0` 断言——语料 :2102/:2107 after 串 `<br><div><br></div>` **根本不含 selection marker**，compare 仅核 innerHTML，caret 未被语料锁。c5-div 的 DIV@0 通过纯属宿主与新块同标签巧合（实为 (host,0)），c5-p-sep 的 P@0 断言暴露矛盾。语料未锁的面不造约束：C5 双面钉测改 innerHTML-only（`expSel=null` 跳过），控制键面（语料 :2131 注释明言 caret 入新段）保留锁定。探针包装法（monkey-patch `__zw_ce_dissolve_emptied_div_tail`/`__zw_ce_paragraph_split_at` 记 args/ret/中间态）是本轮定位手段，探针测试已撤、不常驻。

## 验证

- part122 四形钉测绿（0.24s）。
- enginelib 全量 **4045P/0F**（4044 旧 + part122 新；part113-121 编辑钉测群零漂移——dissolve 双调用点共享路径改动无回归）。
- cargo fmt --all --check 绿；clippy -p zero-engine --all-targets -D warnings 绿。
- 语料门禁批 c5p1（23:05:33 发射→00:51:16 终态，锚 `7802cc494`，bin `d64934676e` 全程恒一，ip/ab/mt 乘数2）：
  - **ip A/B 双腿**：7195 行 **7182P/13F/0T**（wall 1141s/1062s）——F2P=2 恰 C5 两键（`?6001-7000 :: [["defaultparagraphseparator","div"/"p"],["insertparagraph",""]] "<div>[abc]</div>" compare innerHTML`），comm 对 v1 基线（c4p-v1-ip-{a,b}.log 7180P/15F）零新增红，A/B 红集逐行恒一。rc=2 系 make 层退出码（editing-gate 纪律：门=计数＋judge 双证，均已过）。
  - ab 10092/0/0（267s）／mt 10090/2 X37 双键恒一（1900s）／ilb 1658/33/0（39s 零暴露）／enginelib strict 4045P/0F（16.3s）。
  - 残红 13F 与 census 重述吻合：真红 3（I5 pre `<pre>foo&#10;[]</pre>` 1＋I4 CE=false ol/dl 列表项块化 2）＋窗伪影 10 除名项维持。批脚本 `.acceptance/dps/c5p1-tail.sh`（md5 `41c1e0c2` 冻结，发射后未改），log `c5p1-{ip-a,ip-b,ab,mt,ilb,lib}.log`（ephemeral 不入库）。
