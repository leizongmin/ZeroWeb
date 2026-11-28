# html-semantics M2 兼容性守成巡查（M2 切片 soak/patrol）

> **当前轮次：s818 收账完毕（2026-11-28 19:22 收账；all.done 19:16:53）｜wob r135 双 P hold 三窗驻留 PRIMARY＝bridge 刀后伤亡形驻立首窗 1842==b818 零差 fail=1 同键 rcb=101＝runner 118th 五点恒二窗 8bd19b98/131,935,144＝stable 8515/d36164e4 二窗 EXACT＝cmp 5ba7bb2a 廿二窗＝ob r250 锚恒 a9e79b58＝band 五十六连＝FLIPS 28 八窗恒＝solos 廿三演＝批墙钟 5m14s＝零盲点＝下一步 s819 部署发射**

## 本轮状态（s818 收账＝刀后驻留窗＝六主读全命中）
- **窗形＝零移窗三演＋批墙钟 5m14s＋premature 廿七窗核过**：judge 19:11:39（clean-soak 3×45s procs=0 load<3 HEAD=e659713c3）→legs.head **e659713c3** settled（**窗内 0 commits＝arm 19:07:45/e659713c33c3f7f140c1fca9a72eaecc1b32a836 → window 19:11:39/e659713c3＝fire-last 廿四演＝窗零移三演**）＝leg1/leg2 rc=2/2 C=0/0 链静双零→bridge 23.41s FAILED→legs.done 19:16:14→analyze 19:16:20→solos/all.done 19:16:53＝**批墙钟 5m14s**＝done 序 fresh＝premature 非复发廿七窗＝日界未跨（19:11 窗）。
- **arm 记录（fire-last 廿四演）**：armstart 2026-11-28 19:07:45/e659713c33c3f7f140c1fca9a72eaecc1b32a836＝fetch 预检 behind=0（pull 入 peer 两笔 e659713c3 R80000 收账＋58ce21e7b 刀73 归因全 docs＝四介入 commit 全 docs 零码差＝清单 HEAD 预期 ed3fa86db 被 peer docs 擦肩刷成 e659713c3＝docs-only 臂静承）→warm rc=0 **C=6（log=0/err=6）计数非判据常形＝产物字节恒等 8bd19b98/131,935,144＝C 形同形臂时兑现（118th hold）**→probe rc=0 C=1（log=0/err=1）**四演常形**→arm solo rc=2/2 wall 16/16s C=0/0→armdisk+face1 8bd19b98＝pids waiter 3047920/rearm 3047950/post 3047956（**直接形绝对路径字面量点火＝坑九加固形＝零残壳＝ps 字段法三点全对**）。
- **wob＝r135 双 P hold 延续 PRIMARY 命中**：**l1 Pass＋l2 Pass＝r134 hold 延续＝驻留延续三窗＝r118 closed-form hold 多窗先例 r94/r95 对直证**（dual-F/单腿摆落空＝非盲点）。
- **bridge＝刀后伤亡形驻立首窗（预注册主读分支兑现）**：「FAILED. 1842 passed; 1 failed」23.41s＝rc=101＝keys **1842 md5 a44eb6a780ee08e3bdba958312effd4d == b818 字节恒等零差 NEW=0 GONE=0**＝fail=1 **同伤亡键 test_execcommand_format_apply_r3254_m3**（非二次伤亡非新伤亡）＝**刀后伤亡形驻立＝heal return 未发**（keys 1843 分支落空＝非盲点＝heal 候续 watch）。
- **runner＝118th 五点恒二窗**：pre=mid=post=bridge.runner=armdisk **全等 8bd19b98/131,935,144**＝二窗 hold 主读兑现。
- **stable＝8515/d36164e4 二窗 EXACT**：stable=8515 disagree=0 l1only=0 l2only=0＝keys md5 **d36164e459aad1badf7336d809cb259e 与 s817 字节恒等**＝census **234 Fail/8274 Pass/6 Timeout 恒**＝**img sizes-auto 无回摆无新面＝s817→s818 corpus 面零移动**（ledger GONE=3/NEW=1 为对 s634 长基线常驻账承袭）＝norms 8517/8517＝l1u/l2u 8515。
- **cmp＝双腿 byte-EXACT 廿二窗**：**l582-1==l582-2==5ba7bb2a80b5d109c69ae5a1980b2406**＝值恒分支兑现＝廿二窗弧续。
- **ob＝2F/2F 锚恒＋band 五十六连**：l1/l2 各 2F/8P＝**f1k==f2k==ob.s559 a9e79b583edd6e2ae3ee9f836c7a14b3 三方恒承**；**band r251 8/8×2＝五十六连**（canon 盘文件直读 present=8/8 双腿）。
- **哨位＝其余七项全持**：dialog r252 2P×2 retirement-held＋select r257 279 rows 8F/8F Fset IDENT se.577＋password r257 225 keys 0F＋mpreload r280 15P×2＋waapi r339 checkmark F/picker P×2（**checkmark heal 候二窗未发＝续 watch**）＋fd r298 121P×2＋dual-emit dup(2)×2＋rc1=2 rc2=2 rcb=101。
- **FLIPS＝28 c92accb1 八窗 byte-IDENT 恒**（s818.flip.keys==s817.flip.keys 字节恒等＝刀零扰承）＋**solos C=0/0 rc=2/2 廿三演**。
- **主预判落定率＝零盲点（六主读全命中无落空分支）**：wob r135 PRIMARY＋bridge 驻立 1842==b818＋runner 118th 二窗＋stable 8515 二窗＋cmp 廿二窗＋ob r250 锚恒＝全命中＝盲点 0＝日界未跨。
- **杂记＝部署脚本锚笔误订正**：s818 清单 phase5 OS 首锚写「短形 count 2」，实况 COMMON 头含 arm-tree 同形子句＝长形 count 1（ENTRY r134 interval 方为短形）＝phase5 首跑 assert 即断＝按活体文件改锚一次跑绿＝**部署时先读行 58 现值核对首锚纪律承（配方七十四轮在案）**。

