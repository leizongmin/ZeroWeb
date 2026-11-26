# 刀㊴ 判读（s10k39）——TT 004 stale-pick 簇＋policy 回调空返回＋脚本 detached 抢跑

日期：2026-11-26 · 域：trusted-types / DOM 查询 · 树锚：本刀 patch `knife39-final.patch` md5 `48e799f6ce9914ed04e984bc7baec548`

## 目标簇（池内入刀前）

- `script-enforcement-004.html` 4F（池内最大单页）：moveBefore／splitText(normalize)／cloneNode／cloneContents 四面。
- `policy-without-return-value.sub.html` 4F：默认策略回调不返回值（undefined）面。

## 根因与修复（5 文件，码刀见 commit）

### ① part12 `_zwTtPolicyMethod`：回调缺省返回 → 空串

- 旧：回调返回 undefined/null → throw（TypeError）。spec「create a trusted type」算法：回调不返回值 → **空字符串**（crbug 430335244 回归语义）。回调缺失仍 TypeError；回调抛出仍穿透。
- 面清理：pwrv 4 面（createScript/createScriptURL/setHTML 等 sink 空串载体）。

### ② part04 URL 反射 getter：SCRIPT.src 在场空串仍解析

- 旧口径 `_r5uRaw === ''` 一律返 `''`；spec reflect：absent → `''`，**在场（含空串）即 parse**，`''` 对 base URL 解析成功＝base 自身＝文档 URL（同 formAction `parse('')` 口径）。
- 收窄：仅 SCRIPT tag 改臂；TRACK/iframe/video/audio 维持既有专用空串臂（各自 corpus 钉形），表内余 tag 无空串语料面，泛化待证据。
- 面清理：pwrv `script.src` 期望 document.location.href 面。

### ③ part03 `_zwScriptPrepConnected`：getter 真＋父非 null ≠ 连接（链爬升定论）

- 004 moveBefore 面根因：script 摘除后挂 **detached root**（createElement div 等），变异中点 `isConnected` getter 真（宿主快照残留原位）＋父链非 null → prepare 抢跑：E1 形执行序提前（CREATE_SCRIPT 先于 append）；跨容器搬文本形中点读陈旧快照文本（`;`）判 trusted 静默 eval 抢章 → append 时点 prepare 被 run-once 跳过（语料 [SET,APPEND] 静默形）。
- 修复：getter-true 臂走 parentNode 链爬升（达文档 true／fragment false／detached 容器 false）；链断（null，plain isConnected 缺席形态）回落原「getter 真＋父非 null」直信臂——plain 链零变化。
- 面清理：004 moveBefore 面（E1/E2/E3 执行序探针复验）。

### ④ part05 `_zwHostAttrConstraint`／`_zwAttrConViolated`＋part06 document 查询入口消费：宿主命中 `[attr]` 谓词 lw 复验

