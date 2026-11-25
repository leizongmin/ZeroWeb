# kc5r131 verdict——shadow-defs 刀：shadow 内联样式作用域＋GCS 桥＋FnChain 链查表＋flat-tree 容器链（2026-11-26）

## 结论

- **刀域**（三面协同；判据页 function-shadow-container.html 4 Fail＋
  mixin-shadow-dom.html 3 Fail＋function-shadow-cache.html 1 Fail）：
  1. **Face A（管线）**：engine `pipeline::collect_all_shadow_scopes`＋
     `collect_shadow_inline_texts`——shadow 树内联 `<style>` 接入
     ShadowScopeStylesheets 作用域收集（scopes 全集＝外链条目＋文档 shadow
     树内联表；此前判读面只接外链条目，declarative 内联表整体缺位）。树序
     走查元素 `<style>` 过门收文本（k131 位次＝聚合 order）；嵌套 ShadowRoot
     子树整体跳过（其 `<style>` 属该层作用域）。shadow 表 parse 携文档级
     `@mixin` 聚合（css-parser `MixinDefs` 加 `Clone`，`Parser` 聚合先于
     展开——shadow 内 `@apply` 跨表命中、先 apply 后 def 形两轮式收敛）。
  2. **Face B（gCS 桥）**：`computed_style.rs`／`computed_style_cache.rs`
     三通道（静态/handle 视图/全量兜底）scopes 全集接线。shim
     part04/part05 查询域统一内容绑定（`_zwDeclarativeShadowEnsureContent`，
     part04 getElementById 分派点＋part05 `_handleQueryFirst/All`）——修
     s173 惰性镜像与 s176 内容绑定同根两查身份分裂（getElementById 先触装
     镜像、querySelector 先触绑内容 → shadowRoot.getElementById 产物
     `__zwSelector=null` → gCS 空＝mixin-shadow-dom 判读黑根因）；镜像保留
     注册面，查询域统一内容绑定。触发面收窄（childNodes 枚举宽触发撤下）。
  3. **Face C（style-system）**：
     - `CascadeOrder.scope_root` 归域（级联声明归属 shadow scope 根）；
     - `FnChain`（custom_function.rs +89 行）：@function 声明定义域链查表
       （内层屏蔽外层、链尾文档表——css-shadow-1 tree-scoped reference；
       全链未命中＝调用形保留原文 guaranteed-invalid 通道不变）；
     - `container` 简写臂（shorthand/mod.rs，css-contain-3
       §container-shorthand `<'container-name'> [ / <'container-type'> ]?`；
       斜杠形左名右型词法自识、单值形按 keyword 区分）——语料全库唯一
       `container: --cont / size` 形此前整体丢弃＝容器缺席根因一；
     - flat-tree 容器链快照（lib.rs `container_chain_snapshots`＋
       `flat_container_chain_for_slot`＋light-children 分支 swap/restore）：
       slotted 元素容器查询按 flat-tree 链求值（slot 位快照＝最近容器祖先
       push 后全量链；ShadowRoot parent=host 恰为 flat 跳边；无容器 flat
       链＝空链不回落 home tree）＝容器缺席根因二。
- **指纹**：`ZW-KC5R131-SHADOW-DEFS`（css_wire.rs `#[used]` 22B）＋
  CARRY-K130 `ZW-KC5R130-M2C-UNBOUND` 双指纹独立重核。
- **电池代**：b1＝gen27 `r131-shadow-defs-runner`（md5
  f09de3e3a99d7e2da6a908b5038751c8，撤诊断洁净重建 42.31s；判值代
  gen26＝50ab29a5869041222613a93830b4d1b9 含临时 K131DIAG 诊断码，两代
  判值逐位同——诊断码为 env 门/纯 log 无语义）；b0＝k130 归档 bin
  `r130-m2c-runner`（md5 8e1fdd1783a932a937f2169e7a39221d）洁净锚。
