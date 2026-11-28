# html-semantics M2 兼容性守成巡查（M2 切片 soak/patrol）

> **当前轮次：s822 收账完毕（2026-11-28 21:15 收账；all.done 21:08:13）｜runner 120th 二窗 hold 五点恒 d62a446a/131,943,880＝bridge 新面驻留首窗 keys 1844==b822 零差 NEW=0 GONE=0（伤亡键 execcommand 五窗 heal 未返 rcb=101）＝wob r139 双 P hold 七窗 PRIMARY＝dialog r256 l1 1F swing re-surge＝stable 8514/f2fa7074（dialog none 键离场 disagree=1＝l1 Fail/l2 Pass 单腿摆＝census Pass 8274→8273）＝cmp canon 双腿断（l582-1 970da992≠l582-2 5ba7bb2a 值恒承＝廿五窗弧终）＝ob r254 锚恒＝band 六十连＝FLIPS 28 十二窗恒＝solos 廿七演＝批墙钟 5m18s（judge 21:02:55 host load 延拓 soak）＝主读三命中二落空均预注册分支＝下一步 s823 部署发射（b823＝1844/795a50a3 同值滚转二窗）**

## 本轮状态（s822 收账＝120th 二窗 hold＝dialog 单腿摆）
- **窗形＝零移窗七演＋批墙钟 5m18s＋premature 卅一窗核过**：judge **21:02:55**（clean-soak 3×45s **拉长 9.4 min＝host load 高位 waiter 负载门延拓**＝procs=0 load<3 终过 HEAD=7fcd7cea7）→legs.head **7fcd7cea7** settled（**窗内 0 commits＝arm 20:52:50/7fcd7cea7f36fe48eeecb0f24476482a3adf7503 → window 21:02:55/7fcd7cea7＝fire-last 廿八演＝窗零移七演**）＝leg1/leg2 rc=2/2 C=0/0 链静双零→bridge 23.61s FAILED→legs.done 21:07:32→analyze 21:07:40→solos/all.done 21:08:13＝**批墙钟 5m18s**＝done 序 fresh＝premature 非复发卅一窗＝日界未跨（20:5x 窗同日）。
- **arm 记录（fire-last 廿八演＝零顺延形）**：fetch 预检 **behind=0**（20:52）→**armstart 一次成型 20:52:50/7fcd7cea7f36fe48eeecb0f24476482a3adf7503**→warm rc=0 C=log=0/err=6→probe rc=0 C=log=0/err=1 **八演**→arm solo rc=2/2 wall 17/16s C=0/0→armdisk+face1 **d62a446a/131,943,880＝120th 锚恒等**＝pids waiter 3108299/rearm 3108300/post 3108301（点火与 ps 分离调用＝三点 ppid=1 字面量直接净形＝坑九零暂态）。
- **runner＝120th 二窗 hold**：**pre=mid=post=bridge.runner=armdisk 五点恒全等 d62a446a0efc5ea180b7d7ee05f77931/131,943,880**＝世代交替第二弧二窗承（**≠臂未触发＝121st 候零**＝二窗 hold 主读命中）。
- **bridge＝新面驻留首窗**：「FAILED. **1844 passed**; 1 failed」23.61s＝rc=101＝keys **1844 (b822=1844) NEW=0 GONE=0**＝**1844==b822 零差＝桥键新面驻留首窗主读命中**＋**伤亡键 test_execcommand_format_apply_r3254_m3 仍在 fail=1＝伤亡形驻留五窗 heal 未返**（heal return keys 1845 分支未兑现＝刀75 后无刀触 execcommand）。
- **dialog＝r256 l1 1F swing re-surge 兑现（本轮唯一 corpus 面动）**：dialog-overlay「computed overlay initially none」键 **l1 Fail/l2 Pass 单腿摆**＝**stable 8515→8514（该键离场）＋disagree 0→1＋census Pass 8274→8273**＝comm -3 差恰此一键（LC_ALL=C 复核＝dialog-overlay 另一键「changes computed overlay」双腿 2P 仍在册）＝**FLIPS 28 册外新摆**（roster 无此键＝flip md5 不动）＝r256 watch l1 1F 分支在册兑现。
- **stable/can＝8514/f2fa7074＝dialog 键离场形**：stable=8514 keys md5 **f2fa7074d89fb8edddd826b814a960cf**＝s821 d36164e4 −1 键＝census **234 Fail/8273 Pass/6 Timeout/1 Unsupported**（stable.canon 状态行口径＝8514 闭；s821 同口径 234+8274+6+1=8515 闭）＝norms **8517/8517**（8515 unique＋dual-emit dup2×2＝unique 闭 8514+1 disagree）＝img sizes-auto 站立形无新动。
- **wob＝r139 双 P hold 延续 PRIMARY 命中**：**l1 Pass＋l2 Pass（band echo hits=1 双腿 Pass 直读）＝r138 hold 延续＝驻留延续七窗＝r118 多窗先例 r94/r95 对直证**（dual-F/单腿摆落空＝非盲点）。
- **cmp＝canon 双腿断（廿五窗弧终）**：**l582-1＝970da992d8e044a95af83d362beecede（dialog 摆侧）≠l582-2＝5ba7bb2a80b5d109c69ae5a1980b2406（值恒承）**＝dual-EXACT 廿五窗弧终＝l2 侧恒值在册＝split 形入账。
- **ob＝2F/2F 锚恒＋band 六十连**：l1/l2 各 2F/8P＝**f1k==f2k==ob.s559 a9e79b583edd6e2ae3ee9f836c7a14b3 三方恒承**；**band r255 8/8×2＝六十连**（canon 盘文件直读 held=8/8 双腿）。
- **哨位＝其余七项全持**：select r261 279 rows 8F/8F Fset IDENT se.577＋password r261 225 keys 0F＋mpreload r284 15P×2＋waapi r343 checkmark F/picker P×2（**checkmark heal 候六窗未发＝续 watch**）＋fd r302 121P×2＋dual-emit dup(2)×2＋rc1=2 rc2=2 rcb=101。
- **FLIPS＝28 c92accb1 十二窗 byte-IDENT 恒**（s822.flip.keys==s821.flip.keys 字节恒等）＋**solos C=0/0 rc=2/2 廿七演**。
- **主预判落定率＝主读三命中二落空（落空均预注册分支在册＝非盲点）**：runner 120th 二窗＋wob r139 PRIMARY＋bridge 新面驻留＝命中；stable 六窗/cmp 廿六窗两主读落空＝**dialog l1 1F swing 分支兑现**（r256 watch l1 1F 分支＋stable watch disagree re-open 分支均在册）＝日界未跨。
- **杂记**：s822 四脚本全绿（deploy rm=92 keep=22／rotate wc 245 全等／phase5 delta −870 lens58=218770／head oc=(32,32) head=7fcd7cea7）＝**head 门禁拦预注措辞 '121st candidate' 违 121st==0 gate→改 next-ordinal 措辞重跑绿**（gate 纪律先例）＝部署时行 58 现值九锚 count 1 预核承（OS/OE/TAIL_POS/TS/wob 头/chains/win/states）。

