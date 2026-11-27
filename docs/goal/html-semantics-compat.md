# html-semantics M2 兼容性守成巡查（M2 切片 soak/patrol）

> **当前轮次：s792 收账完毕（2026-11-28 06:25 收账；all.done 06:19:26）｜窗开 06:10:21/legs.head 7fd973f97→窗中 2609493c9（账户侧 pull 06:10:45 落 mid-leg1＝携上游 刀57 8a620f6a8 fix(engine) 码刀＋R26000-H/批四十九 docs＝窗区间 4 commits via pull）＝**106th 窗中换代 realize**（pre/mid＝3ca7ddeb 二点恒〔leg1 C=0 move-without-rebuild〕→post＝**f4c91d3814e9da2ff6098fb06f559c5c/131,614,640**〔Δ+4,096 B〕＝leg2 C=6 rebuild＝generation point＝s792 窗 leg2 rebuild＝s791 rebuild-shift 形〔位置〕×103th 码刀形〔字节变〕两先例合演＝105th realize-superseded 形首记〔arm 时面+窗二点恒，post 超越〕＝106th realize 候强 at s793）＝bridge SET-stability 五演 definite ON 106th（1843==b792==b791 三 md5 32e005b9＝rc=0 1843 passed 39.43s＝刀56/57 零 bridge pin）＝stable 六连断（467d5339/8516−1＝234F/8276P）＝**刀56 casualty**（submission-checks Pass→Fail 双腿一致＝迟窗显影）＋**刀57 heal**（form-double-submit runTest2 Fail→Pass＝disagree=1 唯一键＝legs-asymmetry 探测器〔leg1 刀56 vs leg2 +刀57〕首真捕获＝LEGS IDENT 八代断非 flake）＝FLIPS 28＝wob r109 十六演＋band r225 三十连＝**premature cluster 事故**（deploy 僵尸 legs.done 短路＝post.sh 管线早死＝手动补跑 post.sh 幂等救援）＝窗后区间 0 commits｜下一步＝s793 部署发射**