- 004 normalize/cloneNode/cloneContents 三面根因＝**stale-pick 双消费**：TT 004 helper `create_svg_script_with_trusted_source_text` 用 `querySelectorAll("svg script[type='unknown']")`＋textContent find 拾取预制脚本；脚本被拾取消费后经 remove → removeAttribute('type') → 改文本 → 重挂 container 复位。宿主 dom_html 对「Remove 已 apply 后的属性变更」不同步（apply 侧 SetAttr/RemoveAttr 按**位置 sel 寻址**，元素摘除后同位置兄弟漂移使命中错节点或 detached warn-skip → 属性变更丢失）→ document 级 QSA 候选列恒含已摘 type 的已消费脚本；find 的活 textContent 过滤失效（split/merge 保文本＋部分已消费脚本文本被 SET 成裸 LOG_RUN 与 clone 面拾取文本重合）→ **双消费**：对已 started 脚本二次 prepare 被 run-once 静默跳过 → [SET,APPEND] 空跑。
- 可行性探针定谳（identity）：宿主命中已搬移元素的 sel 串＝原 `__zwSelector`（probe `hit0 sel=...svg:nth-child(1) > script:nth-child(1)` 恒等）；`_zwQueryWrapIdentity` 反查回原 proxy（sameObj=true）；proxy `getAttribute` 走 LW 真值层（type=null 正确）。→ shim 侧查询边界后过滤可行。
- 修复：s81 id 约束（`_zwSelIdConstraint`/`_zwIdConViolated`）同型推广到一般 `[attr]` 谓词——`_zwHostAttrConstraint(q)` 解析末位复合段 attrs（选择器列表／unsupported 伪类段／无 `[` → null＝host 行为保持；仅单 complex，s81 同款列表不接补偿）；`_zwAttrConViolated(proxy, attrs)` 经包装 proxy `getAttribute`（LW）复验 `=`/`~=`（R124 ASCII 分词）`|=`/`^=`/`$=`/`*=`／i 旗标，任一不满足即 violated（stale 剔除）。消费点：document.querySelectorAll（out161 包装后过滤）＋document.querySelector（首查＋pending 回落循环两臂）。
- tag/class/id 约束不在本刀（id 有 s81 专面；class/tag 同窗漂移无语料钉，泛化待证据）。

## 判读

### solo 页级（post 树）

| 页 | base | post | rc |
|---|---|---|---|
| script-enforcement-004 | 27P/4F | **31P/0F** | 0 |
| policy-without-return-value.sub | 0P/4F | **4P/0F** | 0 |
| block-text-node-insertion-into-script-element | 7P/3F | 7P/3F（恒等，3F 不在本刀位） | 2（留痕） |
| script-enforcement-006 | 1P/3F | 1P/3F（恒等，module/importmap 新切片位） | 2（留痕） |
| script-enforcement-007 | 1P/3F | 1P/3F（恒等） | 2（留痕） |

### 电池四腿＋TT 同树 A/B（字节绑定 `48e799f6…`，setsid pid 4121145）

四腿＋TT A/B 全数落定（driver `/tmp/s10k39/driver-k39.log`，全程 13:42:21→14:16:07Z＝34min）：

| 腿 | k38 基线（HEAD 树） | k39 post | 判 |
|---|---|---|---|
| san | 713P/65F | 713P/65F | **SET-IDENTICAL** |
| dp | 1314P/221F/1T | 1314P/221F/1T | **SET-IDENTICAL** |
| wc | 17568P/920F/20T | 17674P/920F/20T | F/T multiset **恒等**；P +106 全落 gethtml.html（3307→3413）＝截断抖动已知噪声页（±200 家族） |
| dom | 57447P/362F/12T | 57440P/362F/12T | F/T multiset **恒等**（GONE=0 NEW=0）；P −7 同族截断抖动——QSA `[attr]` 后过滤共享路径零回归 |
| TT base | — | 2185P/74F/20T | 与 k38 tt-post 逐键同带＝基座稳定 |
| TT post | base 2185P/74F/20T | **2194P/65F/20T** | **+9P/−9F，NEW=0**，Timeout 20 恒等 |

GONE 9 键（全真实页，无探针页）：004 ×4（moveBefore/normalize/cloneNode/cloneContents 全清）＋pwrv ×4 全清＋**script-enforcement-003 ×1 红利**（`Normalizing script source via Element.normalize()`＝stale-pick 同根因，QSA 过滤器免费修复；solo 32P/0F rc=0）。

**TT residual 池 74F → 65F（真实页 58＋探针页 7）＋Timeout 20**。**累计 84,227 → 84,236（距 ≥99,947 差 15,711）**。

## 事故记

- 本 session 页名笔误 `policy-without-return-value`（真名 `policy-without-return-value.sub.html`）→ runner 0.008s 即退 rc=1 零输出；一过性误判为「语料页被删／码回归」，ls 核名即解。语料目录另见前期刀 zz-knife3*/3b2* 探针页遗留（gitignored，未清）。
