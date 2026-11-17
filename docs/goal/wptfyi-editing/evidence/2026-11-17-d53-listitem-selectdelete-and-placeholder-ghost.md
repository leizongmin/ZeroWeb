# d53 列表项选删归一＋占位 br 换代幽灵滤除 A/B 证据（2026-11-17）

## 码面（part01.js +83/−5、part05.js +19，git numstat）

1. **`_sdLiKid` 列表容器元素锚单 li 跨度归一**（deleting-in-list-items 语料定谳）：
   sc===ec 元素锚 ul/ol 跨恰一 li 子——摘空后按后继兄弟定形：有后继（跳过折叠
   空白）摘 li 壳、无后继保壳＋占位 br；gtr 随动作形（保壳面降首/尾文本叶、
   摘壳面元素锚跨度）。② splice 臂同形收口。
2. **BS/FD 文内臂塌缩门**：重读覆盖（during-and-after-dispatch 语料）限域
   `range.collapsed`——非塌缩选区默认动作删选区而非重读位单单元（语料
   `ist-item1` 塌坍形实证）。FD 镜像同款。
3. **文内臂整节点摘除占位**：`[list-item1]` 全文形清空所在块 → 占位 br
   （`<ul><li><br></li></ul>` 期望形）；`_bsWholeShell`/`_fdWholeShell` 剪枝
   面不入。
4. **`_zwCePlaceholderBr` 换代记账门**（见根因链 2）；append 后以
   `__zwHandle`→`_zwApplyGeneration` 记账（`globalThis.__zwPbrGenByHandle`）。
5. **part05 融合视图并桶点跨代滤除**（R382 邻位同款 splice 剔除）：pending
   added 条目命中占位 br 记账且代际 ≠ 当前 → 剔除不并入。

## 根因链（探针序贯定谳，zz-probe-d53b/c/k 三轮 trace）

1. **d53b 序贯毒化**：同页先 `[list-item1]` 后 `{list-item1}`——case-2 CE
   walk 域读视图时 stale pending-added 桶把 case-1 遗留占位 br 再并入 li 读取
   视图，「尚有可见内容 → 不占位」门拒收 → 引擎树无 br。probea（单形隔离）
   两二进制全 8/8 P＝跨形序贯污染非形语义回归。
2. **判别面两轮证伪**：parentNode identity 门失效（li/br 包装对象跨 rebake
   复用，trace `eqLastPbr=true`）；`__zwPbrGen` expando 失效（set trap 丢弃，
   trace `g=undefined`）；`__zwHandle` 持久（`h=__n1` 双端可读）→ handle→gen
   记账成立。handle 记账后 probeb 16/16、probea 8/8。
3. **dil 全量六面 P2F**（filter 前轮）：跨列表 join 族 `<ul><li>list-item1[</li>
   </ul><ol><li>}list-item2` innerHTML 残 `<ol></ul>` 空壳。trace5/7 二轮
   （[SD-prune]/[SD-X] elView）定谳：B 形 select-delete 入口 elView 已带跨代
   幽灵（`<UL><LI>[li1]<LI>[li2]<BR>[]…`，lca=UL 错几何）——part05 并桶点对
   占位 br 条目无代际滤除（R382 的宿主快照确认对 rebake 后 handle 失配恒
   并入）。part05 滤除后 dil P2F 6→0。LI 幽灵（三面 A+A2+B 最小序）不毒化
   B 形（截断序列残留面见「残留」2）。
4. **HEAD 对照**（e4fb2c42e 净树 runner）：dil 对 old-base 0/0——六面 P2F
   全数本刀序列位移所致，非 rebase 落窗刀（evt-reject/img breakout）。

## 权威账（专项 A/B，base=old-base 9404d6d41 序账；HEAD 对照 0/0）

| 腿 | base | wip（final3 无痕二进制） | 共键 F2P | P2F |
|---|---|---|---|---|
| dil 四 variant | 232P/1032F (1264) | 304P/960F (1264) | 72 | 0 |
| backspace 主文件 | 81P/82F (163) | 81P/82F (163) | 0 | 0 |
| forwarddelete 主文件 | 100P/76F (176) | 100P/76F (176) | 0 | 0 |
| non-collapsed 三 variant | 106P/101F (207) | 106P/101F (207) | 0 | 0 |
| joining-dl 两文件 | 752P/424F (1176) | 752P/424F (1176) | 0 | 0 |

