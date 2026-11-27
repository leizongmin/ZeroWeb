# html-semantics M2 兼容性守成巡查（M2 切片 soak/patrol）

> **当前轮次：s782 收账完毕（2026-11-27 13:12 收账；all.done 13:06:02）｜窗开 13:00:03/311925425＝armstart 12:58:33 后 1m30s＝**单判净快窗形**（load 0.76→clean-soaked 3×45s procs=0 load<3 rearm 预算 0＝s781 收账+臂双笔落库后机器即净＝九坑 s780/s781 两连 1h+ 长候窗后首快窗；legs 13:00:03→leg1 13:02:38→leg2 13:04:36→bridge 23.65s→all.done 13:06:02＝批墙钟 5m59s 零 ABORT＝窗 HEAD 311925425 即本流 s782 臂记录 commit＝legs.head 即现承）｜区间 37cce359b..311925425＝4 commits＝0 code＋4 docs（臂时 staging 3＝0 code＋3 docs＋本流 s782 臂记录 1＝**窗 HEAD==臂 HEAD＝窗中漂移 0＝码集恒同**）｜runner 97th hold realize＝46d43a701/131,480,088 三点恒持＝臂盘读值窗内零换代＝97th 恒持二演窗＝join identity hold 六演＝bridge SET-IDENTICAL＝FLIPS 27 集二演＝**全持形收账（六 watch 全部落空＝零漂移窗）**＝genuine-knife 十四演 face 恒持 carry｜s783 已臂（2026-11-27 13:22:41/a9982ad03＝waiter 817396＋rearm 817397＋post 817398 三链 argv 形核；b783=8515be9c/1831 值恒基二轮（cp b782.keys＝SET-IDENTICAL carry）＋b782 对照保留一轮＋b781/b780 drop〔对照终轮满＝滚动三轮窗〕；python 轮转八坑修正承＋九坑进位兑现（comm 基线指针 b782→b783＋label echo `(b783=`＋wc target `< b783.keys` 三处显式替换＋wc -l 全等审计 11 脚本＋bash -n ×11＋chmod +x 承）；head139 oc 28/28（97th×3/98th×0）＋phase5 配方第三十九轮全 gate（r99 ENTRY delta 0 自相似＋junction r98 tail 直贴 32-run depth 32＋watch bump r99→r100＋states 97→98 append v782=8517＋rounds 98 值尾 s782＋ordinal 97th×5/98th×0）；**staging 现读分型＝区间 311925425..a9982ad03＝3 commits＝1 code＋2 docs**＝f634df5f2 对端 fix(engine,cargo) 红测试清零刀（js_dom_shim/part05.js 源＋js_dom_bridge_tests/part16.rs 测试面＋Cargo 依赖清理 −87 行＝kb307/kb308 product-version 级联先例承；刀笔 13:02:45 落 s782 窗体内但窗 HEAD 未含＝窗后 staging 携入形＝s782 窗账 0 code 不变）＋a9d6fea27 对端 R12000-H 收账＋本流 s782 收账 1＝**码刀入区间＝genuine-knife 十五演 candidate**；probe rc=0 15.13s C=13（1 Compiling＋12 Checking＝刀面重查形＝部署面不归因承）；arm solo1 C=13 rc=2 77s（**刀级联 rebuild＝13 crate 垂直 zero-dom→zero-wpt-runner＝换代 rebuild 形**）＋solo2 C=0 rc=2 慢性 16s＋快照 cp ×4；runner 臂盘读 **b0b5a2aa/131,477,768＝−2,320 B vs 97th 46d43a701/131,480,088＝换代臂盘读**（97th 恒持两轮后换代＝f634df5f2 刀面字节效应直证）＝**98th realize candidate**（臂时 realize＝代点臂 solo rebuild；序数窗内定谳＝不预写承〔窗 legs 三点定谳〕）；对端 batch48 test262 新批在跑（ZeroWeb-3 test-guard 806429＝comm 命中阻断清单）＝**长候窗 candidate**（s781 窗跨越 1h11m05s 判例同构承）｜下一步＝s783 窗开 join 收账）

## 本轮状态（s782 收账＝全持形：identity hold 六演＋bridge SET-IDENTICAL＋97th hold realize＋FLIPS 27 集二演＋wob r99 闭形持六演＋mpreload hold 八轮）

