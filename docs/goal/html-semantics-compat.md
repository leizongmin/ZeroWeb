# html-semantics M2 兼容性守成巡查（M2 切片 soak/patrol）

> **当前轮次：s808 收账完毕（2026-11-28 15:20 收账；all.done 15:18:33）｜点火 15:08:34→judge 15:13:10（soak 两轮拍即时形＝零对端占用/legs.head 3344d65fc settled＝fire-last 十四演兑现）＝**runner 116th hold＝五点恒 PRIMARY 命中**（530f3f4c/131,697,032 pre=mid=post=bridge.runner=armdisk 全等＝116th 候二窗 hold＝刀71 分支落空＝树上零源码变化一致）＝**bridge 二十一演 carry**（keys 1842/e9c9aa0e==b808 零差＝NEW=0 GONE=0＝fail=1≡r11 九窗 carry）＝**wob r125 dual-P return-to-held＝PRIMARY 命中＝两窗闭**（r117/r123 先例＝摆-回-摆-回 周期形 s805 out/s806 in/s807 out/s808 in）＝**stable 8517/057764d3＋canon 793bd6cd byte resume 三演＝s804/s806 面复原**（disagree=0 GONE 解消＝cmp legs-IDENT 复立＝canon 双腿 6251237e byte-EXACT 十三窗＝leg2 从 d2ec5a30 resume 复立）＝FLIPS 28 c92accb1 byte-IDENT＝band 四十六连＝solos 十三演＝哨位全持＝零 MISS｜下一步＝s809 部署发射（fire-last 十五演＋bridge 二十二演 carry watch＋wob r126 closed-form hold PRIMARY＋stable byte-IDENT hold 候＋runner 116th hold 三窗候/刀71 分支 watch）**

