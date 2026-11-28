# html-semantics M2 兼容性守成巡查（M2 切片 soak/patrol）

> **当前轮次：s819 收账完毕（2026-11-28 19:46 收账；all.done 19:42:08）｜wob r136 双 P hold 四窗驻留 PRIMARY＝bridge 伤亡形驻留二窗 1842==b819 零差 fail=1 同键 rcb=101＝runner 118th 五点恒三窗 8bd19b98/131,935,144＝stable 8515/d36164e4 三窗 EXACT＝cmp 5ba7bb2a 廿三窗＝ob r251 锚恒 a9e79b58＝band 五十七连＝FLIPS 28 九窗恒＝solos 廿四演＝批墙钟 5m18s＝零盲点＝下一步 s820 部署发射**

## 本轮状态（s819 收账＝驻留二窗＝六主读全命中）
- **窗形＝零移窗四演＋批墙钟 5m18s＋premature 廿八窗核过**：judge 19:36:50（clean-soak 3×45s procs=0 load<3 HEAD=900836aee）→legs.head **900836aee** settled（**窗内 0 commits＝arm 19:33:17/900836aee37e9aa4e431755da720db35eb5db83e → window 19:36:50/900836aee＝fire-last 廿五演＝窗零移四演**）＝leg1/leg2 rc=2/2 C=0/0 链静双零→bridge 23.85s FAILED→legs.done 19:41:29→analyze 19:41:35→solos/all.done 19:42:08＝**批墙钟 5m18s**＝done 序 fresh＝premature 非复发廿八窗＝日界未跨（19:36 窗）。
- **arm 记录（fire-last 廿五演）**：armstart 2026-11-28 19:33:17/900836aee37e9aa4e431755da720db35eb5db83e＝fetch 预检 behind=0（push 后 pull 无新进＝8cb58a064 R81000 收账 docs 已入树）→warm rc=0 **C=6 计数非判据常形＝产物字节恒等 8bd19b98/131,935,144＝C 形同形臂时兑现（118th hold）**→probe rc=0 C=1（log=0/err=1）**五演常形**→arm solo rc=2/2 wall 16/17s C=0/0→armdisk+face1 8bd19b98＝pids waiter 3062428/rearm 3062444/post 3062455（直接形绝对路径字面量＝坑九加固形零残壳＝ps 字段法三点全对）。
- **wob＝r136 双 P hold 延续 PRIMARY 命中**：**l1 Pass＋l2 Pass（canon 尾态直读 hits=3 双腿 Pass）＝r135 hold 延续＝驻留延续四窗＝r118 多窗先例 r94/r95 对直证**（dual-F/单腿摆落空＝非盲点）。
- **bridge＝伤亡形驻留二窗（预注册主读分支兑现）**：「FAILED. 1842 passed; 1 failed」23.85s＝rc=101＝keys **1842 md5 a44eb6a780ee08e3bdba958312effd4d == b819 字节恒等零差 NEW=0 GONE=0**＝fail=1 **同伤亡键 test_execcommand_format_apply_r3254_m3**＝**驻留二窗＝heal return 未发**（keys 1843 分支落空＝非盲点＝heal 候三窗 watch 承）。
- **runner＝118th 五点恒三窗**：pre=mid=post=bridge.runner=armdisk **全等 8bd19b98/131,935,144**＝三窗 hold 主读兑现。
- **stable＝8515/d36164e4 三窗 EXACT**：stable=8515 disagree=0 l1only=0 l2only=0＝keys md5 **d36164e459aad1badf7336d809cb259e 与 s818 字节恒等**＝census **234 Fail/8274 Pass/6 Timeout 恒**＝img sizes-auto 无回摆无新面（ledger GONE=3/NEW=1 对 s634 常驻账承袭）＝norms 8517/8517＝l1u/l2u 8515。
- **cmp＝双腿 byte-EXACT 廿三窗**：**l582-1==l582-2==5ba7bb2a80b5d109c69ae5a1980b2406**＝值恒分支兑现＝廿三窗弧续。
- **ob＝2F/2F 锚恒＋band 五十七连**：l1/l2 各 2F/8P＝**f1k==f2k==ob.s559 a9e79b583edd6e2ae3ee9f836c7a14b3 三方恒承**；**band r252 8/8×2＝五十七连**（canon 盘文件直读 present=8/8 双腿）。
- **哨位＝其余七项全持**：dialog r253 2P×2 retirement-held＋select r258 279 rows 8F/8F Fset IDENT se.577＋password r258 225 keys 0F＋mpreload r281 15P×2＋waapi r340 checkmark F/picker P×2（**checkmark heal 候三窗未发＝续 watch**）＋fd r299 121P×2＋dual-emit dup(2)×2＋rc1=2 rc2=2 rcb=101。
- **FLIPS＝28 c92accb1 九窗 byte-IDENT 恒**（s819.flip.keys==s818.flip.keys 字节恒等）＋**solos C=0/0 rc=2/2 廿四演**。
- **主预判落定率＝零盲点（六主读全命中无落空分支）**：wob r136 PRIMARY＋bridge 驻留二窗＋runner 118th 三窗＋stable 8515 三窗＋cmp 廿三窗＋ob r251 锚恒＝全命中＝盲点 0＝日界未跨。
- **杂记**：s819 四脚本一次全绿（deploy rm=90 keep=18／rotate wc 245 全等／phase5 delta −780／head oc=(29,29)；head 首跑括号失衡 −1 即修＝ paren 平衡断承效）＝滚动窗 KEEP 承（s817 stable/flip keys 本轮 drop 落实）。