- **窗与区间**：窗 13:00:03 HEAD=311925425（armstart 12:58:33 后 1m30s＝**单判净快窗形**＝load 0.76→clean-soaked 3×45s procs=0 load<3 rearm 预算 0；legs.head 即现承）；legs 13:00:03→leg1.done 13:02:38→leg2.done 13:04:36→bridge 23.65s→all.done 13:06:02＝批墙钟 5m59s 零 ABORT；区间 **37cce359b..311925425＝4 commits＝0 code＋4 docs**（臂时 staging 3＝3b9ba915a 对端 wptfyi-html 门十七演＋ce1d48855 对端 R12000-H 发射笔＋526d3db3c 本流 s781 收账＝0 code；臂后 1 笔＝311925425 本流 s782 臂记录＝**窗 HEAD==臂 HEAD＝窗中漂移 0**＝kb305/kb306 携入形后的首零漂移窗）。
- **genuine-knife face 恒持窗＝十四演 definite carry**：区间 docs-only（0 code）＝无新刀＝十四演 face 无新事件；**FLIPS 27 集二演**（s782.flip.keys **14f7aada/27**==s781 exact＝cmp 零差）＝112dbadf8 五愈 face＋刀㊾ TT 池面恒持 carry；**module/inline-async retained 四演窗兑现**＝knife-48 伤亡 carry 续；dialog 域 20 持。
- **bridge＝SET-IDENTICAL**：keys 1831 vs b782（**8515be9c/1831 exact 同值**）NEW=0 GONE=0＝**face 稳定 watch 兑现＝112dbadf8 五愈后首稳窗**＝b782 值恒基一轮确立；fail 4 恒（r5249＋r5010＋t2pb1＋r3318＝e77b9d11==s781 fail keys exact）＝fail 4 态持 watch 兑现；**b781=727032bb/1826 对照一轮完成＝s783 drop 预注**；rcb=101 慢性承。
- **runner＝97th hold realize**：**pre=mid=post=46d43a701/131,480,088 三点恒持＝臂盘读值窗内零换代**＝**97th 恒持二演窗**（97th realize 首记后 hold 形；代点＝s782 臂 solo rebuild 承＝+12,416 B vs 96th 不变）＝窗 leg1 C=6 Fresh=0（HEAD-move docs 级联 526d3db3c→311925425 rebuild **产出字节恒同＝确定性 rebuild 形四演**〔s780 leg1/s781 leg1/s782 臂 solo 三承后续〕）／leg2 C=0 Fresh=0 字节复用＝**98th 序数不预写承兑现**（hold 形臂盘读＋legs 三点恒持定谳 hold）。
- **join＝identity hold 六演**：stable **8517** disagree=0 l1only=0 l2only=0 NEW=0 GONE=0＝**stable.keys md5 057764d3==s634 冻结锚==s779==s780==s781==s782 完全一致**＝identity base 六演持锚（破缺/count 变 watch 落空）；stable.canon **e5e5f4df**==s781 exact＝canon 恒 hold 兑现＝**LEGS byte-identical 六连**（canon＋norm 双 cmp 零差＝flake absent-form 六连演）。
- **哨位轮转**：dialog r216 双 2P×2＝归休持形持 ✓＋select r221 279 8F IDENT se.577 双腿 ✓＋password r221 225 0F ✓＋mpreload r244 15/15P×2＝**hold 八轮 realize**（r237..r244 八连 15/15P）✓＋waapi r303 1F/1P ✓＋fd r262 121/121P ✓＋ob r214 2F/8P×2 Fset 双腿 IDENT＝restore-held 持 ✓＋band r215 8/8×2＝**twenty-carry 兑现**（r214 nineteen-carry 后续）✓＋**wob r99 双 P＝闭形持六演兑现**（r94..r99 六连 dual-P held）✓＋dup(2)×2 ✓。
- **probe/solos**：probe（臂时 rc=0 19.16s C=0＝docs-only 区间零重查形承＝部署面不归因）；臂 solo1 C=6 rc=2 52s（确定性 rebuild 三演承）＋solo2 C=0 rc=2 慢性 14s＋快照 cp ×4；窗 solo ×2 15s/14s rc=2/2 慢性；rc1=2 rc2=2 慢性＋so 全 0。

## 基线决策（s783）

