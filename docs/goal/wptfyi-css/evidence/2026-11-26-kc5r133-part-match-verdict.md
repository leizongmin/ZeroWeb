# kc5r133 verdict——part-match 刀：`::part(<ident-list>)` 匹配＋文档表/shadow 双通道＋树上下文轴（2026-11-26）

## 结论

- **刀域**（style-system 两文件；判据页 css/css-shadow part/ 域 53 页池）：
  1. **Face M（matcher/mod.rs 裸形双通道门）**：
     `collect_part_declarations_with_media` 增第 7 参 `bare_form_matches` 下传
     `collect_part_from_rules`（4 处递归全下传）→`part_selector_matches` 裸形门：
     shadow 树通道 bare=false——无 `:host`/宿主复合前缀即拒（css-shadow §5.4
     「::part() only matches anything when the originating element is a shadow
     host」：树内裸形 originating element 是 part 元素自身非 host，零命中）；
     文档表通道 bare=true 维持裸形放行（all-hosts 判据页形：host 全体在文档树内）。
  2. **Face L（lib.rs 1.3 段双流＋树上下文轴）**：part 段收集拆双流
     `part_matching_shadow`（bare=false）／`part_matching_doc`（bare=true，
     `view.for_document()` 文档表）——元素自身树声明（主匹配/随机展开/inline
     三段）在 part 竞争窗翻 `own_tree_ctx=1`（内树上下文），doc-part 独立展开段
     保持 ctx=0（外树）。css-cascade-5 §6.1 树上下文轴由 cascade 比较器完成：
     normal 外树胜内树（不问 specificity）、important 反转内树胜外树；
     doc-part 非空时禁 `cascade_cacheable`（doc-part 声明未入 CascadeKeyRef 键
     序列，FIXME(k134+) 键集成随嵌套树排序一并做——跨调用陈旧命中实证钉）。
  3. **伪元素早退门补臂**：`pseudo.is_some()` 空匹配早退门增
     `&& doc_part_matching.is_empty()`（45 号 M2 同型）——仅文档表
     `::part(x)::before` 页主匹配恒空但 doc 段非空，不得早返。
- **fail-stop 全程**（P2F 红线纪律实证）：run1（bin 1aeece67）P2F=2 即收账
  作废→双案归因（均真实红非 flake/伪红）→修复→bin 重建换代→r2 全绿。
  - **P2F①** part/host-stylesheet.html：断言树内裸 `::part(partp)` 不命中
    （span green 应胜）——run1 shadow 通道裸形过宽命中。→Face M 门。
  - **P2F②** part/simple-important-important.html：断言树内
    `span red!important` 胜文档表 `#c-e::part(partp) green!important`
    （§6.1 important 树序反转）——run1 part 段并入段序按 specificity 比。
    →Face L 翻转＋doc 流独立 ctx=0（首版 doc 流混入 matching 同翻 ctx=1
    树轴失效，拆流修复；二次缺陷 cascade_cache 陈旧命中由 hit_flip 钉抓出，
    缓存门修复）。
- **指纹**：`ZW-KC5R133-PART-MATCH` 四代齐（KC5R130-M／KC5R131-SHADOW-DEFS／
  KC5R132-SELECTOR-VALIDITY／KC5R133-PART-MATCH strings 各 1）。
- **电池代**：b1＝`r133-part-match-runner` 归档 md5
  cb83fa3c271b692565bf577e06b2c392（终代，07:04:29-07:08:51 三腿全 RC=0）；
  中间代 1aeece67e85f（run1，P2F=2 作废不入账）；b0＝k132 归档 bin
  `r132-selector-validity-runner`（md5 3aecc612f40e52f47f6f468fab8ac25f）。
- **三语料终判**（KEY 级差分 diff-all-k133.py，净恒等式逐 suite 验算
  DIFF-RC=0）：
  - **css-shadow（主靶）P 228 → 240 = +12**：F2P=12 **P2F=0** GONE=NEW=0。
    F2P 全在 part/ 池内（12 页各 1，all-hosts 2）：all-hosts（2）／
    complex-matching／grouping-with-checked／grouping-with-disabled／
    inner-host／invalidation-change-part-name／invalidation-complex-selector／
    multiple-parts／serialization／simple-inline／simple。F2P 12＞预测 11
    （invalidation 两页额外翻＝树上下文轴增益二进证）。
  - **css-mixins（红线）P 485 → 488 = +3**：F2P=3 **P2F=0** GONE=NEW=0。
  - **css-values（tripwire）P 6841 → 6844 = +3**：F2P=3 **P2F=0** GONE=NEW=0。
