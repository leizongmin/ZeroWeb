# 刀㊸ 判读（s10k43）——iframe 本地 scheme CSP 继承三面＝srcdoc require F 清零＋data/blob 四缺失转 Pass（真实池 44→43F）

日期：2026-11-27 · 域：trusted-types / inheriting-csp-for-local-schemes（iframe/local scheme CSP 继承域）· 树锚：刀体 patch md5 `d24e57955f2986aef47757fa4afd0022`（发射 TREE-MD5-OK＋TT 重放 REAPPLY-MD5-OK 双断言）；**落库码刀 `63b792449` patch md5 恒等 `d24e5795`——电池域=落库域零差**（刀㊷ 的注释订正差本刀不存，判读后未再改字节）。落库字节 TT 全量已含在电池 post 腿（同一 bytes）。

## 目标簇（散布池最大单键，入刀前 1P/1F/1T/7NR）

页五组子测＝srcdoc/data/blob/about:blank/blank.html 各两臂（`require-trusted-types-for` 继承＋`trusted-types` 名单继承）。刀前形态：srcdoc 对 1P/1F（require F＋list P）；data/blob 四臂全缺失（T＋3NR）；about:blank require T；blank.html 对 2NR。runner 有 `__zw_fetch` 无 iframe 本地 scheme 管道是缺失主体。

## 根因（三面同源：iframe 域门与册的全局/缺失）

- **face A（srcdoc require F）**：iframe 文档 createElement 被覆盖（part05:3404）产 `_zwIframeCreateElement` 平对象元素，其 R181 innerHTML setter **无 TT 门**——主文档 sel 形代理（part04:16564）与 detached/factory M 形（part03 刀㉒）均已有门。帧内 enforcement 标志 TRUE（part14 `_zwTtEnforced` 读 head meta 全局缓存）、直调 `__zwTtSinkValue` 也抛错，唯独元素形式裸写 → require 臂 F。探针 zz-s10k43-probe2 p02e/p02g 双面定谳（元素 form/hasOwnIh/creator 链）。
- **face B（data/blob 四缺失）**：`_zwLoadIframeEntry` 的 data:/blob: src 落进 `__zw_fetch` 分支（http(s) only）——fetch 分支先命中且**早退**，else 回落体是死分支（挂账注记里的「data/blob 臂」原位不可达）。探针 probe3 boot 灯塔直证：data 帧 cd=null 永挂、blob 帧得空 text/plain wire。
- **face C（重名 TypeError）**：`trustedTypes.createPolicy` 重名门用全局 `_zwTtCreatedNames` 册——帧脚本（data/blob/srcdoc 内 boot 页）再 `createPolicy('allowedPolicy')` 抛 `policy with the same name already exists`。spec 每 Document 独立 TrustedTypePolicyFactory。

## 修复（四文件，三面各一）

- face A：part05 R181 setter 镜像刀㉒门——`__zwTtSinkValue('TrustedHTML', …, 'Element innerHTML')` 先行转换。
- face B：part01 `_zwLoadIframeEntry` srcdoc 分支后、fetch 分支前插**先截臂**：data: 走 `_zwDataURLProcessor`（RFC 2397，text/html only）→ `'200\x1fOK\x1fcontent-type\x1e<ct>\x1e\x1f'+body` wire 进 `_zwFinishIframeEntry`；blob: 走 `_zwBlobStore` 精确命中＋origin 同源校验（`/^([^\/]+:\/\/[^\/]*)\//` 提取 vs `location.origin`）；未命中 → 新助手 `_zwIframeEntryFallNoSrc43`（no-src 回落，保持 R130 旧账语义）。
- face C：part14 导出 `__zwTtActivePolicyNames`（WeakMap 以 `__zwActiveIframeDoc` 为键的名册）；part05 帧 `win.postMessage` 派发窗设/还原 active-doc 槽位（s339/s382 同型 try/finally）；part12 createPolicy 重名门按活动文档分册（`allowDuplicates` 或无槽位回落全局册＝主文档行为零变化）。

## 探针方法学（zz 页 gitignored，已移出语料，存 /tmp/s10k43/）

