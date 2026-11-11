# tq table partial-span formatBlock qV/qI 读面刀收口（2026-11-11）

## 刀体

commit **2efbddec0**（part01g 单文件，75+/31−，guard 隔离扫描代理 PASS 1 文件 75 行 0 发现）：

- **part01g `_zwFbqEdgeLeaf` 端点折叠域分派**（Chrome delta，VisiblePosition 归一边界点折入锚点同侧邻子树）：START 内区锚（0<o<len）前折 child[o] 子树首可见叶（行走域 [o,len) 早返），END 内区锚后折 child[o-1] 子树末可见叶（域 [0,o) 逆序早返）；**折叠域内无可可见叶时回落全域旧走法**；边界锚（o=0 ∨ o≥len）全域旧走法全保、文本锚直返恒一。
- **`__zw_ce_fb_value` / `__zw_ce_fb_indeterm` 端点 isEnd 贯通**：qI 漏透传会使 table 形 b1/b2 失配返 true 抵消 qV 收益（静态推演缺口，首版修复）。

目标域：formatblock.js :83/:88/:418/:423 `<tr data-start=1 data-end=2>` 部分跨列选区 4 行（双 dps × 双 fb 目标），qV-after 录音 "div"/"p"——本引擎终点 (host,2) 全域读首可见叶落 td1 内旧块爬回非 FB 名或 table barrier 返 ""，恒红。

## 折叠回落臂归因（TEMP-TRACE 诊断闭环）

首版折叠无回落臂，solo 预验 P2F=8：`{<p><p> <p>foo</p>}` 4 条目 × qV/qI-after。TEMP 插桩（part01g `__zwTqTrace` + 诊断页 zz-fbq-dbg-tq.html assert_unreached dump）实证锚形：apply 后新块含 `[" ","foo"]` 两文本节点，qV-after 范围终点为**内区锚 (div,1)**，END 折叠域 [0,1) 仅空白文本（`_zwJusIsVisible` 不可见）→ 返 null → qV ""。旧全域走法逆序不早返能越过空白找到 "foo" 爬回新块。

修复：折叠域 walk 返 null 且锚为内区时，按刀前同锚形语义重走全域（`walk(0, len, false, false)`——逆序不早返终值=域内树序首可见叶，逐字节保持刀前行为）；折叠仅在邻子树确有可见叶时生效。qV-before 通过面同轨归因：边界锚 (host,0)/(host,3) 全域逆序末赋值落空 p1（空 P 可见）爬回 "p" 恰录音——刀前行为原样保留。commit 前插桩零残留（`__zwFbTrace`+`__zwTqTrace` 双查入批特征串门）。

## 批前预验（rbv2-fb.log 为基线，solo）

fb 全页 5029P/17F → **5033P/13F**（rows 5046 恒一），judge F2P=4 恰白名单（table partial-span qV-after ×4 双 dps×双 fb 目标）**P2F=0 post-only 0 lost 0**；残 13F 与挂账三分表一致（compare ×2：cE=false span 容器 + h1 合并形 dps-p；qV·qI 读面 ×11：h1 合并形 ×2 / dl qI ×2 / del·quasit·xmp·listing·pre-div·h5·h1 qV-before 各 ×1）。日志 /tmp/tq-fb-pre2.log（分析 tq-pre-analyze.py）。

## 门禁批

batch-tq-gate-v1.sh @2efbddec0，00:08:48 发射（ensure_solo + flock + 十点 md5 锚 + 特征串门 + EXPECTED_HEAD）。锚：九点恒一沿推送树 + **bridge 75add5ae（s390 域复算）** + part01g 刀锚 3b9918c4 + part11-color 96e55a0e 漂移层锚。入站 rebase 5843bbd8d..f0a23470f 六笔他域提交（s390 form 指针 / individual transforms / css corpus .any.js / UBA LTR / wptfyi docs），autostash 字节保真九 shim 锚恒一、脏区与远端 hunk 不相交先核后 rebase。腿序：probe（f6bwin 2092/0 + k11 224/17 judge 零漂移）→ fb 主门 5033/13（judge F2P=4 恰白名单 + P2F=0）→ mt 20182/2（judge 零翻越 + F2P 非 0 须 formatblock 签名）→ 单五腿/exec×3 零漂移 → enginelib 7-red 双门 → battery 七腿同数 → binary sig。

