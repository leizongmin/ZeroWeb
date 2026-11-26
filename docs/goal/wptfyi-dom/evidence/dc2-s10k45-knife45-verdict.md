# 刀㊺ 判读（s10k45）——TT 散布三簇＝'default' 政策重名无条件 throw＋三数据接口 serializable toJSON＋容器 handle 纯文本 innerHTML 镜像补文本节点（TT 语料 +6P/−6F，真实池 38→33F）

日期：2026-11-27 · 域：trusted-types 散布收敛 · 树锚：刀体 patch md5 `fb42e13e29b552aae566a36c06e5c8d0`（发射 TREE-MD5-OK＋TT 重放 REAPPLY-MD5-OK 双断言）；落库码刀 `19d31b651` patch md5 恒等 `fb42e13e`（`git diff 19d31b651^ 19d31b651` 复核）——本刀两文件域=落库域零差。电池树态声明：TT 腿树＝HEAD＋本刀两文件＋**兄弟席在飞 part02.js 字节**（同 session 并行轮 TT javascript: 预导航检查，同挂「刀㊺」标签），判读按 A/B 结构精确归因（下详），兄弟席效应不计本刀账。

## 目标簇（TT 池散布余 5F，四页）

k44b 后池 38F＝module 切片 16F＋散布 22F；本轮收敛散布中已定谳三簇 5F＋关联探针外溢：

| 页 | 面 | 根因链 |
|---|---|---|
| block-string-assignment-to-ShadowRoot-innerHTML | 2F（两转换脸） | 策略值＝INPUTS/RESULTS.HTML 均纯文本 → part04 set-trap 内标签臂：16798 纯文本分支清空容器镜像 `_handleChildren[handle]=[]`＋16832 文本视图仅入注册表；容器 handle 读路径（innerHTML 2468/textContent 3889/childNodes 2000 三臂）全走 `_handleChildNodes` 镜像单源（part05:8741，不融合注册表）→ 赋值成功读回空。R100（元素 handle 双计域）_registry-only 形态对容器失效——容器无融合故无双计面 |
| trusted-types-duplicate-names（CSP `trusted-types 'allow-duplicates' duplicatename default`） | 1F | 'default' 重名 throw 原在 hasList 分支内且被 'allow-duplicates' 豁免——spec default 唯一性是 factory 内在规则（§create-a-TrustedTypePolicy），allow-duplicates 只豁免普通重名 |
| trusted-types-duplicate-names-without-enforcement（无 CSP） | 1F | 同根因：无名单页 default 重名原容忍，spec 须 throw（无 violation 派发） |
| trusted-types-tojson | 1F | TrustedHTML/Script/ScriptURL 三接口缺 toJSON → JSON.stringify({x: 实例}) 落 Object 序列化 '{}'；spec 三接口 serializable |

## 修复（两文件三面，44 行）

- **face ① 'default' 重名无条件 throw（part12）**：检查移到名单门之后（spec 名单 block 步先于 default 唯一性步——名单外名字 violation 面优先）、`_zwTtDefaultPolicy !== null` 直查，无 violation 派发；throw 在 policy 组装前（失败创建不污染 default 槽）。回归面 census：语料 248 页扫描同页双建 default 仅 duplicate-names 两页（正测 throw）＋worker 族 3 页（inline main-doc＋worker 内 importScripts support——worker realm 与主文档分域，电池 NEW=0 实证无碰撞）。
- **face ② serializable toJSON（part12）**：三数据接口 prototype 挂非枚举 toJSON 返 `__zwTtData`（同 toString 基座）；TrustedParserOptions/TrustedTypePolicy/TrustedTypePolicyFactory 不挂。
- **face ③ 容器纯文本镜像补文本节点（part04）**：纯文本分支 `_zwRegisterTextEl` 注册后，容器 handle 把注册表同一 text 节点写入 `_handleChildren[handle]=[node]`——innerHTML 序列化/textContent/childNodes 三读全通，元素 handle 行为不变（R100 域）。外溢：zz-knife3b2-probe9 v9a 面（setHTMLUnsafe 纯文本入 shadow 读回，同容器读路径）F→P。

