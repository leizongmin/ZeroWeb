# M4 切片 R49 — observe options 校验/隐含启用 + textContent childList 语义 + 注册文本节点可编辑

**日期**: 2026-08-15
**里程碑**: M4 / DC-3（nodes MutationObserver）
**证据**: [../evidence/2026-08-15-r49-mo-options-and-textcontent.json](../evidence/2026-08-15-r49-mo-options-and-textcontent.json)

## 修复五件

1. **observe options 校验**（spec `dom-mutationobserver-observe`）：childList/attributes/characterData 全缺抛 TypeError；attributeOldValue=true 而 attributes!==true 抛；attributeFilter 存在而 attributes!==true 抛
2. **隐含启用**：attributeOldValue/attributeFilter/characterDataOldValue **存在**（非 undefined）即隐含对应观测
3. **filter 不提供 oldValue**：oldValue 仅 attributeOldValue===true 时提供（`_mo_obs_wants_attr_old` 收紧——filter 只筛 record）
4. **textContent= spec 语义**：同值 no-op（**写前**读比较 + **本地注册文本优先**——两轮回归修正）；异值发 childList（removed=旧子 + added=[新文本节点]）**不发 characterData**（R3027 pragmatic 移除）；firstChild/lastChild 消费 `_zwLocalChildNodes`（childNodes 原已接）
5. **注册文本节点可编辑**：`_zwRegisterTextEl` 节点补 data/nodeValue setter + 五方法（写经 `__zw_set_child_text` 索引 0 + 经新全局 hook `__zw_mo_notify_text` 发 characterData record，target=文本节点）

### host 侧配套

- SetChildText apply：child_index 越界 fallback 父 set_text_content；加入 `rewrite_pending_id_selectors` 追链集

## 同轮回归（两起，均 probe 定位）

1. 同值 gate 读在 host 写**后**（latest-wins 已含新值恒同值→注册被跳→firstChild 复 null）→ 移到写前
2. 本地编辑后同值误判（data= 只改本地+pending，元素级 lw 读不到）→ gate 本地注册文本优先

## 遗留测试更新（R3025/R3027/R3028）

旧测试锁 R3027 pragmatic 语义（filter 隐含 oldValue / textContent 发 characterData）——按 R49 spec 语义更新（childList records / null oldValue / characterData-only observer 收 0）。

## 结果

| 用例 | 前 | 后 |
|------|-----|-----|
| MutationObserver-sanity | 11P/4F | **16P/0F（100%）** |
| MutationObserver-takeRecords | 1P/2F | **3P/0F（100%）** |
| MutationObserver-attributes | 38P/0F | **40P/0F（100%，+2 filter）** |
| MutationObserver-childList | 15P/1F | **18P/1F（+3）** |
| dom/nodes polyfill | 2566P | **2579P（+13）** / native 2549P |

零回归：events 189 / collections 24 / traversal 9 / ranges 39 / classlist 1420 / textContent 17。

**R45–R49 MutationObserver 族五轮累计**：attributes 40P/0F、characterData 19P/0F、sanity 16P/0F、takeRecords 3P/0F（**四个 100%**）、childList 18P/1F（遗留=同步兄弟链 M1 L2）；dom/nodes **2507→2579（+72）**。

## 验证门禁

- 单测 `test_mo_observe_options_and_textcontent_records_r49` + 3 个遗留测试更新
- engine v8 2131 / quickjs 1415 全绿；quickjs 矩阵 14 crate 全绿
- clippy 双矩阵零警告，fmt 无 diff