## 判读（2026-11-11 终态化）

### v1+tail 门禁批（锚 2efbddec0，全绿）

- **v1**（00:08:48 发射）：probe 双窗过门（f6bwin 2092/0 + k11 224/17 judge 恒一 vs bfl）+ fb 主门过门（**5033P/13F**，rows 5046 恒一；judge vs rbv2-fb **F2P=4 恰白名单**——table partial-span qV-after 双 dps×双 fb 目标四行、P2F=0、post-only 0、lost 0，VERDICT OK）；mt 腿 ABORT（00:56，P=20114/2 ≠ 期望 20182/2）。
- **v1 mt ABORT 三角归因**（台账三条独立复算收敛，否证核验方初判「跨案状态链毒化 setup」方向）：①键集恒一（unique lost 0 / new 0，10,092/10,092）②per-key 状态零漂移（F2P=0/P2F=0 恒成立，F=2 恰 X37 双行既有签名）③缺 68 行 = multitest-ab.html 无窗页**首打块尾连续段**（rbv2 result 行 10024–10091，恰 ab→multitest.html 页界）——非行为漂移非覆盖缺失；首判「runner 退出 flush 截断」被三判否证（丢的是首打块尾非日志尾），指向 ab 页（全量单页 10,092 断言无窗）运行至尾部触页级预算/静默跳案（17 案签名 = delete/inserttext 系 × flex/table 部分选区已知慢簇；ZW_SCRIPT_TIMEOUT_MS/页预算机制待定谳）。tail mt 复跑全量 20182 过门 = 时序瞬态定谳，非确定行为漂移。
- **tail**（batch-tq-gate-v1-tail.sh，01:02:49 发射，同锚同门不放宽）：mt **20182/2**（judge F2P=0/P2F=0/lost 0 VERDICT OK + 归因门外来 F2P=0）→ 单五腿（iol 1759 / iul 2061 / ip 6989 / ilb 1658 / ihtml 1446）+ exec×3 judge 零漂移 → enginelib 3,833P/**7F**（7-red 名册双门恒一）→ battery 七腿 420/880/1104/552/3456/2607/1455 全 F=0 T=0 → binary sig（tq-knife ×2 + wrap-only/harvest/indeterm/boolean/fbqRet2 全中）。
- **判读 = v1（probe/fb）+ tail（mt 尾链）合并 VERDICT OK，绑定 2efbddec0**。残 13F 与挂账三分表一致（compare ×2：cE=false span 容器 + h1 合并形 dps-p；qV·qI 读面 ×11：h1 合并形 ×2 / dl qI ×2 / del·quasit·xmp·listing·pre-div·h5·h1 qV-before 各 ×1）。

### 复绑批（推送候选树 474338234，判读域终绑）

pull --rebase 并入 origin/main 3e2f0e417→cdfa1a16e 七笔他流提交（grid CSSOM shim 面接线 part06 +20/part03 +74/part05 +18/callbacks.rs +69、css values serialize_math、individual transforms、s391 form owner reset-on-move、DC-3/DC-2 docs、wptfyi cron）——入站触 shim/bridge 编辑运行时面，v1+tail 判读域 ≠ 推送候选树，按 fbpara/fbwrap 复绑先例全腿重立红线；刀体重放 **part01g md5 3b9918c4 逐字节恒一**（非刀域位移零）。批 mt 腿内置截断守卫（首跑计数失配 ∧ judge 零逐键漂移 ∧ lost>0 → 同门重跑一次；复现仍失配 ABORT 升根因刀）。

**复绑批 batch-tq-rebind-v1.sh ALL-DONE 05:02:22（03:15:48 发射，1h47m），判读域终绑 474338234**：锚复读（head 十点 + part03 d237e44f / part05 7a2db924 入站漂移层锚 + 静态串全中）→ probe 双窗（f6bwin **2092/0** + k11 **224/17** judge 零漂移 vs bfl）→ fb 主门 **5033P/13F**（rows 5046 恒一；judge F2P=4 恰 table partial-span qV-after 白名单、P2F=0、VERDICT OK——入站 grid CSSOM/css values/transforms/s391 域对刀域零扰动）→ mt **20182/2 首跑即过**（judge F2P=0/P2F=0/lost 0 VERDICT OK + 归因门外来 F2P=0；v1 疑似截断零复现，自动重跑守卫未触发）→ 单五腿（iol 1759 / iul 2061 / ip 6989 / ilb 1658 / ihtml 1446）judge 零漂移 → exec×3 零漂移 → enginelib 3,833P/**7F**（7-red 名册双门恒一）→ battery 七腿 420/880/1104/552/3456/2607/1455 全 F=0 T=0 → binary sig（tq-knife ×2 + 五串全中）。**VERDICT OK——tq 刀红线在推送候选树（含他流七笔组合态）全腿重立，knife10c/fbpara 复绑先例形**。

## 核验方补账（2026-11-11 00:58，旁观轮独立验证 mt ABORT）

tq gate v1 ABORT @mt（20114/2 vs 期望 20182/2）。judge 域逐行对账定谳：

- **−68 行 = 17 案 × 4 断言行，全部在 multitest-ab.html 无窗段**；窗化
  multitest.html 段 10092 行零漂移（F2P/P2F=0）——同语料双页跑，窗化页完好。
- 零 Timeout、零键集缺失、无 JS 错误痕迹 → 17 案在 ab 连续段**静默零发射**
  （setup 阶段异常被 harness per-case catch 吞掉），非引擎判定翻转。
- 17 案签名：delete/inserttext 系命令 × **flex/table 部分选区**形
  （`<div style=display:flex><span>…<b>{def` 跨 span、`<td>a<td><b>[b</b>…
  <td><b>d]</b>` 跨 cell）；multitest.js 中表格形为双实体（:168384/:168880）。
- **归因注意**：tq 批树含 rbv2 基线之后他流三笔 engine 提交
  （91ab28c56 source-phase imports、f0a23470f s390 form 指针侧表+shim 同树门、
  5843bbd8d css-parser values trig）——mt 漂移归因须在 tq 刀 vs s390 shim 门
  之间 bisect（先 multitest-ab 单页复跑 tq 树证 intra-page 状态链，再倒换
  变量定位毒化源）。窗化段零漂移支持「连续段跨案状态链毒化 setup」方向
  （窗界重置掩盖链效应，与 editing-corpus 窗界伪影先例同构）。
- fb/probe/单文件/exec 各腿过门数据不受影响（本批红线在 mt 腿之后未跑）。

### 复绑链 v3→v7（组合态前进重立红线，判读域终绑 13535525b，2026-11-11 晨收口）

v1 复绑后他流三波入站（s396→4a0d1ba65、s397/Typed-OM 相对色/grid var()→718e0f0e6、position-area 文法基座→13535525b）均触运行时面，按复绑先例逐波全腿重立；judge 基线 rbv2-\*（fbpara 复绑 v2 批）+ bfl-probe-win/k11（boolface 批），同门不放宽。

- **v3 双 ABORT（batch-tq-rebind-v3，08:3x 发射 @d85dc7114）**：①fb 主门自 ABORT——修复页实测 5042/4 vs 预测 5043/3，10 伪影键 9 翻绿、R7（idx171 pre-div qV-before）未翻；fb13 预研预证 Chrome 录音会话在 idx171 前自身复位（非 exec 链），单门版静默执行模型不可复现 → R7 归录音协议伪影（#37 判据）除名挂账，fb 门改 5042/4、白名单 10→9；②rollover pull 并入 s396 → HEAD 4a0d1ba65，mt 腿起将 rebuild 混合二进制，孤儿发射轮/监视轮清场后停批，probe 双腿（旧二进制）判读不跨批传递。
- **v4 静默死亡定谳（08:54:46 发射 @4a0d1ba65）**：probe 双窗过门 → fb 计数过门（5042/4 恰期望）→ judge 基础度量真 OK（P2F=0/lost 0/rows 恒一）→ **白名单断言 False → heredoc `|| exit 1` 无标记静默退出**（09:13，mt 发射前）。定谳＝判读规格错、行为零漂移：v4 白名单 9 键**漏计 4 个 tq 刀键**（rbv2-fb 为 tq 刀前基线，该 4 键 vs 刀前基线必现 F2P），F2P 实测 13=9+4 恰正确集。修复＝白名单 9→13 键（逐字节取自 v4 实测日志）+ heredoc 失败路径补 ABORT 标记。
- **v5 入站漂移自弃（09:26 ABORT）**：重锚 718e0f0e6 后 09:17:40 pull fast-forward 入站 5 提交（s397 module parse→part01c+6/es_module/webview；Typed-OM 相对色→part10-typed-om+55/part11-computed-color+84；grid CSSOM var()→grid_cssom.rs+40）——launch 锚捕获 part01c md5 漂移自弃；域净（part01g 刀锚 3b9918c4 恒一）。v6 重锚 part01c a3ae1b41 + **5 点入站漂移层扩锚**（part10-typed-om/part11-computed-color/grid_cssom/es_module/webview）。
- **v6 停批 ABORT-EXTERNAL（09:30:02 发射 @718e0f0e6，09:44 接手轮执行）**：在飞中 rollover 轮开工 pull（09:33）并入 **13535525b**（position-area：position_area.rs 新建 +470 / **js_dom_bridge.rs +14——锚表内 6fcefc20→27f402be** / callbacks.rs +44 锚表盲区新文件）。f6bwin 腿跑旧二进制过门 2092/0；k11 腿 make rebuild → **混合二进制**（k11 224/17 judge OK＝probe 域零漂移实证，判读不跨批传递），fb 腿 3min 处接手轮停批（v3② 同型先例；杀序 脚本→腿树→fd 穷举锁零残留）。
- **v7 全绿 ALL-DONE（09:43:55 发射 @13535525b，11:24:17 ALL-DONE，墙钟 1h40m 含 foreign 租户争抢窗）**：bridge 重锚 27f402be + **2 点新锚 callbacks c9a06c39 / position_area 3c72fbbd**（补锚表盲区）；锚复读 OK → probe f6bwin 2092/0 + k11 224/17 judge OK → fb **5042/4** 恰期望（judge rows 5046 恒一 P 5029→5042 F 17→4 **F2P=13 恰集**＝9 伪影+4 刀键，P2F=0 post-only 0 lost 0）→ mt **20182/2** 恰期望（judge F2P 0/P2F 0/post-only 0/lost 0，X37 2F 稳定）→ singles iol 1759/iul 2061/ip 6989（残 206F 恒）/ilb 1658/ihtml 1446 零漂移 → exec×3 零漂移 → enginelib 3846P/7F 名册双门 OK → battery 420/880/1104/552/3456/2607/1455 全 F=0 T=0 → binary sig 六串 + tq-knife ×2。**VERDICT OK——判读域终绑 13535525b（含他流 s396/s397/typed-om/grid-var/position-area 组合态）**。日志 batch-tq-rebind-v7.log（tqr7-\* 前缀，本地 .acceptance/dps）。

**链教训三条**：①judge 白名单期望须含「刀键 vs 刀前基线」的必现 F2P（v4 漏计静默死亡先例；heredoc 失败路径必须带 ABORT 标记防静默退出）；②复绑批锚表须覆盖入站 range 的全部 crates/ 运行时面**含新增文件**（v5 五点扩锚、v7 又两度补锚实证）；③rollover 开工 pull 与在飞批互斥——开工先查在飞批（flock/pid/日志终态标记），在飞时先停批或延 pull，launch 锚断言不复核中途入站（v6 毒化实锤）。

**残账重组与 R3/R4 定谳（返修 r1 补链，2026-11-11 非作者复核）**：复绑链判读（修复页 F2P=13＝9 伪影+4 刀键）实证标准页 13F 中 9 键＝窗界伪影（del/quasit/xmp/listing/h5 单块形 ×5 + h1 合并形 ×4，修复页翻绿溶解）；fb 修复页终态残红 **4**＝真差异 1（R13 cE=false span wrap 能力缺口）＋伪影 3（R3/R4 dl qI 不可满足互斥对＋R7 录音复位）除名挂账；「qV·qI 单块形读面族下一刀候选」撤销，下一簇＝indent 27F 普查。逐键数据与互斥对四行实证见 [fb13-r34-artifact-verdict](2026-11-11-fb13-r34-artifact-verdict.md)。
