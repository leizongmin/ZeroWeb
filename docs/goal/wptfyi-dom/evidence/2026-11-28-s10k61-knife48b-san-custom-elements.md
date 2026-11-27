# s10k61 刀㊽b：san 残池 CE 簇四根因——custom-elements-is 8F＋sethtml-with-custom-elements 3F 清零

日期：2026-11-28。刀体 commit `5ff02c7bd`（发射号 4815c8495，rebase 重放：上游 css 流码刀 5fed0f2d4〔css-mixins owner sheet 生命周期，part04.js @11818+／js_dom_bridge.rs @5902+〕并入致两文件整文件 blob 变化，**hunk 域零相交**〔本刀 part04 全部 hunk @2689-6425、bridge @3349〕，余 6 文件 blob 恒等）。8 文件：part03.js＋part04.js＋part05.js＋parser.rs＋document/mod.rs＋snapshot.rs＋serializer.rs＋js_dom_bridge.rs，197 增 1 删。guard 扫描 PASS（0 发现 0 提醒，8 blob 绑定）。本刀独占 s10k61（.acceptance/dps/ 无并发轮，solo 窗口）。刀号撞号注记：㊽ 已被 s10k49（module R1-b）占用，按 ㊻b 先例取 **㊽b** 消歧，commit 标题＋evidence 目录双锚。

## 刀位与基线

控制面点名簇：san 残池 29F 中 custom-elements-is 8F＋sethtml-with-custom-elements 3F（11F 簇）。锚 **749P/29F**（ver-san60-post2.log＝刀㊼ 收账态）。

## corpus ground truth

- `custom-elements-is.tentative.html`（8F）：容器 `createElement("div")`＝**handle 域**。removed 套件断言序 innerHTML（剥净 `<div>` 无 is）／hasAttribute false／matches `:not(:defined)`／ctor 计数 false／conn 计数 false；kept 套件 innerHTML（`<div is="x-foo">` 保留）／hasAttribute true／matches `:defined`／removeAttribute 后 innerHTML **is 再物化**／ctor true／conn true。
- `sethtml-with-custom-elements.html`（3F）：容器 `document.body.firstElementChild`＝**sel 域**。==1 三案（x-probe allowed／is-blocked／sanity `div is=x-div` kept 后 connected==1）。

## 四根因与订正

### ① setHTML CE 升级钩子缺失（ctor 轴）

setHTML/setHTMLUnsafe 产物代理种子不触发 CE 升级树遍历——spec insertion steps 要求解析后升级。sel 域：种子块补 `__zwCeAttachForAdded(_uAdded, _ceParentConnected(...))`（完整升级含 ctor）；handle 域：物化岛 `_wcRebuildAsHandle` 经 createElement 建元素——autonomous 已构造并盖 `_ceUpgraded` 章（R90 创建面），cbi（`<div is=…>`）createElement('div') 不反查、is 属性是重建循环 setAttribute 后置的**无升级窗口**（探针 kept-empty-cfg ctor=0/conn=0 实证——探针容器即 createElement 域）。补同款钩子：已构造元素经 WeakMap 章早退零重放，conn 由 `_ceConn` was 门去重；物化失败回退代理（无 handle）入列表无害（nodeType 门跳过）。

### ② `_ceApplyConn` plain 分支 cbi 反查缺失（conn 轴，sel 域）

plain 解析代理 tag 是宿主名（div）、注册名是 CE 名（foo-bar），tag 直查恒 miss→connectedCallback 永不派（sanity 案「div is=x-div kept 后 connected==1」根因；SELHOST 探针 conn=0 定谳）。补 cbi 反查三段：`_ce_cbi_count > 0` 门（define 只增不减的结构性空快道）→ `getAttribute('is')` → `__zwCERegistryLookup(is, 宿主名)`。升级原语 `_ceUpgradeNode` 同链已有反查，连接派发侧缺同款——本刀补齐。断开 walk 同样命中（键组删除对称）。

### ③ `:defined` 同 turn 快照 stale（matches 轴）

matches trap registry 查询在同一 microtask 内 define 后仍读旧快照（custom-elements-is 8F 的 matches 轴全数）。shim matches trap 裸伪类特判 `_zwIsPotentialCustomElementName3301(tag)`（首字符小写字母＋含连字符＋PCEN 集）静态近似，与 dom query.rs R3299 同源。探针 PLAINDIV 精确：div=true／xprobe（未 define）=false。

### ④ is 值序列化丢失（serialization 轴）

spec §4.13.1.4：is 属性不驻 DOM（hasAttribute false／getAttribute null）但创建时值保存于元素（is value）、序列化时回写输出。双域分治：
- **sel/引擎面**：`DomBuilder.is_value_map`（解析期印记）→ `Document.is_value()`/`set_is_value()` → serializer HTML-ns 元素臂无 is 属性时重物化 → `snapshot.rs` 随搬移 → `copy_subtree_from` 移植（setHTML sel 域写路径经此）。
- **handle/journal 面**：attr 读全走 mutation journal（`__zw_attr_names_handle`/`__zw_get_attr_handle`/`shared_state_inner_html`），引擎表**不可达**——`_wcRebuildAsHandle` attrs 循环 HTML 臂印 `el.__zwIsValue` expando（该面唯一载体）；part04 handle-only outerHTML 臂（innerHTML/getHTML 融合序列化均经子 outerHTML 级联）经 **proxy 面**读章再物化（get trap `this`=handler 非 proxy，`function(_t, prop)` 签名无 receiver——必须 `_makeProxy(sel, handle)` 幂等缓存面读）；removeAttribute 后真实属性缺席而印章仍在→重物化 `is="value"`（转义同 R123 全集）。

