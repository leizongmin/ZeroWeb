# html-semantics M2 兼容性守成巡查（M2 切片 soak/patrol）

> **当前轮次：s812 收账完毕（2026-11-28 16:38 收账；all.done 16:31:48）｜点火 16:19:54→judge 16:26:26（clean-soak 慢判 6m32s＝负载等待形）＝runner 116th hold 五点恒六窗＝bridge seek_epsilon 驻留 carry 二演＝e9c9aa0e==b812 零差＝wob r129 held 连持四窗＝stable byte-IDENT hold 七演＝cmp byte-EXACT 十七窗＝band 五十连＝下一步 s813 部署发射**

## 本轮状态（s812 收账＝runner 116th hold 六窗＝bridge seek_epsilon 驻留 carry 二演＝wob r129 held 连持四窗＝stable byte-IDENT hold 七演）
- **窗形＝clean-soak 慢判 6m32s＋批墙钟 5m22s＋premature cluster 廿一窗核过**：点火 16:19:54→judge 16:26:26（**慢判形＝负载等待**＝s809-s811 三连 90/103/94s 单拍即时形破＝waiter load-wait 恢复形＝procs=0 load<3 across 3x45s checks 达成前 6.5min 轮询）＝legs.head b5e29b956 settled（arm-same-tree 0 commits no-move＝窗零 .git 移）＝leg1 16:26:26→16:28:27（2m01s）→leg2 16:28:27→16:30:30（2m03s）rc=2/2→legs.done 16:31:04→analyze 16:31:16→solos/all.done 16:31:48＝**批墙钟 5m22s（5m23s 四窗恒 ±1s 方差＝同形持）**＝done 序 legs→analyze→solos≈all（**solos/all 同秒 11ms 微倒序形＝all 4866ms<solos 4877ms＝同秒收尾记账＝premature cluster 非复发廿一窗＝四 marker 全 fresh**）＝日界未跨。
- **runner＝116th hold PRIMARY 命中＝五点恒＝六窗 hold 兑现**：armdisk 530f3f4c2006ecdcfb5222523b177ab7/131,697,032==臂时读数＝窗内 pre=mid=post=bridge.runner=armdisk **五点全等**＝刀73/74 双落空（fetch 预检 behind=0 ×2＝树零源差＝docs-only 对端承）＝**臂时 C 形 rebuild ×6（product-version→net→engine→page-runtime→webview→wpt-runner 链）产物 byte 恒等（Δsize 0）＝重编产物恒等形〔R73000/R74000 先例同形〕＝面零效应直证＝116th 世代锚不变**＝warm 39s C=6（s810/s811 两演 C=0 破＝C 形有由＝product-version 链指纹漂移）＝probe rc=0 C=5（check 侧方差形三演＝s810/s811/s812 同值）＝arm solo rc=2/2 C=0/0（wall 21/35s）。
- **bridge＝seek_epsilon 驻留 carry 二演＝主读命中**：rc=101「FAILED. 1842 passed; 1 failed」＝keys 1842 md5 **e9c9aa0e==b812 零差**＝NEW=0 GONE=0＝fail=1≡**r11 十三窗 carry**（test_element_local_name_r11）＝bridge.runner==armdisk==530f3f4c 承＝单窗 flake 定谳后续演＝驻留三窗形（s810 归返→s811 驻留→s812 驻留二演）。
- **wob＝r129 dual-P held＝连持四窗 PRIMARY 命中**：l1 Pass＋l2 Pass（canon 尾态直读）＝r127 held→r128 held→r129 held＝**hold streak 四窗**＝r94-r109 hold 先例群主读兑现＝摆链 s808 in/s809 in/s810 in/s811 in/s812 in 恒持＝swap re-open/mirror-swap/dual-F/silent 四分支全落空＝FLIPS 29 watch 落空（FLIPS 28 恒）。
- **stable＝byte-IDENT hold 七演 PRIMARY 命中**：stable=8517 disagree=0 l1only=0 l2only=0＝keys 057764d3==s811＋canon 793bd6cd 恒＝census 234 Fail/8277 Pass 形恒（canon 6251237e 跨窗 byte 恒传递）＝**cmp legs-IDENT byte-EXACT 6251237e 十七窗**（l1==l2 byte-EXACT）＝刀74 面效应落空。
- **哨位＝十二项全持＋band 五十连**：dialog r246 2P×2 retirement-held 持＋select r251 279 rows 8F/8F Fset 双腿 IDENT se.577＋canon 6251237e byte-IDENT 封印承＋password r251 225 keys 0F 双腿＋mpreload r274 0F＝15P×2 hold-continued＋waapi r333 checkmark F/picker P×2＋fd r292 121 [121 Pass] ×2＋dual-emit dup(2)×2＋**ob r244 锚 Fset 三方恒 a9e79b58（f1k==f2k==ob.s559 双腿 IDENT＝2F/2F）**＋**band r245 8/8×2 Fail＝五十连**＋rc1=2 rc2=2 rcb=101＋stack overflow 三零。
- **solos＝C=0/0 十七演＋rc=2/2**：窗内 opt-solo1 rc=2（16:31:04→16:31:32＝28s）＋opt-solo2 rc=2（→16:31:48＝16s）＝C=0/0 零级联＝warm-build 处方承＝armstart 两行 16:19:54/b5e29b956f8fe3fb97ee2f96eab6392b830b2a9d＝pids 2815271/2815272/2815273 ps 字段锚 ANCHOR-OK。
- **主预判落定率＝全 PRIMARY 命中＋驻留二演主读精确命中＝零盲点**：runner 116th hold（命中＝刀74 落空）＋wob r129 held 连持四窗（命中）＋stable byte-IDENT hold 七演（命中）＋cmp byte-EXACT 十七窗（命中）＋**bridge seek_epsilon 驻留 carry 二演＝keys 1842 零差主读命中**＋band 五十连＋dialog/select/password/mpreload/waapi/fd/ob/dual-emit 全持＋FLIPS 28＋solos 十七演＋fire-last 十八演（C 形 rebuild 恒等形新账）＋premature 廿一窗＋日界未跨＝盲点落空 0。
- **杂记＝s791 链惰性残留 7 件**：leg1/leg2.s791 系 7 文件 21 代链传（family glob 's791.*'/'s791-*' 不匹配 'leg1.s791.end' 前缀形＝STALE 清单缺口）＝窗效应零（analyze2 全读 s812 名新产物）＝s813 deploy STALE 一次性清理项；norms 'Fail' 子串 236 ≠ census 234（键名含 Fail 子串伪差＝canon 封印为准承）。
## 基线决策（s813）
- **s812 臂记录（16:19:54）**：三链点火 waiter 2815271/rearm 2815272/post 2815273（分号分隔形＋ps 字段法核承）；armstart 两行 2026-11-28 16:19:54/b5e29b956f8fe3fb97ee2f96eab6392b830b2a9d；deploy rm=96 files=37 sh=11 keep=18（b812=cp s811.bridge.keys＝md5 双验 e9c9aa0e/1842 行＝**驻留基线兑现**＝b811 kept control＋b810 drop＋s808 系 drop 滚动窗承）；fetch 预检 behind=0→warm build 39s **C=6（C 形 rebuild＝product-version 链×6）→产物 byte 恒等 530f3f4c/131,697,032（Δsize 0）＝重编产物恒等形**→probe rc=0 C=5（check 侧方差形三演）→arm solo C=0/0 rc=2/2（wall 21/35s）→armdisk 530f3f4c/131,697,032＝**116th hold 臂时确立＝四点恒承（六窗 hold 候）**。
- **s812 收账（2026-11-28 16:38）**：runner 116th hold 六窗＋bridge seek_epsilon 驻留 carry 二演＝keys 1842 e9c9aa0e==b812 零差（fail=1≡r11 十三窗）＋wob r129 dual-P held 连持四窗（PRIMARY＝r94-r109 hold 先例群主读兑现）＋stable 8517/057764d3＋793bd6cd byte-IDENT hold 七演＝cmp byte-EXACT 十七窗＝band 五十连＝全 PRIMARY＋驻留主读精确命中＝零盲点＝明细见本轮状态。
- **基线轮转（s813 部署时执行）**：**b813＝cp s812.bridge.keys＝e9c9aa0e/1842 值锚（驻留基线三演）**〔s813 窗 NEW=0 GONE=0 carry 三演主读＝seek_epsilon 驻留 carry 三演 watch＝再翻 fail 侧＝flake 复发 watch＝fail=2 形＝二次伤亡 watch；cp 先行后 rm 承＝md5 双验 e9c9aa0e2aad1d71bd1b7cdf4e2a1e9c/1842 行〕＋b812 kept one-round control〔同值对照＝e9c9aa0e/1842；s814 drop 预注〕＋b811 drop〔kept-control 滚动窗承＝s813 drop 预注兑现〕＋s809 系 drop（s634/s810/s811/s812 stable keys＋flips s810/s811/s812 保留＝滚动窗承）＋**s791 链残留 7 件一次性清理**（leg1/leg2.s791 系＝STALE 缺口补）。
- **runner 世代锚＝530f3f4c2006ecdcfb5222523b177ab7/131,697,032（116th 候 hold 七窗承）**＝s813 窗三点恒此值＝hold；armdisk≠530f3f4c＝刀75 落树分支现役（**116th 已收六窗 hold＝七窗 hold 候＝读数后回填收账轮派生 117th 候**＝读数后回填纪律承）；**C 形 rebuild 恒等形承：warm C>0 不判面变＝产物 md5/size 双验为准〔s812 轮 C=6 恒等形直证〕**。
- **主预判（s813）＝全 hold 窗**：**runner hold watch PRIMARY-candidate**（armdisk==530f3f4c＝116th 候五点恒七窗持＝主读／≠530f3f4c＝刀75 embed 分支＝新面候读数后回填）＋**wob r130 dual-P held 连持五窗 watch PRIMARY**（r129 held 连持四窗→连持五窗形＝r94-r109 hold 先例群／swap re-open／mirror-swap＝摆-回周期再开 watch／dual-F＝FLIPS 29 watch／silent＝held watch）＋**stable byte-IDENT hold watch PRIMARY**（8517/057764d3＋793bd6cd 恒＝hold 八演候／wob 摆→8516 形 watch／census Δ＝刀75 面效应 watch）＋cmp legs-IDENT（byte-EXACT 6251237e 十八窗候 PRIMARY）＋ob r245 anchor Fset IDENT（2F/2F hold 续）＋band r246 五十一连＋dialog r247 retirement-held＋select r252 8F IDENT＋password r252 0F＋mpreload r275 15P＋waapi r334＋fd r293 121P＋dual-emit dup(2)×2＋**bridge seek_epsilon 驻留 carry 三演 watch**（keys 1842 e9c9aa0e==b813 零差＝驻留 carry 三演主读／keys 1841 GONE=1＝seek_epsilon 二度翻 fail＝flake 复发 watch＝fail=2 形／fail=3＝二次新伤亡 watch）＋**solos C=0/0 十八演**＋**fire-last 十九演核**＋premature 廿二窗核＋日界 watch。

