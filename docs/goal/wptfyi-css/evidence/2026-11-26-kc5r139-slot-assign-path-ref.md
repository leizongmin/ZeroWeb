# kc5r139 verdict——SlotAssign 路径形引用：plain Text／sel 路径 innerHTML 双 miss 节点（2026-11-26）

## 结论

- **电池净 +2，P2F=0**：b0（k138 bin）/b1（k139 bin）双腿 css-shadow 全量名册
  90 页 362 键；F2P 2（changing-002 两键）、P2F 0、NEW/GONE 0。两腿同构
  1 Timeout（非本刀域，同位恒持）。b0 288P/73F/1T → b1 290P/71F/1T。
- **同 bin 对账恒等**：k139 b0（k138 bin md5 `595998361ab8ed1e9c67b541b2e241cc`）
  与 k138 b1 键集全等（362）＋零 status diff；换代硬证 b1
  `d4f6a87eab0374d66161bdf982cb2c74`（r139-slot-pathref-runner）。
- **solo 预验**：ONLY_SUBSTR=changing-002 精确 2 页——目标页 3P/0F（test 1 持绿
  ＋test 2/3 翻绿），functional-changing-002 tentative 持红（k137 定谳踢出域）。

## 根因链（两层，均实证）

1. **shim skipSl 断链（主因，两键同根）**：`HTMLSlotElement.assign()` 渲染镜像
   循环取 `__zwHandle || __zwSelector` 双 miss 即 `skipSl=true` 跳过**整个 slot
   同步**（`__zw_slot_assign` 不发 → 引擎手动表不落 → `:has-slotted` 恒假 →
   gCS 期望 green 得 blue）。双 miss 面＝plain 世界 Text（页面静态文本子，两域
   皆无引用）＋sel 路径 innerHTML 解析代理产物（t8r-4：sel 路径 setter 不做本地
   物化——div 是 `_zwMEl` 解析代理）。shim 侧 JS 语义面（`__zwManualAssigned`）
   无条件更新，故 `assignedNodes().length` 断言绿、gCS 断言红（b1 失败 message
   分界实证）。
2. **引擎 childNodes SR 位次（path 解析面，钉测二轮实证）**：引擎模型 ShadowRoot
   挂宿主 children（spec childNodes 不含 SR）——路径下行若按原生
   `child_nodes` 序，attachShadow 后宿主的 spec 位次整体错位（场景 2 sweep 后
   children=[SR, div]，path[0] 命中 SR）。k138 `replace_all_sweep_children`
   同款过滤先例。

## 支路考古（cutoff 疑点消解）

`__zw_get_computed_style_by_handle` 的 `style_version = mutations.len()`
（R3030 键契约）——SlotAssign 入队即换代，`ensure_handle_view_slot` 重放窗口
`[applied..count)` 全量含 SlotAssign，视图槽通路**本就健全**；blue 的真因是
SlotAssign 从未入队（skipSl），非视图陈旧。init 版假设「HANDLE_VIEW_CACHE 键
不含队列长度」被代码否证（键含 style_version＝队列长度），存疑即查、不落码。

## 实现面（两处＋钉测）

1. **shim assign 臂**（part03.js）：双 miss 节点上传路径形引用
   `"@path:<anchorRef>|<idx.chain>"`——anchor＝parentNode 链上最近可引用祖先
   （handle/sel），链＝自锚下行 childNodes 全节点序 child-index（`__zw_child_nodes`
   JSON 序口径）；链组不出（脱离树/pending-removed）才维持 skipSl 保回退内容。
   FIXME(45-M3+) plain Text 渲染面覆盖——本刀消除。
2. **engine SlotAssign arm**（js_dom_bridge.rs）：nodes 解析第四级——`@path:`
   前缀解析（anchor 走既有三级 handles→persistent→shadow 包含查询；链按 spec
   childNodes 口径**本地下行**过滤 ShadowRoot 位次）。非该前缀走既有三级零漂移；
   `walk_child_path`（t7 语义面，corpus 固化口径）不动。
3. **钉测**（shadow_forest.rs tests）：k139_slot_assign_path_ref_plain_and_
   innerhtml——plain Text assign→`:has-slotted` 真→清空回落假→innerHTML 后
   assign(div) 路径命中新子，三段全链。修复前形态（Err "no node __n9"）先证
   后修。

## 门禁账

- **engine --lib**：4350P 3F——3F＝s330/r3046/p99s 恒红名单同名同键（k127..k138
  账面延续），零新红。
- **clippy**：`-p zero-engine -p zero-dom -p zero-style-system --all-targets
  -D warnings` RC=0。
- **solo**：changing-002 3P/0F（唯一 JSON_DIR/results-solo）。
- **电池**：flock `zw-k139-battery.lock` 独占；test-guard 逐腿（--per-proc-mem 4
  --total-mem 16 --time-limit 3000，CASE_WALL_S=60）；唯一 JSON_DIR/transcript；
  LEG-EXIT rc=0×2＋BATTERY-DONE；serialize-gate 三探（无活跃 scope/cargo/裸 bash
  ok）后 setsid systemd-run --user --scope（MemoryMax=20G）detached 发射；
  键级差分 P2F=0（diff-k139.py Counter，exit 0）。

## 残 F 挂账

- **handle 岛形**（FIXME）：t8r innerHTML 物化的 handle 容器场景——shim 物化
  wrapper handle 是 detached 复制岛，引擎 SetInnerHtml(OnHandle) 不注册该
  handle → assign 上传岛 handle 三级 miss 硬错（钉测初版 `no node __n9` 实证
  形态）。完整修复属 r80 bind journal 统一切片（clone 臂已有
  `__zw_bind_handle_by_path`，innerHTML 臂缺位）；本刀 path 形只覆盖双 miss 面
  （sel/plain），岛形不在 changing-002 语料（页面宿主是 sel-form）。
- functional-changing-001..004.tentative 4 键：k137 定谳踢出刀域维持。
- 既有挂账不变：`:is(:has-slotted)` over-match FIXME；exportparts；`::part`
  盒布局 7 键（渲染流域）；marker 树位路径编码 FIXME(45-M3+)（id 缺失降级面，
  本刀 path 形是运行时通道、非序列化通道，互不替代）。