## 本轮状态（s808 收账＝runner 116th hold 五点恒＝bridge 二十一演 carry＝wob r125 return-to-held＝stable byte resume 三演）
- **窗形＝soak 两轮拍即时形＋批墙钟 5m23s＋premature cluster 十七窗核过**：点火 15:08:34→judge 15:13:10（**4m36s 即时形＝soak 两轮拍方差形**〔s807 95s 单拍→本窗两拍＝arm build 后 load 缓降＝waiter 设计内〕＝零对端占用＝procs=0 load<3 三连 45s 达成即判）；HEAD=3344d65fc settled（arm 同树＝**0 commits no-move**＝窗零 .git 移）；leg1 15:13:10→15:15:05 1m55s **C=0**＋leg2 15:15:05→15:17:09 2m04s **C=0**＝双腿零级联十四演（Compiling=0 双腿＝树上零源码变化承）；bridge →legs.done 15:17:44→analyze 15:18:00→solos 15:18:33→all.done 15:18:33＝done-marker 顺序净（十七窗核过）＝批墙钟 5m23s 零 ABORT。
- **runner＝116th hold PRIMARY 命中＝五点恒**：armdisk 530f3f4c2006ecdcfb5222523b177ab7/131,697,032==臂时读数＝窗内 pre=mid=post=bridge.runner=armdisk **五点全等**＝**116th 候二窗四点恒 hold 兑现**＝刀71 分支落空（树上零源码变化一致＝docs-only 3344d65fc 承）＝warm build 76s C=0（零新级联＝s807 fire 已吸收刀70 承＝新鲜度扫描形）＝probe C=0。
- **bridge＝SET-stability 二十一演 carry＝r11 九窗**：rc=101「FAILED. 1842 passed; 1 failed」＝keys 1842 md5 e9c9aa0e==b808 零差＝NEW=0 GONE=0＝fail=1≡`test_element_local_name_r11` **九窗 carry**＝rcb=101 恒＝bridge.runner==armdisk verified ON 116th hold＝b809 二十二演候 staged（carry watch）。
- **stable＝8517/057764d3＋canon 793bd6cd byte resume 三演＝PRIMARY 命中＝s804/s806 面复原**：stable=8517 disagree=0 l1only=0 l2only=0 NEW=0 **GONE=0**＝keys md5 057764d3==s806.stable.keys（==s804 密封面）＋canon md5 793bd6cd==s806 stable.canon＝**双锚 byte resume 三演兑现＝GONE 解消＝wob 键归队 Pass**；census 234 Fail/8277 Pass/5 Timeout/1 Unsupported 双腿；**cmp legs-IDENT 复立＝canon 双腿 6251237e byte-EXACT＝十三窗兑现（leg2 从 d2ec5a30 resume 复立＝leg1 十三窗连续持）**（norms 双腿 8519 行）。
- **wob＝r125 dual-P return-to-held＝PRIMARY 命中＝两窗闭**：r124 l1P/l2F mirror-swap 单窗→r125 双腿 Pass 归队＝**r117/r123 先例 return 两窗闭**（swap/mirror 后 return 周期形闭合）＝direction flip-back/r122-form 逆 swap/dual-P held 迟到 hold/dual-F/silent 均落空；**wob 摆链 stable.keys 四点＝s805 out/s806 in/s807 out/s808 in＝摆-回-摆-回 周期形**（r122 swap→r123 return→r124 mirror-swap→r125 return＝2 周期）；ob count l1 2F/l2 2F＝ob family wob-domain overlap form resolved（r123 同形＝锚外净形复原）。
- **哨位＝十二项全持＋band 四十六连**：dialog r242 2P×2 retirement-held＋select r247 279 rows 8F/8F Fset 双腿 IDENT se.577＋**canon 6251237e byte-IDENT 封印承**〔select 域 seal 探针纪律＝canon 派生形为准＝norm 派生探针（状态前列格式）不可作跨窗封印＝s807 轮 sorted 形纪律细化承〕＋password r247 225 keys 0F 双腿＋mpreload r270 0F＝15P×2 hold-continued＋waapi r329 checkmark F/picker P×2＋fd r288 121 [121 Pass] ×2＋dual-emit dup(2)×2（wrapping-transformation＋execution-timing/149 双腿 n=2）＋**ob r240 锚 Fset 三方恒 a9e79b58（f1k==f2k==ob.s559 双腿 IDENT＝2F/2F 净形复原＝wob 键退出 l2 F 侧）**＋**band r241 8/8×2 Fail＝四十六连**＋rc1=2 rc2=2 rcb=101＋stack overflow 三零。
- **solos＝C=0/0 十三演**：窗 solo C=0/0 rc=2/2＝臂 solo C=0/0 rc=2/2（**wall 34/36s vs 前窗 16/17s＝方差形记账**）＝处方 hold 十三窗连续；warm build 76s C=0＝probe C=0。
- **主预判落定率＝零 MISS（PRIMARY 命中 11）**：wob r125 return-to-held（PRIMARY 命中）＋stable byte resume（PRIMARY 命中）＋cmp legs-IDENT byte-EXACT 十三窗（PRIMARY 命中）＋**runner 116th hold（PRIMARY 命中＝刀71 分支落空）**＋bridge 二十一演 carry（PRIMARY 命中）＋band 四十六连＋dialog/select/password/mpreload/waapi/fd/ob/dual-emit 全持＋FLIPS 28＋solos 十三演＋fire-last 十四演（judge 即时形兑现）＋premature 十七窗＋日界未跨＝盲点落空 0。
- **证据纪律＝fire-last 处方十四演兑现**：收账 commit/push 3344d65fc（14:58）→fetch 预检 behind=0→warm build（76s C=0）→probe（C=0）→arm solo ×2（repo 根 C=0/0）→armdisk 读（530f3f4c/131,697,032＝116th hold）→armstart 两行（15:08:34/3344d65fc 全 sha）→三链点火（ps cmdline 核＝waiter 2710580/rearm 2710579/post 2710581＝**waiter pid 文件 wrapper 假象坑复踩＝&& 前置致 $! 捕外层＝ps 字段精确法重写＝锚形三 OK 承**）。