## 探针（zz-s10k45-probe1，判读前已移出语料）

诊断页（arm1 策略赋值面＋arm2 default 面，dump 经 assert_false 载体）。修后事实链：`read=Quack, I want to be a duck!`＋`cn=1`（修前 read=''＋cn=0）、nodeType=11 容器域、proto-innerHTML=acc／ownAfter=none（trap 路径非 own 属性遮蔽）；arm2 无 CSP 页直穿＝spec 一致（无 TT 处理时无 default policy 调用）。

## solo 页级（post 树，电池前先行）

四目标页全绿 EXIT=0：block-string-assignment 5P/0F（修前 3P/2F）、tojson 1P、duplicate-names 1P、duplicate-names-without-enforcement 2P。

## 电池六腿＋TT 同树 A/B（solo 串行，2026-11-27 19:59:32→20:32:44 UTC，墙钟 33m12s，无 ABORT；k42→k44b 协议，scoped 断言/checkout＝恰本刀两文件）

| 腿 | 计数 | 判 |
|---|---|---|
| sanitizer-api | 713P/65F/0T | F/T SET-IDENTICAL vs k44b |
| domparsing | 1314P/221F/1T | SET-IDENTICAL |
| web-components | 17500P/920F/20T | F/T SET-IDENTICAL；P −129＝gethtml.html 单页（3368→3239，已知巨页截断抖动族，F 集零差排除） |
| dom | 57496P/362F/12T | F/T SET-IDENTICAL；P +21＝Range-mutations 双页（−2/+23，已知截断抖动族） |
| TT base（scoped checkout＝HEAD 含刀㊹b） | 2227P/40F/19T | vs k44b-tt-post +5P/−5F——GONE 恰＝**navigate-to-javascript-url 001/002/003/004/008 五面＝兄弟席 part02 在飞字节效应被 base 腿单独隔离实证**（本刀缺席它仍翻→不属本刀） |
| TT post（重放 md5 断言过） | **2233P/34F/19T** | **post − base ＝本刀效应 +6P/−6F**：GONE=5 目标面＋1 探针外溢（zz-knife3b2-probe9 v9a）；F NEW=0；T IDENTICAL（19 恒） |

判读行提取 LC_ALL=C awk＋comm 前 wc -l 对账协议（k44 教训守恒）。四腿 rc=2 均为既有基线失败集（rc 只留痕）。兄弟席 part02 字节对 san/dp/wc/dom 四 corpus 零漂移（四腿 SET-IDENTICAL 实证）＝该在飞刀为 TT-only 域。

## 归属与计账边界

- **本刀计账：TT 语料 +6P/−6F（5 目标＋1 探针外溢），真实池 38→33F**（套件 F 45→40−外溢探针面不在池：套件 F 40−probe F 6=34−6=28F 为电池树态；扣除兄弟席 5 面＝本刀域 33F）。**累计 84,264→84,270（距 ≥99,947 差 15,677）**。
- 兄弟席 navigate-to-javascript-url 五面（+5P/−5F）＝part02 在飞字节域，本席不 commit 不计账（其落库轮自结；本席 COORD 已落账：.acceptance/dps/s10k45-coord.md——含击杀回执：兄弟席误用本席脚本路径发射的污染批已按在飞批纪律叶先 -9 清链，part02 字节全程未触碰）。

## 结论

刀㊺ 净增 **+6**（TT 散布三簇收敛＋探针外溢），TT 真实池 **38→33F**，Timeout 19 恒，**累计 84,264→84,270（距 ≥99,947 差 15,677）**。池余：module 子系统切片 16F（立项评估待做）＋散布 17F＋兄弟席 navigate 5F（其域）。battery 树态兄弟席并发刀在飞为首次 scoped-A/B 归因先例：base 腿隔离效应证明可行，后续共享树双刀沿用。
