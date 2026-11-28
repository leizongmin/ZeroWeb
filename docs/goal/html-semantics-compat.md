# html-semantics M2 兼容性守成巡查（M2 切片 soak/patrol）

> **当前轮次：s798 收账完毕（2026-11-28 11:04 收账；all.done 11:01:35）｜窗开 10:56:15/legs.head c015cbe90 settled 树＝fire-last 处方四演兑现（legs C=0/C=0 四演＝批墙钟 6m50s 零 ABORT）＝**110th hold 兑现**（pre=mid=post=armdisk 四点恒 d3576588/131,668,208＝hold 分支胜出＝无新刀窗）＝bridge 十一演 ON 110th（b798=32e005b9/1843 NEW=0 GONE=0 fail=0）＝stable 8517 四窗双层 EXACT hold（keys 057764d3 byte-IDENT s797＋canon 793bd6cd＝s794/s796/s797/s798 四窗同面）＝wob r115 silent hold 二演＝哨位十二项全持＝solos C=0/0 三演＝canon 6251237e 六窗恒＝**连续二窗零 miss**｜下一步＝s799 部署发射（fire-last 五演＋bridge 十二演＋stable hold 五窗候）**

## 本轮状态（s798 收账＝110th hold＝bridge 十一演＝stable 四窗双层 EXACT hold＝连续二窗零 miss＝face 全恒窗）
- **窗形＝快窗 6m50s 全程＋fire-last 处方四演兑现＋premature cluster 七窗核过**：点火 10:54:45→judge 10:56:15（1m30s＝3×45s 三查即过＝procs=0 load<3＝HEAD=c015cbe90 settled 树）＝对端零占窗；legs.head==arm HEAD==c015cbe90＝窗零 .git 移承；leg1 10:56:16→10:58:09 1m53s **C=0**＋leg2 10:58:09→11:00:13 2m04s **C=0**＝双腿零级联四演；bridge 23.59s rc=0→legs.done 11:00:47→analyze 11:01:06→solos 11:01:35→all.done 11:01:35＝done-marker 顺序净（七窗核过）＝批墙钟 6m50s 零 ABORT。
- **runner＝110th hold 兑现＝四点恒**：pre(10:56:15)=mid(10:58:09)=post(11:00:13)=armdisk(10:54) 全等 d3576588769bff196b2a83dea5b8cd52/131,668,208＝**hold 分支胜出**（主预判 PRIMARY 兑现＝刀65 落树分支落空）＝110th realize→hold 两窗承接形（s796 109th hold→s797 110th realize→s798 110th hold＝面恒三窗链）。
- **bridge＝SET-stability 十一演 definite ON 110th**：rc=0「1843 passed; 0 failed 23.59s」＝keys 1843==b798==s798.bridge 双 md5 32e005b9751df4048bbdac6c19d2b17f＝fail=0 NEW=0 GONE=0＝bridge.runner.md5=d3576588＝b797 kept control 满＝b799 十二演候 staged＝b798 转 kept control＋b797 drop 预注承。
- **stable＝四窗双层 EXACT hold＝keys byte-IDENT 前窗**：keys 8517 md5 057764d38ee311a47c34ecfc3e1e3192（==s797 byte-IDENT）＋canon md5 793bd6cd20679beb564d75185335ff9b＝**s794/s796/s797/s798 四窗逐字节同面**；census 234 Fail/8277 Pass/5 Timeout/1 Unsupported＝NEW=0 GONE=0＝identity hold carry＝刀65 未落＝零扰窗。
- **LEGS＋canon＋FLIPS＝全恒**：disagree=0 l1only=l2only=0（0/0 carry）；canon l582-1==l582-2==6251237e34e8ed34240f9706bffbfb6f 双腿 byte-EXACT＝**六窗恒 realized**（s792/s793/s794/s796/s797/s798）；FLIPS 28 md5 c92accb1249d615a53bd6fc2be4844cd＝byte-IDENT carry＝script-7 heal watch 十四连空承＝dialog domain 20 held＝刀56-64 九刀零 corpus 面承。
- **哨位＝十二项全持＋wob r115 silent hold 二演**：wob1 r115 Checkboxes Pass/Pass hits=1 双腿＝silent hold carry（episode 五演持：r110 swap/r111 dual-P/r113 heal/r114 silent/r115 silent）＋ob r230 2F/8P×2 Fset IDENT（anchor hold 二窗）＋band r231 8F×2 IDENT＝**三十六连**＋dialog r232 2P×2 retirement-held＋select r237 279 keys 8F IDENT se.577＋password r237 225 keys 0F＋mpreload r260 15P×2 hold-continued＋waapi r319 1F/1P＋fd r278 121P×2＋dual-emit dup(2)×2＋rc1=2 rc2=2 rcb=0＋stack overflow 三零。
- **solos＋legs＝C=0/0 三演＋C 形归档 hold 承**：窗 solo1 C=0/solo2 C=0（warm-build 处方三窗连续）＋arm solo C=0/0 rc=2/2（10:54 warm build 39.9s 后）＝处方 hold；legs C=0 双腿＝embed 级联四窗全消＝fire-last+warm-build 处方面恒窗四演。
- **主预判落定率＝连续二窗零 miss**：stable hold＋wob silent＋ob hold＋band 三十六连＋dialog/select/password/mpreload/waapi/fd/dual-emit 全持＋LEGS 0/0＋canon 六窗恒＋FLIPS 28 carry＋bridge 十一演＋runner 110th hold＋solos C=0/0 三演＋fire-last 四演＋premature 七窗＝**十五项全中**＝日界未跨（窗全程 11-28 上午）。
- **证据纪律＝fire-last 处方四演兑现**：值+mtime 双读承＋LC_ALL=C comm 承＋静态段零触碰＝s799 处方五演：收账 commit/push→warm build→probe→arm solo ×2（repo 根起）→armdisk 读→三链点火。

