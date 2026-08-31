# R141 — characterSet-normalization 654F→0F（encoding.py 虚拟化 + meta charset label 归一化）

**日期**: 2026-08-21
**里程碑**: M4（WPT dom 上游基线扩展）
**驱动用例**: `dom/nodes/Document-characterSet-normalization{,-2}.html`（315 + 339 subtest）

## 根因（单一根因，654F 全同源）

用例经 `<iframe src="encoding.py?label=X">` 取子文档，onload 后断言
`iframe.contentDocument.characterSet/inputEncoding/charset` === WHATWG encoding label
表归一化后的编码名。三层缺口：

1. **runner fetch handler 不认 .py**：上游 `encoding.py` 是 wptserve Python 脚本
   （读 `?label=` 返回 `<!doctype html><meta charset="X">`），wpt-data 无静态文件
   → fetch 404 → iframe contentDocument 恒 null → `Cannot read properties of null` 654F。
2. **相对 src 不解析**：shim 的 iframe 加载只处理根相对（`/x`）与绝对 URL，
   `encoding.py?label=X`（文档相对）不在 `https://wpt.test` 域内 → fetch miss。
3. **meta charset → 编码名归一化缺失**：iframe 子文档的 characterSet 恒
   'UTF-8'（detached doc 默认），无 label 表映射。

## 修复（三处）

- **testharness.rs `wpt_data_fetch_handler`**：内置 `dom/nodes/encoding.py` 生成器——
  percent-decode `?label=` 参数，返回 `<!doctype html><meta charset="{label}">`
  （与上游 wptserve 脚本逐字等价，pin `3159769`）。
- **part04 iframe URL 解析**：相对 src（`x`、`./x`、`../x`）按 `location.href`
  目录段解析（`../` 逐层弹出），根相对/绝对保持原逻辑
  （spec HTML iframe src「resolve a URL」）。
- **part05 `_zwMakeIframeDoc`**：解析 `<meta charset>` → `_zwEncodingFromLabel`
  归一化（ASCII 大小写不敏感 + ASCII 空白剥离 + `_ZW_ENC_LABELS` 全表
  ——WHATWG encoding spec labels 全集 34 编码）→ 覆写 doc 的
  characterSet/charset/inputEncoding getter。**HTML meta 特例**（spec
  documentEncoding）：utf-16/utf-16be/utf-16le → UTF-8、x-user-defined →
  windows-1252。**首跑补漏**：ISO-8859-6 的 `iso-8859-6-e`/`iso-8859-6-i` 两 label
  首版表漏（6F 暴露）已补。

## A/B 结果（polyfill / native 双路径）

| 套件 | 结果 |
|---|---|
| characterSet-normalization-1 | **315P/0F 双路径 100%** |
| characterSet-normalization-2 | **339P/0F 双路径 100%** |
| dom/nodes 全量 | 5452P/232F（R140 口径 4799P/874F → **+653P/-642F**；charnorm 654F 全消；其余 fail 集零新增——3 个 crash/query-target Timeout 为并行调度 flake，隔离复跑全 Pass） |
| dom/events | fail 集与 R140 基线**完全一致**（2 个 incumbent-global-subframe Timeout 为调度 flake，隔离 Pass） |
| dom/traversal | fail 集一致（50P/6F） |
| dom/collections | 49P/0F 全绿 |
| `make test` | 66 套件全绿（双矩阵） |
| fmt / clippy | 零 diff / 零警告 |

## 单元测试

`encoding_py_handler_generates_meta_charset_from_label_query`（testharness.rs，4 断言段）：
label 透传 / percent-decode / 缺 label 空 charset / 非 .py 路径回落静态文件。