## 基线决策（s819）
- **s818 收账（2026-11-28 19:22）**：wob r135 三窗驻留＋bridge 刀后伤亡形驻立 1842/a44eb6a7 fail=1 同键 rcb=101＋runner 118th 五点恒二窗＋stable 8515/d36164e4 二窗 EXACT＋cmp 5ba7bb2a 廿二窗＋ob/band/哨位全持＋批墙钟 5m14s＝明细见本轮状态。
- **基线轮转（s819 部署时执行）**：**b819＝cp s818.bridge.keys＝1842/a44eb6a7 同值滚转（刀后伤亡形驻立基线二窗）**〔s819 窗 keys 1842==b819 零差 rcb=0＝**伤亡形驻留二窗主读**／fail=2 二次伤亡 watch／**heal return watch＝keys 1843 fail=0 rc=0 NEW=1＝刀侧回归修复候（归 peer 流）**；cp 先行后 rm＝md5 双验 a44eb6a780ee08e3bdba958312effd4d/1842 行〕＋b818 kept one-round control〔同值对照＝s820 drop 预注〕＋**b817 drop**（s634 keys/canon＋s816/s817/s818 stable keys＋flips s816/s817/s818 保留＝滚动窗承）。
- **runner 世代锚＝8bd19b988df5a91b3342570549e33c18/131,935,144（118th hold 三窗承）**＝s819 窗五点恒此值＝hold 主读；armdisk≠8bd19b98＝对端新刀分支＝新面候读数后回填派生 119th 候；C 形恒等形 vs 真刀面变双判据并行承。
- **主预判（s819）＝刀后驻留二窗**：**runner 118th 五点恒三窗 watch**（armdisk==8bd19b98＝三窗 hold＝主读／≠＝对端新刀）＋**bridge 伤亡形驻留二窗 watch**（1842==b819 零差 rcb=0＝主读／fail=2 二次伤亡／heal return watch）＋**wob r136 hold 延续 watch PRIMARY**（r135 双 P hold→l1P+l2P hold 四窗驻留主读／dual-F＝FLIPS 29＋census watch／单腿摆 watch）＋**stable 8515/d36164e4 三窗 watch**（主读／再漂＝对端新刀候）＋**cmp 廿三窗 watch**（5ba7bb2a 值恒主读／census 动＝新值双腿一致／wob 行破 watch）＋**ob r251 watch**（2F/2F 锚 IDENT 持＝主读／计数变 watch）＋band r252 五十七连＋dialog r253 retirement-held＋select r258 8F IDENT＋password r258 0F＋mpreload r281 15P＋waapi r340 checkmark heal 候三窗＋fd r299 121P＋dual-emit dup(2)×2＋FLIPS 28 carry watch＋**solos C=0/0 廿四演**＋**fire-last 廿五演核**＋premature 廿八窗核＋日界 watch（晚间窗＝跨日候）＋**img sizes-auto 回摆 watch 续**。

