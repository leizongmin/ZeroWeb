# html-semantics M2 兼容性守成巡查（M2 切片 soak/patrol）

> **当前轮次：s817 收账完毕（2026-11-28 18:58 收账；all.done 18:46:56）｜runner 118th 五点恒首窗＝bridge re-casualty rc=101＝键 execcommand_format_apply_r3254_m3＝stable 8515 face 变 d36164e4 census 234F/8274P/6T＝GONE=3+NEW=1 img sizes-auto 文件形变＝FLIPS 28 roster c92accb1 恒＝cmp 双腿 byte-EXACT 新面 5ba7bb2a 廿一窗＝wob r134 双 P hold＝ob 锚恒＝peer 刀 0213b2bdc 归因记账＝批墙钟 5m43s＝下一步 s818 部署发射**

## 本轮状态（s817 收账＝runner 118th 首窗＝bridge re-casualty＝stable face 变＝wob r134 hold）
- **窗形＝零移窗＋批墙钟 5m43s＋premature 廿六窗核过**：judge 18:41:13（clean-soak 3×45s procs=0 load<3）→legs.head **5043b94dc** settled（**窗内 0 commits＝arm 18:37:53/5043b94dc62253537b2db2e113ecd1872483d49d → window 18:41:13/5043b94dc＝fire-last 廿三演＝先 pull 后 arm 承＝窗零移二演**）＝leg1 18:41:13→18:43:18（2m05s）→leg2 →18:45:24（2m06s）rc=2/2→bridge 22.58s→legs.done 18:46:10→analyze 18:46:23→solos/all.done 18:46:56＝**批墙钟 5m43s（s816 5m03s +40s＝leg 墙钟 +13/+9s post-knife 观察项＋bridge 伤亡记账）**＝done 序 fresh＝premature 非复发廿六窗＝日界未跨（18:41 窗）。
- **runner＝118th 五点恒首窗 PRIMARY 命中（新世代确立）**：**armdisk≠c5ee4ef9 分支兑现＝8bd19b988df5a91b3342570549e33c18/131,935,144（Δsize +213,616 vs 117th 131,721,528）＝真刀面变（s813 判例形）＝118th 世代臂时确立**＝窗内 pre=mid=post=bridge.runner=armdisk **五点全等 8bd19b98**＝leg1 C=0＋leg2 C=0 链静双零（warm 预编承）。
- **arm 记录（fire-last 廿三演）**：armstart 2026-11-28 18:37:53/5043b94dc62253537b2db2e113ecd1872483d49d＝fetch 预检 behind=0（peer 5de2f5905 R78000＋cea48ec86 刀72 订正全 docs 已 rebase 入树＝0213b2bdc 唯一真码刀）→warm rc=0 **40s C=6（log=0/err=6）＝product-version→net→engine→page-runtime→webview→wpt-runner 全链真刀重编**→probe rc=0 C=1（log=0/err=1）**三演常形确立**→arm solo rc=2/2 wall 17/16s C=0/0→armdisk+face1 8bd19b98＝pids waiter 3031138/rearm 3031137/post 3031139（args 绝对路径字面量全对；**waiter pids 初值 wrapper 残壳 3031136＝cd && nohup & 复合形邻坑九＝cmdline 匹配订正 3031138＝记账新形**）＝**warm.build.C/solo1.C.arm/solo2.C.arm 盘文件本轮显式落盘＝缺位异常三演止**。
- **bridge＝re-casualty 分支命中（治愈驻留四窗弧终结）**：「test result: FAILED. 1842 passed; 1 failed」（runner 自身 22.58s）＝**rc=101＝rcb=101＝预注册 fail=1 re-casualty watch 分支兑现**＝keys **1842 md5 a44eb6a780ee08e3bdba958312effd4d ≠ b817**＝GONE=1 fail=1 NEW=0＝**伤亡键 js_dom_bridge::tests::test_execcommand_format_apply_r3254_m3**（execCommand format apply＝StyleFlushBarrier/gCS 路径涟漪）＝vs b816 单键差恰同键＝单伤亡净账＝s809 治愈→s817 刀断四窗驻留弧承。
- **stable＝face 变 8515/d36164e4（census shift 分支兑现）**：stable=8515 disagree=0 l1only=0 l2only=0＝keys md5 **d36164e459aad1badf7336d809cb259e**（8517/057764d3 三窗主读落空＝刀面变分支）＝census **234 Fail/8274 Pass（−3）/6 Timeout（+1）**/1 Unsupported＝**GONE=3＋NEW=1 全落 img sizes-auto-dynamic-attributes.html**（3 子测消失＋文件级条目新现＝新 Timeout 行＝gCS flush 路径涟漪归因）＝**FLIPS 28 roster c92accb1 六窗恒（flip 集未动＝刀零扰）**＝norms 8517/8517（−2 恒承）。
- **cmp＝双腿 byte-EXACT 新面廿一窗**：**l582-1==l582-2==5ba7bb2a80b5d109c69ae5a1980b2406**＝6251237e 值恒持主读落空＝**预注册「census moved→NEW byte value both-legs-agreed new face」分支兑现**＝双腿一致新面＝廿一窗 byte-EXACT 弧续。
- **wob＝r134 双 P hold 延续 PRIMARY 命中**：**l1 Pass＋l2 Pass（canon 尾态直读 hits=1×2）＝r118 closed-form hold 多窗先例＝驻留延续二窗**（dual-F/单腿摆落空＝非盲点）。
- **ob＝2F/2F 锚恒**：l1 2F/8P＋l2 2F/8P＝**f1k==f2k==ob.s559 a9e79b583edd6e2ae3ee9f836c7a14b3 三方恒承**。
- **哨位＝其余八项全持＋band 五十五连**：dialog r251 2P×2 retirement-held 持＋select r256 279 rows 8F/8F Fset IDENT se.577＋password r256 225 keys 0F 双腿＋mpreload r279 15P×2 hold-continued＋waapi r338 checkmark F/picker P×2（**checkmark heal 候落空＝刀未愈此键＝非盲点**）＋fd r297 121P×2＋onblur r249 2F/8P 锚＋dual-emit dup(2)×2＋**band r250 8/8×2＝五十五连**＋rc1=2 rc2=2 rcb=101＋stack overflow 三零。
- **solos＝C=0/0 廿二演＋rc=2/2**：窗内 opt-solo1 rc=2＋opt-solo2 rc=2＝C=0/0 零级联（盘文件本轮在位直证）＝norms 8517/8517。
- **主预判落定率＝零盲点（两主读落空皆落预注册分支）**：runner ≠c5ee4ef9 刀分支兑现＋bridge fail=1/rc101 分支兑现＋wob r134 hold 主读命中＋ob 锚恒＋band 五十五＋FLIPS 28 carry＋solos 廿二演＋fire-last 廿三演＋premature 廿六窗＋日界未跨＝**stable 三窗主读落空→census shift 刀分支＋cmp 值恒主读落空→新面一致分支＝两落空皆预注册分支兑现＝盲点 0**。
- **杂记＝跨流碰撞归因记账**：**bridge 伤亡＋img sizes 形变归因 peer 刀 0213b2bdc（fix engine animation registry/flush＝对端流工作面＝engine 共享面）＝本流只记录不修＝跨流碰撞记账承（对端 R79000 处方自有 animation 面回归钉测）**＝peer 5de2f5905（R78000 收账 docs）＋cea48ec86（刀72 订正 docs）窗前落树 docs-only＝fire-last 计数勘误 d2370ae2e 已订正（s816 窗＝廿二演/s817＝廿三演）。