- probe1：状态 dump＋双写＋srcdoc 诊断＋四帧可达性（有界 race 400-600ms 防 `__timedout` 级联）。
- probe2：拆断言 p02a–p02g＋帧内诊断（form/ctor/hasOwnIh/odIsDoc/creator/w1/w2/直调 sinkValue）——p02e 期望值 `[object HTMLDivElement]` 系探针自身陈旧（工厂元素按设计是 `[object Object]`），p02f/p02g 证明元素路径可用。
- probe3：boot 灯塔（帧→parent postMessage 带标签）＋parent 侧 contentWindow/Document 状态——定谳 face B 死分支。方法学教训：模板字面量内 `</script>` 须写 `<\/script>`（parse 级提前终结外层 script 元素，两撞）；OR 断言不定位（`a || b` 恒真遮蔽单臂），拆严格断言＋每帧标签。

## solo 页级（post 树，终版字节）

| 页 | base（k42-tt-post 权威） | post | 判 |
|---|---|---|---|
| inheriting-csp-for-local-schemes | 1P/1F/1T/7NR | **6P/0F/1T/3NR（rc=0）** | +5＝srcdoc require F 清零＋data/blob 四臂全转 Pass |

余 about:blank require T＋blank.html 对 2NR＝**挂账刀㊸b**：about:blank/blank.html 走 no-src 空文档回落，页脚本经 `document.write` 进已完成文档——执行链未通；blank.html 对另需**每文档 CSP**（enforcement＋名单不继承）：全局门架构在该域产错误抛错（master 刊载「列表与 enforcement 都按 head meta 全局读」），write-exec 与 per-doc CSP 须同刀落否则收 2F。

## 电池四腿＋TT 同树 A/B（solo 串行，2026-11-27 17:24:34→17:58:13 UTC，墙钟 33m39s，无 ABORT）

| 腿 | 计数 | 判 |
|---|---|---|
| sanitizer-api | 713P/65F/0T | F/T 多重集 SET-IDENTICAL vs k42 |
| domparsing | 1314P/221F/1T | SET-IDENTICAL |
| web-components | 17216P/920F/20T | F/T SET-IDENTICAL；P −271＝gethtml.html 单页（3226→2955，已知巨页截断抖动族） |
| dom | 57384P/362F/11T | Fail 多重集恒等；P +201＝Range-mutations 双页（+45/+156，已知截断抖动族）；T 形变＝replaceChildren 单页截断落位（2 子测 T→页级 T，同族 solo 时序伪影） |
| TT base（checkout＝HEAD 含刀㊷） | 2208P/51F/20T | vs k42-tt-post **F/T SET-IDENTICAL**（对锚正确性自证） |
| TT post（重放 md5 断言过） | **2213P/50F/20T** | **+5P/−1F；GONE 恰＝srcdoc require；F NEW=0；data/blob 四缺失转 Pass（页行 dump 逐行对账）** |

四腿 rc=2 均为既有基线失败集（rc 只留痕）。页级 dP=+5 的构成（judge 页行 dump）：F srcdoc require→P＋T data require→P＋3×NR→P；`dP==-(dF+dT)` 检查不适用（NR→P 不入该式），以页行 dump 为权威。逐页 P census 对账：wc/dom P 漂移全部落在 3 个已知截断噪声页（F 集零差排除），零真实漂移。

## 回归面度量（静态 grep）

dom/ 与 web-components/ 语料：iframe 静态 `src="data:|blob:"` 0 命中；动态 `.src="data:|blob:"`／createObjectURL 赋 iframe 0 命中——先截臂跨套件面≈0；face C 名册分册仅帧域行为变化（主文档回落全局册）；postMessage 槽位包裹改帧 handler 内定时器上下文捕获为帧文档（spec 正确向，基线 quirk 变化）——四腿 SET-IDENTICAL 实证无回归面。

## 结论

刀㊸ 净增 **+5**，TT 真实池 **44→43F**（套件 F 51→50；master 收账对本页记 2，其中 data require 面在 k42/k43 权威基线显示为 Timeout 非 Fail，两面均已由本刀清出池），**累计 84,255（距 ≥99,947 差 15,692）**。散布池余键：SharedWorker 2＋ServiceWorkerContainer 2（worker 面）＋其余散布归簇；挂账刀㊸b（about:blank/blank.html write-exec 链＋每文档 CSP 不继承）。