- **s783 臂记录（13:22:41）**：三链点火 waiter 817396/rearm 817397/post 817398（argv 形绝对路径承）；b783=8515be9c/1831 值恒基二轮（cp b782.keys＝SET-IDENTICAL 值恒 carry）＋b782 对照保留一轮〔值恒对照；s784 drop 预注〕＋b781/b780 drop〔对照终轮满＝滚动三轮窗承〕；python 轮转八坑修正承＋九坑进位兑现（comm 基线指针 b782→b783＋label echo `(b783=`＋wc target `< b783.keys` 三处显式替换＋wc -l 全等审计＋bash -n ×11＋chmod +x 承）；head139 oc 28/28（97th×3/98th×0）＋phase5 配方第三十九轮全 gate（r99 ENTRY delta 0 自相似＋junction r98 tail 直贴 32-run depth 32＋watch bump r99→r100＋states 97→98 append v782=8517＋rounds 98 值尾 s782＋ordinal 97th×5/98th×0）；**staging 现读分型＝区间 311925425..a9982ad03＝3 commits＝1 code＋2 docs**＝f634df5f2 对端红测试清零刀（js_dom_shim/part05.js 源＋js_dom_bridge_tests/part16.rs 测试面＋Cargo 依赖清理＝刀笔 13:02:45 落 s782 窗体内但窗 HEAD 未含＝窗后 staging 携入形＝窗账 0 code 不变＝**码刀入区间＝genuine-knife 十五演 candidate**）＋a9d6fea27 对端 R12000-H 收账＋本流 s782 收账＝**staging 预读即现读纪律 live 承**（s780 臂时翻案判例承＝禁沿用区间记忆）；probe rc=0 15.13s C=13（刀面重查形＝部署面不归因承）；arm solo1 C=13 rc=2 77s（**刀级联 rebuild＝13 crate 垂直 zero-dom→zero-wpt-runner＝换代 rebuild 形**〔s782 docs-move C=6 形对照〕）＋solo2 C=0 rc=2 慢性 16s＋快照 cp ×4；runner 臂盘读 **b0b5a2aa/131,477,768＝−2,320 B vs 97th 46d43a701/131,480,088＝换代臂盘读**（97th 恒持两轮后换代＝f634df5f2 刀面字节效应直证）＝**98th realize candidate**（臂时 realize＝代点臂 solo rebuild；序数窗内定谳＝**98th primary candidate 不预写**承〔窗 legs 三点定谳〕）；对端 batch48 test262 新批在跑（test-guard 806429＝comm 命中阻断清单）＝**长候窗 candidate**（s781 窗跨越判例同构承＝clean-soaked 3×45s 判据不变）。
- **b783 = cp b782.keys（**8515be9c/1831＝值恒基二轮**：SET-IDENTICAL 后值恒 carry〔＝s782.bridge.keys 同值〕；**b782 对照保留一轮**〔face 变更后新值基对照；s784 drop 预注〕＝**b781/b780 已 drop 承**〔对照终轮满＝滚动三轮窗〕）＝s783 部署时执行。
- **runner 世代锚＝46d43a701/131,480,088（97th 恒持二演）**。**s783 预判＝hold-or-change 双歧**（区间 docs-only→byte 复用/确定性 rebuild hold candidate＝代点臂 solo rebuild 前置 mirror 形 watch〔97th/96th/95th 多连臂时形承＝窗 leg1 C=6 docs-move 级联字节恒同形续 watch〕；区间码刀→98th definite candidate 不预写；**98th 序数不预写**承）。
- **主预判＝窗后区间归因（311925425..部署 HEAD 左锚承）**：本流 s782 收账/部署 docs＋对端流续笔＝docs-only candidate 第一（**预读一律现读 git log 现场分型，禁沿用区间记忆〔s780 臂时翻案判例承〕**；窗中 pull 携入风险形承＝kb305/kb306 判例＝窗 HEAD 收账时同锚复核）；落码刀则 genuine-knife 十五演 candidate。
- **stable identity hold 六演后 s783 判读**：8517 集锚＝**s634 冻结锚 057764d3 持锚形**（identity hold 七演 candidate＝持锚 watch；再破缺＝破缺 watch／wob1 再 F＝swap 再开 watch／dialog-overlay 键 l1F＝swing 回潮 watch／键他员出走＝member-swap watch；count 变＝count 变 watch；either leg lost-block→lost-block face watch）；stable.canon **e5e5f4df 观察基三轮**（flip 集 27 恒＝canon 恒 hold candidate＝再现新 flip＝集变 watch／再退役＝flip 续退 watch）。
- **bridge 值恒基 s783 判读**：b783=1831 值恒基二轮（SET-IDENTICAL 二演窗＝face 稳定 carry；NEW 现＝continued recovery watch＝残红续愈 face〔112dbadf8 余项 ci 面〕／GONE 现＝tests 退场 watch）；fail 4 态持 expected（r5249＋r5010＋t2pb1＋r3318 刀面态持 watch；fail<4＝续愈 watch／fail>4＝knife 新伤亡 watch）；rcb=101 慢性承。
- **FLIPS 27 判读**：27 集三演窗（vs s782.flip.keys cmp：再恒＝27 集三演 carry＝十四演 face 恒持续；再现新 flip＝码刀续击 watch＝刀㊾ corpus face 候补〔TT 池面〕；旧员再退＝flip 续退 watch）；dialog 域 20 持；**module 员五演窗**（retained 五演＝knife-48 伤亡定谳 carry／退役＝修复回潮 watch）。
- **phase5 锚纪律（配方第三十九轮）**：s783 部署条目锚＝r99 ENTRY（闭形持六演＋identity hold 六演＋97th hold absorb＋bridge SET-IDENTICAL＋FLIPS 27 集二演形）插入 depth 现测承（K 唯一 run 探测＋junction 链承＝r98 tail 直贴 run）；**next-generation 序数不预写**承（r99 ENTRY 携 97th hold 引用＝ordinal gate 部署时现测）。
- **标签轮转值**：dialog r217（归休持 watch 续：双 2P 持＝归休持形 watch／l1 1F＝swing 回潮 watch／l2-only F＝leg-swap watch）／select r222（279 8F set=se.577）／password r222（225 0F）／mpreload r245（hold 八轮 realize 后：dual-leg 15/15P＝hold 九轮 watch／单腿 F＝spectrum 二演 watch／spread 加宽＝leg-drift watch）／waapi r304（picker P／checkmark F）／fd r263（121 全 P）／onblur r215（2F/8P Fset IDENT vs ob.s559＝restore-held 持 watch／Fset 变＝扩员 watch）／band r216（8 键 per-leg；8/8×2 held＝twenty-one-carry expected after r215 twenty-carry；re-absent＝absent-form watch）。
- **watch bump 锚**：post-phase5 形 wob head `wobble-key band wob1 r100 (r99 dual-P held closed-form hold sixth showing realized at s782` 段前缀定位＋s782→s783 语义重写（r99 dual-P held 闭形持六演入 wob record）。
- **states lockstep**：现尾 97 项→s783 count-guard 97→98 append **v782＝8517**（identity hold 8517 值形）＝/r99 evolutions 链＋/s782 rounds 链（98 值尾 s782 断言）。
- **analyze2 轮转承＋九坑进位**：逐行枚举式 python 轮转（双排除 analyze2:58 wob head＋bridge:1 head139 整行 verbatim 原位保）＋轮转后 wc -l 与上一代相等＋双锚 grep 各 1 审计＋chmod +x 固化承＝八坑修正承＋**九坑修正进位＝bridge-extract.sh comm 基线指针 `b782`→`b783` 三处显式替换（comm 路径＋label echo `(b782=`＋wc target `< b782.keys`）＝b783.keys 承**。

