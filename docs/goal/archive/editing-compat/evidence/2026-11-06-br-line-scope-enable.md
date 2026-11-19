# br compare 族 ×6 收口：enable 行粒度预遍（br 刀）

日期：2026-11-06。基线 = dps 刀收口面（iol 1747P/14F / iul 2047P/18F / mt 10,092 键 9,676P/416F / enginelib 3,766P/5F）。归因铺底见 [br 归因](2026-11-06-br-family-attribution.md)（含 9a579316b 勘误：A 形行界 br 契约 = 随提取移除）。

## 结果

iol 1747P/14F → **1750P/11F**，iul 2047P/18F → **2050P/15F**：judge-dps 双腿 **F2P 3+3 / P2F 0 / postF 0 / lost 0**，rows 1761/2065 恒等，VERDICT OK ×2。iol 残差 14→11（?1001-last rtl ×3 + nested ③ ×3 + blockquote ×2 + styled ×1 + noneditable ×1 + qVb 伪影 ×1），iul 残差 18→15（nested ×2 + blockquote ×2 + divbr straddle ×2 + rtl ×3 + div 边界 ×3 + noneditable ×1）。

mt 全文件腿：10,092 键 P 9,676→9,676 / F 416→416，**F2P 0 / P2F 0**，rows 恒等，VERDICT OK。键级全量补审（20116 共享键 verdict-diff 0）：原始行计数 +35（33P+2F）= `multitest.html?9001-last` 切片尾超时阴影——基线与 post 同触该页 timeout（"1 test(s) unreported"），post 多报 35 案；2 新见 Fail 为 delete 族旧红（`display:grid` 双 span delete+inserttext g/h、`<table><td><b>[b…d]</b>` delete+inserttext B），不经 toggleLists，入 delete 族残差清单备查。enginelib：3,773P/**5F 五红名册恒一**（p56_adopted_snapshot_restore_hook / test_p99s_rule_style_preserves_custom_property_case / test_stylesheet_add_rule_remove_rule_r3276 / test_stylesheets_rule_style_r2810 / test_stylesheets_write_r2809，与 dps 轮 canonical 名单逐字一致；3766→3773 为间期提交净增测试数）。

探针：zz-probe-br **6/6 got 与语料期望树逐字相等**（iol/iul × A/B/C；A `foo<ol><li>[bar]</li></ol>` 越界 br 清、B `<ol><li>f[oo</li><li>b]ar</li></ol>baz` 双 li 切分 baz 留外无幽灵壳、C `<ol><li>[foo]</li></ol><p>bar</p>` ol 拆出 p 残段净）；零扰动 zz-probe-qv13 26/26、zz-probe-dps 11/11。执行体证据：`grep -a -c _zwListEnableLineScope` = 2（二进制）。

## 修复（part01e 四处，唯 part01e 改动）

统一契约（Chrome 录制，语料 :13/:18/:23 exp 实证）：enable 对纯扁平块是**行粒度**——选区行整体入列、行外内容不动、行间 br 消费为 li 切分、提取边界邻接 br 移除（唯空行载体 br 不动）。vendored blockExtend 无行概念（爬到 host 整域入列），主循环 br 停聚臂把界外 br 吸进壳、wrap 的 ViaHtml/flat 重写臂对 br/文本混合成员序列化失配——三缺陷形（got `foo<ol><br><li>` / bar 遗壳外+尾部幽灵壳 / ol 嵌 p）同根。

1. 新增 `_zwListEnableLineScope(capS, capE)`（`_zwListToggle` 前）：纯扁平块守卫（origin 块直子全 inline/br ∧ brCount≥1 ∧ 双端点同块）∧ **列表语境排除**（b 或祖先为 ol/ul/li/dl/dt/dd 恒 null，见下迭代节）∧ anyVisible 门（eb1 空形守护）。命中则：选区行 span 外紧邻 br 移除（隔位 br 不动）→ 行壳临时 div 收壳（`_zwJusMovePreservingRanges` 逐成员），返回 `{block, kept}`。行壳借主循环 p/div retag 臂成 li（成员为元素后绕开 ViaHtml 臂）。
2. `_zwListToggle` blockExtend **之前**调用（时序关键：预遍删 br/移壳会位移块子索引，事后跑会 skew newRange 端点致 nodeList 漏员；capS/capE 端点身份快照为输入，不受预遍影响）。
3. nodeList enable filter 后追加 kept 过滤：爬升到预遍块直子，仅行壳放行（A 形 foo 不入列、C 形 bar 不入列），块外/已脱块成员放行。
4. 壳臂 freshShell 后补 `_zwListFixDisallowedAncestors(newShell)`（vendored toggleLists :5928 等价；lineScope 门控，置于 dps 归并出口之前——C 形 ol 落 p 内，ol ∈ prohibited-paragraph children ∧ p ∈ inlineContents → allowedChild 祖先链首检 false → splitParent 拆出，残段净 `<p>bar</p>`；A/B 形新壳落 div host，ol/ul 对 div 放行 no-op；非预遍面恒不触发）。

## 迭代：v1 批 P2F 2 定谳与守卫补笔

首全腿批（14:21，守卫缺列表语境排除）iol 腿即触红线：P2F 2（`.acceptance/dps/regress-br-iol.txt` 14:23 存证，iul 镜像同）——`<ul><li>[foo]<br>bar<li>baz</ul>` / `<ul><li>foo<br>[bar]<li>baz</ul>` 族：b 爬升命中 **li**（display 非 inline，是 block 节点），选区在既有列表 li 内时预遍误入，把行壳塞进 li 造嵌套错形；该族 enable 语义是整壳 retag（ul→ol 期望 br 原位保全）。修复 = 列表语境排除（b 或任意祖先 ol/ul/li/dl/dt/dd → null 走旧径）。批中改源码纪律：v1 批 stop（未修码余腿无收口价值），修后探针三连复验（br 6/6 + qv13 26/26 + dps 11/11）再发 v2 全腿批（zw-br-legs2.service，systemd 脱离，14:31 start，15:08 ALL-DONE）。

## 门禁账目

- 腿日志：`.acceptance/dps/batch-br-legs.log`（批摘要）、`batch-iol-post.log` / `batch-iul-post.log` / `batch-mt-post.log`（全腿原文）、`batch-enginelib.log`（五红原文）——.acceptance 本地不入库（gitignored），数字已录本档。
- 批脚本：`.acceptance/dps/batch-br-legs.sh`（flock 单实例 + marker 门 + judge 三段 + 五红名册核对；v1 批 enginelib ROSTER DRIFT 为脚本 grep 缩进 bug 假警报，名册经 batch-enginelib.log 原文直读定谳恒一）。
- 判读器：`.acceptance/dps/judge-dps.py`（行级元组 F2P/P2F/postF/lost，dps 轮同款复用）。
- 探针：`tests/wpt-runner/wpt-data/editing/run/zz-probe-br.html`（6 案 dump 式，期望树逐字取语料 insertorderedlist.js / insertunorderedlist.js）。
- 构建行：v2 批 cargo build 0.18s no-op 复用同二进制（marker hits=2 先核后跑）。
