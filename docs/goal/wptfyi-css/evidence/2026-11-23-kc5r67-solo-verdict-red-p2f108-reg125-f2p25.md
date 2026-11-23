# KC-5-r67 solo 判读 verdict-red（verify 流，2026-11-23 08:2x）

## 判定：三门全红且 RESOLVED 崩盘，r67 face 不落库（REDLINE P2F>0 fail-stop）

- **RESOLVED 2 / RESIDUAL-P2F 108 / REG 125 / F2P-BREAK 25，GT-missing 0**（5080 键两侧齐，14 页×双臂）。
- 对 v5 预测（105/5/0/1）再度三面全失准：RESOLVED 2 vs 105、RESIDUAL 108 vs 5、REG 125 vs 0；F2P 25 vs 1 是唯一接近轴。
- 判读工具 `kc5r66-verify-json.py solo-r67-snap-verify0823`（JSON 直读链，先快照后判读）；输出 `/tmp/kc5r67-judge-out.txt`。
- 与 r66（49/61/112/375）对照：**RESOLVED 49→2 崩盘、RESIDUAL 61→108 恶化**，但 **F2P-BREAK 375→25 大幅收敛**（r5 基恢复兑现了「r5 赢面 parity」设计目标）——单刀未同时压三门的历史第三次重演。

## 双臂 provenance

- b0 臂 = `probe-bin/b0-runner` md5 `16ce5984`（GT 基线同 md5）；b1 臂 = `probe-bin/r67-runner` md5 `a8c9e799`（08:18:20 构建，mtime 晚于 face 文件末次编辑 08:16:00，无 stale-bin；strings 正负检 trio 标记齐：r66_pair_suppress b1=2/b0=0、pair_endpoint_class b1=1/b0=0、fold_consult_for_selector b1=4/b0=0、parse_lp_terms b1=3/b0=0）。
- 脚本绑定静态核：`kc5r67-solo-verify.sh:24` b1 臂显式绑 `$PB/r67-runner`（不依赖 `$ARM-runner` 模板——r66 pass1 同型陷阱已防御）。
- solo 发射 08:18:25（本流实例，flock 持有），08:23:27 收官；竞争实例 08:18:36 撞单实例锁 exit 42（防双写门生效，其 SOLO-EXIT=42 行混入共享日志曾致 watcher 早触发误报——实例存活核验后排除）。

## 形态漂移：px echo → token 化 keyword echo（r67 新伪影）

- r66 残面 got 全为静态 px echo（`'50px '`/`'200px '`/`'0px '`，r65 基 used_size_echo 栈伪影）；r67 残面主簇 got 变为 **from-keyword token 化 echo**：`'min - content '`、`'auto '`、`'max - content '`、`'fit - content ( 100 % ) '`（token 间带空格＝keyword 经 echo 路径逐 token 序列化）。
- 机制指向：关键词涉足 shape（KW→cs、KW↔KW、NEU→KW、to-neutral）的插值复合在 r67-b1 内**塌缩到 from 端点 keyword**，经 cascade/echo 通道返回起始值——动画复合本体未产出，与 cssa 透传无关（b0 同树同 cssa 全过）。
- ⇒ **GT 面破坏由 r5 基 12 文件 patch 自身携带**（apply/cascade/animation 系 hunks），非 cssa 门：「cssa 回 HEAD 恢复透传＝GT 面可过性」假设被实测证伪——透传只救回 2 键（calc-size-height-interpolation，r66 同值），其余 107 GT 键塌 from-keyword。通道对称化（apply-往返 vs cascade 不对称，GT-110 根机制）是必要刀，透传非充分条件。

## F2P 轴：r5 基恢复基本兑付

- F2P-BREAK 375→25：残 25 簇＝width:LP→KW 7、width:KW→cs 6、width:NEU→KW 4、height:KW→cs 4、to-neutral 3、nomatch 1。r66 的 375 破坏主簇（r65 基坏栈拖累）随 r5 基回植消解。
- 对照：预测残 5 键（calc-h cs→LP 2 超额兑付／which-value＋cis 持位／1em→10vw 重分类）在 r67 残面全部复现同位（nomatch 784px、WA cis `50px 60px ` 在 RESIDUAL 列表可见）。

## REG 轴

- REG 112→125（+13），可见簇＝width:LP→KW 7、width:KW→cs 6、to-neutral 3、nomatch 1（显示截断，全量以 JSON 快照为准）；r66 的 max-*:NKW→LP WA 28 键簇未进显示前列。

## 判读样本与工序注记

- **transcript 尾段丢失**：本流日志 `/tmp/kc5r67-solo-all.log` 于发射窗内被并发流 rm 重建（fd 1 指向 deleted inode），RUN 行 24/28 经 `/proc/<pid>/fd/1` 快照抢救（`/tmp/kc5r67-solo-mine-final.log`）；末 4 腿（min-width/width 两页）RUN 行随进程退出丢失，**键级数据无损**（judge 直读 arm-dir JSON，5080×2 齐）。
- 结果快照：`results-kc5r6-fame2b/solo-r67-snap-verify0823/`（28 JSON 全非空，指纹 md5 `3d1564239dfb6c4dc3311b86509ffcbd`），canonical `solo-r67/` 同内容；快照先行防并发流重发覆写。
- 单轮判读即 RED（P2F=108 远超瞬态噪声域，无需确定性双跑；r66 的双证协议用于绿门验证，红门 fail-stop 不依赖重跑）。
- 教训：①共享日志名＋并发流清理＝deleted-inode 输出丢失，唯一日志名按流绑定（`solo-<face>-<stream>.log` 级）；②watcher 完成判据不得裸 grep 共享日志退出标记（他流撞锁行混入），以 PID 存活＋自身 RUN 计数为准；③JSON 直读判读链对 transcript 形态/丢失免疫，arm-dir JSON 才是判读权威输入。

## 对照池（GT 三门轨迹累计）

| 轮 | RESOLVED | RESIDUAL | REG | F2P-BREAK |
|---|---|---|---|---|
| r6.3 (w3) | 93 | 17 | 211 | 573 |
| r6.4 (w4) | 49 | 61 | 73 | 332 |
| r65 | 94 | 16 | 184 | 551 |
| r66 | 49 | 61 | 112 | 375 |
| **r67** | **2** | **108** | **125** | **25** |
| v5 预测 | 105 | 5 | 0 | 1 |

- **RESIDUAL 单调最优仍是 r65（16）**：r65 的输出侧门＋宽抑制组合至今是 GT 轴唯一有效解；r5 基（r67）与 r6.4 基（r66）都不承载 GT 轴。r68 若延续 face 路，基座选择须先回答「r5 patch 哪些 hunks 塌缩插值复合」——逐 hunk 二分是下一步前置，谓词/抑制层在其上才有意义。

## 工件索引

- 判读输出：`/tmp/kc5r67-judge-out.txt`（本 doc 数字源）；判读工具 `.acceptance/wptfyi-css/kc5r66-verify-json.py`（`solo-r67-snap-verify0823` 参数）
- 结果快照：`.acceptance/wptfyi-css/results-kc5r6-fame2b/solo-r67-snap-verify0823/`（canonical `solo-r67/` 同内容）
- 本流日志快照：`/tmp/kc5r67-solo-mine-final.log`（24/28 RUN，deleted-inode 抢救）
- 判读脚本：`.acceptance/wptfyi-css/kc5r67-solo-verify.sh`（b1 显式绑 r67-runner）
- 前轮 verdict：`2026-11-23-kc5r66-solo2-verdict-red-p2f61-reg112-f2p375.md`
