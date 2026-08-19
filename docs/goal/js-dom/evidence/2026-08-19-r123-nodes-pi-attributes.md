# R123 — M4 nodes：ProcessingInstruction 属性层（PI-attributes 7P/133F→140P/0F 全 100%，+133 净）

**日期**: 2026-08-19
**里程碑**: M4（WPT dom 上游基线建立与扩展）
**驱动用例**: `dom/nodes/processing-instruction-attributes.html`（7P/133F→140P/0F）
**规范**: https://github.com/WICG/declarative-partial-updates（PI 带属性接口——Chrome
「Parse processing instructions in HTML」同期演进）

## 结果摘要

| 路径 | 前（R122） | 后 | 净 |
|------|----|----|----|
| polyfill nodes 全量 | 7673P | 7806P | +133（PI-attributes 133F→P，零新增 fail——A/B per-case 唯一 diff 为本簇） |
| traversal / events / collections | 1595P/9F / 419P/27F / 48P/0F | 同值 | 零回归 |
| native PI-attributes | — | 128P/12F | 记叠加路径对齐候选（native document 环境下 main-parser 快照/MO 分支行为差，R109 同族） |

## 根因与修复（七层）

1. **PI 无属性接口**（`pi.hasAttributes` undefined 全簇崩）：part04 `isPI` 分支补属性
   五件套（hasAttributes/getAttributeNames/getAttribute/hasAttribute/setAttribute/
   removeAttribute/toggleAttribute）——读写统一经 part05 `_zwPiParseAttrs`/`_zwPiSetData`
   （data 即属性序列化源：`a="b" x="yy"` ⇔ [['a','b'],['x','yy']]，改属性后 data 重序列化
   ——改值原位、新增尾追、移除收缩）。mutation record 记 **characterData** 类型
   （spec PI 属性变更是 data 变更的观察面）。
2. **值转义三面一致**：`_zwPiEscape`/`_zwPiUnescape`（& " < > U+00A0 全集）+ 解析层反转义
   （setAttribute 原值往返）。对齐路径：part04 handle outerHTML attr 转义（旧只转 & "）+
   part03 `_zwMEscapeAttr` 同步全集——与 Rust `escape_html` 三面一致（WPT
   check-attribute-value 簇 `pi.data === element.outerHTML 提取值` 逐串相等）。
3. **`data =` 重解析**：set trap 的 PI 分支写 `_piHandles.data`（属性层解析源随之刷新）+
   `pi.data=''` → hasAttributes false + `'blabla=""'` → getAttribute('blabla')=''。
4. **bogus comment 双 parse 派生 PI 视图**：`<?t …?>` 经 HTML tokenizer 落为 bogus
   comment（data '?…?'）。`_zwMPiFromBogus`（part03）：剥壳拆 target/data + 属性五件套 +
   ownerDocument + CharacterData 编辑方法。两路接入：innerHTML 子树（`_zwMBuildNode` C
   分支）+ 主文档 parse（part05 `_wrapNodeEntry`）。
5. **观测回链**：PI 视图无 sel/handle——`__zwFragHostHandle`（`_zwFragmentAdded` 顶层子
   盖宿主印章）+ `__zwMoSelfKey`（XML doc createPI 自观测键）双通道接 MutationObserver
   observe 回落与 piNotify 投递；record.target 身份对齐（`__zwNotifyTarget`）。
6. **XML 文档面**（DOMParser parseFromString）：createProcessingInstruction（spec 校验）+
   XML createElement 大小写敏感（spec——旧统一大写断 `el` regex 匹配）+ 轻量属性/
   outerHTML 面 + firstChild 前导 `<?…?>` PI 合成（`<?xml` 声明跳过）。负向前瞻字符类
   正则在含 `?>` 值上失配（三 case 实证）——改 split 提取。
7. **Name production 校验**：`_zwPiValidName`（'=' '>' '/' 空白 '<' '"' →
   InvalidCharacterError；'$' '_' 等 XML Name 宽集合法——WPT invalid/valid 名单全对齐）。

## 回归与修正（过程中三处）

- **MutationObserver-characterData PI 回归**（1F）：`_wrapNodeEntry` PI 视图首版只接
  data setter——deleteData/replaceData undefined。补全 CharacterData 编辑方法（写统一经
  comment 节点 `_write`：本地同步 + host SetChildText + record）+ record.target 身份
  （mutationobservers.js 断言 `record.target === 观察的 PI 视图`，内部 comment 对象
  identity 不等）。
- **lit 首渲染回归**（3 e2e 失败）：lit 模板串尾注入 `<?>` 占位（bundle `t[s]||"<?>"`）——
  bogus comment data='?' 被转 PI 视图后 lit TreeWalker 的 `r.data===marker` part 定位
  失败（首渲染插值全空）。**收紧守卫**：'?target …?' 形态（剥壳后非空合法 target +
  空格分隔）才转 PI，'?' 裸壳保回 comment（`_zwMBuildNode` + `_wrapNodeEntry` 双路）。
- **转义二版反复**：首版 PI 转义含 < >、element 侧不转（两面不等）；二版全去掉（值含
  < > 时 regex 提取截断）；终版**两面同加全集**（element outerHTML 提取值与 PI data
  对转义后串相等——regex 在 element 侧因 < > 已转义不再提前截断）。

## 验证

- PI-attributes 140P/0F（polyfill 100%）；MutationObserver-characterData 19P/0F
- nodes 全量 7806P（+133 净，A/B per-case 零新增 fail）
- traversal 1595P/9F、events 419P/27F、collections 48P/0F 逐值零回归
- engine 单测 `test_pi_attribute_layer_r123`（20 断言段）+ lit e2e 3 组回归修复后全绿
- `make test` 66 套件 17,499 全绿 exit 0；fmt 无 diff；clippy `-D warnings` 零警告
- 账本：`tests/wpt-runner/imported-tests.txt`（R123 条目）

## 设计注记

- **PI 属性是 JS 侧纯视图**：host 无 PI 属性存储，data 是唯一序列化源——与 R122
  `_zwAttrInstances`（host 扁平存储的多实例覆盖层）相反方向：PI 的权威在 JS，host 只收
  SetTextOnHandle 供渲染。
- **bogus comment → PI 的判别式是形态而非上下文**：'?target data?'（可剥出合法 target）
  才转——lit 的 '<?>' 占位证明「以 '?' 开头」不足（框架 marker comment 会被误吞）。
- **check-attribute-value 的相等条件是转义后串**：pi.data 与 element.outerHTML 提取值
  都须是转义后的形态（< > 也转），两面同款转义是唯一稳定解——element 侧不转时 regex
  会在原始 < > 处提前截断。