## 基线决策（s818）
- **s817 收账（2026-11-28 18:58）**：runner 118th 五点恒首窗 8bd19b98/131,935,144＋bridge re-casualty 1842/1 failed rc=101 键 execcommand_format_apply_r3254_m3＋stable 8515/d36164e4 census 234F/8274P/6T＋GONE=3/NEW=1 img sizes-auto 形变＋FLIPS 28 c92accb1 恒＋cmp 双腿 byte-EXACT 新面 5ba7bb2a 廿一窗＋wob r134 双 P hold＋ob 锚恒＝明细见本轮状态。
- **基线轮转（s818 部署时执行）**：**b818＝cp s817.bridge.keys＝1842/a44eb6a7 值锚（刀后伤亡形基线＝新面滚转非同值滚转）**〔s818 窗 keys 1842==b818 零差 rcb=0＝**刀后伤亡形驻留主读**／fail=2 二次伤亡 watch／**heal return watch＝execcommand 键复 P→NEW=1+fail=0+rc=0＝刀侧回归修复候（归 peer 流）**；cp 先行后 rm 承＝md5 双验 a44eb6a780ee08e3bdba958312effd4d/1842 行〕＋b817 kept one-round control〔1843/32e005b9 对照＝s819 drop 预注〕＋**b816 drop**＋**s814 系 drop**（s634 keys/canon＋s815/s816/s817 stable keys＋flips s815/s816/s817 保留＝滚动窗承）。
- **runner 世代锚＝8bd19b988df5a91b3342570549e33c18/131,935,144（118th hold 二窗承）**＝s818 窗五点恒此值＝hold 主读；armdisk≠8bd19b98＝对端新刀分支＝新面候读数后回填派生 119th 候（读数后回填纪律承）；**C 形恒等形 vs 真刀面变双判据并行承**。
- **主预判（s818）＝刀后驻留窗**：**runner 118th 五点恒二窗 watch PRIMARY**（armdisk==8bd19b98/131,935,144＝二窗 hold＝主读／≠＝对端新刀＝新世代候）＋**bridge 刀后伤亡形驻留 watch**（keys 1842 a44eb6a7==b818 零差 rcb=0＝主读／fail=2 二次伤亡 watch／heal return watch）＋**wob r135 hold 延续 watch PRIMARY**（r134 双 P hold→l1P+l2P hold 三窗驻留主读＝r118 多窗先例／dual-F＝FLIPS 29＋census watch／单腿摆 watch）＋**stable 8515/d36164e4 二窗 watch**（刀后面稳二窗＝主读／再漂 watch＝对端新刀候）＋**cmp 双腿 byte-EXACT 恒持 watch**（5ba7bb2a 廿二窗候主读／wob 行破 watch）＋**ob r250 watch**（2F/2F 锚 Fset IDENT 持＝主读／计数变 watch）＋band r251 五十六连＋dialog r252 retirement-held＋select r257 8F IDENT＋password r257 0F＋mpreload r280 15P＋waapi r339 checkmark heal 候续记＋fd r298 121P＋dual-emit dup(2)×2＋FLIPS 28 carry watch＋**solos C=0/0 廿三演**＋**fire-last 廿四演核**＋premature 廿七窗核＋日界 watch（晚间窗＝跨日候）＋**img sizes-auto 形变归档 watch**（GONE=3/NEW=1 是否随对端修复回摆＝回摆即回摆记账）。