## 基线决策（s809）
- **s808 臂记录（15:08:34）**：三链点火 waiter 2710580/rearm 2710579/post 2710581（ps cmdline 锚形核承＝waiter pid wrapper 假象坑 ps 字段法重写承）；armstart 两行 2026-11-28 15:08:34/3344d65fc9f700e3fd90baf6624dfff64b8d6552；deploy files=37 sh=11 keep=18（b808=cp s807.bridge.keys＝md5 双验 e9c9aa0e/1842 行＝b807 kept control＋b806 drop＋s804 系 drop 滚动窗承）；fetch 预检 behind=0→warm build 76s C=0→probe C=0→arm solo C=0/0 rc=2/2（wall 34/36s 方差承）→armdisk 530f3f4c/131,697,032＝**116th hold 臂时确立＝四点恒二窗承**。
- **s808 收账（2026-11-28 15:20）**：runner 116th hold 五点恒＋bridge 二十一演 carry（r11 九窗）＋wob r125 dual-P return-to-held（PRIMARY＝两窗闭＝摆-回-摆-回 周期形）＋stable 8517/057764d3＋793bd6cd byte resume 三演＝s804/s806 面复原＋cmp legs-IDENT byte-EXACT 十三窗＝零 MISS＝明细见本轮状态。
- **基线轮转（s809 部署时执行）**：**b809＝cp s808.bridge.keys＝e9c9aa0e/1842 值锚**（SET-stability 二十二演候承＝cp 先行后 rm 承＝md5 双验 e9c9aa0e2aad1d71bd1b7cdf4e2a1e9c/1842 行）＋b808 kept one-round control〔二十一演 carry 后零差对照；s810 drop 预注〕＋b807 drop〔kept-control 滚动窗承＝s807 drop 预注兑现〕＋s805 系 drop（s634/s806/s807/s808 stable keys＋flips s806/s807/s808 保留＝滚动窗承）。
- **runner 世代锚＝530f3f4c2006ecdcfb5222523b177ab7/131,697,032（116th 候 hold 三窗承）**＝s809 窗三点恒此值＝hold；armdisk≠530f3f4c＝刀71 落树分支现役（**116th 已收二窗 hold＝三窗 hold 候＝读数后回填收账轮派生 117th 候**＝读数后回填纪律承）。
- **主预判（s809）＝全 PRIMARY hold 窗**：**runner hold watch PRIMARY-candidate**（armdisk==530f3f4c＝116th 候四点恒三窗持＝主读／≠530f3f4c＝刀71 embed 分支＝新面候读数后回填）＋**wob r126 dual-P held closed-form hold watch PRIMARY**（r125 return 单窗→r118 先例 return 后 hold 承＝stable join hold 续／silent＝held watch／swap re-open＝re-open watch／mirror-swap＝direction flip watch＝摆-回周期再开 watch／dual-F＝agreed-Fail reopen＝FLIPS 29 watch）＋**stable byte-IDENT hold watch PRIMARY**（8517/057764d3＋793bd6cd 恒＝面复原承恒／wob 摆→8516 形 watch／census Δ＝刀71 面效应 watch）＋cmp legs-IDENT（byte-EXACT 6251237e 十四窗候 PRIMARY／恰 1 行差 wob form watch）＋ob r241 anchor Fset IDENT（2F/2F hold 续／摆动回潮 2F/3F watch／扩锚 family re-open watch）＋band r242 四十七连＋dialog r243 retirement-held＋select r248 8F IDENT＋password r248 0F＋mpreload r271 15P＋waapi r330＋fd r289 121P＋dual-emit dup(2)×2＋**bridge 二十二演 carry watch**（PRIMARY＝keys 1842/e9c9aa0e==b809 零差＝rcb=101 fail=1 r11 十演＝carry 确立／NEW=1 r11 回归＝peer bridge heal watch／fail>1＝新 casualty watch）＋**solos C=0/0 十四演**＋**fire-last 十五演核**＋premature 十八窗核＋日界 watch。

