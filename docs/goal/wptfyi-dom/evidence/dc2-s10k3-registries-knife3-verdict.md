# DC-2 s10-registries 刀③（no-src iframe R365 槽接线 ＋ DSR-attr registry 印记全链）判读收账（2026-11-24）

本席 rally 新轮（Round 2）单席完成：S1 iframe 面 → SR 语义矩阵定谳 → 引擎/出桥/JS
五面实施 → transfer 面槽传播修复 → walk 无槽写回分支 → 探针刮擦验证 → 常驻钉测
part149 → 电池判读。全程无并发轮碰撞（本轮窗内 `.acceptance/dps/` 无新增他流条目）。

## 范围

刀②续判读档挂账首项：effective-global-registry 残 48F＝adopt/append 矩阵深域
（DSR `shadowrootcustomelementregistry` 属性语义全链）＋ S1：no-src iframe
contentDocument/contentWindow 首访问面（k3probe 刮擦探针 4 子测定位的 18P/48F 前置域）。

## SR registry 语义矩阵（上游语料实证，本刀实施基准）

| 产物 | registry 语义 | adopt 后 |
|---|---|---|
| DSR 无属性 | 无槽 → live doc 默认 | 重钉 adopt doc default |
| DSR 带 `shadowrootcustomelementregistry` | 粘滞 null（`_ZW_CE_DECL_NULL_REG` 单例印） | **sticky**（不重钉，子元素 null） |
| attachShadow `{registry: null}` | own 槽 `_ZW_CE_NULL_REG` | 重钉（非 sticky） |
| attachShadow 无 option | 无槽 → live doc 默认 | 重钉 adopt doc default |
| scoped slot | 恒持久 | 持久（子树 `_eff` 归零） |

## 修复面（七件）

1. **part04.js**（S1，identity-chain ~1785-1863）：no-src entry 首访问（contentDocument/
   contentWindow 同站）即物化 fallback win——no-src 文档即终态文档（about:blank 初始
   导航，spec navigate-to-multiple-frames 面语义）；R365 realm registry 槽接线
   （`doc._zwCERegistry = win.customElements`，同 part01 加载路径）；R139 fallback win
   记账（后续读同 identity）。
2. **part03.js**（标记＋walk＋resolver，~13299）：`_ZW_CE_DECL_NULL_REG` 单例印＋
   `_zwCeIsRegNullMark` 共享判据；`_zwCeDocDefault` null 判据升级；resolver SR 臂删
   declarative→null 早退（无属性根改走 host→doc default live）；walk SR 臂重写四分支
   （显式非 scoped 重钉 / null 印 declarative sticky、imperative 重钉 / 无槽写回重钉
   / scoped 持久）；`_eff` 归零判据带 `__zwCeNullRegistry`。
3. **part04.js**（R5433 克隆臂 ~8180）：declarative 分支镜像源 wrapper own 槽（decl
   印随克隆），无属性根不盖章（旧恒 `_ZW_CE_NULL_REG` 钉扎删除）；imperative 保持
   `_zwCeBindRegistry`（WeakMap）。
4. **part05.js**（getter＋三盖章面）：`_zwCeSrGetter` 删 declarative→null 早退落 doc
   default live 解析（基准 host.ownerDocument）；`_zwAttachDeclarativeSRView`/
   `_zwDeclarativeShadowHydrate` 按 `sr.r` 盖 decl 印；**`_zwTransferDeclarativeSR`
   槽传播**——首版按 `sr.r` 判（视图对象无 `r` 键恒 false 死码，shape 探针 t2
   `slot=undef` 实证），终版读视图根 `_zwCERegistry` 槽判 `_zwCeIsRegNullMark` 后
   原样拷贝。
5. **crates/dom node.rs**：`ShadowRootData.custom_element_registry: bool` 新字段
   （reference_target post-hoc 同款模式，不动 `attach_shadow_full` 签名波及面）。