**dil 净 +72（F2P 72 / P2F 0）**，new=0/gone=0。F2P 面：`[list-item1]` 全文形
innerHTML ×8（BS/FD × ul/ol/嵌套）＋ `{list-item1}`/`{<li>…</li>}` splice 族
innerHTML＋gtr 键＋`list-item2[</li>` 中位形——明细见 /tmp/d53-ab.py 输出存档。

## 哨兵

- input-events 全套件（1500s 完整轮）：对 base 共键 **F2P=108/P2F=0**，
  gone=0；wip 6475 keys（new 968＝zz-probe 面＋rebase 落窗刀面自增）。
  首轮 600s 截断轮伪 P2F=11/gone=160 作废（timeout 截断伪影，见残留 1）。
  **HEAD 净树（e4fb2c42e suite-head）→ wip 对拍：common 6475 键恒等，
  F2P=199/P2F=0/gone=0/new=0**（更强回归陈述：rebase 落窗刀增益并入后
  WIP 仍零 P2F，199 面全为 d53 本刀 F2P）。
- selection corpus（53,317 键全量）：base 49,019P/4,298F → wip
  **49,025P/4,292F，共键 F2P=6/P2F=0/other-flip=0**；new=gone=1＝
  selection/selection-nested-video.html 同文件名漂移非码面。F2P 面：
  editing/event.html ×3（collapse 门面）＋ insert-or-paste-image ×1＋
  delete-chars-following-collapsible-white-space ×1＋ restoration ×1。
- editing/other：base 13101P → wip 13102P，**F2P=1/P2F=0**。
- editing/run（后台轮回填，4 分片 base/wip，2026-11-18 03:20–04:39 产物）：共键
  99,474 **F2P=0/P2F=0/other-flip=0**（base 98,289P/99,500 → wip
  98,284P/99,494）；new=20/gone=26 全落分片尾键
  （forwarddelete?6001-7000/multitest?9001-last/zz-probe-f6bwin?9001-last 等）
  ＝文件级超时吞键族带截断伪影非码面（残留 3 同谱）。
- enginelib：**4137P/0F**（test-guard --compile-first --per-proc-mem 4
  --total-mem 8 --time-limit 1500；对 d52 4134P +3＝rebase 落窗刀自增，零 F）。
- 探针终态（final3）：probea 8/8、probeb 16/16（序贯毒化复现面清零）、
  probea/c 单形 2/2。

## 残留与归域

1. **runner 全套件轮须充足墙钟**：600s 截断轮产生伪 P2F=11（zz-probe 尾部
   文件被截）；完整轮 1500s 清零。后续 suite 哨兵一律 ≥1500s。
2. **③ 域（generation-aware 归一刀）不动**：LI 幽灵跨代并入在册域（d53k
   探针 elView 实证 li2/li3 跨代幽灵可致 lca=UL 错几何）——本刀只滤占位 br
   记账条目；全量序未现 LI 幽灵毒化面（d53g 截断序 Delete 形残留 F 为截断
   序列伪影，全量序 P2F=0）。B 族 LI 面留 ③ 域归一刀（d48/d47 负样本在册）。
3. **首 dil 轮 22-key GONE 单发**：Backspace,ul variant 键吞 22，复跑全回
   ＝族带时序谱（s552 同款 file-level timeout 吞键），非码面。
4. **runner CWD 敏感**：runner 以进程 CWD 解析 wpt 根——`/tmp` 下调用全文件
   `load testharness.js ENOENT`；一律仓根调用（排障三轮实录）。

## 运行产物

- /tmp/d53-{dil,bs,fd,nc,jdl}-wip.json（final3）、/tmp/d53-dil-head.json
  （HEAD 对照）、/tmp/d53-suite-wip.json（1500s）、
  /tmp/d53-editing-other-wip.json、/tmp/d53-enginelib.log
- /tmp/d53-{base2,head,final3,trace5,trace7}-runner（对照/无痕/trace 轮二进制）
- 探针：zz-probe-d53{a,b,c,d,e,g,i,j,k}.html（wpt-data 本地不入树；k＝前缀
  复现轮、g＝68 块截断轮）
- trace 轮备份：/tmp/d53k-trace7.json（[SD-X] elView 幽灵实录）
