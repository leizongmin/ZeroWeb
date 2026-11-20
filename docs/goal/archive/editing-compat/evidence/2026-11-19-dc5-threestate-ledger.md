# DC-5 editing 三态清零路径记账（2026-11-19）

- 浏览器基准（wptfyi-cache aligned stable）：{'chrome': ('158.0.8127.36', '71b58bb191'), 'firefox': ('160.0.1', '71b58bb191'), 'safari': ('27.2 (22625.2.8.11.9)', '71b58bb191')}
- universe（三家并集 /editing/）：files=888 subtests=117243
- ZW 结果条目：289 文件 → 合并 761 路径（editing 面）

## 三态总表

| 态 | 文件数 | 子测试数 |
|---|---:|---:|
| 已执行 executed | 581 | P=108448 F=2878 T=0 其他=0 |
| 具名排除 named-excluded | 47 | 1275 |
| 上游不存在/域外划界 upstream-absent | 260（域外 232＋pin 缺 28） | 4150 |
| 未记账缺口 unaccounted（应为 0） | 0 | 0 |

## 具名排除分理由

- **https**：4 文件 / 4 子测试
- **iframe**：43 文件 / 1271 子测试

## 具名排除文件清单（reason | path | 子测试分母）

- https | /editing/other/copy-paste-in-styled-inline-editing-host.https.html | 1
- https | /editing/other/paste-multiline-text-as-plaintext.tentative.https.html?white-space=pre | 1
- https | /editing/other/paste-multiline-text-as-plaintext.tentative.https.html?white-space=pre-line | 1
- https | /editing/other/paste-multiline-text-as-plaintext.tentative.https.html?white-space=pre-wrap | 1
- iframe | /editing/other/delete-in-child-of-head.tentative.html?designMode=off&method=backspace | 17
- iframe | /editing/other/delete-in-child-of-head.tentative.html?designMode=off&method=forwarddelete | 17
- iframe | /editing/other/delete-in-child-of-head.tentative.html?designMode=on&method=backspace | 17
- iframe | /editing/other/delete-in-child-of-head.tentative.html?designMode=on&method=forwarddelete | 17
- iframe | /editing/other/delete-in-child-of-html.tentative.html?designMode=off&method=backspace | 12
- iframe | /editing/other/delete-in-child-of-html.tentative.html?designMode=off&method=forwarddelete | 11
- iframe | /editing/other/delete-in-child-of-html.tentative.html?designMode=on&method=backspace | 12
- iframe | /editing/other/delete-in-child-of-html.tentative.html?designMode=on&method=forwarddelete | 11
- iframe | /editing/other/exec-command-without-editable-element.tentative.html | 474
- iframe | /editing/other/insertparagraph-in-child-of-head.tentative.html?designMode=off&white-space=normal | 9
- iframe | /editing/other/insertparagraph-in-child-of-head.tentative.html?designMode=off&white-space=pre | 9
- iframe | /editing/other/insertparagraph-in-child-of-head.tentative.html?designMode=off&white-space=pre-line | 9
- iframe | /editing/other/insertparagraph-in-child-of-head.tentative.html?designMode=off&white-space=pre-wrap | 9
- iframe | /editing/other/insertparagraph-in-child-of-head.tentative.html?designMode=on&white-space=normal | 9
- iframe | /editing/other/insertparagraph-in-child-of-head.tentative.html?designMode=on&white-space=pre | 9
- iframe | /editing/other/insertparagraph-in-child-of-head.tentative.html?designMode=on&white-space=pre-line | 9
- iframe | /editing/other/insertparagraph-in-child-of-head.tentative.html?designMode=on&white-space=pre-wrap | 9
- iframe | /editing/other/insertparagraph-in-child-of-html.tentative.html?designMode=off&white-space=normal | 9
- iframe | /editing/other/insertparagraph-in-child-of-html.tentative.html?designMode=off&white-space=pre | 9
- iframe | /editing/other/insertparagraph-in-child-of-html.tentative.html?designMode=off&white-space=pre-line | 9
- iframe | /editing/other/insertparagraph-in-child-of-html.tentative.html?designMode=off&white-space=pre-wrap | 9
- iframe | /editing/other/insertparagraph-in-child-of-html.tentative.html?designMode=on&white-space=normal | 9
- iframe | /editing/other/insertparagraph-in-child-of-html.tentative.html?designMode=on&white-space=pre | 9
- iframe | /editing/other/insertparagraph-in-child-of-html.tentative.html?designMode=on&white-space=pre-line | 9
- iframe | /editing/other/insertparagraph-in-child-of-html.tentative.html?designMode=on&white-space=pre-wrap | 9
- iframe | /editing/other/insertparagraph-in-non-splittable-element.html | 14
- iframe | /editing/other/typing-around-link-element-at-collapsed-selection.tentative.html?target=ContentEditable | 37
- iframe | /editing/other/typing-around-link-element-at-collapsed-selection.tentative.html?target=ContentEditable&child=b | 30
- iframe | /editing/other/typing-around-link-element-at-collapsed-selection.tentative.html?target=ContentEditable&parent=b | 37
- iframe | /editing/other/typing-around-link-element-at-collapsed-selection.tentative.html?target=ContentEditable&parent=b&child=i | 30
- iframe | /editing/other/typing-around-link-element-at-collapsed-selection.tentative.html?target=DesignMode | 37
- iframe | /editing/other/typing-around-link-element-at-collapsed-selection.tentative.html?target=DesignMode&child=b | 30
- iframe | /editing/other/typing-around-link-element-at-collapsed-selection.tentative.html?target=DesignMode&parent=b | 37
- iframe | /editing/other/typing-around-link-element-at-collapsed-selection.tentative.html?target=DesignMode&parent=b&child=i | 30
- iframe | /editing/other/typing-around-link-element-at-non-collapsed-selection.tentative.html?target=ContentEditable | 31
- iframe | /editing/other/typing-around-link-element-at-non-collapsed-selection.tentative.html?target=ContentEditable&child=b | 21
- iframe | /editing/other/typing-around-link-element-at-non-collapsed-selection.tentative.html?target=ContentEditable&parent=b | 21
- iframe | /editing/other/typing-around-link-element-at-non-collapsed-selection.tentative.html?target=ContentEditable&parent=b&child=i | 21
- iframe | /editing/other/typing-around-link-element-at-non-collapsed-selection.tentative.html?target=DesignMode | 31
- iframe | /editing/other/typing-around-link-element-at-non-collapsed-selection.tentative.html?target=DesignMode&child=b | 21
- iframe | /editing/other/typing-around-link-element-at-non-collapsed-selection.tentative.html?target=DesignMode&parent=b | 21
- iframe | /editing/other/typing-around-link-element-at-non-collapsed-selection.tentative.html?target=DesignMode&parent=b&child=i | 21
- iframe | /editing/run/undo-redo.html | 69

