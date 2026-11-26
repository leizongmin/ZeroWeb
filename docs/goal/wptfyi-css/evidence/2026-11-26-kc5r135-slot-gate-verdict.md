# kc5r135 verdict——slot-gate 刀：伪元素存储门扩 background＋handle 通道 shadow 地址读路径（2026-11-26）

## 结论

- **刀域**（style-system 三门＋engine 三文件＋shim 一臂；判据页
  css/css-shadow/part/pseudo-elements-after-part＋innerHTML 物化族）：
  ① **Part A** style-system lib.rs 三存储门扩 background 判据——before 门
  （~1534）/after 门（~1556）/placeholder 门（~1241）在原 color-diff 门上
  增 `!matches!(X.background_color, ColorValue::Transparent)` 值探针臂；
  **不能用 elem_style 比较**：background 非 inherited，R5052 UA 表单默认
  误存同型陷阱（selection 门 R5065 Transparent 探针先例）。
  ② **Part B** engine handle 读路径——shim part01.js gCS handle 臂随读
  上传单跳 shadow 地址 `"<rootHandle>:<elemOrdinal>"`（s45-M1 focus 铸址
  同源寻址原语：`_zwNodeParent`/`_shadowHandles`/`_handleChildren` 三表；
  伪元素无参读形空串占位第三参）＋callbacks.rs
  `__zw_get_computed_style_by_handle` 第 4 参透传＋computed_style_cache.rs
  `computed_style_by_handle` 新参 `shadow_addr: Option<&str>`＝解析
  **最优先于 `nodes.get`**（双子假源：`__zw_create_element` 的
  CreateElement mutation 使视图 nodes 表命中 detached 双子自身＝"" 假源
  短路——实施中途改形，初版 addr 放 nodes.get miss 臂被双子命中）。
  ③ 指纹 `ZW-KC5R135-SLOT-GATE`（css_wire.rs `[u8;20]`）。
- **根因链**（k134 归因半误订正）：k134 verdict 记「handle 通道已通但槽
  None 回落本体 bg=""」——实测半误。真实缺口＝shim innerHTML 物化走
  `_wcRebuildAsHandle` detached 双子（part04.js ~16551-16620，注释自认与
  宿主树 handle 不同源）→ 真树内元素无 handle 绑定 →
  `computed_style_by_handle` `nodes.get` miss → forward 表无条目 →
  **元素级读也 ""**（diag T3 探针证）。k134 仅修 cloneNode 产物族
  （interaction 4＋part-mutation 1）；innerHTML 族元素级＋伪元素级读全 miss。
- **电池**（b0=k134 归档 bin c097b644／b1=新代 505036c1，指纹 strings=1；
  锁键 zw-k135-battery.lock；六腿全 RC=0；POST MD5 与发射前恒等）：
  **css-shadow P 245→259＝F2P=14 P2F=0 NEW=0 GONE=0（键级 362 恒等）**
  ——其中 13 键确定性 Fail→Pass＋1 键 NotRun→Pass
  （css-scoping-shadow-dynamic-remove-style-detached；双臂单页 solo 探针
  定谳确定性：b0 bin 单页复现 NotRun／b1 bin 单页 Pass＝face 跑全性修复
  入账）＋css-mixins 488→488 F2P=0 P2F=0＋css-values 6844→6844 F2P=0
  P2F=0＝**净 +14 红线成立**。invalidation-part-pseudo 双臂
  Timeout→Timeout 稳定＝边界噪声不入账。
- **F2P 分解**（14 键＝预测 4＋超预期 10；13 Fail→Pass＋1 NotRun→Pass）：
  - pseudo-elements-after-part.html 4 键（::before/::after/::placeholder/
    ::selection backgroundColor computed style）＝A+B 复合
    （selection 槽 k134 已存仅需 B；before/after/placeholder 需 A 存储＋B 读）。
  - grouping-with-checked 3＋grouping-with-disabled 3＋host-part-001 1＋
    host-part-nesting 1＋style-sharing 1＝**仅 Part B**——innerHTML 物化族
    元素级 gCS 读整体翻绿（定谳预测仅 4 键，电池实得 13：addr 读路径
    修复面＝innerHTML 族整体，非 slot 族专属）。三页采样程序化核
    （attachShadow＋.innerHTML＋getComputedStyle 形）。
  - css-scoping-shadow-dynamic-remove-style-detached.html 1 键
    （NotRun→Pass）：双臂单页 solo 探针（/tmp/zw-k135/probe-nr＋
    probe-nr-b1）定谳确定性——b0 bin 单页复现 NotRun（测试不报告）／
    b1 bin 单页 Pass（断言全过）＝face 跑全性修复，非 flake
    （具体通道未深究）。
- **钉测**：style-system kc5r135_slot_gate 5 钉（before_bg_only_slot／
  after_bg_only_slot／placeholder_bg_only_slot／bg_no_rule_no_misstore
  span 自身 bg 规则＋仅 ::after 规则→before 不误存／color_only_undrifted）
  ＋engine kc5r135_slot_gate_pins 4 钉（E1 innerHTML 子 ::selection bg 读
  rgb(4,5,6)／E2 ::before bg 读 rgb(1,2,3)＝A+B 复合端到端／E3 元素级读
  rgba(0,0,0,0)／E4 无地址 plain 子零漂移）＋k134 4 钉同域复验绿
  ＋style-system 全量 2745P 0F。
- **门禁**：clippy zero-style-system＋zero-engine -D warnings 绿；engine
  lib 红名单三键 s330/r3046/p99s 恒对账零新红。
- **残 F 挂账 k136+**：异种槽位 10 键（backdrop/details-content/
  file-selector-button/grammar-error/highlight/search-text/spelling-error/
  target-text 等）＋interaction first-line/first-letter 2 键（helper
  shadow_view plumbing）＋布局 7 键（::part 伪元素盒 rect）＋深链
  FIXME(k135+)（单跳地址→嵌套 shadow 逐跳扩链）＋exportparts 跨树转发
  ＋:has-slotted 22 键。
- 电池工件：`.acceptance/wptfyi-css/k135/results-{b0,b1}-{css-shadow,
  css-mixins,css-values}`；差分脚本 /tmp/zw-k135/diff-k135.py（键级
  Counter，status 判读 lower().startswith）；运行注记 /tmp/zw-k135/NOTES.md。