- **钉测三面**：
  - style-system `shadow_part_segment.rs` 9 钉：hit_flip（文档表裸形命中／树内
    裸形不命中 span 独胜／宿主不吃／名单零命中）＋名单交集＋ASCII 大小写＋
    checked/hover 伪类＋before 尾对齐（含**文档表专属 `::part(x)::before` 臂**
    ＝早退门补臂回归面）＋light 子零命中＋文档表 host 复合/复杂前缀＋树内
    `:host::part` 复合＋custom property 暂缓过滤＋**树上下文三轴**
    （normal 外树胜不问 specificity／important 内树反转）。
  - css-parser `kc5r133_part_parse.rs` 3 钉（解析面，本轮零 css-parser 改动
    沿用绿）。
  - engine `kc5r133_part_match_pins.rs` 4 钉：P1 `:host::part` gCS green／P2
    before_pseudo 存储＋投影（规则色 rgb(0,0,4) 脱离默认黑巧合）／P3 投影
    阴性/回退分界（first-line None／marker 槽 miss 回退）／P4 文档表跨树命中。
- **门禁**：clippy 三 crate --all-targets -D warnings 绿（v2 链 single_match
  一 lint 修复后补验；engine clippy v2 链未达本次补验过）；style lib 2740P
  0F；css-parser lib 3422P 0F；engine lib 4327P/3F＝既有红名单三钉
  （dialog_focus_s330／expando_r3046／p99s_inline_custom_property）恒对账
  零新红（4327 比 k132 基线 +1＝engine P4 新钉）。
- **并发环境账**：session 内 5+ resume 实例风暴（NOTES 仲裁件
  /tmp/zw-k133/NOTES.md 磁盘先手认链）；并发 rule_pos 半成品（兄弟实例
  MatchingDecl 第 6 元未完成态）外科剥离 22 hunks＋2 内联调用（语义不回滚，
  保全 /tmp/zw-k133/face-with-rulepos-0651.patch 可续做——对两 P2F 页冗余
  不入本刀）；姊妹 clone ZeroWeb-3 trusted-types 流 cwd 天然隔离零碰撞。
- **挂账 k134**：exportparts 跨树转发／part 段 custom property 声明恢复
  （@container unknown 域）／::first-line/::first-letter 存储面／gCS ::part
  复合伪元素读（pseudo-elements-after-part 族，21 键）／嵌套树深排序＋
  CascadeKeyRef doc-part 键集成（lib.rs FIXME）／function-shadow-container
  2 键／:has-slotted 22 键／invalidation 域剩余／part-name-idl／文档表长
  复合链形。
- 判据页 <https://drafts.csswg.org/css-shadow-1/#part>／
  <https://drafts.csswg.org/css-shadow-parts/#part>／
  <https://drafts.csswg.org/css-cascade-5/#shadow-tree>。

## 证据链

- 电池：run1 b1 三腿（06:38-06:41）bin 1aeece67 P2F=2 作废；r2 b1 三腿
  （07:04:29→07:08:51）bin cb83fa3c 全 RC=0。ARM-b1 BIN md5=cb83fa3c
  FACE 指纹 4/4（battery-k133.log）。
- 差分：/tmp/zw-k133/diff-all-k133.py（KEY 级＋part/ 池交叉核＋净恒等式
  验算）对 results-k133-b{0,1}-* 现算，DIFF-RC=0。
- 钉测运行：style `cargo test -p zero-style-system --lib` 2740P（shadow_part
  段 9 钉全绿含文档表 before 臂）；engine lib 4327P/3F 红名单恒等。
- 工件：/tmp/zw-k133/（battery-arm-k133.sh／diff-all-k133.py／gates-v2.log／
  bin-r2.log／batt-r2-launch.log／NOTES.md 仲裁件＋face-with-rulepos-0651.patch
  保全）；归档 bin .acceptance/wptfyi-css/k133/bins/r133-part-match-runner
  （cb83fa3c）；电池结果 .acceptance/wptfyi-css/k133/results-k133-b{0,1}-*。