## 基线决策（s799）
- **s798 臂记录（10:54:45）**：三链点火 waiter 2280414/rearm 2280406/post 2280407（锚定 `^/bin/bash /tmp/s798/<名>$` 全等形 cmdline 白名单核验＝launcher-pid delta 二坑锚定形承）；armstart 两行 2026-11-28 10:54:45/c015cbe90；deploy rm=92 files=38 sh=11 keep=18（b798=cp s797.bridge.keys＝cp 先行后 rm 承＝md5 双验 32e005b9/1843 行＝十一演候承＋b797 kept control＋b796 drop＋s794 系 drop 滚动窗承）；**显式 warm build 39.9s**；probe rc=0 C=1（check profile embed 级联非预测性承）；arm solo C=0/0 rc=2/2 全 warm（wall 14/15s＝repo 根起承＝s797 cwd 坑不复发）；armdisk **d3576588769bff196b2a83dea5b8cd52/131,668,208 恒＝110th 面承＝110th hold 分支现役兑现**；phase5 配方第五十四轮（**首次单跑过＝十坑锚规则落位＝part A 前窗实现子句锚按 s796 写**）。
- **s798 臂记录保留承**（上记 10:54:45 条＝s797 臂记录 10:43:00 条滚出〔滚动单承记〕＝收账-点火解耦形承）
- **s798 收账（2026-11-28 11:04）**：110th hold＋bridge 十一演＋stable 四窗双层 EXACT hold＋wob silent hold 二演＋连续二窗零 miss＝明细见本轮状态＝主预判十五项全中。
- **基线轮转（s799 部署时执行）**：**b799＝cp s798.bridge.keys＝32e005b9/1843 值恒锚**（SET-stability 十二演候承＝cp 先行后 rm 承＝md5 双验 32e005b9751df4048bbdac6c19d2b17f/1843 行）＋b798 kept one-round control〔十一演 definite 后零差对照；s800 drop 预注〕＋b797 drop〔对照终轮满＝滚动三轮窗承〕＋s795 系 drop（s634/s796/s797/s798 stable keys＋flips s796/s797/s798 保留＝滚动窗承）。
- **runner 世代锚＝d3576588769bff196b2a83dea5b8cd52/131,668,208（110th hold 基准＝generation-constant 基准）**＝s799 窗三点恒此值＝hold；窗中変面＝knife-in-window watch（对端新刀落树候＝mid-window 换代形 vigilance＝103th/s792/s793/s797 四重先例）；armdisk≠d3576588＝刀65 落树分支现役（111th 臂时面候＝读数后回填收账轮）。
- **主预判（s799）＝全指标 hold 窗**：stable 8517/057764d3＋793bd6cd 五窗双层 hold（identity 恒 carry＝census 漂移＝单键 watch／真面 Fail＝knife-in-window casualty watch）＋wob r116 silent hold（r115 silent carry＝silent=held watch／single-leg F＝swap watch／dual-F＝agreed-Fail reopen watch）＋ob r231 anchor 2F IDENT hold＋band r232 三十七连＋dialog r233 retirement-held＋select r238 8F IDENT se.577＋password r238 0F＋mpreload r261 15P hold＋waapi r320＋fd r279 121P＋dual-emit dup(2)×2＋LEGS 0/0 carry＋canon 6251237e 双腿恒（七窗恒）＋FLIPS 28 c92accb1 hold（script-7 heal 十五连空候）＋**bridge 十二演**（NEW=0 GONE=0 fail=0 on b799=32e005b9/1843）＋**runner 110th hold**（四点恒 d3576588；armdisk≠＝刀65 分支）＋**solos C=0/0 四演**＋**fire-last 五演核**（窗零 .git 移＝legs C=0 双腿候）＋premature 八窗核＋日界 watch 承（窗跨 11-28/11-29 嵌日＝环境面形承＝s786 先例）。

