# html-semantics M2 兼容性守成巡查（M2 切片 soak/patrol）

> **当前轮次：s821 收账完毕（2026-11-28 20:40 收账；all.done 20:30:55）｜runner 世代交替第二弧＝119th 单窗终（220f8870/131,939,944 founding-only）→120th FOUNDING 首窗五点恒 d62a446a/131,943,880＝对端刀75 f0ff6a275 面变 Δsize +3936＝bridge 桥域双动 keys 1844 NEW=2 GONE=0（伤亡键 execcommand 驻留四窗 heal 未返 rcb=101）＝wob r138 双 P hold 六窗驻留 PRIMARY＝stable 8515/d36164e4 五窗 EXACT corpus 面零动＝cmp 5ba7bb2a 廿五窗＝ob r253 锚恒＝band 五十九连＝FLIPS 28 十一窗恒＝solos 廿六演＝批墙钟 5m23s＝零盲点＝下一步 s822 部署发射（b822＝1844/795a50a3 桥键新面滚转首窗）**

## 本轮状态（s821 收账＝世代交替第二弧＝bridge 桥域双动）
- **窗形＝零移窗六演＋批墙钟 5m23s＋premature 三十窗核过**：judge 20:25:32（clean-soak 3×45s procs=0 load<3 HEAD=f0ff6a275）→legs.head **f0ff6a275** settled（**窗内 0 commits＝arm 20:22:27/f0ff6a2758fe6a128e289c16a943128f4beff79a → window 20:25:32/f0ff6a275＝fire-last 廿七演＝窗零移六演**）＝leg1/leg2 rc=2/2 C=0/0 链静双零→bridge 23.20s FAILED→legs.done 20:30:15→analyze 20:30:22→solos/all.done 20:30:55＝**批墙钟 5m23s**＝done 序 fresh＝premature 非复发三十窗＝日界未跨（20:2x 窗同日）。
- **arm 记录（fire-last 廿七演＝臂顺延形）**：首轮 armstart 20:21:00/1dcfba16d 后 fetch 预检 **behind=1＝对端刀75 f0ff6a275 fix(engine) gCS 视图 post-drain 窗口身份回植（js_dom_bridge callbacks 1 行＋computed_style_cache +146 行＝bridge 域直涉＋dynamic-changes/both-at-once 4F 回收）**→pull fast-forward（刀75 建在 1dcfba16d 之上＝双方已有）→**armstart 重录 20:22:27/f0ff6a2758fe6a128e289c16a943128f4beff79a**→warm rc=0 C=log=0/err=6（真重编）→probe rc=0 C=log=0/err=1 **七演**→arm solo rc=2/2 wall 17/16s C=0/0→armdisk+face1 **d62a446a/131,943,880≠119th 锚＝真刀面变臂时直证**＝pids waiter 3092139/rearm 3092141/post 3092144（点火同调用 ps 暂态读 ppid=3092138 launcher sleep 未退＝壳退后复测三点 ppid=1 字面量＝坑九净形非残壳）。
- **runner＝世代交替第二弧 120th FOUNDING 首窗**：**pre=mid=post=bridge.runner=armdisk 五点恒全等 d62a446a0efc5ea180b7d7ee05f77931/131,943,880（Δsize +3936）**＝**119th 单窗世代弧终（220f8870/131,939,944 s820 founding-only 存在恰一窗＝刀74→刀75 连刀形）→刀75 f0ff6a275 真链面变＝120th founding 定谳**。
- **bridge＝刀75 桥域双动（预注册 heal/新键 watch 双臂之一兑现）**：「FAILED. **1844 passed**; 1 failed」23.20s＝rc=101＝keys **1844 (b821=1842) NEW=2 GONE=0**＝**伤亡键 test_execcommand_format_apply_r3254_m3 仍在 fail=1＝伤亡形驻留四窗 heal 未返**（heal return keys 1845 分支未兑现＝刀75 未触 execcommand 伤亡）＋**2 新键入册＝computed_style_cache 刀新增测试＝桥键基线 1842/a44eb6a7→1844/795a50a34648d6679fe9b378ec927c58 面变**（伤亡形弧首次基线换代）。
- **stable＝8515/d36164e4 五窗 EXACT＝corpus 面零动**：stable=8515 disagree=0＝keys md5 **d36164e459aad1badf7336d809cb259e 与 s820 字节恒等**＝census **234 Fail/8274 Pass/6 Timeout 恒**＝**刀75 4F 回收不在 corpus 腿域**（integration/桥域收账＝corpus 无感）＝img sizes-auto 无回摆＝norms 8515/8515。
- **wob＝r138 双 P hold 延续 PRIMARY 命中**：**l1 Pass＋l2 Pass（norm 4185 行双腿 Pass 直读）＝r137 hold 延续＝驻留延续六窗＝r118 多窗先例 r94/r95 对直证**（dual-F/单腿摆落空＝非盲点）。
- **cmp＝双腿 byte-EXACT 廿五窗**：**l582-1==l582-2==5ba7bb2a80b5d109c69ae5a1980b2406**＝值恒分支兑现＝廿五窗弧续。
- **ob＝2F/2F 锚恒＋band 五十九连**：l1/l2 各 2F/8P＝**f1k==f2k==ob.s559 a9e79b583edd6e2ae3ee9f836c7a14b3 三方恒承**；**band r254 8/8×2＝五十九连**（canon 盘文件直读 held=8/8 双腿）。
- **哨位＝其余七项全持**：dialog r255 2P×2 retirement-held＋select r260 279 rows 8F/8F Fset IDENT se.577＋password r260 225 keys 0F＋mpreload r283 15P×2＋waapi r342 checkmark F/picker P×2（**checkmark heal 候五窗未发＝续 watch**）＋fd r301 121P×2＋dual-emit dup(2)×2＋rc1=2 rc2=2 rcb=101。
- **FLIPS＝28 c92accb1 十一窗 byte-IDENT 恒**（s821.flip.keys==s820.flip.keys 字节恒等）＋**solos C=0/0 rc=2/2 廿六演**。
- **主预判落定率＝零盲点（主读全命中）**：runner 120th founding（≠臂方向命中且五点恒齐）＋wob r138 PRIMARY＋bridge 新面 NEW=2（heal 分支未兑现＝改读新面）＋stable 8515 五窗＋cmp 廿五窗＋ob r253 锚恒＝全命中＝盲点 0＝日界未跨。
- **杂记**：s821 四脚本一次全绿（deploy rm=92 keep=20／rotate wc 245 全等／phase5 delta −840 lens58=216788／head oc=(32,32) head=1dcfba16d）＝KEEP 锚计数 18→20（滚动窗 +stable/+flip 对）＝head 盘面预注 220f8870 二窗与臂时实况 ≠ 不改盘（先例承＝收账以实况承）。