## 探针链（probe-ceis→post→post2→post3→post4→post5）

六案 DIAG 断言（探针页语料文件运行时读盘，跑后删除）：

| 案 | 域 | 终态（probe5 全绿） |
|---|---|---|
| KEPT-EMPTY-CFG | handle | ih 保留 is＋ctor=1 conn=1 ✓ |
| REMOVED-DEFAULT | handle | 剥净＋ctor=0 conn=0 ✓ |
| PLAINDIV | — | `:defined` 静态近似精确（div=true／xprobe=false）✓ |
| INNERHTML-SETTER | handle | ih 保留 is＋conn=1；**ctor=0 已知挂账**（`_ihMat` 物化臂缺升级钩子，无语料钉面） |
| SELHOST-EMPTY-CFG | sel | ctor=1 conn=1 ✓（② 验证面） |
| RA-FACE | handle | removeAttribute 后 stamped 在＋has=false＋再物化串恒等 ✓ |

迭代：probe2 误诊 KEPT ctor=0 为钩子缺陷——实际探针容器 createElement=handle 域，sel-branch 钩子不在路径上（教训：先核元素寻址域再归因）；probe3 SELHOST conn=0 定谳②；RA-FACE `stamped=foo-bar inh=true` 而序列化缺 is→get trap `this` 陷阱→proxy 面读，probe5 全绿。

## ==0 面回归风险分析

sethtml-with-custom-elements 全部 ==0 案 sanitizer 配置剥 is 或删元素→净串无升级触发；`_ceUpgraded` 章＋`_ceConn` was 门防双跑双派。

## 判读

| 臂 | 锚 | 实跑 | 归因 |
|---|---|---|---|
| san 全臂 | 749P/29F（ver-san60-post2.log） | **760P/18F/0T** | 四象限 comm：翻绿 11（custom-elements-is 8＋sethtml-with-custom-elements 3）、**新回潮 0**、ΔP+11=ΔF−11 闭合 |
| dom 臂 | 57524P/361F/12T（ver-dom60.log） | **57488P/361F/12T** | **F·T 测试级集恒等**（翻绿 0／新回潮 0）＝刀零 dom 影响；P−36＝页预算截断噪声族（Range-mutations ±150+ 既知形，36 在带内） |
| tt61 | 2290P/14F/18T（ver-tt60.log） | **2290P/14F/18T** | 逐位恒等零漂移 |
| ref61 | 725/725（ver-ref60.log） | **725/725**（Layout 521/521＋Text 204/204） | 零漂移 |

chain61 EXIT rc=0（16:13:13Z→16:47:52Z，san→dom→tt→ref 四腿）。

### 字节绑定（双 bin 闭合）

- 发射 bin `92260d0eb743`＝4815c8495 树（＝guard 暂存 8 blob）：四腿判读域。发射断言通过（expected-bin.md5）。
- rebase 并入上游 5fed0f2d4 后 push `5ff02c7bd`：净树复验腿 chain61b 重编译 bin `62c430e6ed93`＝5ff02c7bd 树，san 复跑 **760P/18F/0T＋F 集逐测试恒等（comm 差 0 行）**——主判读腿在提交域字节上复验闭合。dom/tt/ref 恒等回归腿的上游字节风险面（css-mixins owner sheet）不触及三臂判读面（hunk 零相交佐证）。

## fmt 既有漂移复认

`cargo fmt --all -- --check` 15 区既有 diff（if_function.rs 11＋mixin_apply.rs 1＋callbacks.rs 1＋css_wire.rs 1＋js_dom_bridge.rs 1）——与刀㊼ 挂账集一致，本轮编辑区零新增，挂账专刀不变。

## 残相挂账

- san 残池 **18F**（本刀后）：declarative-shadow-root 7F＋svg-animate SMIL 1F＋其余簇 10F census 待下轮。
- innerHTML setter `_ihMat` 物化臂 cbi ctor 缺口（探针 INNERHTML-SETTER ctor=0 conn=1，无语料钉面）。
- handle 域 t8r 重建岛连接态补派发镜像；TEMPLATE 容器 context 激活；sel 域 outerHTML 产物视图透传。
- `:defined` registry 感知精确化（三通道：registry 含 tag/namespace/构建中）；spec 保留名（annotation-xml 等 5 个）近似偏差。
- post-drain is 固化为真实属性（hasAttribute 翻 true，引擎/handle 两序列化面同族 FIXME）。
- census：+11 → **84,379**，距 DC-3 校准线 ≥94,093＝9,714。
