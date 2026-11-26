# 刀㊹b 判读（s10k44b）——iframe 写入执行链与每文档 CSP 政策容器＝write-into-finished-doc R206 通道＋网络帧自有 delivery/本地 scheme 继承＋about:blank text/html 先截臂（TT 语料 +4P/−1T，真实池 38F 恒）

日期：2026-11-27 · 域：trusted-types（inheriting-csp 帧写入族）＋dom 写入执行链 · 树锚：刀体 patch md5 `3dde4fee9d00771523c6534315bc11b0`（发射 TREE-MD5-OK＋TT 重放 REAPPLY-MD5-OK 双断言）；落库码刀 `45bf86a6f` patch md5 恒等 `3dde4fee`（`git diff 45bf86a6f^ 45bf86a6f` 复核）——电池域=落库域零差。

## 目标簇（inheriting-csp-for-local-schemes.html 余 4 臂：1T＋3NR）

k44 后该页 10 臂形态 6P/1T/3NR——srcdoc/data/blob 六臂 k43 权威已绿；余四臂同根因：

| 臂 | 刀前形态 | 根因链 |
|---|---|---|
| about:blank require／trusted-types ×2 | 1T＋1NR | `src='about:blank'` 落 fetch 分支：host wire 无 text/html 头 → `_zwIframeKindFromUrl` 无扩展名 → **'xml' 空 XML 文档** → `document.write` InvalidStateError（探针 ab 臂 `err=This document is an XML document` 直证）——write 面根本没到 |
| blank.html require／trusted-types ×2 | 2NR | 帧文档经 finish 物化（HTML）但 body 空 → R206 `_zwScriptsRan` 门未置位、写入场无执行链——`document.write` 只解析不执行 |

耦合约束（face P）：blank.html 帧 write-exec 落地后，若全局 TT enforcement 继承进网络交付帧，require 两臂会收集 2F——**网络交付帧政策容器自有**（spec CSP §policy-containers），本地 scheme（about:blank/srcdoc/data:/blob:）继承。无 face P 的 write-exec＝净 2F；两 face 必须同刀。

## 修复（四文件三面）

- **face W 写入执行链（part01＋part03＋part05）**：part03 detached-doc `write()` 快照追加循环后插执行钩——`doc.__r115SetWin` 印＋defaultView 物化的浏览上下文文档，写入体经 `globalThis.__zwRunFrameWriteScripts44b` 桥进 R206 `_zwRunIframeScripts` 通道（合并 Function、with(window) 裸 on* 落帧窗、active-doc 槽位窗、per-段错误隔离）；桥加 `opts44b.force` 直穿 `_zwScriptsRan` once 门（finish 首跑后写入场仍须执行；缺省 undefined 时逐字节原语义）。sandbox 门：part05 `_zwMakeIframeWin`（全物化路径单一咽喉）落 `doc._zwFrameFlags44b` 旗标槽，write 钩读槽校验 `sandboxed && !allowScripts` 不执行（trust boundary 不简化）。FIXME：流内逐 script 时序面待增量 parser（现批次尾一次执行，单写单 script 语料形态等价）。
- **face P 每文档政策容器（part14＋part01）**：`_zwTtPerDocCsp44b` WeakMap（doc→CSP 态）；`__zwTtFrameOwnCsp44b` 落册口记录中性态（＝无 CSP 交付精确近似——宿主 fetch 桥未 wire 帧响应 CSP 头，FIXME 头 wire 后按头解析）；读侧 `_zwTtParseCsp`/`_zwTtEnforced`/`_zwRoActive` 三口记录优先直返（不触全局闩锁/memo），读窗＝`__zwActiveIframeDoc` 槽位（R206/s339/帧消息/write 四执行窗同键）。落册面按 URL scheme 收窄：finish 管线被 srcdoc/data:/blob: 先截臂共用（刀㊸/k43 同管线），仅非 `about:(srcdoc|blank)`/`data:`/`blob:` URL 落册——本地 scheme 继承语义零变化。
- **about:blank 先截臂（part01）**：`src === 'about:blank'` 合成空 text/html wire 走 finish 管线（srcdoc 同构）——初始 about:blank 文档是 text/html（spec iframe-embed-object initial about:blank 语义）；空 body 无 script，R206 零执行。杀掉 fetch 分支 'xml' kind 错型根因。