## 基线决策（s820）
- **s819 收账（2026-11-28 19:46）**：wob r136 四窗驻留＋bridge 驻留二窗 1842/a44eb6a7 fail=1 同键 rcb=101＋runner 118th 五点恒三窗＋stable 8515/d36164e4 三窗 EXACT＋cmp 5ba7bb2a 廿三窗＋ob/band/哨位全持＋批墙钟 5m18s＝明细见本轮状态。
- **基线轮转（s820 部署时执行）**：**b820＝cp s819.bridge.keys＝1842/a44eb6a7 同值滚转（伤亡形驻留基线三窗）**〔s820 窗 keys 1842==b820 零差 rcb=0＝**驻留三窗主读**／fail=2 二次伤亡 watch／**heal return watch＝keys 1843 fail=0 rc=0 NEW=1＝刀侧回归修复候（归 peer 流）**；cp 先行后 rm＝md5 双验 a44eb6a780ee08e3bdba958312effd4d/1842 行〕＋b819 kept one-round control〔同值对照＝s821 drop 预注〕＋**b818 drop**（s634 keys/canon＋s817/s818/s819 stable keys＋flips s817/s818/s819 保留＝滚动窗承）。
- **runner 世代锚＝8bd19b988df5a91b3342570549e33c18/131,935,144（118th hold 四窗承）**＝s820 窗五点恒此值＝hold 主读；armdisk≠8bd19b98＝对端新刀分支＝新面候读数后回填派生 119th 候；C 形恒等形 vs 真刀面变双判据并行承。
- **主预判（s820）＝驻留三窗**：**runner 118th 五点恒四窗 watch**（armdisk==8bd19b98＝四窗 hold＝主读／≠＝对端新刀）＋**bridge 驻留三窗 watch**（1842==b820 零差 rcb=0＝主读／fail=2 二次伤亡／heal return watch）＋**wob r137 hold 延续 watch PRIMARY**（r136 双 P hold→l1P+l2P hold 五窗驻留主读／dual-F＝FLIPS 29＋census watch／单腿摆 watch）＋**stable 8515/d36164e4 四窗 watch**（主读／再漂＝对端新刀候）＋**cmp 廿四窗 watch**（5ba7bb2a 值恒主读／census 动＝新值双腿一致／wob 行破 watch）＋**ob r252 watch**（2F/2F 锚 IDENT 持＝主读／计数变 watch）＋band r253 五十八连＋dialog r254 retirement-held＋select r259 8F IDENT＋password r259 0F＋mpreload r282 15P＋waapi r341 checkmark heal 候四窗＋fd r300 121P＋dual-emit dup(2)×2＋FLIPS 28 carry watch＋**solos C=0/0 廿五演**＋**fire-last 廿六演核**＋premature 廿九窗核＋日界 watch（晚间窗＝跨日候）＋**img sizes-auto 回摆 watch 续**。