- **三语料终判**（judge/verify 预注册通过；KEY 级差分程序化生成）：
  - **css-mixins（主判据红线）P 481 → 485 = +4**：F2P=4 P2F=0
    GONE=NEW=0（census {Pass 481/Fail 39/Timeout 1}→{Pass 485/Fail 35/
    Timeout 1}；净恒等式 +4 == F2P − P2F == 4 − 0）。F2P 四键：
    mixin-shadow-dom.html 3 键（inside mixins／adopted stylesheets／
    outside non-adopted mixins）＋function-shadow-cache.html「Tree scope
    is respected for otherwise identical styles」。旁证真化（非 F2F 计数）：
    function-shadow-container.html「::slotted() can see inner named
    containers」两臂 Pass 形——b0 空洞通过（test_all_shadows assert 双空
    恒过）→ b1 真判值（gen25 dump 探针 P.actual="5px 5px"==
    P.expected）；container 简写＋flat-tree 链刀效由探针定谳非键翻。
  - **css-shadow（Face A 触面预核）P 49 → 95 = +46**：F2P=47 P2F=1
    GONE=NEW=0。F2P 主峰 shadow-cascade-order-001.html ×32＋
    multiple-parts ×2＋15 页单键散布（::part/::slotted 级联序、host/
    has-slotted invalidation、chaining/complex-non-matching 等）。P2F 唯一
    slotted-nested.html「Style invalidation works correctly for nested
    slots」＝**b0 空洞通过暴露**（b0 该页 slotted 规则整体不匹配、兄弟键
    dump 黑色 rgb(0,0,0) 佐证 assert_not_equals 双侧同值恒过；face 使规则
    生效暴露 shadow 树变更失效（invalidation）缺失＝从未存在能力，
    held-exposure 归档非回归）。
  - **css-values（共享通道 tripwire）P 6840 → 6841 = +1**：F2P=1 P2F=0
    GONE=NEW=0——attr-pseudo-element-originating.html「::slotted() attr()
    invalidation on the slotted element」（意外增益：Face C ::slotted
    归域通道连带）。
- **gen27 撤码复验**：K131DIAG 临时诊断码全撤（lib.rs 4 块 csws/push/
  scope/fnx＋pipeline inline-scope 块＋`INLINE_CALLER` thread_local＋
  `set_inline_caller` 8 调用点＋`FnChain::diag_probe`）→ b1 三腿复跑
  **数字逐位复现**（css-mixins 13s RC=0／css-shadow 45s RC=0／css-values
  180s RC=0；485/95/6841＋diff 形全同）。
- **钉测**：engine 常驻钉 4（js_dom_bridge_tests/kc5r131_shadow_defs_pins.rs
  新文件＋include 登记）＝D1 document defs → shadow 内 @apply（gCS
  green）＋D2 shadow 内 def 后置跨表命中＋D3 adopted 汇总 defs 可见
  （线程局部 override 通道）＋D4 scope 定义不回灌 document 聚合隔离红线
  （含阴性对照）。css-parser mixin_apply.rs 钉 +70 行（shadow defs
  聚合展开形）。
- **门禁**＝见 master.md k131 block（gen27 树终跑数字）。
- **挂账**：shadow 树变更失效域＝css-shadow 残 265F 主体＋slotted-nested
  P2F 暴露面（invalidation 独立刀评估）；css-mixins held-F 残键 m7 attr
  provenance／m8+m15 typed 元素级／m11+m11b S5 if() 门（k130 carry）；
  flat-tree 容器链快照为 compute 批内方案（增量路径单元素重算域快照
  命中依赖批缓存——跨批失效场景 FIXME 待评估）。
- 判据页 <https://drafts.csswg.org/css-mixins-1/#using-mixins>／
  <https://drafts.csswg.org/css-shadow-1/>／
  <https://drafts.csswg.org/css-contain-3/#container-shorthand>。

## 证据链

- 探针：probe-solo（gen4 单页隔离）→ gen25 dump 值定谳（P.actual=
  "5px 5px"==P.expected＝真判值，区别于 b0 空洞通过）；probe-b0-pure
  43 页纯语料 481/39 逐位同基线（shim 触发面排除）。
- 电池：battery-k131.log LEG 六腿全 RC=0；B0=8e1fdd17 B1=50ab29a5（gen26）；
  gen27 复验 reverify-k131.log 三腿 RC=0 B1=f09de3e3（FACE=1 CARRY=1）。
- 差分：KEY 级程序化（本证据数字全部由 results JSON 现算——f2p-registry
  形制）；held-F 两臂恒 Fail；registry GONE=NEW=0 全套件。
- 工件：/tmp/zw-k131/（build/probe/battery/reverify/gates 五脚本＋logs）；
  归档 bin .acceptance/wptfyi-css/k131/bins/r131-shadow-defs-runner（gen27）；
  电池结果 .acceptance/wptfyi-css/k131/results-k131-b{0,1,1v2}-*。
