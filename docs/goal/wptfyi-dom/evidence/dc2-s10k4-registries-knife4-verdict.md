# DC-2 s10-registries 刀④（DSR 镜像子 parentNode 接线两针 ＋ attachShadow 缺席默认）判读收账（2026-11-24）

本席 rally 新轮（前轮 14:18 429 死亡接管）：k4 WIP 恢复（stash transit）→ 拼接
SYNTAX_OK → 常驻钉测 part150 三代全绿 → 终态电池判读。轮初收敛两事：① de03c6a0f
异基重放残 commit 判冗余 rebase --skip（css 流接岗轮 656f8669b 已推同批账面 blob，
本流 4bb2f6067 恢复提交 pull 时被 git 判 already-upstream 自动 drop——账面零缺失）；
② 13:52 前轮电池（二进制 06aa59a3）取证定谳＝含 k4 早期迭代镜像臂（二进制内
comment 块字节取证），非 pre-k4 基线，不入判读链。

## 范围（刀③ 残面节首项）

DSR（声明式 shadow root）镜像子 parentNode 语义父接线＋attachShadow 缺席默认
registry。根因（k3 判读档残面节＋k4 探针）：`_zwDeclarativeShadowEnsureMirrors`
镜像子与 s176 内容绑定子的 parentNode 落引擎父视图（shadow 地址 `zwsh:host|local`
父折叠到 host，探针 cePn=1:DIV 实证），registry 解析父链（逐跳 parentNode 上行）
越过盖 decl 印的 SR wrapper 直达文档默认 global——ShadowRoot-init-declarative
test1/test2 expected-null-but-got-object 根因。

## 修复面（三件，part05.js 单文件 +57 行零 Rust 行为面）

1. **镜像子 pending-parent 槽同步**（`_dsBuilt` 臂）：容器 appendChild 的 R5240
   同步槽对镜像子不达（createElement 产物无 `__zwSelector` 延迟铸造），补
   `_zwSelPendingParent = { parentSel: null, parentHandle: rootHandle }`——
   parentNode trap 消费返 `_makeProxy(null, rootHandle)`＝页面持有 SR wrapper
   同一缓存对象（k4 探针手工盖章 afterPnIsSr=true／afterReg=null 实证全链）。
   spec concept-node-pre-insert 步骤 6「set parent」。
2. **内容绑定子同步**（s176 `kids176` 臂）：本函数整表替换 `_handleChildren`
   （镜像对象被弃），枚举子以 selector proxy 面世， parentNode 读走引擎父视图
   ——同款槽形逐子补章。handle 子不经此臂（`_recordHandleChild` 已建反链）。
3. **attachShadow 缺席默认**（`_zwAttachDeclarativeSRView` 缺 option 分支）：元素
   node document 的 customElementRegistry（whatwg/html scoped registries——
   ShadowRoot-init-declarative 标题实证面）。显式盖槽后 getter 不走解析上行，
   decl-null DSR 子树内命令式根不再借道祖先 null 印，同步对齐 upstream
   effective-global 语义。

常驻钉测 **part150_s10k4_dsr_mirror_parent_pins.rs** 两针（同提交）：DSR-attr 根
镜像子 parentNode===SR wrapper＋registry null；无属性 DSR 根同接 wrapper＋registry
live（wrapper 无槽越界 host→doc default）。探针页 k4-decl-probe.html gitignored
刮擦面用后即删，本钉测即常驻回归载体。

## 字节绑定链

| 阶段 | md5 |
|---|---|
| 终态 runner（电池判定二进制） | `c4260af19ec323d45dc514a99544e58e` |
| part05.js（三臂终态） | `1dfb53c9c357a720c93c2e659697e583` |
| js_dom_bridge_tests.rs | `9bacae45f5c9006c4e01c971bcf675ff` |
| part150 钉测 | `cc79a9937268deff6e8bace470ce6ea4` |
| 电池发射器 | `bb1c219f46ee5de70cb51df3a0222889` |

前置钉测：`cargo test -p zero-engine --lib s10k`（test-guard 外单针协议沿 刀③）
＝test_s10k4_dsr_mirror_parent_pins / test_s10k2_dsr_contents_pins /
test_s10k3_iframe_registry_pins 三代全绿 3 passed 0 failed。拼接 29 分片
node --check SYNTAX_OK（`_zwSelPendingParent` 112 处）；终态二进制三臂标记在
（`_zwSelPendingParent`×121／`_e176pp`×1／`_srDefReg`×3）。fmt --all 零新 diff；
clippy -p zero-engine 唯一 warning＝cssa_registry.rs:262 identical-if
（3e8cdfc12 css 流 R2 落库，本刀域外既有，不顺手修）。