6. **crates/dom shadow.rs**：两个声明式物化面（解析期 + 空 contents）捕获
   `shadowrootcustomelementregistry` 属性存在性 post-hoc 置位。
7. **crates/engine js_dom_bridge.rs**：`shadow_root_entry_json` 与
   `declarative_shadow_deep_json_doc` 两发射点加 `"r":{}` 键。

## 探针刮擦验证链（用后即删，gitignored）

- k3probe（no-src iframe 四子测）：槽接线 / contentWindow-first 对照 /
  contentDocument-first 修复面 / global+scoped 矩阵——S1 后 4/4 绿。
- k3-dsr-shape-probe（DSR 形态 dump）：暴露 transfer 面 `sr.r` 死码（t2 slot=undef）；
  槽传播修复后 t2 `slot=nullmark`、t1 `slot=undef`（live）✓。

## 常驻钉测（part149_s10k3_iframe_reg_pins.rs，同提交）

四针全绿（test-guard 外 lib 目标 `cargo test -p zero-engine --lib test_s10k3` 1
passed 0 failed；part148 邻针复跑同绿）：
① no-src iframe 首访问 fallback win 物化＋R365 槽接线；② DSR-attr transfer 印记
（attr 模板 contents 克隆入主文档 SR getter null sticky＋无属性对照 live）；③ 跨文档
adopt 矩阵（DSR-attr sticky null＋SR 内子元素解析 null；DSR-no-attr/imperative null
重钉 iframe doc default——**walk 开门前置**：`__zwCeScopedActive` 门内钉测显式
scoped attachShadow 置位，语料域同型前置）；④ 主文档 hydrate `r` 键（body 标记
DSR-attr 根 null／无属性根 live）。

钉测勘误记录：iframe 合成 doc 无 legacy `customElements` 读面（`fd.customElements`
＝undefined），doc 级读走 `customElementRegistry` getter（part04 1809 注与 k3 矩阵
一致）——独立缺口不入本刀域，挂账残面节。

## 字节绑定链

| 阶段 | runner md5 |
|---|---|
| S1 修前基线探针 | `c368cd3571b6a112ec6018e11dad21c0` |
| S1 后（k3probe 4/4） | `6b080ce9be15833075c0c375dc062431` |
| SR 面全套首探（t2 死码实证） | `0a012d03643c36e53e22759277308d71` |
| transfer 槽传播修复（t2 nullmark＋66P/0F） | `60fade6e97d37ab40224d522a50523b3` |
| walk 无槽写回（电池判定二进制） | `c641b0a322c8f4040ead98d63e6f5a9c` |

源码终态 md5：part03 `2e95df5d3f175125188c09d53ae49c05` / part04
`770e3dab803f4469f7b9bb9b095c2bfc` / part05
`6bcc6d298f3bfc9649ace965767e2257` / shadow.rs `0bf5d0d6a524e70d77aaf59c3d30bf6e` /
node.rs `8696c3268c074b5454adf3b7f1ca8fe2` / bridge.rs
`83c60d452ff9970f195a95317a60edc5` / part149
`a32eb9d5efbb0b5733c9a155f58420dd`。fmt --all CLEAN、clippy -p zero-dom/-p
zero-engine --all-targets 零告警。

## 电池判读（锚＝已推 刀②续 [dc2-s10k2-wc-post-knife3.log](dc2-s10k2-wc-post-knife3.log)）

| 腿 | knife③ 终态 | 对已推 刀②续 | 判读 |
|---|---|---|---|
| testharness-web-components | **18129P/3965F/19T** | **+114P/−31F/+0T** | 六页全限定（下表），其余 463 页零漂移 |
| testharness-domparsing | 1264P/271F/1T | 恒等 | SET-IDENTICAL（逐页 key join 空 diff） |
| testharness-sanitizer-api | 414P/342F/1T | 恒等 | SET-IDENTICAL（逐页 key join 空 diff） |
| testharness-dom | 57545P/496F/12T | −320P | Range-mutations 双页停摆形（下节） |