## 基线决策（s822）
- **s821 收账（2026-11-28 20:40）**：runner 世代交替第二弧 120th founding 首窗五点恒 d62a446a/131,943,880＋bridge 桥域双动 1844 NEW=2（伤亡驻留四窗 heal 未返）＋wob r138 六窗驻留＋stable 8515/d36164e4 五窗 EXACT＋cmp 5ba7bb2a 廿五窗＋ob/band/哨位全持＋批墙钟 5m23s＝明细见本轮状态。
- **基线轮转（s822 部署时执行）**：**b822＝cp s821.bridge.keys＝1844/795a50a34648d6679fe9b378ec927c58 桥键新面滚转首窗**〔s822 窗 keys 1844==b822 零差 rcb=0＝**新面驻留主读**／fail=1 同伤亡键＝**伤亡驻留五窗续（键在册）**／**heal return watch＝keys 1845 fail=0 rc=0 NEW=1＝刀侧回归修复候（归 peer 流）**／GONE watch；cp 先行后 rm＝md5 双验 795a50a34648d6679fe9b378ec927c58/1844 行〕＋b821 kept one-round control〔1842/a44eb6a7 对照＝s823 drop 预注〕＋**b820 drop**（s634 keys/canon＋s817..s821 stable keys×5＋flips ×5 保留＝滚动窗承＝KEEP 22 锚）。
- **runner 世代锚＝d62a446a0efc5ea180b7d7ee05f77931/131,943,880（120th founding 首窗确立）**＝s822 窗五点恒此值＝**二窗 hold 主读**；≠＝对端新刀分支＝新面候读数后回填派生 121th 候；C 形恒等形 vs 真刀面变双判据并行承。
- **主预判（s822）＝120th 二窗 hold＋桥键新面驻留**：**runner 120th 五点恒二窗 watch**（armdisk==d62a446a＝二窗 hold＝主读／≠＝对端新刀）＋**bridge 新面驻留 watch**（1844==b822 零差 rcb=0＝主读／fail=1 同键＝伤亡五窗续／heal return 1845／GONE watch）＋**wob r139 hold 延续 watch PRIMARY**（r138 双 P hold→l1P+l2P hold 七窗驻留主读／dual-F＝FLIPS 29＋census watch／单腿摆 watch）＋**stable 8515/d36164e4 六窗 watch**（主读／再漂 watch）＋**cmp 廿六窗 watch**（5ba7bb2a 值恒主读／census 动＝新值双腿一致／wob 行破 watch）＋**ob r254 watch**（2F/2F 锚 IDENT 持＝主读／计数变 watch）＋band r255 六十连＋dialog r256 retirement-held＋select r261 8F IDENT＋password r261 0F＋mpreload r284 15P＋waapi r343 checkmark heal 候六窗＋fd r302 121P＋dual-emit dup(2)×2＋FLIPS 28 carry watch＋**solos C=0/0 廿七演**＋**fire-last 廿八演核**＋premature 卅一窗核＋日界 watch（下窗跨日高候）＋**img sizes-auto 回摆 watch 续**。