## 下一步（s813 部署清单）
1. 目录复制与 drop 清单：/tmp/s813＝cp -a /tmp/s812＋stale markers 清除执行项（rm legs.done/analyze.done/solos.done/all.done＋legs.head/judge/armstart/pids/三链日志＋runner 四点+armdisk+bridge.runner＋solo markers+logs+walls+快照＋probe 产物＋warm.build×4＋norms+canons+stable.canon＋new/gone/disagree/leg1only/leg2only/both/k1/k2/s1/s2＋s812.bridge.*＋se.cur*/ob.* 窗值＝premature cluster 防复发承）＝**b813.keys=cp s812.bridge.keys**（cp 先行后 rm＝md5 双验 e9c9aa0e2aad1d71bd1b7cdf4e2a1e9c/1842 行＝**驻留基线三演**）＝drop b811.keys＋s809.*＋s809-*＋**s791 链残留 glob 一次性清**（KEEP 18 锚：s634 keys/canon＋s810/s811/s812 stable keys＋flips s810/s811/s812＋b812.keys 对照＋b813.keys＋wob1/band8/fd121＋五哨锚）。
2. 轮转 rotate-s813.py：s812→s813 全量行枚举＋排除行 verbatim 双锚（analyze2.sh:58 wob 头 wob1 r129＋bridge-extract.sh:1 头注 s812 形）＝九坑 b812→b813 三处显式替换（comm 基线指针×2/label echo/wc target）＋body stray 零残留核＋rearm 改名＋wc -l 全等审计（245 总行）＋bash -n ×11＋chmod。
3. head-s813.py：bridge-extract.sh 头注整行重写（s812 窗账 condensed＋runner 116th hold 六窗＝bridge seek_epsilon 驻留 carry 二演 e9c9aa0e==b812 零差＝r11 十三窗＋wob r129 held PRIMARY 连持四窗＝stable byte-IDENT hold 七演＝C 形 rebuild 产物恒等形＋s813 预注册 incl. runner 116th hold PRIMARY-candidate 七窗＝wob r130 held 连持五窗 PRIMARY＝stable byte-IDENT hold＝bridge seek_epsilon 驻留 carry 三演 watch＝b813 驻留基线注记；gate：**116th≥1＝117th==0**＝b813≥4＝530f3f4c＋e9c9aa0e＋b5e29b956 命名＝fire-last 命名；幂等重写形承〔old 断言容 s812/s813 双形〕＝序数门禁 catch 纪律承）。
4. phase5_s813.py（配方第六十九轮＝十坑承：part A 前窗实现子句锚按 s811 写＝rotation s812→s813 只触 s812 token；rep_a 续跑形承）：标签 r247/r252/r252/r275/r334/r293/r245(ob)＋band r246（五十连 realized 承）＋ob 子句 r244 锚 Fset IDENT held realized 承（wob r129 held count l1 2F/l2 2F＝锚 held 承）＋wob r130 前缀（r129 dual-P held realized 承＝r94-r109 hold 先例群连持四窗承）＝COMMON 首尾锚对提取（首锚「(interval at-window feff02536..feff02536 settled-tree unchanged window = judge 16:01:23 immediate form = single-shot-94s form = peer zero occupation = 」**count 2 find-first**〔COMMON 与 ENTRY r128 interval 同形＝共享短前缀锚承〕＋TAIL_POS「arm 12:46:22/0ef657441 to window 12:47:54/0ef657441 0 commits no-move」前位约束〔ENTRY r121 尾〕＋尾锚「batch wall 5m23s zero ABORT)」恰 1 次〔COMMON 专属＝ENTRY 尾逗号形〕＋sanity 承：'via pull' 不在＋长度窗＋特征词〔s811 COMMON＝`116th hold five-point constant`＋`fifth-window hold` 特征词校验〕）＝NEW_COMMON（s812 实况＝judge 16:26:26 慢判形〔6m32s 负载等待＝即时形三连破〕＋runner 116th hold 五点恒 530f3f4c/131,697,032 六窗＝bridge seek_epsilon 驻留 carry 二演 e9c9aa0e==b812 零差 r11 十三窗＝wob r129 held 连持四窗＝批墙钟 5m22s ±1s 方差）＝paren 平衡即断＝ENTRY r129 insert（**TAIL_STAMP＝ENTRY r128 尾戳「arm 15:59:49/feff02536 to window 16:01:23/feff02536 0 commits no-move」＋')'*32**＝代锚坑承＝r129＝116th hold 六窗＋seek_epsilon 驻留二演＋wob held 连持四窗＋stable byte-IDENT hold 七演 group＝首行括注形 (o_ins,c_ins−32)=(5,3)＝**ENTRY 首行括注只开 5 对括号〔s810 轮 (6,4) 断言拦截例承＝cmp 子句内联形〕**＝新尾戳「arm 16:19:54/b5e29b956 to window 16:26:26/b5e29b956 0 commits no-move」＋')'*32）＝chains r2..r129/s685..s812/counts 128 值尾 8517＝states 位置法核证。
5. fire-last 处方十九演：master 收账 commit/push 完成→fetch 预检（刀74/75 候＝有则 pull --rebase 后重跑 warm build＋arm 顺延＝armstart 记新 HEAD；**.rs 触碰核对面效应承**）→warm build（**C 形 rebuild 有由承：C>0 列箱直读＋产物 md5/size 双验＝恒等形记账／面变＝刀分支**〔s812 轮 C=6 恒等形例〕）→probe（C=0 主读＝C>0＝check 侧方差形记账承〔s810/s811/s812 C=5 三演〕）→arm solo ×2（LC_ALL=C FILTER=select-home-end-pagedown-pageup.optional make testharness-html-semantics＝**repo 根起**＝rc/wall/C 快照）→armdisk 读（md5sum＋stat 真二进制直读）＝116th hold 形预期：530f3f4c/131,697,032 恒＝hold（六窗承）；armdisk≠530f3f4c＝刀75 分支＝新面候读数后回填。
6. 三链点火：nohup 绝对路径 waiter/rearm/post＋**点火命令逐行/分号分隔〔s810/s811/s812 轮分号形承＝wrapper 假象坑防复发〕＋ps cmdline 锚形核（pgrep 复合命令引号假象坑承＝nohup &&链截断坑承）**＋pids 三件＋armstart 两行纪律＝窗零 .git 移＝leg C=0 双腿候。
7. 本提交不含 s813 臂记录（收账-点火解耦承）：点火事实（pids/armstart/时间）盘面为准＝s813 收账轮开读 pids.s813.*＋armstart.s813 核；fetch 预检在 commit 前执行（上游新刀落树候＝有则 pull --rebase 后 arm 顺延＝无则直推后点火）。

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