## 下一步（s809 部署清单）
1. 目录复制与 drop 清单：/tmp/s809＝cp -a /tmp/s808＋stale markers 清除执行项（rm legs.done/analyze.done/solos.done/all.done＋legs.head/judge/armstart/pids/三链日志＋runner 四点+armdisk+bridge.runner＋solo markers+logs+walls+快照＋probe 产物＋warm.build×4＋norms+canons+stable.canon＋new/gone/disagree/leg1only/leg2only/both/k1/k2/s1/s2＋s808.bridge.*＋se.cur*/ob.* 窗值＝premature cluster 防复发承）＝**b809.keys=cp s808.bridge.keys**（cp 先行后 rm＝md5 双验 e9c9aa0e2aad1d71bd1b7cdf4e2a1e9c/1842 行）＝drop b807.keys＋s805.* （KEEP 18 锚：s634 keys/canon＋s806/s807/s808 stable keys＋flips s806/s807/s808＋b808.keys 对照＋b809.keys＋wob1/band8/fd121＋五哨锚）。
2. 轮转 rotate-s809.py：s808→s809 全量行枚举＋排除行 verbatim 双锚（analyze2.sh:58 wob 头 wob1 r125＋bridge-extract.sh:1 头注 s808 形）＝九坑 b808→b809 三处显式替换（comm 基线指针×2/label echo/wc target）＋body stray 零残留核＋rearm 改名＋wc -l 全等审计（245 总行）＋bash -n ×11＋chmod。
3. head-s809.py：bridge-extract.sh 头注整行重写（s808 窗账 condensed＋runner 116th hold 五点恒＋bridge 二十一演 carry r11 九窗＋wob r125 return-to-held 两窗闭＋stable byte resume 三演＝s804/s806 面复原＋s809 预注册 incl. runner 116th hold PRIMARY-candidate＋wob r126 closed-form hold PRIMARY＋stable byte-IDENT hold＋bridge 二十二演 carry watch；gate：**116th≥1＝117th==0**＝b809≥4＝530f3f4c＋e9c9aa0e＋3344d65fc 命名＝fire-last 命名；幂等重写形承〔old 断言容 s808/s809 双形〕＝序数门禁 catch 纪律承〔s807 轮拦截例〕）。
4. phase5_s809.py（配方第六十五轮＝十坑承：part A 前窗实现子句锚按 s807 写＝rotation s808→s809 只触 s808 token；rep_a 续跑形承）：标签 r243/r248/r248/r271/r330/r289/r241(ob)＋band r242（四十六连 realized 承）＋ob 子句 r240 锚 Fset IDENT held realized 承（wob r125 return count l1 2F/l2 2F＝ob family wob-domain overlap form resolved 承）＋wob r126 前缀（r125 dual-P return-to-held realized 承＝r117/r123 先例两窗闭＝摆-回周期形）＝COMMON 首尾锚对提取（首锚「(interval at-window d084f7ff4..d084f7ff4 settled-tree unchanged window = judge 14:50:08 immediate form = peer zero occupation = 」**count 2 find-first**〔COMMON 与 ENTRY r124 interval 同形〕＋TAIL_POS「arm 12:46:22/0ef657441 to window 12:47:54/0ef657441 0 commits no-move」前位约束〔ENTRY r121 尾〕＋尾锚「batch wall 5m23s zero ABORT)」恰 1 次〔COMMON 专属＝ENTRY r124 尾逗号形〕＋sanity 承：'via pull' 不在＋长度窗＋特征词〔s807 COMMON＝`knife-70 branch new-face four-point constant` 特征词校验〕）＝NEW_COMMON（s808 实况＝judge 15:13:10 soak 两轮拍即时形＋runner 116th hold 五点恒 530f3f4c/131,697,032＝bridge 二十一演 carry rcb=101 r11 九窗＝wob r125 return-to-held stable join resume 8517 byte resume 三演＝批墙钟 5m23s）＝paren 平衡即断＝ENTRY r125 insert（**TAIL_STAMP＝ENTRY r124 尾戳「arm 14:48:33/d084f7ff4 to window 14:50:08/d084f7ff4 0 commits no-move」＋')'*32**＝代锚坑承＝r125＝116th hold 五点恒＋二十一演 carry＋wob return-to-held＋stable byte resume group＝首行括注形 (o_ins,c_ins−32)=(5,3)＝新尾戳「arm 15:08:34/3344d65fc to window 15:13:10/3344d65fc 0 commits no-move」＋')'*32）＝chains r2..r125/s685..s808/counts 124 值尾 8517＝states 位置法核证。
5. fire-last 处方十五演：master 收账 commit/push 完成→fetch 预检（刀71 候＝有则 pull --rebase 后重跑 warm build＋arm 顺延＝armstart 记新 HEAD）→warm build→probe→arm solo ×2（LC_ALL=C FILTER=select-home-end-pagedown-pageup.optional make testharness-html-semantics＝**repo 根起**＝rc/wall/C 快照）→armdisk 读（md5sum＋stat 真二进制直读）＝116th hold 形预期：530f3f4c/131,697,032 恒＝hold（三窗承）；armdisk≠530f3f4c＝刀71 分支＝新面候读数后回填。
6. 三链点火：nohup 绝对路径 waiter/rearm/post＋**ps cmdline 锚形核**（pgrep 复合命令引号假象坑承＋nohup &&链截断坑承＝**点火命令逐行/分号分隔〔s808 轮 && 前置 wrapper 假象复踩＝ps 字段法重写承＝本次首链禁 && 前置〕**）＋pids 三件＋armstart 两行纪律＝窗零 .git 移＝leg C=0 双腿候。
7. 本提交不含 s809 臂记录（收账-点火解耦承）：点火事实（pids/armstart/时间）盘面为准＝s809 收账轮开读 pids.s809.*＋armstart.s809 核；fetch 预检在 commit 前执行（上游新刀落树候＝有则 pull --rebase 后 arm 顺延＝无则直推后点火）。

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
