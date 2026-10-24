# M2 切片 64（s64）——document-metadata 分诊：color-scheme 归属 + link/style 事件族 + 窄面收口

日期：2026-10-24。三件事：①meta color-scheme 归属核对（10F 死因 = corpus
checkout gap，断言面归渲染流）；②link/style 事件族分诊（35F+18T 四类落档，
可 shim 收口窄面 7F 本轮全收）；③corpus 缺失 helper 本地修复（步进归因显式
入账）。

## 0. corpus 修复（本地 gitignored，不入库）

color-scheme 11 文件中 10 个死于 `assert_root_color_scheme is not defined`
（page script threw，declared tests: 0）——wpt-data 打包裁剪子集缺
`color-scheme/support/compute-root-color-scheme.js`（上游 WPT 有）。按
「打包缺口升级时按需本地补」先例（scripts/fetch-wpt-subdir.sh 同型）从上游
恢复。步进归因：**52P/54F/18T → 52P/66F/18T**（+12F = 10 文件声明真实子
测试，纯 corpus 步进非引擎变化）。

## 1. color-scheme 归属核对（定档：渲染流记账回流）

helper 恢复后 23F **全塌缩到单断言**：`getComputedStyle(div).color` 返空串
——inline `style.color = CanvasText` + `style.colorScheme = only light/dark`
的系统色解析面（light/dark 须解析出不同 computed color）。断言链 = 系统色
used-value × color-scheme 应用值 + `colorScheme` CSSOM IDL + root 应用色随
meta 变化，全在 applied-color（style-system/getComputedStyle 面）→ 归渲染
流记账回流，本流不修。

## 2. link/style 事件族分诊（35F+18T 四类落档）

- **资源管道面 ~13F**（link 侧）：外部样式表 fetch+apply
 （expected rgb(0,128,0) 得黑——stylesheet-change/remove/tokenization/
  document-without-browsing-context/stylesheet-not-removed）+ load/error
  事件与 parsing-blocking 次序（link-load-fired-before 等）→ 渲染流/net 面。
- **applied CSSOM 面 ~13F**（style 侧 + styling/）：media 门
 （style_media×3）、type 门（style_type_change 3 + style_type_html 2）、
 disabled（1）、load 事件（style_load_event 1）、style2 alternate 空 title
 sheet-null（LinkStyle 1）、interactions-of-styling-and-scripting 3F →
 渲染流（sheet 存在性/应用性由样式管线裁决）。
- **CSP tentative 底座缺口 7F+5T**：Declarative CSS Modules import maps
 （"Module foo does not provide an export named default"）+ CSP/securitypolicy
 violation 面 tentative 族 → 底座挂账。
- **parse-world 底座 1F**：non-parser-created-doc（DOMParser 文档 style 元素
 无 sheet——parse-world CSSOM 接线，s54/s55 同架构面）→ 底座挂账。
- **shim 窄面 7F：本轮全收（R5271）**。

## 3. 修复面（R5271，GAIN 6 / REG 0）

1. **link.fetchPriority enumerated reflection**（part04，crossOrigin/as 臂
 邻位）——fetchpriority「limited to only known values」：合法关键字
 ascii 小写返 canonical，missing/invalid value default 均 'auto'
（attr-link-fetchpriority 2F 收口）。FIXME: IDL setter corpus 未钉暂缺。
2. **link.relList supported tokens**（part04 R374 relList 分支内 LINK 臂）
 ——link types 表适用 link 的 20 关键字全集（HTML §4.6.7 表逐行提取），
 supports() 精确匹配；link-rellist `supports("stylesheet")` 1F 收口。
 FIXME: a/area.relList 各有适用子集，corpus 未钉暂无表。
3. **style.sheet [SameObject] + 指纹失效 + connected 门 + StyleSheet
 brand**（part06 `_makeStyleSheet` 包裹层）：
   - **[SameObject] 缓存**（WeakMap per owner proxy）——旧每读重建，跨读
     identity 断言必炸（mutations.window.js「空 DocumentFragment」案根因，
     探针钉定 `.sheet !== .sheet` 跨读即红）。
   - **直接子指纹失效**（nodeType + data/nodeValue 串接）——有效子变更
    （comment 插入/数据变更/normalize/移除）换新对象（CSSOM remove+create
     语义，corpus mutations.window.js 四案钉定）；零子插入（空 fragment）
     指纹不变身份保持。
   - **connected 门**——元素不在文档 sheet 返 null（LinkStyle「not in a
     Document」案钉定；CSS-in-JS 先 append 后读时序不受影响）。
   - **StyleSheet 基接口**（globalThis.StyleSheet + CSSStyleSheet.prototype
     挂链 + ss setPrototypeOf brand）——`instanceof StyleSheet` 断言可达
    （LinkStyle「must return StyleSheet object」案；disabled 等式同案连带
     过）。外部合成条目（owner=null）不缓存不设门。

## 验证

- document-metadata 域 **52P/66F/18T → 58P/60F/18T**（GAIN 6 / REG 0；对
  s63 记账口径 52P/54F/18T 为 corpus 步进 +12F 后净 -6F，逐文件 F 清单
  diff 仅 LinkStyle 3F→1F 与 fetchpriority/rellist/mutations 消失）
- tabular-data **130P/4F**（终版二进制复跑）；interactive-elements
  **439P/44F/0T** 逐值不变；zero-engine lib **3458P/0F**（+2 为 oracle 流
  新增；connected 门/brand/缓存对 R2808/R2809 CSSOM 契约零回归——契约用
  已挂载 `<style>`）
- 拼接 shim node --check 通过；fmt/clippy 不适用（diff 仅 JS shim）

## 未解决项（document-metadata 余 60F+18T 挂账序）

- color-scheme 23F + 资源管道 ~13F + applied CSSOM ~13F + style2 1F →
  渲染流记账回流
- CSP tentative 7F+5T + non-parser-created-doc 1F → 底座挂账
- **http-equiv-and-name 2F**：本流 shim 面最后未分诊项——下一片候选
- 域收口后下一域：text-level-semantics（M1 基线 21P/13F）开片