## 下一步（s818 部署清单）
1. 目录复制与 drop 清单：/tmp/s818＝cp -a /tmp/s817＋stale markers 清除执行项（rm legs.done/analyze.done/solos.done/all.done＋legs.head/judge/armstart/pids/三链日志＋runner 四点+armdisk+bridge.runner＋armdisk.face1.bin＋solo markers+logs+walls+快照＋probe 产物＋warm.build×6（含 C）＋norms+canons+stable.canon＋new/gone/disagree/leg1only/leg2only/both/k1/k2/s1/s2＋s817.bridge.*＋se.cur*/ob.* 窗值＝premature cluster 防复发承）＝**b818.keys=cp s817.bridge.keys**（cp 先行后 rm＝md5 双验 a44eb6a780ee08e3bdba958312effd4d/1842 行＝**刀后伤亡形基线**）＝drop b816.keys＋s814.*＋s814-*（KEEP 18 锚：s634 keys/canon＋s815/s816/s817 stable keys＋flips s815/s816/s817＋b817.keys 对照＋b818.keys＋wob1/band8/fd121＋五哨锚 s477×4+ob.s559）。
2. 轮转 rotate-s818.py：s817→s818 全量行枚举＋排除行 verbatim 双锚（analyze2.sh:58 wob 头 wob1 r134＋bridge-extract.sh:1 头注 s817 形）＝九坑 b817→b818 三处显式替换（comm 基线指针×2/label echo/wc target）＋body stray 零残留核＋rearm 改名＋wc -l 全等审计（245 总行）＋bash -n ×11＋chmod。
3. head-s818.py：bridge-extract.sh 头注整行重写（s817 窗账 condensed＋runner 118th 首窗＝bridge re-casualty rc=101 伤亡键＝stable 8515 face 变＝GONE/NEW img sizes 形变＝FLIPS 恒＝cmp 新面廿一窗＝wob r134 hold＝ob 锚恒＋s818 预注册 incl. wob r135 hold 延续 watch PRIMARY＝bridge 刀后伤亡形驻留 watch＝stable 二窗 watch＝cmp 廿二窗 watch＝ob 锚恒 watch＝b818 伤亡形滚转注记；gate：**118th≥1＝119th==0**＝b818≥4＝8bd19b98＋a44eb6a7＋**收账 commit 后实际 HEAD 命名**〔以 commit 后 git rev-parse 短值为准＝清单不预写 hash〕＝fire-last 命名；幂等重写形承〔old 断言容 s817/s818 双形〕＝序数门禁 catch 纪律承）。
4. phase5_s818.py（配方第七十四轮＝十坑承）：标签 r252/r257/r257/r280/r339/r298/r250(ob)＋band r251（五十五 realized 承）＋ob 子句（r249 2F/2F 锚 a9e79b58 恒承）＋wob r135 前缀（**r134 双 P hold realized＝r118 多窗先例承**）＝COMMON 首尾锚对提取（首锚「(interval at-window d4427be14..d4427be14 zero-move window arm-tree same-form restore = judge 18:12:30 」**count 2 find-first**〔COMMON 与 ENTRY r133 interval 共享短前缀形承＝部署时先读 s817/analyze2.sh 行 58 现值核对首锚〕＋TAIL_POS「arm 17:34:38/811797c90 to window 17:37:53/2c55e0e7d 1 commits docs-move」前位约束〔ENTRY r132 尾〕＋尾锚「batch wall 5m03s zero ABORT)」恰 1 次〔COMMON 专属〕＋sanity 特征词〔s816 COMMON＝`117th hold five-point constant`＋`wob r133 dual-P held`＋`fourth-window hold` 校验承＝部署时现读核对〕）＝NEW_COMMON（s817 实况＝judge 18:41:13 clean-soak＝窗零移二演＝runner 118th 首窗 8bd19b98＝bridge re-casualty rc=101 键 execcommand＝stable 8515/d36164e4 census 234F/8274P/6T＝GONE=3/NEW=1 img sizes 形变＝FLIPS 28 恒＝cmp 新面 5ba7bb2a 廿一窗＝wob r134 hold＝批墙钟 5m43s）＝paren 平衡即断＝ENTRY r134 insert（**TAIL_STAMP＝ENTRY r133 尾戳「arm 18:08:54/d4427be143a0ee5a1013aeaf9e07562cddc95b6d to window 18:12:30/d4427be14 0 commits no-move」＋')'*32**＝代锚坑承＝r134＝118th 首窗＋bridge re-casualty＋stable face 变 group＝首行括注形 (o_ins,c_ins−32)=(5,3)＝新尾戳「arm 18:37:53/5043b94dc62253537b2db2e113ecd1872483d49d to window 18:41:13/5043b94dc 0 commits no-move」＋')'*32）＝chains r2..r134/s685..s817/counts 132 值尾 8515＝states 位置法核证。
5. fire-last 处方廿四演：master 收账 commit/push 完成→fetch 预检（对端刀候＝有则 pull --rebase 后 arm 顺延＝armstart 记新 HEAD；**先 pull 后 arm 承**＝窗零移候选）→warm build make zero-wpt-runner-release（118th hold 预期 C=0 链静／对端新刀＝C>0 真链）→probe（C=1 三演常形承／C 方差记账）→arm solo ×2（LC_ALL=C FILTER=select-home-end-pagedown-pageup.optional make testharness-html-semantics＝**repo 根起**＝rc/wall/C 快照）→armdisk 读＝118th hold 形预期：8bd19b98/131,935,144 恒＝hold；≠＝对端新刀＝新世代候读数后回填。
6. 三链点火：nohup 绝对路径字面量承＋坑九禁令承＝**禁 cd 前缀复合形**（s817 waiter pids 残壳邻坑九教训＝直接 nohup /tmp/s818/wait-and-run.sh & 形）＋armstart 两行纪律＋ps 字段法权威核＋cmdline 匹配订正承＝窗零 .git 移候选。
7. 本提交不含 s818 臂记录（收账-点火解耦承）：点火事实（pids/armstart/时间）盘面为准＝s818 收账轮开读 pids.s818.*＋armstart.s818 核；fetch 预检在 commit 前执行。

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
