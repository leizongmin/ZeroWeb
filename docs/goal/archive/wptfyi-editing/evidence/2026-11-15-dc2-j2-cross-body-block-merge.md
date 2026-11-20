# DC-2 J2 跨 body 边界块合并切片（d36）——del-html 余 8F 清账

日期：2026-11-15（d36 会话，接 d35 plaintext-only styling 门批；上一 session 429
中断遗留 WIP，本 session 完成验证与落账）
范围：part05.js `_b1JoinMembers` body 吸收臂内新增 J2 块并块分支 +
`_b1BlockTags` 块级标签面表（两处，零 Rust 变更）
语料面：`delete-in-child-of-html`（主目标）、del/IP-in-child-of-{head,html}
守护、tal 消费面、b1s3/d31-unlink 探针、engine lib pin

## 结论

- `delete-in-child-of-html` **34P/12F → 42P/4F 净 +8**（F2P=8 / P2F=0）：
  8 个 J2 形（div↔div 跨 body 边界 delete/forwarddelete 并块）全转 Pass；
  余 4F = d17 XOR 伪失败族（`assert_true(isConnected ^ isConnected)`
  返 number，任何浏览器同红，出队不动）。
- HEAD 基线二进制定谳（本切片 before/after 双跑）：base 34P/12F →
  WIP 42P/4F，翻红子测逐名 = 8 个 J2 形，两二进制 Pass 集恒等（零 P2F）。
- tal 全消费面对拍（base vs WIP，status+name 级）：**逐行恒等**——DM 变体
  125P+84P、CE 主窗残余、内壳形 `child=b`/`parent=b&child=i` ×4 Timeout
  全数不变，J2 门条件（body 单可见块 + src 可见块）在 tal 形不触发。
- 守护面零漂移：delete-in-child-of-head 64P/4F、insertparagraph-in-child-of-html
  64P/8F、insertparagraph-in-child-of-head 64P/8F、zz-probe-b1s3 四姊妹
  （main 4P/9F + eng 3F + solo 8F + diag 2F）、zz-probe-d31-unlink 8P/0F。

## J2 形语义（为何展平失账）

del-html 语料 J2 形：宿主 div 在 html 根直下（body 之外），caret 在 div 末/首，
delete 跨 body 边界与 body 内唯一可见块（如 `<body><div>def</div></body>` 并
`<div>abc</div>` 宿主）并合。corpus 期望块级吸收——`<body><div>abcdef</div></body>`
（两 div 并一，靠前块存续）；既有展平臂产 `srcInner + bodyHtml` 字面拼接，
解析后为 `<div>abc</div>def` 两段形失账。修后按块并块：body 可见内容恰为
单块元素 B（其余仅空白文本/不可见/占位 br）且 src 亦为可见块 → 以靠前块
为载体（属性随靠前块）innerHTML 重写（R220 纪律同展平臂），缝 caret 落
靠前块文本末端（解析器并合相邻文本后即缝位）。块判定按标签面直判
（`_b1BlockTags` 表）——子文档域 computed display 未接，解析值判定误收
inline（表内注释同述）。规范依据 delete 跨块边界并块语义
https://w3c.github.io/editing/execCommand.html#deleting-text 。

## 两处修改（part05.js）

1. `_b1BlockTags` 块级标签面表（`_b1Tag` 后）：address–xmp 36 标签静态面。
2. `_b1JoinMembers` body 吸收臂 J2 分支：①并块条件扫描（body childNodes
   逐员——空白文本跳过、非空白文本/bad 形翻 bad、占位 br 跳过、不可见跳过、
   族标签/第二可见块/非块翻 bad）；②靠前块存续重写（src 在 body 前 → src
   壳为载体并入 B 位；src 在后 → B 壳吸收 src 内容原位存续；属性序列化
   `&`/`"` 转义）；③缝 caret（并块内首文本节点 offset=靠前块文本长度，
   缺失/越界落 body 首）。非 J2 形走原展平臂，字节等价。

## 面账（d36 终态）

| 面 | 基线（d35 账） | 实得 |
|---|---|---|
| `delete-in-child-of-html` | 34P/12F | **42P/4F**（F2P=8/P2F=0） ✓ |
| `delete-in-child-of-head`（守护） | 64P/4F | 64P/4F 恒 ✓ |
| `insertparagraph-in-child-of-html`（守护） | 64P/8F | 64P/8F 恒 ✓ |
| `insertparagraph-in-child-of-head`（守护） | 64P/8F | 64P/8F 恒 ✓ |
| `typing-around-link` 三文件（base vs WIP） | — | status+name 级逐行恒等 ✓ |
| `zz-probe-b1s3`（四姊妹） | 4P/9F + 3F/8F/2F | 逐文件恒 ✓ |
| `zz-probe-d31-unlink`（哨兵） | 8P/0F | 8P/0F 恒 ✓ |
| engine lib（test-guard，独立 target） | 4033P/0F | 见下节 ✓ |

## 验证与执行注记

- shim 串联语法检查（node --check，part01–part06 主窗装配）：SHIM-SYNTAX-OK。
- before/after 双二进制：`zw-runner-base-d36`（HEAD，stash 往返构建）vs
  `zw-runner-d36`（WIP）；del-html/tal 两面各双跑，fail 集对拍见上表。
- runner 通道勘误（本 session 记档）：child-doc 语料归 **selection 通道**
  （`testharness-selection`，SELECTION_TEST_SUBDIRS 含 editing/other 且
  无 `<iframe` 跳过）；`testharness-editing-other` 通道的
  `editing_case_skipped` 对 `<iframe` 语料整文件跳过（该通道此前
  "editing/other 12 快页" 口径为去 iframe 面）。filter 子串 `delete-in-child-of-html`
  在 selection 通道命中 4 variant 全谱。
- 引擎 lib pin：`cargo test -p zero-engine --lib`（test-guard 包裹，
  CARGO_TARGET_DIR=/tmp/d19tgt）4033P/0F 恒；clippy -p zero-engine
  --all-features 无告警；cargo fmt --check 过。

## 遗留挂账（del-html 余 4F 出队，其余不变）

- del-html 余 4F = d17 XOR 伪失败族（S8 `assert_true(a.isConnected ^ b.isConnected)`
  返 number 非严格 boolean，harness 固有，任何浏览器同红——d33 记档出队）。
- FIXME(d33) 跨文档收养缺口（IP-html 余 8F，引擎级 adoptNode）。
- FIXME(d24) unlink 余样式重裹 4F。
- tal 内壳形收敛（前置 d24 单一补偿面 + caret 引擎同位快照）；tal CE 主窗
  变体残余（collapsed ~122F/non-collapsed ~77F 本轮对拍恒等持位）。