## 下一步（s822 部署清单）
1. 目录复制与 drop 清单：/tmp/s822＝cp -a /tmp/s821＋stale markers 清除执行项（rm legs.done/analyze.done/solos.done/all.done＋legs.head/judge/armstart.s821/pids.s821.*/三链日志＋runner 四点+armdisk+bridge.runner＋armdisk.face1.bin＋solo markers+logs+walls+快照＋probe 产物＋warm.build×6（含 C）＋norms+canons+stable.canon＋new/gone/disagree/leg1only/leg2only/both/k1/k2/s1/s2＋s821.bridge.*＋se.cur*/ob.* 窗值＝premature cluster 防复发承）＝**b822.keys=cp s821.bridge.keys**（cp 先行后 rm＝md5 双验 795a50a34648d6679fe9b378ec927c58/1844 行＝新面滚转）＝drop b820.keys（KEEP 22 锚：s634 keys/canon＋s817/s818/s819/s820/s821 stable keys＋flips ×5＋b821.keys 对照＋b822.keys＋wob1/band8/fd121＋五哨锚 s477×4+ob.s559）。
2. 轮转 rotate-s822.py：s821→s822 全量行枚举＋排除行 verbatim 双锚（analyze2.sh:58 wob 头 wob1 r138＋bridge-extract.sh:1 头注 s821 形）＝九坑 b821→b822 三处显式替换（comm 基线指针×2/label echo/wc target）＋body stray 零残留核＋rearm 改名＋wc -l 全等审计（245 总行）＋bash -n ×11＋chmod。
3. head-s822.py：bridge-extract.sh 头注整行重写（s821 窗账 condensed＋**runner 世代交替第二弧 119th 单窗终→120th founding 首窗五点恒 d62a446a/131,943,880＝刀75 f0ff6a275 面变**＝bridge 桥域双动 keys 1844 NEW=2 伤亡驻留四窗 fail=1 同键 rc=101＝wob r138 六窗驻留＝stable 8515 五窗 EXACT＝cmp 廿五窗＝ob 锚恒＝band 五十九＝FLIPS 十一窗恒＝solos 廿六演＋s822 预注册 incl. wob r139 hold 延续 watch PRIMARY＝bridge 新面驻留 watch（1844==b822／fail=1 同键伤亡五窗续／heal 1845）＝runner 二窗 watch＝stable 六窗 watch＝cmp 廿六窗 watch＝b822 新面滚转注记；gate：**120th≥1＝d62a446a 命名＝119th 仅史值**＝b822≥4＝795a50a3＋**收账 commit 后实际 HEAD 命名**〔以 commit 后 git rev-parse 短值为准＝清单不预写 hash〕＝fire-last 命名；幂等重写形承〔old 断言容 s821/s822 双形〕）。
4. phase5_s822.py（配方第七十八轮）：标签 r256/r261/r261/r284/r343/r302/r254(ob)＋band r255（五十九 realized 承）＋ob 子句（r253 2F/2F 锚 a9e79b58 恒承）＋wob r139 前缀（**r138 双 P hold realized＝驻留延续六窗承**）＝COMMON 首尾锚对提取（首锚「(interval at-window c79a8238d..c79a8238d zero-move window fifth showing = judge 20:05:52 」count 1〔含 fifth showing 限定＝COMMON 专属；ENTRY r137 interval 为短形无 fifth showing＝部署时先读行 58 现值核对首锚纪律承〕＋TAIL_POS「arm 19:07:45/e659713c33c3f7f140c1fca9a72eaecc1b32a836 to window 19:11:39/e659713c3 0 commits no-move」前位约束〔ENTRY r135 尾〕＋尾锚「batch wall 5m43s zero ABORT)」恰 1 次＋sanity 特征词〔s820 COMMON＝`119th FOUNDING first window`＋`casualty-form residency third window`＋`wob r137 dual-P hold continuation` 校验承＝部署时现读核对〕）＝NEW_COMMON（s821 实况＝judge 20:25:32 clean-soak＝窗零移六演 fire-last 廿七演臂顺延形＝**runner 世代交替第二弧 119th 单窗终→120th founding 首窗五点恒 d62a446a/131,943,880＝刀75 面变**＝bridge 桥域双动 1844 NEW=2 伤亡驻留四窗 fail=1 同键 rc=101＝stable 8515/d36164e4 五窗 EXACT census 234F/8274P/6T＝FLIPS 28 十一窗恒＝cmp 5ba7bb2a 廿五窗＝wob r138 hold 六窗＝solos 廿六演＝批墙钟 5m23s）＝paren 平衡即断＝ENTRY r138 insert（**TAIL_STAMP＝ENTRY r137 尾戳「arm 20:02:02/c79a8238dd25af522c396868fa316b9e77b61f16 to window 20:05:52/c79a8238d 0 commits no-move」＋')'*32**＝r138＝wob 六窗驻留＋bridge 桥域双动＋stable 五窗＋120th founding group＝(o_ins,c_ins−32)=(5,3)＝新尾戳「arm 20:22:27/f0ff6a2758fe6a128e289c16a943128f4beff79a to window 20:25:32/f0ff6a275 0 commits no-move」＋')'*32）＝delta −840→−870＝chains r2..r138/s685..s821/counts 136 值尾 8515＝states 位置法核证。
5. fire-last 处方廿八演：master 收账 commit/push 完成→fetch 预检（对端刀候＝有则 pull --rebase 后 arm 顺延＝armstart 记新 HEAD；**先 pull 后 arm 承**＝窗零移候选）→warm build make zero-wpt-runner-release（120th 二窗预期 C 计数非判据＋产物字节恒等 d62a446a／对端新刀＝真链新世代）→probe（C=1 七演常形承／C 方差记账）→arm solo ×2（LC_ALL=C FILTER=select-home-end-pagedown-pageup.optional make testharness-html-semantics＝**repo 根起**＝rc/wall/C 快照）→armdisk 读＝120th 二窗形预期：d62a446a/131,943,880 恒＝二窗 hold；≠＝对端新刀＝新世代候读数后回填。
6. 三链点火：**直接形绝对路径字面量承（坑九加固形＝禁 cd 前缀复合形）**＝nohup /tmp/s822/wait-and-run.sh & 形＋armstart 两行纪律＋ps 字段法权威核（ppid=1＋args 字面量；点火同调用内 ps 读数若遇 launcher 暂态＝壳退后复测为凭）＝窗零 .git 移候选。
7. 本提交不含 s822 臂记录（收账-点火解耦承）：点火事实（pids/armstart/时间）盘面为准＝s822 收账轮开读 pids.s822.*＋armstart.s822 核；fetch 预检在 commit 前执行。

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