## 下一步（s820 部署清单）
1. 目录复制与 drop 清单：/tmp/s820＝cp -a /tmp/s819＋stale markers 清除执行项（rm legs.done/analyze.done/solos.done/all.done＋legs.head/judge/armstart.s819/pids.s819.*/三链日志＋runner 四点+armdisk+bridge.runner＋armdisk.face1.bin＋solo markers+logs+walls+快照＋probe 产物＋warm.build×6（含 C）＋norms+canons+stable.canon＋new/gone/disagree/leg1only/leg2only/both/k1/k2/s1/s2＋s819.bridge.*＋se.cur*/ob.* 窗值＝premature cluster 防复发承）＝**b820.keys=cp s819.bridge.keys**（cp 先行后 rm＝md5 双验 a44eb6a780ee08e3bdba958312effd4d/1842 行＝同值滚转）＝drop b818.keys（KEEP 18 锚：s634 keys/canon＋s817/s818/s819 stable keys＋flips s817/s818/s819＋b819.keys 对照＋b820.keys＋wob1/band8/fd121＋五哨锚 s477×4+ob.s559）。
2. 轮转 rotate-s820.py：s819→s820 全量行枚举＋排除行 verbatim 双锚（analyze2.sh:58 wob 头 wob1 r136＋bridge-extract.sh:1 头注 s819 形）＝九坑 b819→b820 三处显式替换（comm 基线指针×2/label echo/wc target）＋body stray 零残留核＋rearm 改名＋wc -l 全等审计（245 总行）＋bash -n ×11＋chmod。
3. head-s820.py：bridge-extract.sh 头注整行重写（s819 窗账 condensed＋wob r136 四窗驻留＝bridge 驻留二窗 1842==b819 fail=1 同键 rc=101＝runner 118th 三窗＝stable 8515 三窗 EXACT＝cmp 廿三窗＝ob 锚恒＝band 五十七＝FLIPS 九窗恒＝solos 廿四演＋s820 预注册 incl. wob r137 hold 延续 watch PRIMARY＝bridge 驻留三窗 watch＝runner 四窗 watch＝stable 四窗 watch＝cmp 廿四窗 watch＝b820 同值滚转注记；gate：**118th≥1＝119th==0**＝b820≥4＝8bd19b98＋a44eb6a7＋**收账 commit 后实际 HEAD 命名**〔以 commit 后 git rev-parse 短值为准＝清单不预写 hash〕＝fire-last 命名；幂等重写形承〔old 断言容 s819/s820 双形〕）。
4. phase5_s820.py（配方第七十六轮）：标签 r254/r259/r259/r282/r341/r300/r252(ob)＋band r253（五十七 realized 承）＋ob 子句（r251 2F/2F 锚 a9e79b58 恒承）＋wob r137 前缀（**r136 双 P hold realized＝驻留延续四窗承**）＝COMMON 首尾锚对提取（首锚「(interval at-window e659713c3..e659713c3 zero-move window third showing = judge 19:11:39 」count 1〔含 third showing 限定＝COMMON 专属；ENTRY r135 interval 为短形无 third showing＝部署时先读行 58 现值核对首锚纪律承〕＋TAIL_POS「arm 18:08:54/d4427be143a0ee5a1013aeaf9e07562cddc95b6d to window 18:12:30/d4427be14 0 commits no-move」前位约束〔ENTRY r133 尾〕＋尾锚「batch wall 5m14s zero ABORT)」恰 1 次＋sanity 特征词〔s818 COMMON＝`118th hold five-point constant`＋`bridge post-knife casualty-form residency`＋`wob r135 dual-P hold continuation` 校验承＝部署时现读核对〕）＝NEW_COMMON（s819 实况＝judge 19:36:50 clean-soak＝窗零移四演 fire-last 廿五演＝runner 118th 三窗 8bd19b98＝bridge 驻留二窗 1842/a44eb6a7 fail=1 同键 rc=101＝stable 8515/d36164e4 三窗 EXACT census 234F/8274P/6T＝FLIPS 28 九窗恒＝cmp 5ba7bb2a 廿三窗＝wob r136 hold 四窗＝solos 廿四演＝批墙钟 5m18s）＝paren 平衡即断＝ENTRY r136 insert（**TAIL_STAMP＝ENTRY r135 尾戳「arm 19:07:45/e659713c33c3f7f140c1fca9a72eaecc1b32a836 to window 19:11:39/e659713c3 0 commits no-move」＋')'*32**＝r136＝wob 四窗驻留＋bridge 驻留二窗＋stable 三窗 group＝(o_ins,c_ins−32)=(5,3)＝新尾戳「arm 19:33:17/900836aee37e9aa4e431755da720db35eb5db83e to window 19:36:50/900836aee 0 commits no-move」＋')'*32）＝delta −780→−810＝chains r2..r136/s685..s819/counts 134 值尾 8515＝states 位置法核证。
5. fire-last 处方廿六演：master 收账 commit/push 完成→fetch 预检（对端刀候＝有则 pull --rebase 后 arm 顺延＝armstart 记新 HEAD；**先 pull 后 arm 承**＝窗零移候选）→warm build make zero-wpt-runner-release（118th hold 预期 C=6 计数非判据常形＋产物字节恒等 8bd19b98／对端新刀＝真链新世代）→probe（C=1 五演常形承／C 方差记账）→arm solo ×2（LC_ALL=C FILTER=select-home-end-pagedown-pageup.optional make testharness-html-semantics＝**repo 根起**＝rc/wall/C 快照）→armdisk 读＝118th hold 形预期：8bd19b98/131,935,144 恒＝hold 四窗；≠＝对端新刀＝新世代候读数后回填。
6. 三链点火：**直接形绝对路径字面量承（坑九加固形＝禁 cd 前缀复合形）**＝nohup /tmp/s820/wait-and-run.sh & 形＋armstart 两行纪律＋ps 字段法权威核（ppid=1＋args 字面量）＝窗零 .git 移候选。
7. 本提交不含 s820 臂记录（收账-点火解耦承）：点火事实（pids/armstart/时间）盘面为准＝s820 收账轮开读 pids.s820.*＋armstart.s820 核；fetch 预检在 commit 前执行。

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