## 本轮状态（s792 收账＝**106th 窗中换代 realize**＝pre/mid 3ca7ddeb 二点恒→post f4c91d38＝leg2 C=6 rebuild＝bridge SET-stability 五演 ON 106th＝stable 六连断刀56/57 分账＝LEGS 探测器首捕＝wob 十六演＝premature cluster 事故救援）
- **窗形＝judge 06:10:21 open＋premature cluster 事故**：三链 06:07:51 点火（pids mtime）→waiter clean-soaked 3x45s→judge 06:10:21＝arm 06:05:28→窗开 4m53s（2m30s soak）；**post.sh 06:07:51 即判 legs 完成**（deploy copytree 携带 s791 僵尸 legs.done〔mtime 05:41:28〕＝无新鲜度校验）→缺失 leg logs 上空跑 analyze/analyze2/bridge/solos（06:07:51-06:08:21 30s＝空 keys＋假 all.done）→**管线早死**；救援＝手动补跑 post.sh（幂等）06:17:35→06:19:26 真完＝zombie mtime lesson 首次真实咬人＝**教训升级 s793 deploy 清除执行项**；无 rearm（预算 0/3 fresh）。
- **窗区间＝4 commits via pull＋窗中本流笔移动形二演**：legs.head 7fd973f97（open 时）→账户侧 arm 笔 commit d3f9ec2e6 06:10:41＋pull --rebase **2609493c9 06:10:45 落 mid-leg1**（reflog 定谳＝judge 后 24s）＝携上游 3 commits（**刀57 8a620f6a8 fix(engine) .rs 码刀**＋R26000-H 收账＋批四十九 docs）＋本流臂记重放；s791 首演 docs-only byte-恒→s792 二演 byte-**变**（码刀 rides in）＝**结构形：push 前 pull 纪律 × 上游码刀＝窗中换代必至＝s793 处方＝点火前 fetch 预检＋预注**。
- **runner 三点＝106th 窗中换代 realize＝s792 primary 兑现**：pre（06:10:22）=mid（06:12:22）＝**3ca7ddeb/131,610,544 二点恒**（armdisk 面承；leg1 2m00s C=0＝pull 落后 cargo 已过解析＝**move-without-rebuild 形**）→**post（06:15:10）＝f4c91d3814e9da2ff6098fb06f559c5c/131,614,640（Δ+4,096 B vs 105th）＝leg2 2m47s C=6 rebuild**（06:12:22 起跑 cargo 见树变）＝**generation point＝s792 窗 leg2 rebuild＝106th realize**＝s791 rebuild-shift 形（位置）×103th 码刀形（字节变）两先例合演＝**105th realize NOT definite＝realize-superseded 形首记**（arm 时面＋窗 pre/mid 二点恒，post 被超越〔序数不预写承兑现〕）＝**106th realize 候强 at s793 窗**。
- **bridge SET-stability 五演 definite＝首验 ON 106th 产品**：keys **1843**==b792==b791＝三 md5 **32e005b9751df4048bbdac6c19d2b17f** 全等＝NEW=0 GONE=0 fail=0＝rc=0 **1843 passed; 0 failed 39.43s**＝**bridge.runner.md5=f4c91d38＝106th 产品（刀56+刀57 面）全桥验证**＝刀56/57 零 bridge pin 证明＋b791 对照零差＝SET-IDENTICAL 五 carry。
- **corpus face 分账＝stable 六连断＝刀56 casualty＋刀57 heal**：s792.stable.keys **467d5339ce84443868c1b25550841200/8516**（−1）≠057764d3/8517＝**stable identity hold 六连断（count-change 形）**＝234 Fail/8276 Pass/5 Timeout/1 Unsupported（+1F −2P：submission-checks P→F＋disagree 键离场）；**刀56 casualty definite candidate**＝submission-checks `<form> in a navigated document cannot navigate` **Pass→Fail 双腿一致**（s791 双腿 Pass＝leg1 刀56 面已 Fail＝唯 .rs delta 刀56＝**迟窗显影**：刀56 落 s791 窗后 s792 leg1 首测即显）＝FLIPS **28**（27 持＋1 新）c92accb1＝script-7 held＝刀②b heal watch 七连落空；**刀57 heal definite candidate**＝form-double-submit runTest2（button/callRequestSubmit:false/callSubmit:true/preventDefault:false/passSubmitter:true）**leg1 Fail→leg2 Pass＝disagree=1 唯一键＝GONE=1 同键**＝**legs-asymmetry 探测器首真捕获**（leg1=刀56 面 vs leg2=刀56+刀57 面＝LEGS IDENT 八代断＝l582-1 **28b0168a** vs l582-2 **6251237e**＝非 flake 回潮＝刀面差异直读）。
- **哨位全持＝刀56/57 哨位域零伤亡互证**：wob r109 dual-P＝**闭形持十六演**（r94..r109）＋band r225 **8/8×2 held＝三十连兑现**＋mpreload r254 **15/15P×2**＋dialog r226 **2P×2 retirement-held**＋select r231 **279/8F IDENT se.577**＋password r231 **225 0F**＋waapi r313 picker P/checkmark F＋fd r272 **121P×2**＋ob r224 2F/8P **Fset IDENT×2**＋dual-emit dup(2)×2＝form-submission-0 域外零移动。
- **窗 solos C=0/0＋批墙钟 9m05s**：窗 solos 06:18:54-06:19:26 **C=0/0**（leg2 暖缓存承＝15s+16s）＝批墙钟 06:10:21→06:19:26＝**9m05s**（含救援补跑 1m51s）零 ABORT；.arm 快照 ×4（臂时 C=6/C=0）承＝快照纪律价值三演。
- **窗后区间＝0 commits 零移动**：2609493c9..origin/main **0 commits**（06:10:45 pull 已尽收上游）＝s793 arm solo 预期 **C=0 cache-warm**＝armdisk 恒 f4c91d38 预期（106th 臂时面承）。
- **meta-stale 陷阱近失误读＋证据域分离再证**：首轮三点 cat 时 mid/post 为 s791 拷贝陈值（3a8425d0，mtime 05:37/05:40 陈）——**盘读纪律升级＝值+mtime 双读先行**（免误账）；premature 簇产物（空 keys＋假 all.done）与真窗产物分离＝补跑幂等承。
## 基线决策（s793）
- **s792 臂记录（06:05:28）**：三链点火 waiter 1907501/rearm 1907502/post 1907503（/bin/bash 绝对路径直启形＝argv 白名单核对 /tmp/s792 归属＝**cd 子壳整体形承**〔pids/重定向全落 /tmp/s792＝仓根零残留核＝cwd 坑订正承〕）；b792=32e005b9/1843 **SET-stability 五演候**承继（cp s791.bridge.keys＝cp 先行后 rm 承＝md5 双验 32e005b9751df4048bbdac6c19d2b17f/1843 行）＋b791=32e005b9/1843 对照保留一轮〔值恒对照面＝四演 definite 后零差对照＝NEW 首形 watch；s793 drop 预注〕＋b790 drop〔对照终轮满＝滚动三轮窗承〕；s788 系基线 drop（滚动窗承＝s634/s789/s790/s791 保留）；python 轮转承（rotate-s792.py＝wc -l 全等 245 行审计＋bash -n ×11＋chmod 承＋**二跑幂等核证确认**〔analyze2.sh:58 wob 头 r109 锚 assert=0 即已应用＝analyze.sh 二轮旋转 no-op 零害＝s791 幂等形承〕）＋九坑路径形承（盲转先换目录前缀 /tmp/s792＝九坑锚「/tmp/s792/b791.keys」形＝comm 基线指针/label echo「(b792=」/wc target 三处显式替换＝body stray 零残留核）；head148 **oc 36/36 len 8609**（**105th realize 候强 pre-reg**〔≥1＝候选面非 realize definite〕＝106th/107th×0＝**刀56=35ac9d5f8 订正号文案**〔0380a258a→35ac9d5f8 rebase 重放 blob 恒承〕）；phase5 **配方第四十八轮全 gate 过（regate 承）**：标签 r226/r231/r231/r254/r313/r272/r224＋band r225＋wob r109 前缀＝**dialog prior 子句 s 号坑订正**（rotate 只移 s791→s792＝s790 prior 子句由 phase5 rep 移 s790→s791＝锚序承〔首轮锚错 s791 assert=0 即止零写＝订正重跑〕）＝**COMMON 首尾锚对提取承**（平衡游走止于首组＝多组序列形非单组＝尾锚「batch wall 6m15s zero ABORT)」count 1）＝NEW_COMMON 首段双闭合并承（interval 组 +1 未闭 assert 22/21 即止＝「re-proof))」修）＝**junction 32-run 移 no-pull 侧新形**（s791 零移动窗诚实戳 via no-pull＋gate 侧移＝首跑 gate 捕获 BLOCK 尾缺戳〔runs [1,1,2,2] 无 32〕＝regate-s792.py 修复「zero-move via no-pull」＋全 gate 重验绿＝链尾戳真值承）＝via pull runs {1,1,2}＝r107 尾 interior len-2 32→30＝needles r104..r108 深 30＝chains r2..r108/s685..s791/counts 107 值尾 8517＝watch bump r109 十六演候＋evo /r108＋m2 /s791＋states 六元 8516/8517×6＝max ordinal 105＝105th 入账候选＝106th/107th×0＝链尾空格 strip 承；**probe rc=0 5.76s C=0**（docs-only C=0 先例形回归＝s791 C=1 异形未再演＝hash-embed 假设仍非证＝后续臂 probe watch 续）；arm solo1 **C=6** rc=2 release rebuild（HEAD 7fd973f97＝刀56 fix(engine) .rs 面嵌入级联＝wall 79s）＝armdisk **3ca7ddeb2b192a9ead3496a9bf93ec5d/131,610,544＝Δ0 B vs 104th 3a8425d0/131,610,544（size 恒 md5 变＝刀56 嵌入形）＝105th 臂时换代面成立**（**102th/s791 双先例形＝generation point＝s792 arm solo rebuild**＝105th 序数窗三点定谳不预写＝窗 pre/mid/post 三点恒承 3ca7ddeb 则 realize definite＝若对端再落码刀则窗中换代形警戒〔pre 读 3ca7ddeb mid 读更新二点恒形〕）＝**meta-md5 陷阱自正**（armdisk 初读误中 zero-wpt-runner.d 依赖文件〔md5 1aac4eff/43798B〕＝即时订正重读真二进制＝runner-md5 盘读纪律再证）；solo2 C=0 rc=2 慢性 cache-warm（wall 15s＝15s＝快照 cp ×4 ×2 .arm 后缀形承）；s792 primary＝**runner 换代 watch（105th realize candidate strong＝armdisk 3ca7ddeb 已换代＝无对端新码刀则窗三点恒承 3ca7ddeb）**＝bridge SET-stability 五演候 on 1843（NEW=0 GONE=0 候）＋stable identity hold 六连候＋**刀56 corpus face watch**（TreeWalker assert_node 28F 整簇清零域＝零伤亡候＝伤亡现＝genuine-knife casualty definite candidate）＋FLIPS 十二演 watch＋script-7 heal watch 续（六连落空承）＋wob r109 十六演＋band r225 三十连 watch；候窗形 open（s791 1m23s 短候形承＝对端 kb316 批占窗形候＝TIMEOUT 形非异常承＝日界 watch 承：窗跨日 26.11.29 date 嵌入→byte-change 环境面形）。
- **s792 臂记录保留承**（上记 06:05:28 条＝s791 臂记录 05:30:22 条滚出〔滚动单承记〕＝首燃 cwd 坑订正教训承继在案＝二跑幂等核证承）
- **s792 收账（2026-11-28 06:25）**：106th 窗中换代 realize（pre/mid 3ca7ddeb 二点恒→post f4c91d38/131,614,640＝leg2 C=6＝generation point＝s792 窗 leg2 rebuild＝105th realize-superseded 形首记）＝bridge SET-stability 五演 definite ON 106th＝stable 六连断＝刀56 casualty（submission-checks Pass→Fail 双腿一致）＋刀57 heal（form-double-submit runTest2 Fail→Pass＝disagree=1）＝FLIPS 28＝LEGS 探测器首真捕获＝wob r109 十六演＋band r225 三十连＝premature cluster 事故（僵尸 legs.done 短路＝补跑救援）＝窗后区间 0 commits＝详见本轮状态。
- **基线轮转（s793 部署时执行）**：**b793＝cp s792.bridge.keys＝32e005b9/1843 值恒锚**（SET-stability 六演候＝五演 definite 进位＝cp 先行后 rm 承＝md5 双验 32e005b9/1843 行）＋**b792=32e005b9/1843 对照保留一轮**〔值恒对照面＝五演 definite 后零差对照；s794 drop 预注〕＋**b791 drop**〔对照终轮满＝滚动三轮窗承〕；**s789 系基线 drop**（滚动窗承＝s634/s790/s791/s792 保留）；**deploy 清除执行项新增**（rm 僵尸 done-markers/窗产物＝premature cluster 教训升级）。
- **runner 世代锚＝f4c91d3814e9da2ff6098fb06f559c5c/131,614,640（106th 窗中换代 realize＝generation point 承 s792 窗 leg2 rebuild＝105th 3ca7ddeb superseded 承）**：s793 预判＝**窗三点恒承 f4c91d38 则 106th realize definite**；窗后新码刀则窗中换代形再演警戒（103th/s792 双先例承）；窗后区间 0 commits＝arm solo **C=0 cache-warm 候**＝armdisk 恒 f4c91d38 预期（106th 臂时面承；armdisk≠f4c91d38＝排查 watch）。
- **主预判（s793）＝stable identity hold 恢复形候**：healed 键以 Pass 回场预测＝s793 stable **8517**＝keys md5 **057764d3 EXACT 恢复候**（s792 断为双腿异面 artifact 非成员丢失＝identity 恢复形首演 watch）＝canon 值域 **234F/8277P**（submission-checks P→F 持＋healed 键 F→P）＝**FLIPS 29 候**（28 持＋heal flip Fail→Pass 新入）；**bridge SET-stability 六演候 on f4c91d38**（1843 三 md5 32e005b9 承＝NEW=0 候）；**LEGS IDENT 回归候＋canon 值换代形 watch**（单面双腿 byte 恒恢复＝canon 值离 e5e5f4df 八代链＝面换代形如实记）；**刀56 casualty 处置 watch**（submission-checks face-consistent 持＝s793 确认则升级对端 engine 账候选）；script-7 heal watch 续（七连落空承）；wob r110 十七演＋band r226 三十一连＋mpreload r255/dialog r227/select r232/password r232/waapi r314/fd r273/ob r225 哨位 watch；rearm 承（0/3 fresh＝TIMEOUT 形非异常承）；日界承（窗跨 11-28/11-29 date 嵌入→byte-change 环境面形＝s786 先例）。

