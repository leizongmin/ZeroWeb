# M2 切片 38 — 挂账：appendChild move 后兄弟读 stale + pending 视图插位（2026-10-23）

## 范围

s32 起挂账（dialog 集群）：focus-after-close.html「moved in between」案
红在 `assert_not_equals`（appendChild 移位检查）——比焦点恢复更底层。
探针钉定：`body.appendChild(input)` 真移了（parent/lastElementChild 都
新），但 `input.nextElementSibling` 仍读旧位置的 dialog。

## 探针钉定（zz-probe-s37-append.html，验证后已删）

- R334 appendChild/insertBefore 的结构 wire **异步 drain**，同 turn
  `__zw_element_siblings` 读 host 旧树——兄弟读无 pending 补偿
  （parentNode 有槽位补偿、兄弟没有）。
- 两遍调用序列探针进一步钉出**共享视图插位缺陷**：appendChild 后
  cleanup `insertBefore(input, dialog)`，body 融合视图成
  [dialog, input, script]，真树是 [input, dialog, script]——R182 重定位
  `pos` 在旧位剔除**前**计算，剔除后索引左移未同步，`splice` 把节点插
  到 ref 后一位。
- 「moved to shadow root」案：sel 子挂 shadow 容器时上一子测试残留的
  `_zwSelPendingParent` 槽（parentSel=body）未覆写——读链命中 stale 槽。

## spec 钉定

- nextElementSibling/previousElementSibling
  （dom-nondocumenttypechildnode-*）：兄弟读按当前树；pre-insert 步骤 4
  「remove」同步生效——appendChild/insertBefore 后同 turn 读须反映新位
  （语料首断言 `input.nextElementSibling === dialog` 与移位后
  `!== dialog` 两面都依赖此语义）。
- insertBefore(node, ref) 移动语义 = remove(node) + insert before ref
  （concept-node-pre-insert）——cleanup 恢复原序。

## 落地（R5240，三刀）

1. **part04 兄弟 trap pending gate**：sel 元素兄弟读在 host 直读前查
   ①`_zwSelPendingParent` 槽（parentSel 非空）→ 新父融合元素子列表定位
   （R317 视图源，R182 重定位语义）；②容器记账 `_zwAriaSyncParent[sel]`
   （R5009 片 e）→ 容器 registry 元素定位。self 识别三重：proxy
   identity / `__zwPendReselGet` 登记 / sel 串匹配（无 handle 门）。
2. **part05 overlay R182 插位修正（共享路径）**：旧位剔除发生在 `pos`
   计算后，剔除索引 < pos 时 `pos--` 同步——修共享融合视图，全部
   childNodes/children/firstElementChild 消费面受益。
3. **part04 容器挂载路径槽覆写**：sel 子挂 handle 容器时同步写
   `{parentSel: null, parentHandle, nextSibling: null}` dual-form 槽
   （R97 noWire 同款形状）——stale parentSel 槽清除；parentSel 空值时
   既有槽消费面（parentNode getter/overlay/in-doc 门）回落 host 读，
   行为不变。

## 集群验证

- focus-after-close.html 7 subtests：3F+1T → **2F+1T**。「moved in
  between」案全绿（GAIN 1）。
- interactive-elements 全域 98 files 482 subtests：415P/65F/2T →
  **416P/64F/2T**，对 s37 账本逐 key diff **REG 0 / GAIN 1**。
- forms 域 401 files **4627 subtests 3899P/707F/20T/1U 对 s36b 基线逐
  位一致**（REG 0 / GAIN 0）——共享视图插位修正零回归
  （数据 [2026-10-23-m2s38-pending-move-sibling.json](2026-10-23-m2s38-pending-move-sibling.json)）。

## 已知边界

- 「moved to shadow root」案两处兄弟断言已过，红点推进到
  `shadowRoot.activeElement`（focus-in-shadow 面：ShadowRoot.activeElement
  供给 + focusing steps 跨 shadow 恢复）——与 dialog-focus-previous-outside
  3F（`host.shadowRoot` null）同族的跨流缺口，记挂账待后续切片。
- 「Complex dialog usage」案 focus 恢复红（activeElement 得 div）——
  dialog focus 恢复深层面，既有。
- 文件级 Timeout（s32 起既有挂账）不变。

## 门禁

zero-engine lib **3430P/0F**（基线 3429P +1 为渲染流新增）；clippy
`-D warnings`、fmt clean；零新增依赖。runner release 二进制重建。
R5240 撞号检查通过。