## 本地 pin 缺文件（universe 在、语料无）

- /editing/other/copy-table-selection-ending-at-row-end-in-contenteditable.html | 2
- /editing/other/copy-table-selection-ending-at-row-end.html | 2
- /editing/other/delete-object-fallback.html?block-content-range | 1
- /editing/other/delete-object-fallback.html?caret | 1
- /editing/other/delete-object-fallback.html?forward | 1
- /editing/other/delete-object-fallback.html?inline-block-range | 1
- /editing/other/delete-object-fallback.html?inside-backward | 1
- /editing/other/delete-object-fallback.html?inside-forward | 1
- /editing/other/delete-object-fallback.html?inside-range | 1
- /editing/other/delete-object-fallback.html?nested-cross-range | 1
- /editing/other/delete-object-fallback.html?nested-range | 1
- /editing/other/delete-object-fallback.html?paragraph-control | 1
- /editing/other/delete-object-fallback.html?pre-wrap-range | 1
- /editing/other/delete-object-fallback.html?range | 1
- /editing/other/delete-object-fallback.html?undo-redo | 1
- /editing/other/editing-host-with-only-out-of-flow-child.html | 4
- /editing/other/insert-text-with-surrogate-pairs.html | 1
- /editing/other/inserttext-before-unnecessary-linebreak.html | 14
- /editing/other/paste-as-plaintext-data.tentative.https.html | 6
- /editing/other/trailing-linebreak-of-inline-editing-host.tentative.html?linebreak=br | 224
- /editing/other/trailing-linebreak-of-inline-editing-host.tentative.html?linebreak=linefeed | 224
- /editing/run/conformance-subset-command-prefix.html?1-1 | 2
- /editing/run/conformance-subset-command-prefix.html?2-2 | 2
- /editing/run/conformance-subset-command-prefix.html?3-4 | 3
- /editing/run/conformance-subset-prerequisites.html?1-18&css=true | 19
- /editing/run/conformance-subset-prerequisites.html?12-18&css=false | 8
- /editing/run/conformance-subset-prerequisites.html?13-18&css=false | 7
- /editing/run/conformance-subset-prerequisites.html?13-18&css=true | 7

## 域外划界目录文件：232 条 / 3612 子测试（edit-context, plaintext-only, whitespaces, chrome-compat, crashtests, manual）

## 未解析上报（不在 universe，不进分子分母）：180 条
- /editing/other/zw-probe-d123.html
- /editing/other/zw-probe-ihost-lb.html
- /editing/other/zw-probe-ihost-leak.html
- /editing/other/zw-probe-ihost-pre-min.html
- /editing/other/zw-probe-ihost-shift.html
- /editing/other/zw-probe-lrlp-bs.html
- /editing/other/zw-probe-lrlp-fd.html
- /editing/other/zz-probe-ip-caret0-dump.html
- /editing/other/zz-probe-qv13.html
- /editing/run/multitest-ab.html
- /editing/run/zz-b4ul-g.html
- /editing/run/zz-b4ul-pfx.html
- /editing/run/zz-bisect-c4bq.html
- /editing/run/zz-bisect-c4ul.html
- /editing/run/zz-bisect-nocss.html
- /editing/run/zz-bisect-noflip.html
- /editing/run/zz-bisect.html
- /editing/run/zz-diag-mtab-head.html
- /editing/run/zz-diag-mtab-tail.html
- /editing/run/zz-diag-styleattr.html