## 探针（zz-s10k44b-probe1，判读后已移出语料）

write-exec 链诊断页（`'<scr'+'ipt>'` 拼接防 inline 截断——k43 陷阱二演）：ab 臂揭 XML kind 错型（上表根因）；bh 臂 onload→open/write/close→postMessage 往返全通＝face W 链端到端成立。终版字节复跑 `2P/0F`（EXIT=0）。

## solo 页级（post 树，终版字节，电池前先行）

| 页 | base（k44-tt-post 权威） | post | 判 |
|---|---|---|---|
| inheriting-csp-for-local-schemes | 6P/1T/3NR（10 臂） | **10P/0F** | 四余臂全翻：about:blank 两臂继承门 TypeError 面＋blank.html 两臂自有 delivery null 异常面 |
| declarative-shadow-dom-write-to-iframe（wc 腿域） | 0P/1F（pre-dsw，HEAD 字节） | 见电池 census | face W 邻接面 |
| custom-element-reaction-queue（wc 腿域） | 0P/6F（pre-cerq，HEAD 字节） | 见电池 census | 对照页 |

html/semantics 两写链邻接页（src-repeated-in-ancestor＋move_iframe_in_dom_02）经守卫入口零结果行（枚举跳过，runner exit 1）——harness 不可测面，记证据不立项。

## 电池四腿＋TT 同树 A/B（solo 串行，2026-11-27 19:07:41→19:40:31 UTC，墙钟 32m50s，无 ABORT）

| 腿 | 计数 | 判 |
|---|---|---|
| sanitizer-api | 713P/65F/0T | F/T 多重集 SET-IDENTICAL vs k44 |
| domparsing | 1314P/221F/1T | SET-IDENTICAL |
| web-components | 17629P/920F/20T | F/T SET-IDENTICAL；P −182＝gethtml.html 单页（3550→3368，已知巨页截断抖动族，F 集零差排除） |
| dom | 57475P/362F/12T | F/T SET-IDENTICAL；P −69＝Range-mutations 双页（−36/−33，已知截断抖动族） |
| TT base（checkout＝HEAD 含刀㊹） | 2218P/45F/20T | vs k44-tt-post **F/T SET-IDENTICAL**（对锚正确性自证） |
| TT post（重放 md5 断言过） | **2222P/45F/19T** | **+4P/−1T；F IDENTICAL NEW=0；T GONE=1 恰＝about:blank require 臂旧 Timeout 行** |

四腿 rc=2 均为既有基线失败集（rc 只留痕）。判读行提取全部 LC_ALL=C awk＋comm 前 wc -l 对账（grep 对 wc 日志双侧静默零输出假绿一次，awk 复核矫正——k44 教训复演）。页行 dump：目标页 10/10 全 Pass（套件内）。逐页 P census 对账：wc/dom P 漂移全部落在已知截断噪声三页（F 集零差排除），零真实漂移。**face W 邻接面零动**：declarative-shadow-dom-write-to-iframe（1F 恒）与 custom-element-reaction-queue（6F 恒）——write 钩执行 script 不重解析 DSR，CE reaction queue 时序 unrelated，两页留池待后续刀位。

## 结论

刀㊹b 净增 **+4**，TT 真实池 **38F 恒**（四余臂原为 T/NR 形不在 F 池），Timeout **20→19**，**累计 84,260→84,264（距 ≥99,947 差 15,683）**。inheriting-csp 页 10/10 收敛（刀㊸ 后继挂账闭合）；TT 池余键＝其余散布（约 22F）＋module 子系统切片 16F（立项评估待做）。写链时序面（流内逐 script 增量 parser）与帧响应 CSP 头 wire 后按头解析升级为 FIXME 挂账；DSR-write 与 CE-reaction-queue 两邻接页留池。