## 下一步（s793 部署清单）
1. 目录复制与 drop 清单：/tmp/s793＝cp -a /tmp/s792 ＋ **stale markers 清除新增执行项**（rm legs.done all.done solos.done analyze.done＋leg logs/keys/canons/norms/bridge logs＝premature cluster 事故教训升级＝deploy 僵尸 legs.done 防复发）；drop s789 系基线＋b791.keys；b793.keys＝cp s792.bridge.keys（32e005b9/1843）＋b792.keys 对照保留（cp 顺序坑防：先 cp 后 rm）
2. 轮转脚本 rotate-s793.py：s792→s793 全量行枚举＋排除行 verbatim 双锚（analyze2.sh:58 wob 头 **r109**＋bridge-extract.sh:1 头注 s792）＋九坑 b792→b793 三处显式（comm 基线指针 /tmp/s793/b792.keys／label echo `(b793=`／wc target）＋body stray 零残留核＋wc -l 全等审计（245 行承）＋bash -n ×11＋chmod 承＋二跑幂等核证承
3. head-s793.py：bridge-extract.sh 头注 149 整行重写（s792 窗账 condensed＋**刀56 casualty/刀57 heal 分账文案＋106th realize 候强 pre-reg**）＋oc gate（head148 36/36 承＝新注 paren 实数断言）＋断言 107th×0/108th×0
4. phase5_s793.py（配方第四十九轮）：标签 r227/r232/r232/r255/r314/r273/r225＋band r226＋wob r110 前缀（十六演承）＋COMMON s792 窗账（106th 窗中换代 realize＋SET-stability 五演 ON 106th＋stable 六连断刀56/57 分账＋LEGS 探测器首捕＋premature cluster 事故承）＋ENTRY r109 BLOCK delta +2 自相似承＋**junction 形现读（s792 窗区间 4 commits via pull＝move 窗＝junction 回 via pull 侧候＝诚实戳现读承）**＋heads paren 深 30 恒承＋chains r2..r109/s685..s792/counts 108 值尾 8516＋watch bump r110＋evo /r109＋m2 /s792＋states 七元承＋needles r104..r109 深 31＋max ordinal 106＝链尾空格 strip 承
5. probe（/usr/bin/time -v cargo check -p zero-wpt-runner＝仓根运行承）＋arm solo ×2（LC_ALL=C FILTER=select-home-end-pagedown-pageup.optional make testharness-html-semantics＋rc/wall/.arm 快照 cp ×4）＋armdisk 读（md5sum＋stat＝真二进制直读＝.d meta-md5 陷阱防）＝**hold 形预期：窗后 0 commits＝arm solo C=0 cache-warm 候＝armdisk 恒 f4c91d38＝106th 臂时面承；armdisk≠f4c91d38＝排查 watch；点火前 fetch 上游新刀预检新增（有刀则 armdisk 换代警戒预注＝窗中换代形 s792 先例承）**
6. 三链点火（nohup 绝对路径 waiter/rearm/post＋pgrep -af cmdline 白名单核对 /tmp/s793 归属＋cd 子壳整体形承＝cwd 坑订正）
7. master 已臂笔＋lei-pre-commit-guard PASS＋commit＋pull --rebase＋push