### WC +114P/−31F 逐页归因（六页全限定 registries 簇）

| 页 | 刀②续 | 刀③ | Δ |
|---|---|---|---|
| custom-elements/registries/scoped-registry-effective-global-registry.html | 18P/48F | **66P/0F** | **+48P/−48F（页全绿，刀③主目标）** |
| custom-elements/registries/define.html | 页级死亡（仅页标 1F） | 48P/22F | +48P/+21F（**页复活**：子测全注册，新可见 22F＝`define()` 参数校验矩阵已知残域；页级标记清零） |
| custom-elements/registries/define-customized-builtins.html | 页级死亡 | 14P/1F | +14P（页复活；1F＝bgsound NotSupportedError 子测，非页标） |
| custom-elements/registries/Element-customElementRegistry.html | 6P/5F | 8P/3F | +2P/−2F |
| custom-elements/registries/Document-customElementRegistry.html | 2P/2F | 3P/1F | +1P/−1F |
| custom-elements/registries/scoped-registry-append.html | 5P/11F | 6P/10F | +1P/−1F（walk 无槽写回增益——翻转子测「Declarative shadow DOM without shadowrootcustomelementregistry attribute … should gain effective global registry after adoption」，与写回语义精确对应） |

页复活归因：两页基线仅页级标记（子测名＝页路径形，整页 harness 死亡）；刀③ JS 面
（resolver/getter declarative 早退删除＋标记模型重写）移除崩溃路径后整页复活。复活
新可见的 23F 为既有残域子测（`define()` 校验矩阵 22F＋bgsound 1F），如实入册非回归。
T 全局 19＝基线 19 恒等（零新增停摆）。

### dom 腿（−320P 停摆形）

`Range-mutations-replaceData` 1121→986（−135）、`Range-mutations-dataChange`
1178→993（−185）——F 构成 496 恒等、T 12 恒等、其余 419 页零漂移。P 缺失形（注册
截断）＝master 碰撞记录 2026-11-23 两则已定谳的停摆族（页中途停摆杀后续 async_test
注册，停摆点逐窗漂移：本窗 986/993 vs 历史窗 948/1002/1121/1146/1152/1178）。刀域
（registries JS/DSR 印记）与 dom/ranges 页零交集，无归因路径；判读按噪声族入账。

## 证据日志（同提交 add -f）

dc2-s10k3-wc-post-knife3.log（`a1c0643b32b93a5e5486cbae68b3ea01`）/
dc2-s10k3-domparsing-post-knife3.log（`1b22956cd5c098725eda5e4dacd58207`）/
dc2-s10k3-sanitizer-post-knife3.log（`771db8e9f9533f56798abf925c2e7897`）/
dc2-s10k3-dom-post-knife3.log（`cb88f527fe3cb972cd267f7511f56665`）。
电池窗 12:53:32–13:0x（launcher `/tmp/s10k3-battery-launch.sh` md5
`a72b86fc4770f052db402c48ec8cd0f4`，四腿顺序流无并发争抢）。

## 残面（下刀挂账）

- ShadowRoot-init-declarative 3F：test1 custom-tag 元素解析 got object（resolver
  hyphen 面深域）＋ test3 setHTMLUnsafe DSR shadowRoot null（unsafe 子树 face
  链）——本刀 transfer/getter/walk 面对该页逐字恒等（基线同 3F 同构成）。
- ShadowRoot-init-customElementRegistry 残 1F：template contents 克隆宿主 null
  registry 链（模板内容文档 null 默认域，line 89-103 子测）。
- define.html 新可见 22F＝`define()` 参数校验矩阵（页面复活后入册）、
  define-customized-builtins 1F（bgsound）——下刀按序收。
- element-mutation 15F、R90 trap、parse-time attr、importNode/upgrade 页、IAH
  handle-only——沿 刀② 判读档序不变。
- iframe 合成 doc 无 legacy `customElements` 读面（本刀钉测勘误实证）。