## 基线决策（s823）
- **s822 收账（2026-11-28 21:15）**：runner 120th 二窗 hold 五点恒 d62a446a/131,943,880＋bridge 新面驻留首窗 1844==b822 零差（伤亡驻留五窗 heal 未返）＋wob r139 七窗驻留＋dialog r256 l1 1F swing（stable 8514/f2fa7074 键离场 disagree=1＝census 8273P）＋cmp canon split（l582-1 970da992/l582-2 5ba7bb2a）＋ob/band/哨位全持＋批墙钟 5m18s＝明细见本轮状态。
- **基线轮转（s823 部署时执行）**：**b823＝cp s822.bridge.keys＝1844/795a50a34648d6679fe9b378ec927c58 桥键新面同值滚转二窗**〔s823 窗 keys 1844==b823 零差 rcb=0＝**新面驻留二窗主读**／fail=1 同伤亡键＝**伤亡驻留六窗续（键在册）**／**heal return watch＝keys 1845 fail=0 rc=0 NEW=1＝刀侧回归修复候（归 peer 流）**／GONE watch；cp 先行后 rm＝md5 双验 795a50a34648d6679fe9b378ec927c58/1844 行〕＋b822 kept one-round control〔1844/795a50a3 对照＝s824 drop 预注〕＋**b821 drop**（s634 keys/canon＋s817..s822 stable keys×6＋flips ×6 保留＝滚动窗承＝KEEP 24 锚）。
- **runner 世代锚＝d62a446a0efc5ea180b7d7ee05f77931/131,943,880（120th 二窗 hold 承）**＝s823 窗五点恒此值＝**三窗 hold 主读**；≠＝对端新刀分支＝新面候读数后回填派生 next-ordinal 候；C 形恒等形 vs 真刀面变双判据并行承。
- **主预判（s823）＝120th 三窗 hold＋dialog 摆后形四分支**：**runner 120th 五点恒三窗 watch**（armdisk==d62a446a＝三窗 hold＝主读／≠＝对端新刀）＋**bridge 新面驻留二窗 watch**（1844==b823 零差 rcb=0＝主读／fail=1 同键＝伤亡六窗续／heal return 1845／GONE watch）＋**wob r140 hold 延续 watch PRIMARY**（r139 双 P hold→l1P+l2P hold 八窗驻留主读／dual-F＝FLIPS 29＋census watch／单腿摆 watch）＋**dialog r257 摆后形 watch**（**hold l1 1F＝disagree=1 持续主读**／l1+l2 双 Fail agreed＝stable 8513 census 235F／swing-back dual-2P＝stable 回 8515＋canon 复 EXACT／leg-swap l2-only Fail）＋**stable 8514/f2fa7074 二窗 watch**（dialog hold 持续主读／swing-back 8515／新动 watch）＋**cmp 摆后 watch**（split hold 主读＝l582-1 970da992＋l582-2 5ba7bb2a／复 EXACT＝swing-back 联动＝双腿新值）＋**ob r255 watch**（2F/2F 锚 IDENT 持＝主读／计数变 watch）＋band r256 六十一连＋select r262 8F IDENT＋password r262 0F＋mpreload r285 15P＋waapi r344 checkmark heal 候七窗＋fd r303 121P＋dual-emit dup(2)×2＋FLIPS 28 十三窗 carry watch＋**solos C=0/0 廿八演**＋**fire-last 廿九演核**＋premature 卅二窗核＋日界 watch（夜窗跨日高候）＋**img sizes-auto 回摆 watch 续**。