## Mission

以 WPT html/semantics corpus 分域为验收标准，按子域切片推进（建议序：forms
约束与控件状态 → tabular-data → document-metadata/grouping/text-level →
embedded-content → links/microdata），每域独立收口出数字。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **分域收口制**：一个子域一个数字，不设总通过率门禁（corpus 太大）
3. **轻量修复优先**：约束校验/IDL 面走 shim；触 DOM parser 核心的深结构改动
   单独立项征询

覆盖范围（十个初始子域，fetch 脚本 DIRS 同清单）：forms（最大，含深层子目录
按 web-animations 先例追加）/ embedded-content / document-metadata /
grouping-content / text-level-semantics / tabular-data / interactive-elements /
scripting-1 / links / microdata。

### 排除（明确不在范围内）

- **table/multicol 等布局渲染差异** — rendering-compat 域
- **iframe 浏览上下文树** — navigation-compat M3 门控域
- **媒体元素播放语义** — media 系已归档 goal 域（embedded-content 中的媒体
  元素属性面归本 goal，播放管线不碰）
- **ARIA/无障碍映射** — 未立项域，挂账

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | 元素 IDL 面/约束校验/状态机语义 | part 系 JS 语义 |
| dom/style-system | 属性解析与反射（不碰 parser 核心与布局） | 轻量优先 |
| WPT 资产 | html/semantics 十子域导入（分批） | fetch 脚本 + 账本回填 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 rendering-compat**：布局/绘制差异记账回流（table 布局是其活跃工作面），
  本 goal 不越界修渲染