## 下一步（s783 部署清单）

- **部署门禁**：cp -a /tmp/s782/. /tmp/s783/＋drop 清单（s782.* 窗产物：leg/bridge/rc/log/analyze.out/analyze2.out/bridge.out/solos.out/runner.{pre,mid,post}.*＋armstart.s782/window.s782.judge/waiter/rearm/probe log＋legs.done/all.done/solos.done/analyze.done＋s782.opt-solo*.{log,rc}（含 .arm）＋s782-l*.norm/l582-*.canon/s782.l1u/l2u/s782.{new,gone,disagree,both,leg1only,leg2only,k1,k2,s1,s2}＋s782.bridge.{new,gone,fail,result}.keys/件＋ob.cur*/ob.f*/se.cur* 生成件＋legs.head＋runner.armdisk.*；保留 s782.stable.keys/canon＋s782.flip.keys 基线；**b783.keys＝cp b782.keys（8515be9c/1831 SET-IDENTICAL 值恒基二轮）后 drop s782.bridge.keys 件**；**b782.keys 对照保留一轮**；**b781/b780 drop**〔对照终轮满＝滚动三轮窗承〕）。
- **区间归因左锚承**：deploy 预读一律 `git log 311925425..部署 HEAD` 起算并**现读 code/docs 分型（禁沿用区间记忆＝s780 臂时翻案判例承）**；窗 HEAD 收账时同锚复核（候窗 pull 携入承＝kb305/kb306 判例＝drift note 预埋续承）。
- **runner 读数纪律承**：md5/size 一律 cat 盘 .md5/.size 文件（禁对 .md5 文件跑 md5sum＝meta-md5 陷阱承）；fresh 复核只对二进制 `md5sum target/release/zero-wpt-runner`。


> **历史巡查明细归档（2026-10 至 2026-11-26）**：完整原文已按原样迁至 [html-semantics-compat/archive/rounds-log-2026-10-to-11-26.md](html-semantics-compat/archive/rounds-log-2026-10-to-11-26.md)。

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