## 下一步（s799 部署清单）
1. 目录复制与 drop 清单：/tmp/s799＝cp -a /tmp/s798＋stale markers 清除执行项（rm legs.done/analyze.done/solos.done/all.done＋legs.head/judge/armstart/pids/三链日志＋runner 四点+armdisk+bridge.runner＋solo markers+logs+walls+快照＋probe 产物＋norms+canons+stable.canon＋new/gone/disagree/leg1only/leg2only/both/k1/k2/s1/s2＋s798.bridge.*＋se.cur*/ob.* 窗值＝premature cluster 防复发承）＝**b799.keys=cp s798.bridge.keys**（cp 先行后 rm＝md5 双验 32e005b9751df4048bbdac6c19d2b17f/1843 行）＝drop b797.keys＋s795.* （KEEP 18 锚：s634 keys/canon＋s796/s797/s798 stable keys＋flips s796/s797/s798＋b798.keys 对照＋b799.keys＋wob1/band8/fd121＋五哨锚）。
2. 轮转 rotate-s799.py：s798→s799 全量行枚举＋排除行 verbatim 双锚（analyze2.sh:58 wob 头 wob1 r115＋bridge-extract.sh:1 头注）＝九坑 b798→b799 三处显式替换（comm 基线指针×2/label echo/wc target）＋body stray 零残留核＋rearm 改名＋wc -l 全等审计＋bash -n ×11＋chmod。
3. head-s799.py：bridge-extract.sh 头注整行重写（s798 窗账 condensed＋110th hold＋bridge 十一演＋stable 四窗双层 hold＋连续二窗零 miss＋s799 预注册；gate：110th≥1/111th==0（hold 形）＝b799≥4＝d3576588＋32e005b9 命名）。
4. phase5_s799.py（配方第五十五轮＝十坑承：part A 前窗实现子句锚按 s797 写＝rotation s798→s799 只触 s798 token）：标签 r233/r238/r238/r261/r320/r279/r231(ob)＋band r232（三十六连 realized 承）＋wob r116 前缀（r115 dual-P silent hold realized 承）＝COMMON 首尾锚对提取（首锚「(interval at-window de236d76a..de236d76a settled-tree zero-move window = judge 10:44:30 immediate form = peer zero occupation = 」count 2 find-first＋TAIL_STAMP「arm 10:43:00/de236d76a to window 10:44:30/de236d76a 0 commits no-move」前位约束＋尾锚「batch wall 6m50s zero ABORT)」）＝NEW_COMMON paren 平衡即断＝ENTRY r115 insert（r114 尾锚「0 commits no-move」＋r115＝110th hold＋十一演＋四窗 hold＋silent 二演＋C=0/0 三演 group＝(o_ins,c_ins−32)=(4,2)＝sentinels 无括号形＋五刀 group 不闭合形）＝chains r2..r115/s685..s798/counts 114 值尾 8517＝states 位置法核证＝二跑幂等核证。
5. fire-last 处方五演：master 收账 commit/push 完成→warm build（make zero-wpt-runner-release＝C=0/0 四演承）→probe（cargo check -p zero-wpt-runner＝产物落 /tmp/s799）→arm solo ×2（LC_ALL=C FILTER=select-home-end-pagedown-pageup.optional make testharness-html-semantics＝**repo 根起**＝rc/wall/.arm 快照 cp ×4）→armdisk 读（md5sum＋stat 真二进制直读）＝hold 形预期：d3576588/131,668,208 恒＝110th 面承；armdisk≠d3576588＝刀65 落树分支现役（111th 臂时面候＝读数后回填收账轮）。
6. 三链点火：nohup 绝对路径 waiter/rearm/post＋锚定 pgrep 全等形 pid 核对 /tmp/s799 归属＋armstart 两行纪律＝窗零 .git 移＝leg C=0 双腿候。
7. 本提交不含 s799 臂记录（收账-点火解耦承）：点火事实（pids/armstart/时间）盘面为准＝s799 收账轮开读 pids.s799.*＋armstart.s799 核；fetch 预检在 commit 前执行（上游新刀落树候＝有则 pull --rebase 后 arm 顺延＝无则直推后点火）。

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