## 电池判读（锚＝已推 刀③ [dc2-s10k3-wc-post-knife3.log](dc2-s10k3-wc-post-knife3.log)）

| 腿 | 刀④ 终态 | 对已推 刀③ | 判读 |
|---|---|---|---|
| testharness-web-components | **18135P/3959F/19T** | **+6P/−6F/+0T** | 四页全限定（下表），其余 467 页零漂移 |
| testharness-domparsing | 1264P/271F/1T | 恒等 | SET-IDENTICAL（per-(status,page) 键 0 diff） |
| testharness-sanitizer-api | 414P/342F/1T | 恒等 | SET-IDENTICAL（键 0 diff） |
| testharness-dom | 57916P/496F/11T | +371P/−1T | Range-mutations 双页停摆噪声族（下节） |

### WC +6P/−6F 逐页归因（四页全限定 registries/DSR 簇，逐测试名核对）

| 页 | 刀③ | 刀④ | Δ | 翻转子测 |
|---|---|---|---|---|
| custom-elements/registries/ShadowRoot-init-declarative.html | 3F | 1F/2P | **+2P/−2F（刀④主目标）** | test1/test2「custom/built-in element inside 'shadowrootcustomelementregistry' declarative shadow root should use document's registry as attachShadow default registry」 |
| custom-elements/registries/Element-customElementRegistry.html | 8P/3F | 10P/1F | +2P/−2F | 「element inside a declarative shadow DOM with shadowrootcustomelementregistry should return null」＋「should return the scoped registry after call」 |
| custom-elements/registries/element-mutation-null-registry-removal.html | 4P/1F | 5P/0F | +1P/−1F（页全绿） | 「An element's null customElementRegistry should not mutate after removal from a declarative shadow root」 |
| shadow-dom/declarative/declarative-shadow-dom-basic.html | 16P/6F | 17P/5F | +1P/−1F | 「Declarative Shadow DOM: Basic test」（`sr.firstElementChild.parentNode === sr` 语义面） |

残面精确持位：ShadowRoot-init-declarative 残 1F＝「…parsed via setHTMLUnsafe
should use null registry」＝unsafe 子树 face 链（刀③ 残面节原样、本刀不触，
预判持位兑现）。

### dom 腿（+371P 噪声族）

`Range-mutations-dataChange` 993→1204、`Range-mutations-replaceData` 986→1146
＋replaceData 本窗 TO 解除（1→0）——F 构成 496 恒等、其余页零漂移。P 缺失/回摆
形＝master 碰撞记录 2026-11-23 两则定谳的停摆族（停摆点逐窗漂移，本窗为完全跑通
窗）。刀域（registries JS/DSR 印记）与 dom/ranges 页零交集；判读按噪声族入账。

## 早期迭代辅助证据（13:52 前轮电池，二进制 06aa59a3）

前轮 13:48 构建（`/tmp/s10k4-build*.log` 三连 c7fae29b→17ec3037→06aa59a3）含
镜像臂早期迭代（终态二进制内 comment 块字节取证）；其 WC 腿（`b369d23d`）已见
同四页同六翻转＋k4-decl-probe.html 刮擦页 5F（页已删）——本刀终态电池对之
唯一差异＝刮擦页移除（5F→0），终态三臂对语料零新增翻转。翻转全部归属镜像臂
＋attachShadow 默认臂，kids176 臂为语义完备面（钉测 ② 载体）暂无独立语料翻转，
如实记。该 4 log 留 /tmp 不入库（刮擦迭代非判读锚）。

## 证据日志（同提交 add -f）

dc2-s10k4-wc-post-knife4.log（`3d3974e61cfe6f0e7b2c8f73b2103bde`）/
dc2-s10k4-domparsing-post-knife4.log（`492dc92343e774df4272e0e6a488e627`）/
dc2-s10k4-sanitizer-post-knife4.log（`2b319939d4ec51c4e9cc306e03b98e0b`）/
dc2-s10k4-dom-post-knife4.log（`aaf96661a6c1690628062e8f97439137`）。
电池窗 06:46:45–06:58:51Z（四腿顺序流 solo 无并发构建）。

## 残面（下刀挂账，序沿 刀③ 判读档不变）

- ShadowRoot-init-declarative 残 1F（setHTMLUnsafe unsafe 子树 face 链）、
  ShadowRoot-init-customElementRegistry 残 1F（模板内容克隆宿主 null registry 链）。
- define.html 22F（`define()` 参数校验矩阵）、define-customized-builtins 1F
  （bgsound）、element-mutation 15F、R90 trap scoped 原型臂 instanceof、
  parse-time attr 区分、importNode/upgrade 页、IAH handle-only——沿旧序。
- iframe 合成 doc 无 legacy `customElements` 读面（刀③ 钉测勘误实证）。
