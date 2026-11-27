# html-semantics M2 兼容性守成巡查（M2 切片 soak/patrol）

> **当前轮次：s791 收账完毕（2026-11-28 05:51 收账；all.done 05:42:17）｜窗开 05:35:55/99a0da2f3＝三链重燃 05:34:32 后 **1m23s＝短候窗新记**（s787 1m43s 前记破＝首燃 05:30:22 cwd 坑订正损失 ≈4min＝窗态零损失承＝无 rearm）＝**runner 四点恒 3a8425d0229d8484c5417b2d5a453c3d/131,610,544＝104th realize definite**（generation point＝s791 arm solo rebuild＝102th 先例形＝刀54/55 首嵌兑现＝序数窗三点定谳不预写承兑现）＝**窗中本流笔移动形首记**（臂记录笔 9fc0c0908 05:37:37＋pull 重放 1ec4a449f 05:37:41 落 mid-leg1＝docs-only 双移→leg2 rerun C=6＝rebuild 移位形 leg1→leg2＝byte 恒五演＝零行为效应）＝bridge SET-stability **四演 definite** 1843==b791==b790 三 md5 32e005b9＝NEW=0 GONE=0 fail=0 26.41s＝**首验 ON 104th 产品**＝stable identity hold **五连** 057764d3/8517＝**刀54/55 corpus 零伤亡首证**＋LEGS IDENT 七代 f7fc4041＋canon e5e5f4df 七窗＋FLIPS 27 十一演＝wob r108 十五演＋band r224 二十九连＝窗后 **刀56 35ac9d5f8** fix(engine) 码刀＝s792 arm rebuild 必然＝105th realize 候强｜下一步＝s792 部署发射

## 本轮状态（s791 收账＝runner 四点恒 3a8425d0＝**104th realize definite**＝bridge SET-stability 四演首验 104th＝stable 五连＝LEGS IDENT 七代＝FLIPS 十一演＝wob 十五演＝1m23s 短候窗新记＝窗中本流笔移动形＝窗后 刀56 码刀 105th 候强）

