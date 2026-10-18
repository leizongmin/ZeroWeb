# M2 收口 — 构造器/DataTransfer 面七簇全绿（2026-10-18）

## 运行方式

- 通道与 M1 基线一致：`make testharness-dnd`（test-guard 包裹，`--per-proc-mem 4
  --total-mem 8` 墙钟 900s；`--json` 落账）。分母不变：11 可执行 testharness 案
  / 387 subtest（skip 域见 [M1 基线](2026-10-18-m1-baseline.md) census）。
- 原始账本：[2026-10-18-m2-convergence.json](2026-10-18-m2-convergence.json)。

## 总账：213/387（55.0%）→ **387/387（100.0%）**，净 +174，零回归

| 案 | 子测试 | Pass | Fail |
|---|---|---|---|
| datastore/datatransfer-constructor-001.html | 1 | 1 | 0 |
| datastore/datatransfer-types.html | 5 | 5 | 0 |
| datastore/datatransferitemlist-remove.html | 2 | 2 | 0 |
| dom/draggable.html | 27 | 27 | 0 |
| dom/events.html | 7 | 7 | 0 |
| dom/specials.html | 21 | 21 | 0 |
| historical.html | 1 | 1 | 0 |
| synthetic/001.html | 16 | 16 | 0 |
| the-draggable-attribute/draggable-enumerated-ascii-case-insensitive.html | 1 | 1 | 0 |
| the-draggable-attribute/draggable_attribute.html | 302 | 302 | 0 |
| the-dropzone-attribute/dropzone_attribute.html | 4 | 4 | 0 |
| **合计** | **387** | **387** | **0** |

## 七簇处置（对应 M1 基线聚类）

1. **draggable 反射不在原型链**（157 fail）：三层修复——① part04 get trap
   R2848 分支补 dropzone 反射串列；② part05 element proxy has trap 白名单补
   `draggable`/`dropzone`（`in`/assert_inherits 探测面——has 白名单缺列使恒
   false，media-elements 切片 3 同款根因）；③ part03 HTMLElement.prototype 补
   draggable/dropzone IDL accessor（**plain 世界**元素——_zwMEl 直装对象与
   standalone canvas 无 trap，属性读走原型链；trap 世界 proxy 的 get/set trap
   先于原型命中，accessor 为兜底供给，双世界同源语义）。
2. **video 默认 draggable**（draggable_attribute.html video 案）：part01
   `_defaultDraggable` 去掉 AUDIO/VIDEO（初版 R3188 误并入）——spec dnd 段仅
   img 与带 href 的 a 默认可拖拽。
3. **DataTransfer types FrozenArray 语义**（5 fail）：part05 DataTransfer 改版
   ——**有序 item 列表（`_dt_items`）为唯一事实源**（旧 format map + files 数组
   双源在 string/file 混插时失插入序）；store 版本号 `_dt_storeVer` 缓存
   FrozenArray（未变返同一对象，item list 任何变更换新；缺失 format 的
   clearData/越界 remove/空列表 clear/add 抛 NotSupportedError 均不变）；
   `Object.freeze` 供 `Object.isFrozen` 面。
4. **DataTransferItem disabled mode**（1 fail）：`items.remove(i)` 后被捕获的
   item 对象原位清空（kind/type 空串、getAsFile null、getAsString no-op）——
   item 实例稳定驻留 store，不再每次 access 重建。
5. **element 上 `'ondrag*' in el'`**（7 fail）：has trap 白名单补七名
   （ondragstart/ondrag/ondragover/ondragenter/ondragleave/ondrop/ondragend，
   已知事件类型清单式——不认任意 on* 串，`ondragexit` 保持不可见 =
   historical.html 负对照）。
6. **`HTMLDocument` 未定义**（1 fail）：part03 补 legacy 别名
   `HTMLDocument = Document`（现代浏览器同一构造器；historical.html 取
   `HTMLDocument.prototype` 断言无 ondragexit）。
7. **DragEvent init dict dataTransfer 校验**（1 fail）+ **dropzone 反射
   IDL**（2 fail）：DragEvent 构造器挂外层 wrapper（R109 UIEvent wrapper 先例
   ——`dataTransfer` 显式非 null/undefined 且非 DataTransfer → TypeError；**普
   通构造去 this 载体裸调内层**——直接 apply 会因 prototype 重挂误入 super 分
   支产无 `_makeEvent` 注册对象，dispatchEvent 报 not-of-type-Event，synthetic
   001 四案实证）；dropzone 走 ① 的反射列 + set trap 字符串写（null → "null"）
   + setAttribute/removeAttribute 同步失效 IDL setter 缓存（R5009 片 e 同款
   stale 面——`dropzone=null` 后 setAttribute 读回缓存残值 "null"）。

## 门禁

- `cargo test -p zero-engine js_dom_bridge`：1155 passed / 0 failed（in-tree
  WAB2-M2-s3 / R2948 DataTransfer 语义面保绿——clearData() 保 files、
  items.clear 全清、types join、live 视图、getAsString、add 回写 getData）。
  中途复验：R3046 expando for-in 曾因原型 accessor enumerable 破坏——改非
  enumerable 后恢复（for-in 面 zero change；corpus 仅 assert_inherits 原型
  探测不依赖枚举）。
- `make test`：全腿绿（fmt 门禁前置 + workspace v8/quickjs 组合 + renderer/
  browser 各专项腿）；`make guarded-clippy` 干净；`cargo fmt --all --check` 干净。
- `make reftest`：712/712（Layout 508/508 + Text 204/204）零回归。
- execCommand copy/cut 剪贴板载荷消费面随事实源改版同步迁移
  （part06 `__zwClipboardStoreWrite` 自 item 列表导出，形状不变）。

## M3 门控面挂账（不变）

拖放交互管线（mousedown→dragstart→dragenter/over/leave→drop→dragend 状态机、
drop target 命中、js-file 案的 test_driver.Actions 指针链）——host drag-input
管道依赖，web-api-batch2 同款挂账；**未经用户点名不实现**（DC-3）。
