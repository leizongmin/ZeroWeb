# 刀㊽b 判读（s10k48b）——TT 杂项拼盘：setAttributeNS ToString＋legacy mutation 同步视图（TT 语料 +2P/−2F，真实池 23→21F）

日期：2026-11-27 · 域：trusted-types 杂项簇（S4 setAttributeNS 面＋legacy-trusted-scripts legacy mutation 面）· 树锚：刀体 patch md5 part04 `bce6e29d0691ecf02633f450bc6449c9`＋part05 `2c3b6974c00642526abe783fdb6655b5`（发射 TREE-MD5-OK＋REAPPLY-MD5-OK 断言；patch 常驻 `/tmp/s10k48b/knife48b-part0{4,5}.patch`）· 文件域＝`part04.js`（15+/6−）＋`part05.js`（99+/24−）双文件。码内标签「刀㊽（wptfyi-dom s10k48）」；账面标签刀㊽b（module 席 s10k48/刀㊼ `fa75fd1f5` 已推＋其 s10k49 自标刀㊽——双席位独立编号撞号，b 后缀消歧，COORD `.acceptance/dps/s10k48b-coord.md` 三查在案）。

## 目标簇（2F）

- **block-string-assignment-to-Element-setAttributeNS「accepts null for non-specced accessor」1F**（Face A，part04）：setAttributeNS value 旧 `String(value == null ? '' : value)` 把 null 串化为 `''`；spec `set an attribute value` 对 DOMString 无 [TreatNullAs=EmptyString]——null 恒串化 `"null"`（`String(value)` 同普通 setAttribute 既有规格形）。修：实例层 upsert／host 写／CE attrChange 三汇点统一 `_r122Val = String(value)`。
- **legacy-trusted-scripts「Legacy sinks for TrustedScript accept arbitrary strings.」1F（六断言取尾）**（Face B，part05 六件）：
  1. **R522 sel 域 parsed Text `after` viableNextSibling 入口定锚一次**——旧版循环内重读 `node.nextSibling`，首项插入后即变刚插项，后续各项各插其前 → afterScript 断言 ";321" 反序（桶锚 trace __n12→null、__n13→__n12、__n14→__n13 逐项倒链实证；spec ChildNode.after viableNextSibling 方法入口一次算定 https://dom.spec.whatwg.org/#dom-childnode-after ）。ref 恒基驻节点（原后继或 null）；不得改传未插入参数项为 ref（R300 ref 非本父子校验照抛 NotFound——链锚形不可经 insertBefore 通道）。
  2. **`_appendVariadic` sel 容器字符串项缓冲单次委派＋added 单身份**——同批连续字符串缓冲后单次 `_insertAdjacentVariadic(sel,'beforeend',buf,false)`（prepend 全对形同构）；非字符串项前与循环尾 flush 保交错位序。added 推同 w-proxy 身份（非裸对象）：工厂 notify 与漏斗对同 handle 双登记须单身份——裸对象使桶 6 项且 notify 批内锚按 addedNodes 位序重写 `_zwNodeParent[handle].nextSibling` 劫持 proxy 链锚（";321"+空心壳 7 项根因之二）。
  3. **漏斗串参数物化走元素通道**——`__zw_create_text` 建 Text 后 `_wrapHandle` 入 items（spec concept-node-convert-nodes-into-a-node）；宿主树插入与 JS 反链记账同步落，同 turn 即时可见（旧 `__zw_insert_adjacent_text` 宿主异步 apply 且无 JS 记账＝六断言 stale 根因）。
  4. **R4600 per-item 后继链扩位 beforeend/afterend**——item i 后继 = item i+1、末项 null（beforebegin 同型既有；append 多项同批 null 锚散架实证）。
  5. **R5277ab 锚取 `childNodes[0]`**（旧 `.children` 仅元素子——文本首子容器锚恒 null→首项失锚尾插序错；元素首子容器零变化）。
  6. **s10k9 topo 锚依赖解引用比对**——旧 `_k9left.indexOf(_k9lk2.nextSibling)` 以对象查 int 索引数组恒 −1（类型异质），拓扑序从不生效、ordB 恒按桶序裸跑；改按 `bucket.added[_k9left[_k9lp]] === anchor` 节点对象解引用比对（s10k9 注记本意兑现）。

## 验证

- solo（发射前）：legacy-trusted-scripts 1P/RC=0（六断言全绿，无 Timeout 无合成行）；block-string-assignment-to-Element-setAttributeNS 4P/RC=0。
- 电池六腿（`/tmp/s10k48b/k48b-*.log`，发射 pid 810355 TREE-MD5-OK sib02=32985cd4 sib03=2ded4009 p4=`85a99333` p5=`f12e9130`，墙钟 33m29s 无 ABORT）：SAN/DP/WC/DOM 四 corpus 腿＋TT base/post（scoped checkout/reapply 恰本席两文件，REAPPLY-MD5-OK）。
- 判读（`/tmp/s10k48b/judge-k48b.sh`）：**主判 TT A/B PASS**——base 24F/2243P/19T → post 22F/2245P/19T＝GONE 恰两面（setAttributeNS「accepts null」＋legacy-trusted-scripts 全测）、NEW=0、T 恒 19。corpus san 65F/0T、dp 221F/1T、wc 920F/20T vs k49 基线 F/T SET-IDENTICAL；dom 腿 k49 基线含本席调试探针伪影（见下），真 F 集 362=362 GONE=0 NEW=0 实证恒等。
- **基线污染定谳（dom 腿 DIFF 分解）**：k49 dom 腿（23:39–23:48Z）落在本席 testharness.rs 调试 dump 在树窗内——114 GONE 全为 `t48-trace` 合成行＋108 T-GONE 全为合成行破完成点计数不变量（`state_tests == results.len()`）的页级假 Timeout，非探针行 0、真 F 集恒等。本席探针（dump＋合成行）已全量回退（testharness.rs 空 diff `d41d8cd9` 核过）、探针页 zz-s10k48-probe.html 已删；san/dp/wc 腿跑在探针入树前（SAME 直判有效）。经验已沉淀 memory（runner 侧 trace dump 计数不变量）。
- 观测方法注记：chunk 敏感页页侧插桩＝海森堡（任何页字节变化移动 shim 分块边界翻转断言），本刀经 runner 侧完成点 `execute_script` dump shim 全局 trace 数组定谳（页源零改动）。

## 波及与域隔离

- Face B 六件全落 part05 共享 overlay/漏斗面：corpus 四腿真 F 集 SET-IDENTICAL＝波及面实证闭合。
- 兄弟字节（part02/03 s10k49 WIP＋import_map.rs＋webview.rs）全程未触；其 k49 电池 23:48:25Z ABORT-SIB-DRIFT-05（本席 hoist 后于其快照，守卫正确触发；NOTICE 已落 `/tmp/s10k49/SIBLING-K48B-NOTICE.txt` 含本席冻结字节 md5，重发射按新快照即不再漂移）。
- Face C（S3 CSP violation location sourceFile/line/column）与 Face D（DOMParser iframe contentDocument 占位）**本刀不入**：C 依赖页 creation-001/002＋sink-mismatch-001 页级 Timeout 先解（控制面在案；module 席 s10k49 落库后重探）；D 需重探钉面。留池后续刀位。

## 账目

TT +2P/−2F＝in-suite 24→22F/2243→2245P、T 19 恒＝真实池 23→21F（module 席 29→23F 后）。累计：**84,280→84,282（module 席 84,274→84,280 已推；距 ≥99,947 差 15,665）**。
