# 刀㊵ 判读（s10k40）——insertAdjacentText 同 turn sel-null no-op 吞插入＝TT script 源文本 bypass 簇

日期：2026-11-26 · 域：trusted-types / Element.insertAdjacentText · 树锚：本刀 patch `knife40-final.patch` md5 `80bae3fbc919458c5a1a47195474b7cb`

## 目标簇（池内入刀前）

- `block-text-node-insertion-into-script-element.html` 3F：insertAdjacentText bypass ×2（initial comment／textContent）＋module 面 1F。
- `block-text-node-insertion-into-svg-script-element.html` 2F：insertAdjacentText bypass ×1（textContent svg:script）＋module 面 1F。

## 根因与修复（1 文件，码刀见 commit）

### part04 R293 `insertAdjacentText` 同步子视图脱 sel 门

- TT 对 classic script 源文本的 enforcement 在 **prepare/exec 时**（`_zwTtScriptExecGate`，part14 §6.3）——文本照插 DOM、script 按信任快照缺位拦跑（violation 面）。exec-time 模型本刀零改动（appendChild 文本族三面已绿为证）。
- 失败面根因＝**同 turn sel-null 吞插入**：语料形 `s.appendChild(p); container.appendChild(s); p.insertAdjacentText("beforebegin", …)` 全在同一同步 turn（checkMessage 的 fn() 于 promise executor 内直呼）——ref 元素 p 的 host apply 未轮转 → `sel=null` → R293 臂 `if (sel && …)` 门把宿主 `__zw_insert_adjacent_text` 调用与**同步子视图插入**一并跳过 → 零插入零抛（探针 B3 childNodes 不变；`s.text.includes("postMessage")` 断言假）。
- 修复：同步子视图块**脱门**——`p293.insertBefore(tn293, ref293)`／`self293.insertBefore/appendChild` 的 pending overlay 路径对 handle 元素自记账（探针 C1 手动同形调用实证 `s.text` 即见、宿主侧经该路径自身 mutation 出线 apply），仅宿主直呼保留 sel 门。WPT dom 钉形页（query 后调用＝sel 已赋）路径零变化。
- 语义验证（探针）：文本落位 `s.text="postMessage(…)"` ＋ script 未执行（`ran=[]`，exec gate 拦跑）＝TT 拦截语义完整（文本在、代码不跑）。

## 判读

### solo 页级（post 树）

| 页 | base | post | rc |
|---|---|---|---|
| block-text-node-insertion-into-script-element | 7P/3F | **9P/1F**（余 1F＝module 面） | 2（留痕） |
| block-text-node-insertion-into-svg-script-element | 6P/2F | **8P/1F**（余 1F＝module 面） | 2（留痕） |
| dom/nodes/Element-insertAdjacentText.html（回归钉） | — | **6P/0F** | 0 |

### 电池四腿＋TT 同树 A/B（字节绑定 `80bae3fb…`，setsid pid 4170710）

四腿＋TT A/B 全数落定（driver `/tmp/s10k40/driver-k40.log`，全程 14:29:34→15:03:04Z＝33min）：

| 腿 | k39 基线（HEAD 树） | k40 post | 判 |
|---|---|---|---|
| san | 713P/65F | 713P/65F | **SET-IDENTICAL** |
| dp | 1314P/221F/1T | 1314P/221F/1T | **SET-IDENTICAL** |
| wc | 17674P/920F/20T | 17653P/920F/20T | F/T multiset **恒等**；P −21＝gethtml 截断抖动已知噪声页 |
| dom | 57440P/362F/12T | 57525P/362F/12T | F/T multiset **恒等**（GONE=0 NEW=0）；P +85＝注册深度抖动族——insertAdjacentText 脱门共享路径零回归 |
| TT base | — | 2194P/66F/20T | P/T 与 k39 post 同带；F +1＝**zz-knife40-probe 探针页**（发射前未删，见事故记）非 flake |
| TT post | base 2194P/66F/20T | **2198P/62F/20T** | **+4P/−4F，NEW=0**，Timeout 20 恒等 |

GONE 4 键（全真实页）：block-text-insertion html ×2＋svg ×1（目标簇）＋**script-enforcement-002 ×1 红利**（`Default policy's calls when setting script source via Node.insertAdjacentText()`＝同 turn sel-null 同根因）。三方对账：k39-post 65F（真实 58＋旧探针 7）→ k40-post 62F（真实 **54**＋旧探针 7＋k40 探针 1）＝真实 −4。

**TT residual 池真实 58F → 54F。累计 84,236 → 84,240（距 ≥99,947 差 15,707）**。

## 事故记

- 本轮电池发射前漏删 `zz-knife40-probe.html`（k39 轮先删后射）→ base 腿 F 66 vs k39 post 65 出现 +1 漂移；三方 multiset 对账定位为探针页自身 failing-assert 载体非回归，发射后已删。教训入账：电池发射清单须含「探针页已删」检查位。

## module 面（2F）归新切片

`Test that default policy applies to module script`（html/svg 两页）＝module script 执行管线缺位（text 子节点永不执行 → count 0）——与 006/007/008/009 四页 12F 同族（module 执行／importmap registry／动态 import() 三缺，page-script 管线无此三面），**新子系统切片立项评估**面累计 14F，非 TT sink 残刀续磨位。