## 下一步（s819 部署清单）
1. 目录复制与 drop 清单：/tmp/s819＝cp -a /tmp/s818＋stale markers 清除执行项（rm legs.done/analyze.done/solos.done/all.done＋legs.head/judge/armstart.s818/pids.s818.*/三链日志＋runner 四点+armdisk+bridge.runner＋armdisk.face1.bin＋solo markers+logs+walls+快照＋probe 产物＋warm.build×6（含 C）＋norms+canons+stable.canon＋new/gone/disagree/leg1only/leg2only/both/k1/k2/s1/s2＋s818.bridge.*＋se.cur*/ob.* 窗值＝premature cluster 防复发承）＝**b819.keys=cp s818.bridge.keys**（cp 先行后 rm＝md5 双验 a44eb6a780ee08e3bdba958312effd4d/1842 行＝同值滚转）＝drop b817.keys（KEEP 18 锚：s634 keys/canon＋s816/s817/s818 stable keys＋flips s816/s817/s818＋b818.keys 对照＋b819.keys＋wob1/band8/fd121＋五哨锚 s477×4+ob.s559）。
2. 轮转 rotate-s819.py：s818→s819 全量行枚举＋排除行 verbatim 双锚（analyze2.sh:58 wob 头 wob1 r135＋bridge-extract.sh:1 头注 s818 形）＝九坑 b818→b819 三处显式替换（comm 基线指针×2/label echo/wc target）＋body stray 零残留核＋rearm 改名＋wc -l 全等审计（245 总行）＋bash -n ×11＋chmod。
3. head-s819.py：bridge-extract.sh 头注整行重写（s818 窗账 condensed＋wob r135 三窗驻留＝bridge 刀后伤亡形驻立 1842==b818 fail=1 同键 rc=101＝runner 118th 二窗＝stable 8515 二窗 EXACT＝cmp 廿二窗＝ob 锚恒＝band 五十六＝FLIPS 八窗恒＝solos 廿三演＋s819 预注册 incl. wob r136 hold 延续 watch PRIMARY＝bridge 驻留二窗 watch＝runner 三窗 watch＝stable 三窗 watch＝cmp 廿三窗 watch＝b819 同值滚转注记；gate：**118th≥1＝119th==0**＝b819≥4＝8bd19b98＋a44eb6a7＋**收账 commit 后实际 HEAD 命名**〔以 commit 后 git rev-parse 短值为准＝清单不预写 hash〕＝fire-last 命名；幂等重写形承〔old 断言容 s818/s819 双形〕）。
4. phase5_s819.py（配方第七十五轮）：标签 r253/r258/r258/r281/r340/r299/r251(ob)＋band r252（五十六 realized 承）＋ob 子句（r250 2F/2F 锚 a9e79b58 恒承）＋wob r136 前缀（**r135 双 P hold realized＝驻留延续三窗承**）＝COMMON 首尾锚对提取（首锚「(interval at-window 5043b94dc..5043b94dc zero-move window third showing = judge 18:41:13 」count 1〔含 third showing 限定＝COMMON 专属；ENTRY r134 interval 为短形无 third showing＝部署时先读行 58 现值核对〕＋TAIL_POS「arm 17:34:38/811797c90 to window 17:37:53/2c55e0e7d 1 commits docs-move」前位约束〔ENTRY r132 尾〕＋尾锚「batch wall 5m43s zero ABORT)」恰 1 次＋sanity 特征词〔s817 COMMON＝`118th generation founding`＋`wob r134 dual-P hold continuation`＋`five-point constant first-window` 校验承＝部署时现读核对〕）＝NEW_COMMON（s818 实况＝judge 19:11:39 clean-soak＝窗零移三演 fire-last 廿四演＝runner 118th 二窗 8bd19b98 C 形同形＝bridge 驻立 1842/a44eb6a7 fail=1 同键 rc=101＝stable 8515/d36164e4 二窗 EXACT census 234F/8274P/6T＝FLIPS 28 八窗恒＝cmp 5ba7bb2a 廿二窗＝wob r135 hold 三窗＝solos 廿三演＝批墙钟 5m14s）＝paren 平衡即断＝ENTRY r135 insert（**TAIL_STAMP＝ENTRY r134 尾戳「arm 18:37:53/5043b94dc62253537b2db2e113ecd1872483d49d to window 18:41:13/5043b94dc 0 commits no-move」＋')'*32**＝r135＝wob 三窗驻留＋bridge 驻立＋stable 二窗 group＝(o_ins,c_ins−32)=(5,3)＝新尾戳「arm 19:07:45/e659713c33c3f7f140c1fca9a72eaecc1b32a836 to window 19:11:39/e659713c3 0 commits no-move」＋')'*32）＝delta −750→−780＝chains r2..r135/s685..s818/counts 133 值尾 8515＝states 位置法核证。
5. fire-last 处方廿五演：master 收账 commit/push 完成→fetch 预检（对端刀候＝有则 pull --rebase 后 arm 顺延＝armstart 记新 HEAD；**先 pull 后 arm 承**＝窗零移候选）→warm build make zero-wpt-runner-release（118th hold 预期 C=6 计数非判据常形＋产物字节恒等 8bd19b98／对端新刀＝真链新世代）→probe（C=1 四演常形承／C 方差记账）→arm solo ×2（LC_ALL=C FILTER=select-home-end-pagedown-pageup.optional make testharness-html-semantics＝**repo 根起**＝rc/wall/C 快照）→armdisk 读＝118th hold 形预期：8bd19b98/131,935,144 恒＝hold 三窗；≠＝对端新刀＝新世代候读数后回填。
6. 三链点火：**直接形绝对路径字面量承（坑九加固形＝禁 cd 前缀复合形＝s817 残壳教训承）**＝nohup /tmp/s819/wait-and-run.sh & 形＋armstart 两行纪律＋ps 字段法权威核（ppid=1＋args 字面量）＝窗零 .git 移候选。
7. 本提交不含 s819 臂记录（收账-点火解耦承）：点火事实（pids/armstart/时间）盘面为准＝s819 收账轮开读 pids.s819.*＋armstart.s819 核；fetch 预检在 commit 前执行。

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