- **与 navigation-compat M3**：embedded-content 的 iframe 面只做元素属性，
  frame 树语义挂其门控
- **与 css-parser 流（本会话 fuzz 修复侧）**：CSS 解析面零交集

---

## Done Criteria

- **DC-1** 十子域导入 + 分域基线 + csv planned 行回填（每域独立数据行）
- **DC-2** 逐域收口：forms / tabular-data / document-metadata / grouping /
  text-level / interactive-elements / scripting-1 / links / microdata /
  embedded-content 各自"分级通过率不再下行两个连续轮次 + 余案归因挂账"
- **DC-3** 回归测试账本纪律（imported-tests.txt）；make test 全绿、make reftest
  零回归、product-smoke 逐字节恒值

## 里程碑

M1 导入与分域基线（✅ 2026-10-15）→ M2 forms 域（最大杠杆，多切片）→
M3 其余九域依序推进 → M4 总收口归档。

## 轮次账
- **2026-11-13 M2 切片 415**：ior jQuery append() 新红根因修复收口轮
  （driving 1F→P，shim 四件套：part01 链接臂逐子真锚 + part05
  TreeOrderSync 倒序/BaseIdxShift prev 回落 + part04 iAH 槽锚逐子
  真锚）——四腿勘根闭环：round-1「同源翻转」系树误标（s413 批构建
  20:04 早于 B1 5d26a28eb 20:13 与收口 commit 22:04，实为异源）、
  腿 1 rustc 非确定性真实但退出因果链、腿 2 native 排空假说
  kill-switch 证伪 + 探针 ts 缺失误绑撤回、腿 3 视图源探针定谳
  B1 record 级 prevSibling 逐子镜像 → 同锚顺插 LIFO 倒转批视图。
  首轮 maketest 暴 t8e 多节点批新红（共享/真锚双读方语义冲突）→
  消费方全量审计四件套收口。验证：ior ×3 全 Pass + t8e 7/7；终稿
  sweep ×2 join 双腿恰 2 state-diff（ior F→P + home-end 族内换位）
  new=0 gone=0、cross 仅族内漂移、数字闭合无残差；make test 155 腿
  154 绿 + engine lib 3911P/7F 逐名 IDENTICAL 零位移。
  **在册红 213F+16T→212F+16T**（−1F ior 解挂）。挂账新增 bucket
  重复条目 parentSel 字符串 "null"。详
  evidence/2026-11-13-m2s415-ior-jquery-append-attribution.md（腿 3
  节 + 卷首/腿 2 更正注）。
- **历史轮次账（2026-10-15 M1 至 2026-11-12 M2 切片 413）**：完整原文已按原样迁至 [html-semantics-compat/archive/round-ledger-2026-10-to-11-13.md](html-semantics-compat/archive/round-ledger-2026-10-to-11-13.md)。

## 依赖约束备注

form-validation 遗产为 forms 域起点；html-syntax-compat（编号 40，已收口）的
parser 面资产为消费基础。