- **窗形＝1m23s 短候窗新记**：三链重燃 05:34:25-05:34:32（pids/log mtime 定谳）→judge 05:35:55/99a0da2f3＝**1m23s**（s787 1m43s 前记破＝**短候窗新记形**）＝clean-soaked 3x45s 首检即过＝**无 rearm**（预算 0/3 fresh 承）；首燃 05:30:22 cwd 坑订正损失 ≈4min（pids/日志落仓根→pgrep -af 白名单核杀重燃＝窗态零损失承〔waiter 2min〕）。
- **窗区间＝0 commits 零移动窗＋窗中本流笔移动形首记**：armstart 05:30:22/99a0da2f3→judge 05:35:55/99a0da2f3＝**99a0da2f3..99a0da2f3＝0 commits**（窗前零 pull＝短窗特有形）；臂记录笔 commit 9fc0c0908 **05:37:37**＋pull --rebase 重放 1ec4a449f **05:37:41** 落 **mid-leg1**（reflog 定谳＝judge 后 1m42s）＝docs-only 双移→**leg2 rerun C=6＝rebuild 移位形首记（leg1→leg2）**（leg1 05:35:56 cargo 先起跑未触发＝C=0）＝mid/post 与 armdisk 同值＝byte 恒＝零行为效应直证。
- **runner 四点恒＝104th realize definite＝s791 primary 兑现**：pre（05:35:56）=mid（05:37:58）=post（05:40:52）=armdisk＝**3a8425d0229d8484c5417b2d5a453c3d/131,610,544 四点恒**＝Δ+8,040 B vs 103th 02bd6332/131,602,504＝**104th generation point＝s791 arm solo rebuild（102th 先例形）＝刀54/55 engine js 首嵌兑现**＝序数不预写承兑现（臂时换代面→窗三点恒承 realize definite）。
- **bridge SET-stability 四演 definite＝首验 ON 104th 产品**：keys **1843**==b791 1843==b790 1843＝三 md5 **32e005b9751df4048bbdac6c19d2b17f** 全等＝NEW=0 GONE=0 fail=0 rcb=0＝26.41s＝**bridge.runner.md5=3a8425d0＝104th 产品上首个全桥验证**＝刀54/55 零 bridge pin 首证＋b791 基线零差＋b790 对照保留零差＝SET-IDENTICAL 四 carry。
- **stable identity hold 五连 definite＝刀54/55 corpus 零伤亡首证**：s791.stable.keys **057764d38ee311a47c34ecfc3e1e3192/8517**==s790==s789==s788==s787==s634 冻结锚 EXACT＝disagree=0 l1only=0 l2only=0＝233 Fail/8278 Pass/5 Timeout/1 Unsupported 恒承＝sanitizer table-family＋iframe R255 两刀面零伤亡（stable 集零移动＝五连）。
- **三恒**：stable identity hold 五连（057764d3/8517＝s634 EXACT 五连 definite）＋**LEGS IDENT 七代**（l582-1.canon==l582-2.canon＝md5 **f7fc4041** 05:41:45 新鲜输出；canon **e5e5f4df** == s785..s791 七窗 byte-identical＝flake 退场七窗形）＋**FLIPS 27 十一演 definite**（s791.flip.keys **14f7aada8236b28fa59b8c122ae9c96e**/27 == s790 byte-identical＝script-7 held＝刀②b heal watch 六连落空＝dialog 域 20 held）。
- **哨位全持**：wob r108 dual-P hits=1×2＝**闭形持十五演**（r94..r108 十五连）＋band r224 **8/8×2＝twenty-nine-carry 兑现**（band8 keys hits=8×2）＋mpreload r253 15/15P×2 hold＋dialog r225 dual 2P retirement-held＋select r230 **F-set IDENT se.577**＋password r230 **keys=225 F=0**＋waapi r312 picker P/checkmark F＋fd r271 121P×2＋ob r223 2F/8P Fset IDENT＋dual-emit dup(2)×2＝**刀54/55 哨位域零伤亡互证**。
- **窗后区间＝3 commits＝1 码刀＋2 docs＝.rs 面复归**：1ec4a449f..3a0b7f67f（pull 后现读）＝**刀56 35ac9d5f8**（fix(engine) iframe 移除后 browsing context 丢弃＝contentWindow/contentDocument null 门＋traversal 语料 support 三件补齐＝TreeWalker assert_node 28F 整簇清零〔对端 wptfyi-dom 流＝刀体号订正 0380a258a→35ac9d5f8 rebase 重放 blob 恒承〕）＋19b0a8bd0（刀56 收账）＋3a0b7f67f（订正笔）＝**s792 arm rebuild 必然＝105th realize 候强**（s791 前刀54/55→104th realize 同形递进）。
- **窗 solo C=0/0＋证据域分离承**：窗 solos 05:41:46-05:42:17 **C=0/0**（leg2 暖缓存承＝s790 形）＝batch wall 05:35:55→05:42:17＝6m22s 零 ABORT；臂时 .arm 快照 ×4 承（solo1.C/solo2.C 臂时 05:32:38/05:33:03 与窗 solo 日志分离＝**cat 证据须辨域**〔本轮收账自正：solo1.C=6 为臂时值非窗值〕）。

