# kc5r134 verdict——pseudo-handle 刀：gCS 第二参 handle 通道透传（2026-11-26）

## 结论

- **刀域**（engine 三文件＋shim 一臂；判据页 css/css-shadow part/
  pseudo-elements-after-part＋interaction-with-pseudo-elements）：
  k133 selector 通道第二参透传的 **handle 臂镜像**——动态元素
  （createElement 产物，handle 身份）伪元素读形此前在 shim handle 臂丢弃
  第二参＝恒本体值。runner 判据页两族全动态形（attachShadow/innerHTML/
  customElements.define），电池 27 键全 F 的根因：
  1. **shim part01.js handle 臂**：`__zw_get_computed_style_by_handle(hdl, prop)`
     → `pseudoElt` 非空透传第三参（五槽名单与 selector 通道同形）。
  2. **host callbacks.rs handle 回调**：第三参归一（`::`/`:` 剥前缀小写）→
     五槽名单分派 `pseudo_elt`（与 selector 回调 k133 臂同归一同名单）。
  3. **computed_style_cache.rs `computed_style_by_handle`**：新参
     `pseudo: Option<&str>`——Some 时在 `styles.get(&nid)` 臂头**直接投影**
     `serialize_pseudo_element_property(...).unwrap_or_default()`（元素级
     consult/echo/transition 登记通道全跳过，同 selector 通道 k133 口径）；
     纯读不进视图键。
- **名单不扩**（k133 五槽 before/after/marker/placeholder/selection 持）：
  `::first-line`/`::first-letter` 维持 shim 名单外→本体回落——values 域
  attr-pseudo-elem-invalidation/random-computed 第三参读形键依赖此宽容面
  （k133 收账注释明言）；扩名单＝values 域 P2F 风险。且 style-system
  compute_first_line_pseudo/compute_first_letter_pseudo 的探测与完整级联
  `shadow_view=None` 硬编码——part 通道结构性缺席（k135 挂账：helper
  shadow_view plumbing＋part-tail 收集）。
- **电池**（b0=k133 归档 bin cb83fa3c／b1=新代 c097b644，指纹
  ZW-KC5R134-PSEUDO-HANDLE strings=1＋carry K133/K132 各 1；锁键
  zw-k134-battery.lock；两臂全 RC=0）：
  **css-shadow P 240→245＝F2P=5 P2F=0**（POOL 内 5/5，POOL 外 0）＋
  css-mixins 488→488＝F2P=0 P2F=0＋css-values 6844→6844＝F2P=0 P2F=0＝
  **净 +5 红线成立**。
  - F2P① part/interaction-with-pseudo-elements.html 4 键
    （::before/::after/::placeholder/::selection in selected host is styled）：
    expected green got rgb(0,0,0)→投影槽值 green（shadow 内红常规规则经
    k133 树上下文轴落败不变）。
  - F2P② part/part-mutation-pseudo.html 1 键（Part in selected host changed
    color）：`setAttribute("part", ...)` 前后 `gCS(part, '::before')` 读形
    assert_not_equals——修前两读恒本体值恒等；修后读形投影随 part 匹配
    变化（真实修复尾，非侥幸：断言语义即依赖投影通道）。
- **残 F 域归因**（b1 JSON 程序化核，k135+ 挂账）：
  - pseudo-elements-after-part computed-style 14 键全持 F：①
    after/before/placeholder/selection 4 键＝**slot 存储 gate 未翻**——
    before/after 存储 gate 要求 content ∈ String/Attr/Counter/Counters/
    List/Url（该页规则 background-only 无 content）；placeholder gate＝
    color 差异（同 background-only 不存）——handle 通道已通但槽 None 回落
    本体 bg=""（got "" 形不变）；② backdrop/details-content/file-selector-
    button/first-letter/first-line/grammar-error/highlight/search-text/
    spelling-error/target-text 10 键＝异种伪元素无存储槽（k133+ 挂账）。
  - interaction first-line/first-letter 2 键持 F（got 本体色＝名单外正确
    回落形，非 ""）＝helper shadow_view plumbing 缺席。
  - 布局 7 键持 F（::part 伪元素盒 getBoundingClientRect）＝布局域另刀。
- **钉测**（engine kc5r134_pseudo_handle_pins 4 钉全绿；k133 4 钉＋G1b
  handle echo 零漂移）：K1 handle 通道五槽投影（动态 div＋doc 表
  `div::before` → 槽值 rgb(1,2,3)；视图级联含伪元素槽计算实证）＋K1b
  本体读零漂移（不带第二参恒默认）＋K2 静态 part 五槽 selector 通道 shim
  级端到端（doc 表 `::part(ab-p)::after` 胜 shadow 内常规规则——k133 钉
  为直接 API 形，本钉补 shim 白名单→host 归一→投影接线）＋K3 槽 miss
  宽容面 shim 级（`::marker` 非 li 槽 None→originating 元素自身值）。
- **门禁**：clippy zero-engine -D warnings 绿；engine lib 全量 4337P/3F＝
  红名单三键 dialog_focus_s330/expando_r3046/p99s_inline_custom_property
  恒对账零新红；三探针（直链 rlib 存储面／沙箱 identity/forward-map 链）
  定谳缺口在 handle 臂不在存储面（静态 declarative 形探针即绿）。
- **裸沙箱边界注**：shadow 树内动态 part 元素的 handle→forward-map→
  shadow 查找链沙箱级不可复现（view 物化不含生产 drain 面，探针 4 元素
  本体读恒 ""）——该链由电池判读（interaction 4 键翻即生产链通）；钉测
  K1 以 light-DOM 动态元素等效覆盖 handle 臂＋host 分派＋投影三段。

## 证据

- 电池 log：/tmp/zw-k134/battery-k134.log（ARM-b0/b1-DONE rc_all=0）
- diff 判读：/tmp/zw-k134/diff-k134.txt（净恒等式逐 suite 验算过）
- 判据页：tests/wpt-runner/wpt-data/css/css-shadow/part/pseudo-elements-after-part.html（44 键）、interaction-with-pseudo-elements.html（6 键）、part-mutation-pseudo.html
- 探针：/tmp/zw-k134/k134-probe{,2,3,4}.rs（存储面绿／handle 通道死／identity 定谳／forward-map 链边界）
- bins：.acceptance/wptfyi-css/k134/bins/r134-pseudo-handle-runner（md5 c097b64469e4b49b2343c6df0fccd33d）
- results：.acceptance/wptfyi-css/k134/results-k134-{b0,b1}-{css-shadow,css-mixins,css-values}
