# M2 切片 62（s62）——document-metadata 域开片：title.text 族收口 + processing-model 4F 分诊

日期：2026-10-24。两件事：

## 0. tabular-data 余 4F 分诊（不修，归属定档）

processing-model-1 四案 runner 现值：col-span-limits `offsetWidth` 期望 53/51 得
1023/1024（col span=1000 / colspan=1000 按字面列数摊宽，期望空列 0 宽塌缩）；
rowspan-0 / rowspan-0-quirks `clientHeight` 期望 >18 得 18（rowspan=0 未参与行高
分布）。全部是**表格布局 span 几何**——控制面排除清单「table/multicol 等布局渲染
差异 — rendering-compat 域」「本 goal 不越界修渲染」，归属渲染流记账回流。DOM
语义面（colSpan/col.span/rowSpan IDL clamp 15 断言）已全绿。tabular-data 在本流
收口定格 **130P/4F**（余案全归因挂账）。

## 1. document-metadata 开片（M1 基线 31P/75F/18T → **43P/63F/18T**，GAIN 12 / REG 0）

Mission 序 tabular-data 后下一域。域内从未切片；首轮分诊按文件族聚类：title 12F
（本轮收口）/ link 30F+load-error 时超 / style 20F / base 9F / meta color-scheme
10F / http-equiv 2F；18T 待分诊。

### 修复面（R5269，title.text 族 12F 全收）

1. **HTMLTitleElement.text getter**（part03，OPTION text gate 邻位）——child text
   content：**直接** Text 子 data 依树序拼接，comment/元素子不含（title.text-01
   断言 .text="TEXT" 而 textContent="TEXTELEMENT"）；**无空白归一**（折叠面在
   document.title getter；title.text-03 断言空白/制表/换行原样保留）。旧无 getter
   落属性反射兜底读 ''。sel 形 SetText 后子列表融合视图不载新文本（probe：
   childLen=0）——walk 空且 latest-wins 文本非空才直取 lw（appendChild 变更链
   walk 非空自然落 walk；lw 含元素后代文本不可直接作信号，title.text-01 首版
   回退实证）。
2. **title.text setter**（part04，OPTION 同位）——set child text content，与
   option.text R3907 同口径委托 textContent setter（string replace all 步骤一致，
   继承宿主写回 + 本地文本子注册 + childList 记录）。
3. **document.title setter 落地路径**（part06）——裸 `__zw_set_text('title', v)` →
   `querySelector('title').textContent = v`（该路径自带 __zw_set_text 宿主写 +
   本地文本子视图维护）；旧路径只写宿主快照，title 元素本地文本子 stale
   （title.text-03 setter 案 firstChild.nodeValue 读回旧值实证）。querySelector
   miss（无 title 文档）回退裸通道行为不变；`_doc_title` 缓存语义不动。

### 已知限制（无 corpus 断言，不扩机制）

- title 有元素子时 `title.text =` 后再 appendChild 文本：spec 期望新旧拼接，现读
  侧 walk 丢 SetText 写入段（SetText 后融合视图不载新文本的既有缺口，非本切片
  引入）。
- `_tcSame` 跳写时 title.text 与 spec 整体替换语义的边缘分歧不观测。

## 验证

- the-title-element 13/13 全绿（title-multiple 先绿）；document-metadata 域
  **43P/63F/18T**（GAIN 12 / REG 0）
- tabular-data 130P/4F、interactive-elements 439P/44F 逐值不变；zero-engine lib
  **3454P/0F**（R3035 title 写回共享面零回归）
- 拼接 shim node --check 通过；fmt/clippy 不适用（diff 仅 JS shim）

## 未解决项（document-metadata 余 63F+18T 挂账序）

- **base 族 9F**：HTMLBaseElement.href IDL（空/缺省回落文档地址、data:/javascript:
  回落）+ base 感知 URL 解析（img.src 相对解析）——下一片候选
- **meta color-scheme 10F**：applied-color 语义，归属核对（疑 style-system/渲染面）
- **link/style 事件族 30F+18T**：load/error 事件与外部资源管道，runner file 通道
  能力上限待分诊
- style type/media 族、CSP tentative 族待分诊