## 下一步（s823 部署清单）
1. 目录复制与 drop 清单：/tmp/s823＝cp -a /tmp/s822＋stale markers 清除执行项（rm legs.done/analyze.done/solos.done/all.done＋legs.head/judge/armstart.s822/pids.s822.*/三链日志＋runner 四点+armdisk+bridge.runner＋armdisk.face1.bin＋solo markers+logs+walls+快照＋probe 产物＋warm.build×6（含 C）＋norms+canons+stable.canon＋new/gone/disagree/leg1only/leg2only/both/k1/k2/s1/s2＋s822.bridge.*＋se.cur*/ob.* 窗值＝premature cluster 防复发承）＝**b823.keys=cp s822.bridge.keys**（cp 先行后 rm＝md5 双验 795a50a34648d6679fe9b378ec927c58/1844 行＝同值滚转）＝drop b821.keys（KEEP 24 锚：s634 keys/canon＋s817/s818/s819/s820/s821/s822 stable keys＋flips ×6＋b822.keys 对照＋b823.keys＋wob1/band8/fd121＋五哨锚 s477×4+ob.s559）。
2. 轮转 rotate-s823.py：s822→s823 全量行枚举＋排除行 verbatim 双锚（analyze2.sh:58 wob 头 wob1 r139＋bridge-extract.sh:1 头注 s822 形）＝九坑 b822→b823 三处显式替换（comm 基线指针×2/label echo/wc target）＋body stray 零残留核＋rearm 改名＋wc -l 全等审计（245 总行）＋bash -n ×11＋chmod。
3. head-s823.py：bridge-extract.sh 头注整行重写（s822 窗账 condensed＋**runner 120th 二窗 hold 五点恒 d62a446a/131,943,880＝零顺延形**＝bridge 新面驻留首窗 keys 1844==b822 NEW=0 GONE=0 伤亡驻留五窗 fail=1 同键 rc=101＝wob r139 七窗驻留＝**dialog r256 l1 1F swing＝stable 8514 键离场 disagree=1**＝census 8273P＝cmp canon split（l582-1 970da992/l582-2 5ba7bb2a 廿五窗弧终）＝ob 锚恒＝band 六十＝FLIPS 十二窗恒＝solos 廿七演＝judge load 延拓 soak 注记＋s823 预注册 incl. wob r140 hold 延续 watch PRIMARY＝dialog r257 摆后形四分支（hold l1 1F 主读／dual-F agreed 8513/235F／swing-back 8515＋canon 复 EXACT／leg-swap）＝bridge 新面驻留二窗 watch（1844==b823／fail=1 同键伤亡六窗续／heal 1845）＝runner 三窗 watch＝stable 8514 二窗 watch＝cmp split hold watch＝b823 同值滚转注记；gate：**120th≥1＝d62a446a 命名＝121st==0**＝b823≥4＝795a50a3＋**收账 commit 后实际 HEAD 命名**〔以 commit 后 git rev-parse 短值为准＝清单不预写 hash〕＝fire-last 命名；幂等重写形承〔old 断言容 s822/s823 双形〕）。
4. phase5_s823.py（配方第七十九轮）：标签 r257/r262/r262/r285/r344/r303/r255(ob)＋band r256＋realization tokens：dialog 「; r255 dual 2P x2 = retirement-held held realized = retirement held at s821 = dialog-overlay none key 2P held both legs」→「; r256 l1 1F swing re-surge realized at s822 = dialog-overlay none key l1 Fail/l2 Pass single-leg swing =」〔watch 头句同步改摆后形〕／mpreload 「; r283 15Px2 hold-continued realized at s821」→「; r284 15Px2 hold-continued realized at s822」／waapi 「fifth window carried」→「sixth window carried」〔r343→r344〕／onblur 「; r253 anchor hold realized at s821」→「; r254 anchor hold realized at s822」〔r254→r255〕／band 「fifty-nine-carry realized at s821」→「sixty-carry realized at s822」〔r255→r256〕＋ob 子句（r254 2F/2F 锚 a9e79b58 恒承）＋wob r140 前缀（**r139 双 P hold realized＝驻留延续七窗承**）＝COMMON 首尾锚对提取（首锚「(interval at-window f0ff6a275..f0ff6a275 zero-move window = judge 20:25:32 」count 1＋TAIL_POS「arm 19:07:45/e659713c33c3f7f140c1fca9a72eaecc1b32a836 to window 19:11:39/e659713c3 0 commits no-move」前位约束〔ENTRY r135 尾〕＋尾锚「batch wall 5m23s zero ABORT)」恰 1 次＋sanity 特征词〔s821 COMMON＝`120th FOUNDING first window`＋`casualty-form residency fourth window` 校验承＝部署时先读行 58 现值核对首锚纪律承〕）＝NEW_COMMON（s822 实况＝judge 21:02:55 clean-soak 3×45s **host load 延拓 soak 注记**＝窗零移七演 fire-last 廿八演零顺延形＝**runner 120th 二窗 hold 五点恒 d62a446a/131,943,880**＝bridge 新面驻留首窗 1844==b822 零差 伤亡驻留五窗 fail=1 同键 rc=101＝stable 8514/f2fa7074 dialog none 键离场 disagree=1 census 234F/8273P/6T/1U＝FLIPS 28 十二窗恒＝cmp canon split l582-1 970da992/l582-2 5ba7bb2a 廿五窗弧终＝wob r139 hold 七窗＝dialog r256 l1 1F swing＝solos 廿七演＝批墙钟 5m18s）＝paren 平衡即断＝ENTRY r139 insert（**TAIL_STAMP＝ENTRY r138 尾戳「arm 20:22:27/f0ff6a2758fe6a128e289c16a943128f4beff79a to window 20:25:32/f0ff6a275 0 commits no-move」＋')'*32**＝r139＝wob 七窗驻留＋bridge 新面驻留首窗＋stable 8514 键离场 disagree=1＋120th 二窗 hold group＝(o_ins,c_ins−32)=(5,3)＝新尾戳「arm 20:52:50/7fcd7cea7f36fe48eeecb0f24476482a3adf7503 to window 21:02:55/7fcd7cea7 0 commits no-move」＋')'*32）＝delta −870→−900＝chains r2..r139/s685..s822/counts 137 值尾 8514＝states 位置法核证。
5. fire-last 处方廿九演：master 收账 commit/push 完成→fetch 预检（对端刀候＝有则 pull --rebase 后 arm 顺延＝armstart 记新 HEAD；**先 pull 后 arm 承**＝窗零移候选）→warm build make zero-wpt-runner-release（120th 三窗预期 C 计数非判据＋产物字节恒等 d62a446a／对端新刀＝真链新世代 next-ordinal 候）→probe（C=1 八演常形承／C 方差记账）→arm solo ×2（LC_ALL=C FILTER=select-home-end-pagedown-pageup.optional make testharness-html-semantics＝**repo 根起**＝rc/wall/C 快照）→armdisk 读＝120th 三窗形预期：d62a446a/131,943,880 恒＝三窗 hold；≠＝对端新刀＝新世代候读数后回填。
6. 三链点火：**直接形绝对路径字面量承（坑九加固形＝禁 cd 前缀复合形）**＝nohup /tmp/s823/wait-and-run.sh & 形＋armstart 两行纪律＋ps 字段法权威核（点火与 ps 分离调用＝ppid=1＋args 字面量；点火同调用内 ps 读数若遇 launcher 暂态＝壳退后复测为凭）＝窗零 .git 移候选。
7. 本提交不含 s823 臂记录（收账-点火解耦承）：点火事实（pids/armstart/时间）盘面为准＝s823 收账轮开读 pids.s823.*＋armstart.s823 核；fetch 预检在 commit 前执行。

---

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
  20:04 早于 B1 5d26a28eb 20:13 与收账 commit 22:04，实为异源）、
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