## 基线决策（s792）
- **s791 臂记录（05:30:22）**：三链点火 waiter 1825444/rearm 1825445/post 1825446（/bin/bash 绝对路径直启形＝argv 白名单核对 /tmp/s791 归属＝双 waiter 碰头互锁坑防承）＝**首燃 cwd 坑订正承**（`cd /tmp/s791 && nohup … &` 整体后台化使 `echo $! > pids…`/rearm/post 重定向落仓根＝pids 三件＋rearm/post 0B 日志错位＝.gitignore *.log 兜住 git 面但证据应落 /tmp/s791＝pgrep -af 白名单核杀三链重燃＝waiter 2min 无窗态损失＝订正后 pids 1825444/1825445/1825446 pgrep/ps 双验＝日志三件 0B 落 /tmp/s791）；b791=32e005b9/1843 **SET-stability 四演候**承继（cp s790.bridge.keys＝cp 先行后 rm 承＝md5 双验 32e005b9751df4048bbdac6c19d2b17f/1843 行＝b790 值恒）＋b790=32e005b9/1843 对照保留一轮〔值恒对照面＝三演 definite 后零差对照＝NEW 首形 watch；s792 drop 预注〕＋b789 drop〔对照终轮满＝滚动三轮窗承〕；s787 系基线 drop（滚动窗承＝s634/s788/s789/s790 保留）；python 轮转承（rotate-s791.py＝76 行盲转＋排除行 verbatim 双锚＝analyze2.sh:58 wob 头 r107＋bridge-extract.sh:1 头注）＋九坑路径形承（盲转先换目录前缀 /tmp/s791＝九坑锚「/tmp/s791/b790.keys」形＝basename b790 无 s 前缀盲转不触及＝comm 基线指针/label echo「(b791=」/wc target 三处显式替换＝body stray 零残留核）＋**wc -l 全等 245 行审计**（11 脚本总计面）＋bash -n ×11＋chmod +x 承＝**二跑幂等核证**（二跑九坑锚 assert=0 即已应用＝rotate 幂等形首用）；head147 **oc 38/38 len 9074**（**104th realize 候强 pre-reg 多现承**〔≥1＝候选面非 realize definite〕＝105th×0/106th×0＝**刀54=0cba7aabc/刀55=b3ace97ef 订正号文案**＝rebase 重放号变 blob 恒等承）；phase5 **配方第四十七轮全 gate 过**（r107 ENTRY delta +2 内形 4/2 自相似＝junction 32-run 移 r107 via pull 尾＝**via pull runs {1,1,2,32} 新形**：历史 len-1 ×2〔深 25/27 pre-existing〕＋**r106 尾新 interior len-2〔depth 32→30〕**＋32-run 唯一 junction 32→0 实测＝via no-pull {1,2} 承＝heads r104/r105/r106/r107 深 30 恒×4＝paren 平衡＝line58 165,616 字符＝chains r2..r107/s685..s790/counts 106 值尾 8517＝watch bump r108 十五演候＋evo /r107＋m2 /s790＋states 五元 8516/8517×4＝max ordinal 104＝**链尾空格 strip 承再证**〔s788 坑 lesson 复现〕＝72 内容行承）；**probe rc=0 2.88s C=1 异形**（zero-product-version 单 crate recompile＝Checking 零编承＝HEAD 99a0da2f3 docs-only 双笔移动→hash 嵌入 rebuild 假设〔非证〕＝vs s789 7.21s C=0/s790 3.80s C=0 docs-only 形先例不合＝**C=0 先例与 hash-embed 假设矛盾未解＝如实记异形＝后续臂 probe watch**）；arm solo1 **C=6** rc=2 release rebuild（HEAD 99a0da2f3 移动 rerun 触发五演＝product-version→net→engine→page-runtime→webview→wpt-runner 级联＝wall 2026-11-28 05:32:38）＝armdisk **3a8425d0229d8484c5417b2d5a453c3d/131,610,544＝Δ+8,040 B vs 103th 02bd6332/131,602,504＝104th 臂时换代面成立**（**102th 先例形＝generation point＝s791 arm solo rebuild＝刀54/55 part04/05/13 js 面 engine crate 内嵌首嵌**＝104th 序数窗 legs 三点定谳不预写＝窗 pre/mid/post 三点恒承 3a8425d0 则 realize definite＝若对端再落码刀则窗中换代形警戒〔pre 读 3a8425d0 mid 读更新二点恒形〕）＝markers 同向并集×2（刀51）＋s10k64×4（刀52）恒＝刀51/52 embed 承＝刀53/54/55 无 standing marker（engine C=6 级联＋byte 变化承证）；solo2 C=0 rc=2 慢性 cache-warm（wall 2026-11-28 05:33:03＝25s＝快照 cp ×4 .arm 后缀形承）；臂时区间 762acafe2..99a0da2f3 现读＝**14 commits＝2 码刀＋12 docs**（刀54 0cba7aabc part04+part13-sanitizer＝sanitizer rWC table-family 1F 清零＋刀55 b3ace97ef part05＝iframe R255 92F 清零＝零 .rs 面＋对端 11 docs＋本流收账 99a0da2f3）；s791 primary＝**runner 换代 watch（104th realize candidate strong＝armdisk 3a8425d0 已换代＝无对端新码刀则窗三点恒承 3a8425d0）**＝bridge SET-stability 四演候 on 1843（NEW=0 GONE=0 候）＋stable identity hold 五连候＋**刀54/55 corpus face watch**（sanitizer table-family＋iframe R255＝零伤亡候＝伤亡现＝genuine-knife casualty definite candidate）＋FLIPS 十一演 watch＋script-7 heal watch 续（五连落空承）＋k150 dom face watch；候窗形 open（s790 1h00m17s 长中形承＝对端 kb316 批占窗形候＝TIMEOUT 形非异常承＝日界 watch 承：窗跨日 26.11.29 date 嵌入→byte-change 环境面形）。
- **s791 臂记录保留承**（上记 05:30:22 条＝s790 臂记录 04:11:56 条滚出〔滚动单承记〕＝首燃 cwd 坑订正教训承继在案）
- **s791 收账（2026-11-28 05:51）**：runner 四点恒 3a8425d0＝**104th realize definite**＝bridge SET-stability 四演 definite 首验 ON 104th＝stable identity hold 五连＝LEGS IDENT 七代＝FLIPS 27 十一演＝wob r108 十五演＝band r224 二十九连＝1m23s 短候窗新记＝窗中本流笔移动形（leg2 rerun C=6 byte 恒）＝刀54/55 corpus 零伤亡首证＝窗后 刀56 35ac9d5f8 码刀 105th 候强＝详见本轮状态。
- **基线轮转（s792 部署时执行）**：**b792＝cp s791.bridge.keys＝32e005b9/1843 值恒锚**（SET-stability 五演候＝四演 definite 进位＝cp 先行后 rm 承＝md5 双验 32e005b9/1843 行）＋**b791=32e005b9/1843 对照保留一轮**〔值恒对照面＝四演 definite 后零差对照＝NEW 首形 watch；s793 drop 预注〕＋**b790 drop**〔对照终轮满＝滚动三轮窗承〕；**s788 系基线 drop**（滚动窗承＝s634/s789/s790/s791 保留）。
- **runner 世代锚＝3a8425d0229d8484c5417b2d5a453c3d/131,610,544（104th realize definite＝s791 窗四点恒＝generation point 承 s791 arm solo rebuild＝102th 先例形）**：s792 预判＝**换代 candidate strong**（窗后 刀56 fix(engine) .rs 码刀落树→arm solo rebuild 嵌入→armdisk 预期 ≠3a8425d0＝arm-time generation face＝s791 自证先例→窗三点恒承新值则 **105th realize definite**；armdisk==3a8425d0＝hold 候＝rebuild-missed 排查 watch；**对端再落码刀则窗中换代形警戒**〔pre 读 3a8425d0 mid=post 读新值二点恒形＝103th 先例承〕）。
- **主预判＝窗后区间归因**（1ec4a449f..部署 HEAD 左锚现读承）：窗后落树＝刀56 35ac9d5f8＋对端 docs 现读；**bridge SET-stability 五演候 on 1843**（NEW=0 候＝四演 definite 后面稳定承／NEW 续现＝刀56 bridge pin 候＝browsing context discard 门面＝k 系刀面延续／GONE 现＝tests exit watch）；fail 0 恒承＝fail>0＝new-casualty watch＝**刀56 corpus casualty candidate**；stable identity hold **六连候**（057764d3/8517 锚）；LEGS IDENT 八代候；FLIPS 27 十二演 watch（script-7 retire＝刀②b heal 七连 watch＝flip retire form；新 flip 现＝刀56 corpus face candidate）；**刀56 corpus face watch**（TreeWalker 28F 整簇清零＝零伤亡候＝伤亡现＝genuine-knife casualty definite candidate＝same-key stable joint judgment）；wob r109 十六演 watch＋band r225 三十连 watch＋mpreload r254 hold watch＋dialog r226 retirement-held watch＋select r231/password r231/waapi r313/fd r272/ob r224 哨位 watch；rearm 先例承（s791 无 rearm＝预算 0/3 fresh＝TIMEOUT 形非异常承）；日界承（窗跨 11-28/11-29 date 嵌入→byte-change 环境面形＝s786 先例）。

## 下一步（s792 部署清单）
1. 目录复制与 drop 清单：/tmp/s792＝cp -a /tmp/s791；drop s788 系基线＋b790.keys；b792.keys＝cp s791.bridge.keys（32e005b9/1843）＋b791.keys 对照保留（cp 顺序坑防：先 cp 后 rm）
2. 轮转脚本 rotate-s792.py：s791→s792 全量行枚举＋排除行 verbatim 双锚（analyze2.sh:58 wob 头 r108＋bridge-extract.sh:1 头注 s791）＋九坑 b791→b792 三处显式（comm 基线指针 /tmp/s792/b791.keys／label echo `(b792=`／wc target）＋body stray 零残留核＋wc -l 全等审计（245 行承）＋bash -n ×11＋chmod 承＋二跑幂等核证承
3. head-s792.py：bridge-extract.sh 头注 148 整行重写（s791 窗账 condensed＋**刀56 35ac9d5f8 订正号文案＋105th realize 候强 pre-reg**）＋oc gate（head147 38/38 承＝新注 paren 实数断言）＋断言 106th×0/107th×0/105th 入账候选≤1
4. phase5_s792.py（配方第四十八轮）：标签 r226/r231/r231/r254/r313/r272/r224＋band r225＋wob r109 前缀（十五演承）＋COMMON s791 窗账（runner 四点恒 104th realize definite＋SET-stability 四演首验 104th＋stable 五连＋七代 IDENT＋FLIPS 十一演＋wob 十五演＋band 二十九连＋短候窗新记＋窗中本流笔移动形＋刀54/55 零伤亡）＋ENTRY r108 BLOCK delta +2 自相似承＋junction 形现读（via pull runs 现读承）＋heads paren 深 30 恒承＋chains r2..r108/s685..s791/counts 107＋watch bump r109＋evo /r108＋m2 /s791＋states 五元承＋max ordinal 105＝链尾空格 strip 承
5. probe（/usr/bin/time -v cargo check -p zero-wpt-runner＝仓根运行承〔/tmp 运行 rc=101 坑防〕）＋arm solo ×2（LC_ALL=C FILTER=select-home-end-pagedown-pageup.optional make testharness-html-semantics＋rc/wall/.arm 快照 cp ×4）＋armdisk 读（md5sum＋stat）＝**换代面警戒：armdisk ≠3a8425d0 则 105th arm-time candidate strong（102th/s791 双先例形）＝序数窗三点定谳不预写；armdisk==3a8425d0＝hold 候＝rebuild-missed 排查**
6. 三链点火（nohup 绝对路径 waiter/rearm/post＋pgrep -af cmdline 白名单核对 /tmp/s792 归属＋**cd 子壳整体形承＝cwd 坑订正**〔pids/日志落 /tmp/s792〕）
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
