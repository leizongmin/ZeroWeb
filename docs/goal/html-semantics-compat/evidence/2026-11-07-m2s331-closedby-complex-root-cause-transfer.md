# M2 s331：dialog-popover-closedby-complex 18F 勘根——根因在 shadow 布局域，移交 shadow-dom-rendering

日期：2026-11-07 ｜ 承接：s330（`2026-11-07-m2s330-dialog-focus-restore-declarative-shadow.md`）

## 目标与定谳

interactive 域最大残红簇 `dialog-popover-closedby-complex.html` 18F（shadow-slots 变体全簇；normal-DOM 变体 20/20 全 P）。勘根定谳：**light-dismiss 判定逻辑零缺陷，根因在 declarative shadow 内容的布局盒退化——渲染管线域，移交 shadow-dom-rendering goal（M3 布局/绘制消费面）**。

## 勘根链（三步实证）

1. **失败面析**（`logs/s330-interactive-r1.log`）：18F 全部是 shadow-slots 变体；失败消息模式 = 点击 popoverB 连 dialogB 一起关、点击 popoverA 连自身关——即「点击被分类为全在外」的 dismiss-all 行为。
2. **逻辑隔离探针**（临时 zz-s331-probe.html，用后已删；`logs/s331-probe1.log`）：以正确目标 sel 直驱 `__zw_dialog_light_dismiss_check/_up` 双腿——target=pB 自身坐标 → 四层全保持（1/1/1/1 ✓）；target=dB → 只关 pB（1/1/1/0 ✓）。s176/R5234 flat-tree 包含 + `_zwLdResolveTarget` compound 地址解析对本案**完全正确**。
3. **布局探针**（settle 100ms 后读盒；`logs/s331-probe2.log`）：shadow 栈四元素盒全退化——dA=(8,100,**0×0**)、pA=(8,100,**0×0**)、dB=(8,200,62×36)、pB=(8,200,62×36)。spec 期待四层分离盒（fixed top:100/150/200/250 + dialog/popover UA 水平居中 x≈369）；实际塌成 (8,200) 单盒、左锚 x=8、dA/pA 零尺寸。qSA `body, body *` 只枚举 light 元素（8 个，无 shadow 内容），runner 扫描对栈区域点击 best=null → msel/空回落 → dispatch body → dismiss-all，与 18F 消息模式逐一吻合（`logs/s331-closedby-debug.log` [ptr-resolve] 全程轨迹）。

## 判域与移交

- **主根因**（shadow 栈布局盒退化）= layout-engine/渲染管线对 declarative shadow 内容的布局面——`docs/goal/archive/shadow-dom-rendering` M3（slotted 内容进布局树与绘制输出）章程内。该 goal M0 对账已记布局扁平化 2026-05-31 落地但存在 R1-R6 六缺口；本案为其 testharness 侧实证面。**修复后 `dialog-popover-closedby-complex.html` 可作其验收语料**（normal-DOM 变体全 P 证明 light-dismiss 逻辑面零阻塞）。
- **从属缺陷 D1**（本 goal 挂账）：`resolve_pointer_target`（tests/wpt-runner/src/testharness.rs:13291）JS 扫描输出协议用 `|` 分隔 sel|px|py——compound zwsh 地址（`zwsh<len>:<host>|<local>`）与 msel 回落串都含 `|`，split 后坐标 parse 失败 → (origin, off) 回落。修法：分隔符换 `\u{1f}`（JS 字面 `\u001f` + Rust `split('\u{1f}')`，协议仅本函数内消费）。当前无独立可动面（根因在布局），布局落地后成为 closedby-complex 收口的必要前置——届时随收口切片同改同验。
- **从属面 2**：modal 组合的 `_zwDialogModalFocusGateOk` 包含判定走非 flat 链（s330 已记最小爆炸半径不动）——slot 分配内容在模态 inert 门被误分类，modal 半簇（9F）叠加此面。

## 验证

| 面 | 方式 | 结果 |
|---|---|---|
| light-dismiss 逻辑 | 探针直驱双腿（正确 sel） | ① click pB → 1/1/1/1 ✓ ② click dB → 1/1/1/0 ✓——逻辑零缺陷 |
| shadow 栈盒 | settle 后 gBCR 全量 | dA/pA 0×0、dB/pB 塌 (8,200) 同盒、无水平居中——布局退化实证 |
| runner 扫描行为 | ZW_TD_DEBUG=1 全程轨迹 | 栈区域点击 best=null → msel compound/空回落 → body dispatch → dismiss-all，与 18F 逐一对应 |
| normal-DOM 变体 | 既有 P | 20/20 全 P——非 shadow 路径零回归面 |

日志：`logs/s331-probe1.log`、`logs/s331-probe2.log`、`logs/s331-closedby-debug.log`（ZW_TD_DEBUG 全程；gitignored）。

## 挂账与队列

- `dialog-popover-closedby-complex` 18F → **shadow-dom-rendering goal**（布局根因；本 goal 待其 M3 落地后回收验收，届时补 D1 协议修）。
- 下一候选（本 goal 可动面）：audio_constructor 1F、audio loading=lazy 10F、object width/height rendered-dim IDL、canvas 残红 3F+1T（承 s329）、CSP style-element 8T、microtasks 4F+2T。
